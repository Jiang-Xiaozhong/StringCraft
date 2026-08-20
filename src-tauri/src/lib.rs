mod commands;
mod config;
mod hotkey;
mod logging;
mod macos;
mod selection;
mod transform;
mod tray;
mod update;

use crate::config::{ConfigState, StartupNotice};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::MacosLauncher;

pub const FLOAT_BAR_LABEL: &str = "float-bar";
pub const SETTINGS_LABEL: &str = "settings";

/// 悬浮条逻辑显隐状态（仅 Windows 使用）：
/// 隐藏时不再调用 `hide()`（长时间隐藏会导致 WebView2 页面进程被系统回收，
/// 再次呼出后界面仍在但点击无响应），改为把窗口移出屏幕并开启点击穿透，
/// 保持 WebView2 存活；呼出时恢复原位。
pub struct FloatBarState {
    pub logical_visible: Mutex<bool>,
    pub saved_position: Mutex<Option<tauri::PhysicalPosition<i32>>>,
}

impl Default for FloatBarState {
    fn default() -> Self {
        Self {
            logical_visible: Mutex::new(true),
            saved_position: Mutex::new(None),
        }
    }
}

/// 呼出悬浮条：显示并取消最小化（不主动抢焦点）。
pub fn show_float_bar(app: &AppHandle) {
    let Some(win) = app.get_webview_window(FLOAT_BAR_LABEL) else {
        return;
    };

    #[cfg(target_os = "windows")]
    {
        // 恢复隐藏前的位置，关闭点击穿透，并重新确保不抢焦点
        if let Some(state) = app.try_state::<FloatBarState>() {
            if let Ok(position) = state.saved_position.lock() {
                if let Some(position) = position.as_ref() {
                    let _ = win.set_position(tauri::Position::Physical(*position));
                }
            }
            if let Ok(mut visible) = state.logical_visible.lock() {
                *visible = true;
            }
        }
        let _ = win.unminimize();
        let _ = win.set_ignore_cursor_events(false);
        let _ = apply_no_activate(&win);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = win.show();
        let _ = win.unminimize();
    }
}

/// 隐藏悬浮条，托盘图标保持常驻。
pub fn hide_float_bar(app: &AppHandle) {
    let Some(win) = app.get_webview_window(FLOAT_BAR_LABEL) else {
        return;
    };

    #[cfg(target_os = "windows")]
    {
        if let Ok(mut position) = app.state::<FloatBarState>().saved_position.lock() {
            *position = win.outer_position().ok();
        }
        if let Ok(mut visible) = app.state::<FloatBarState>().logical_visible.lock() {
            *visible = false;
        }
        // 移出屏幕并开启点击穿透，窗口保持“可见”以维持 WebView2 存活
        let off_screen = tauri::Position::Physical(tauri::PhysicalPosition::new(-32000, -32000));
        let _ = win.set_position(off_screen);
        let _ = win.set_ignore_cursor_events(true);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = win.hide();
    }
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

    #[cfg(target_os = "windows")]
    {
        // Windows 下窗口始终“可见”，用逻辑状态判断显隐
        let _ = win;
        let hidden = app
            .state::<FloatBarState>()
            .logical_visible
            .lock()
            .map(|visible| !*visible)
            .unwrap_or(false);
        if hidden {
            show_float_bar(app);
        } else {
            hide_float_bar(app);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
        } else {
            show_float_bar(app);
        }
    }
}

/// 用系统默认浏览器打开 URL（下载更新入口跳转 GitHub Releases 页面）。
pub fn open_in_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows::core::PCWSTR;
        use windows::Win32::UI::Shell::ShellExecuteW;
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let operation: Vec<u16> = "open".encode_utf16().chain(std::iter::once(0)).collect();
        let url_wide: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();
        let result = unsafe {
            ShellExecuteW(
                None,
                PCWSTR(operation.as_ptr()),
                PCWSTR(url_wide.as_ptr()),
                PCWSTR(std::ptr::null()),
                PCWSTR(std::ptr::null()),
                SW_SHOWNORMAL,
            )
        };
        // ShellExecuteW 返回值大于 32 表示成功
        if result.0 as isize <= 32 {
            return Err(format!("打开浏览器失败：错误码 {}", result.0 as isize));
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("打开浏览器失败：{e}"))?;
        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = url;
        Err("当前平台不支持打开浏览器".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("[StringCraft] 启动中…");
    let config = config::load_or_default();
    let startup_hotkey = config.hotkey.clone();
    tauri::Builder::default()
        .manage(ConfigState(Mutex::new(config)))
        .manage(StartupNotice(Mutex::new(None)))
        .manage(FloatBarState::default())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 第二次启动（如双击快捷方式）只呼出悬浮条，不创建新实例
            show_float_bar(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            println!("[StringCraft] 配置已加载，快捷键：{startup_hotkey}");

            hotkey::install(app.handle())?;
            match hotkey::register(app.handle(), &startup_hotkey) {
                Ok(()) => {
                    println!("[StringCraft] 全局快捷键注册成功：{startup_hotkey}");
                    logging::log_event(
                        app.handle(),
                        &format!("全局快捷键注册成功：{startup_hotkey}"),
                    );
                }
                Err(e) => {
                    eprintln!("[StringCraft] 全局快捷键注册失败：{e}");
                    logging::log_event(app.handle(), &format!("全局快捷键注册失败：{e}"));
                    if let Some(notice) = app.try_state::<StartupNotice>() {
                        if let Ok(mut guard) = notice.0.lock() {
                            *guard = Some(format!("全局快捷键注册失败（可能被其他程序占用）：{e}"));
                        }
                    }
                }
            }

            // 设置窗口点击系统关闭按钮时隐藏而不是销毁，保证下次仍能打开。
            if let Some(settings) = app.get_webview_window(SETTINGS_LABEL) {
                let settings_win = settings.clone();
                settings.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = settings_win.hide();
                    }
                });
            }

            tray::create_tray(app.handle())?;
            logging::log_event(app.handle(), "应用启动完成");
            update::start_update_checker(app.handle().clone());
            println!("[StringCraft] 托盘已创建");
            println!("[StringCraft] 启动完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::take_startup_notice,
            commands::save_config,
            commands::apply_no_activate,
            commands::update_float_bar_position,
            commands::update_float_bar_width,
            commands::export_config_to,
            commands::import_config_from,
            commands::check_for_update,
            commands::download_update,
            commands::install_update,
            commands::is_macos,
            commands::macos_accessibility_trusted,
            commands::open_macos_accessibility_settings,
            commands::open_settings,
            commands::hide_float_bar,
            commands::open_in_browser,
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
