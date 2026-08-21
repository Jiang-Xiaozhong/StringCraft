//! 检查更新（需求 4.8）：基于 GitHub Releases，仅访问 GitHub API。
//! v0.21 起移除自动更新（应用内下载/安装），更新统一跳转浏览器下载。

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
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    body: Option<String>,
    html_url: String,
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

    Ok(UpdateInfo {
        latest: has_update,
        version: Some(latest_version),
        notes: release.body,
        url: Some(release.html_url),
    })
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
    let auto_check = app
        .try_state::<ConfigState>()
        .map(|state| {
            state
                .0
                .lock()
                .map(|guard| guard.auto_check_update)
                .unwrap_or(false)
        })
        .unwrap_or(false);
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
