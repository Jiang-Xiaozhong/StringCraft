use crate::config::{default_config, AppConfig};
use crate::{selection, transform};
use tauri::AppHandle;

#[tauri::command]
pub fn get_config() -> AppConfig {
    default_config()
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), String> {
    crate::open_settings(&app);
    Ok(())
}

/// 执行转换。传入文本时直接转换（调试/测试用）；
/// 未传文本时执行完整自动替换链路（复制 → 转换 → 粘贴 → 恢复剪贴板）。
#[tauri::command]
pub async fn execute_button(transform_id: String, text: Option<String>) -> Result<String, String> {
    if let Some(text) = text {
        return Ok(transform::transform(&text, &transform_id));
    }
    tauri::async_runtime::spawn_blocking(move || selection::replace_selection(&transform_id))
        .await
        .map_err(|e| format!("任务执行失败：{e}"))?
}
