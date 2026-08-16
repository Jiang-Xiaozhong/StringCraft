//! 检查更新与自动更新（需求 4.8）：基于 GitHub Releases，仅访问 GitHub API。

use crate::config::ConfigState;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const REPO_API: &str = "https://api.github.com/repos/Jiang-Xiaozhong/StringCraft/releases/latest";
const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub latest: bool,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub url: Option<String>,
    pub asset_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    body: Option<String>,
    html_url: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

/// 查询 GitHub 最新 Release 并与当前版本比较。
pub fn check_for_update() -> Result<UpdateInfo, String> {
    let mut response = ureq::get(REPO_API)
        .header("User-Agent", "StringCraft")
        .call()
        .map_err(|e| format!("检查更新失败：{e}"))?;
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("读取更新信息失败：{e}"))?;
    let release: GithubRelease =
        serde_json::from_str(&body).map_err(|e| format!("解析更新信息失败：{e}"))?;

    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let has_update = is_newer(&latest_version, env!("CARGO_PKG_VERSION"));
    let asset_url = release
        .assets
        .iter()
        .find(|asset| asset.name.ends_with(".exe"))
        .map(|asset| asset.browser_download_url.clone());

    Ok(UpdateInfo {
        latest: has_update,
        version: Some(latest_version),
        notes: release.body,
        url: Some(release.html_url),
        asset_url,
    })
}

/// 下载更新安装包到临时目录，返回本地路径。
pub fn download_update(asset_url: &str) -> Result<String, String> {
    let file_name = asset_url
        .rsplit('/')
        .next()
        .unwrap_or("StringCraft-update.exe");
    let dir = std::env::temp_dir().join("stringcraft-update");
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建临时目录失败：{e}"))?;
    let path = dir.join(file_name);

    let mut response = ureq::get(asset_url)
        .header("User-Agent", "StringCraft")
        .call()
        .map_err(|e| format!("下载更新失败：{e}"))?;
    let bytes = response
        .body_mut()
        .read_to_vec()
        .map_err(|e| format!("读取下载内容失败：{e}"))?;
    std::fs::write(&path, bytes).map_err(|e| format!("写入安装包失败：{e}"))?;
    Ok(path.to_string_lossy().to_string())
}

/// 启动安装包（Windows NSIS）。
pub fn launch_installer(path: &str) -> Result<String, String> {
    std::process::Command::new(path)
        .spawn()
        .map_err(|e| format!("启动安装程序失败：{e}"))?;
    Ok("安装程序已启动".to_string())
}

/// 后台定时检查：启动后延迟 10 秒检查一次，之后每 24 小时检查一次。
pub fn start_update_checker(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(10));
        loop {
            check_once(&app);
            std::thread::sleep(CHECK_INTERVAL);
        }
    });
}

fn check_once(app: &AppHandle) {
    let (auto_check, auto_update) = app
        .try_state::<ConfigState>()
        .map(|state| {
            state
                .0
                .lock()
                .map(|guard| (guard.auto_check_update, guard.auto_update))
                .unwrap_or((false, false))
        })
        .unwrap_or((false, false));
    if !auto_check {
        return;
    }

    let info = match check_for_update() {
        Ok(info) => info,
        Err(_) => return,
    };
    if !info.latest {
        return;
    }

    let _ = app.emit_to("settings", "update-found", &info);
    if auto_update {
        if let Some(asset_url) = info.asset_url {
            if let Ok(path) = download_update(&asset_url) {
                let _ = app.emit_to("settings", "update-ready", path);
            }
        }
    }
}

/// 版本号比较：`a` 大于 `b` 返回 true。
fn is_newer(a: &str, b: &str) -> bool {
    version_tuple(a) > version_tuple(b)
}

fn version_tuple(version: &str) -> (u64, u64, u64) {
    let mut parts = version.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}
