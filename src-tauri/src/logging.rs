//! 本地调试日志（需求第 5 节）：默认关闭，可在设置页开启。
//! 隐私：只记录运行事件，不记录任何被转换的用户文本。

use crate::config::ConfigState;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// 开启调试日志时追加一条带时间戳的运行记录；未开启时直接跳过。
pub fn log_event(app: &AppHandle, message: &str) {
    let enabled = app
        .try_state::<ConfigState>()
        .map(|state| state.0.lock().map(|guard| guard.debug_log).unwrap_or(false))
        .unwrap_or(false);
    if !enabled {
        return;
    }

    let path = match log_path() {
        Ok(path) => path,
        Err(_) => return,
    };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let line = format!("[{}] {}\n", unix_timestamp(), message);
    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(line.as_bytes()));
}

/// 日志文件路径：与配置文件同目录下的 `logs/stringcraft.log`。
fn log_path() -> Result<PathBuf, String> {
    let config = crate::config::config_path()?;
    Ok(config
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("logs")
        .join("stringcraft.log"))
}

/// Unix 秒时间戳，避免引入额外时间库依赖。
fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
