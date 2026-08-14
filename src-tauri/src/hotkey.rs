use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// 默认快捷键：配置统一用 Ctrl 表示主修饰键（macOS 注册时映射为 Cmd）。
pub fn default_hotkey() -> &'static str {
    "Ctrl+Alt+Space"
}

/// 安装全局快捷键插件，统一处理器负责呼入/呼出悬浮条。
pub fn install(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    Ok(app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    crate::toggle_float_bar(app);
                }
            })
            .build(),
    )?)
}

/// 注册快捷键；失败（如被其他程序占用）时返回错误信息。
pub fn register(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    let platform_hotkey = to_platform_hotkey(hotkey);
    app.global_shortcut()
        .register(platform_hotkey.as_str())
        .map_err(|e| format!("快捷键注册失败：{e}"))
}

/// 注销快捷键。
pub fn unregister(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    let platform_hotkey = to_platform_hotkey(hotkey);
    app.global_shortcut()
        .unregister(platform_hotkey.as_str())
        .map_err(|e| e.to_string())
}

/// 配置统一用 Ctrl 表示主修饰键；macOS 注册时映射为 Cmd（需求 4.4）。
fn to_platform_hotkey(hotkey: &str) -> String {
    #[cfg(target_os = "macos")]
    if let Some(rest) = hotkey.strip_prefix("Ctrl+") {
        return format!("Cmd+{rest}");
    }
    hotkey.to_string()
}

/// 快捷键格式校验：至少一个修饰键 + 一个普通按键。
pub fn is_valid_hotkey(hotkey: &str) -> bool {
    let parts: Vec<&str> = hotkey.split('+').map(str::trim).collect();
    if parts.len() < 2 {
        return false;
    }
    let is_modifier = |part: &str| {
        matches!(
            part.to_ascii_lowercase().as_str(),
            "ctrl" | "alt" | "shift" | "cmd" | "option" | "super" | "win"
        )
    };
    let has_modifier = parts.iter().any(|p| is_modifier(p));
    let has_key = parts.iter().any(|p| !is_modifier(p));
    has_modifier && has_key
}
