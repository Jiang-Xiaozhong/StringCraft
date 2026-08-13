use crate::config::{default_config, AppConfig};
use crate::transform;
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

/// 执行转换。M2 已提供 20 个纯函数转换；自动替换链路（获取选中文本）在 M3 接入。
#[tauri::command]
pub fn execute_button(transform_id: String, text: Option<String>) -> Result<String, String> {
    match text {
        Some(text) => Ok(transform::transform(&text, &transform_id)),
        None => Err("自动替换将在 M3 阶段实现".to_string()),
    }
}
