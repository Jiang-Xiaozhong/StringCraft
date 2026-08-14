mod commands;
mod config;
mod hotkey;
mod selection;
mod transform;
mod tray;

use crate::config::ConfigState;
use std::sync::Mutex;
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
            let config = config::load_or_default(app.handle());
            let startup_hotkey = config.hotkey.clone();
            app.manage(ConfigState(Mutex::new(config)));

            hotkey::install(app.handle())?;
            if let Err(e) = hotkey::register(app.handle(), &startup_hotkey) {
                eprintln!("[StringCraft] 全局快捷键注册失败：{e}");
            }

            tray::create_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::apply_no_activate,
            commands::open_settings,
            commands::execute_button
        ])
        .run(tauri::generate_context!())
        .expect("StringCraft 启动失败");
}

/// Windows：为悬浮条设置 WS_EX_NOACTIVATE，点击按钮时不夺取原应用焦点（F1.12）。
#[cfg(target_os = "windows")]
pub fn apply_no_activate(win: &tauri::WebviewWindow) -> tauri::Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE,
    };

    let hwnd = win.hwnd()?;
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    if style == 0 {
        return Ok(());
    }
    let _ = unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_NOACTIVATE.0 as isize) };
    Ok(())
}
