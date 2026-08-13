mod commands;
mod config;
mod hotkey;
mod transform;
mod tray;

use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::MacosLauncher;

pub const FLOAT_BAR_LABEL: &str = "float-bar";
pub const SETTINGS_LABEL: &str = "settings";

/// 呼出悬浮条：显示并取消最小化（不主动抢焦点）。
pub fn show_float_bar(app: &AppHandle) {
    let Some(win) = app.get_webview_window(FLOAT_BAR_LABEL) else {
        return;
    };
    let _ = win.show();
    let _ = win.unminimize();
}

/// 呼出设置窗口：显示、取消最小化并聚焦，便于立即操作。
pub fn open_settings(app: &AppHandle) {
    let Some(win) = app.get_webview_window(SETTINGS_LABEL) else {
        return;
    };
    let _ = win.show();
    let _ = win.unminimize();
    let _ = win.set_focus();
}

/// 切换悬浮条显隐（全局快捷键使用）。
pub fn toggle_float_bar(app: &AppHandle) {
    let Some(win) = app.get_webview_window(FLOAT_BAR_LABEL) else {
        return;
    };
    if win.is_visible().unwrap_or(false) {
        let _ = win.hide();
    } else {
        show_float_bar(app);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 第二次启动（如双击快捷方式）只呼出悬浮条，不创建新实例
            show_float_bar(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            tray::create_tray(app)?;
            hotkey::register_default(app.handle())?;
            position_float_bar(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::open_settings,
            commands::execute_button
        ])
        .run(tauri::generate_context!())
        .expect("StringCraft 启动失败");
}

/// 悬浮条首次启动显示在屏幕右上角（距上/右边缘 16px）。
fn position_float_bar(app: &AppHandle) -> tauri::Result<()> {
    let Some(win) = app.get_webview_window(FLOAT_BAR_LABEL) else {
        return Ok(());
    };
    let Some(monitor) = win.current_monitor()? else {
        return Ok(());
    };
    let size = win.outer_size()?;
    let work = monitor.work_area();
    let margin = 16i32;
    let x = work.position.x + (work.size.width as i32 - size.width as i32 - margin).max(0);
    let y = work.position.y + margin;
    win.set_position(tauri::PhysicalPosition::new(x, y))?;
    Ok(())
}
