use crate::config::{default_config, AppConfig};
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

/// M1 阶段占位命令：真正的转换与自动替换在 M2/M3 实现。
#[tauri::command]
pub fn execute_button(transform_id: String) -> Result<(), String> {
    Err(format!("转换「{transform_id}」将在 M2 阶段实现"))
}
