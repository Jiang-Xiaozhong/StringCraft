use tauri::AppHandle;
use tauri_plugin_global_shortcut::{Builder, ShortcutState};

/// 默认快捷键：Windows 为 Ctrl+Alt+Space；macOS 统一用 Cmd 表示 Ctrl。
pub fn default_hotkey() -> &'static str {
    if cfg!(target_os = "macos") {
        "Cmd+Alt+Space"
    } else {
        "Ctrl+Alt+Space"
    }
}

pub fn register_default(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    app.plugin(
        Builder::new()
            .with_shortcuts([default_hotkey()])?
            .with_handler(|app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    crate::toggle_float_bar(app);
                }
            })
            .build(),
    )
}
