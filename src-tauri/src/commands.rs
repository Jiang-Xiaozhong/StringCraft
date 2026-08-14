use crate::config::{self, AppConfig, ConfigState};
use crate::{hotkey, selection, transform, FLOAT_BAR_LABEL};
use tauri::{AppHandle, Emitter, Manager};

#[tauri::command]
pub fn get_config(app: AppHandle) -> Result<AppConfig, String> {
    let state = app.state::<ConfigState>();
    state
        .0
        .lock()
        .map_err(|e| e.to_string())
        .map(|guard| guard.clone())
}

/// 保存配置：校验 → 应用快捷键/自启动变更 → 持久化 → 通知悬浮条即时刷新。
#[tauri::command]
pub fn save_config(app: AppHandle, mut config: AppConfig) -> Result<AppConfig, String> {
    config::normalize(&mut config);
    config::validate(&config)?;

    let state = app.state::<ConfigState>();
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    let old = guard.clone();

    apply_hotkey_change(&app, &old, &config)?;
    apply_autostart_change(&app, &old, &config)?;

    if let Err(e) = config::save_config(&app, &config) {
        rollback(&app, &old, &config);
        return Err(format!("保存配置失败：{e}"));
    }

    *guard = config.clone();
    drop(guard);
    let _ = app.emit_to(FLOAT_BAR_LABEL, "config-changed", ());
    Ok(config)
}

/// 执行转换。传入文本时直接转换（调试/测试用）；
/// 未传文本时执行完整自动替换链路（复制 → 转换 → 粘贴 → 恢复剪贴板）。
#[tauri::command]
pub async fn execute_button(
    app: AppHandle,
    transform_id: String,
    text: Option<String>,
) -> Result<String, String> {
    if let Some(text) = text {
        return Ok(transform::transform(&text, &transform_id));
    }
    tauri::async_runtime::spawn_blocking(move || selection::replace_selection(&app, &transform_id))
        .await
        .map_err(|e| format!("任务执行失败：{e}"))?
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), String> {
    crate::open_settings(&app);
    Ok(())
}

/// 悬浮条加载完成后调用：设置不抢焦点，并定位到屏幕右上角。
#[tauri::command]
pub fn apply_no_activate(app: AppHandle) -> Result<(), String> {
    let Some(win) = app.get_webview_window(FLOAT_BAR_LABEL) else {
        return Ok(());
    };
    #[cfg(target_os = "windows")]
    crate::apply_no_activate(&win).map_err(|e| e.to_string())?;
    #[cfg(not(target_os = "windows"))]
    let _ = &win;
    Ok(())
}

/// 快捷键变更：先注册新键，成功后再注销旧键；失败则回滚并报错。
fn apply_hotkey_change(app: &AppHandle, old: &AppConfig, new: &AppConfig) -> Result<(), String> {
    if old.hotkey == new.hotkey {
        return Ok(());
    }
    if let Err(e) = hotkey::register(app, &new.hotkey) {
        return Err(format!("快捷键注册失败（可能被其他程序占用）：{e}"));
    }
    let _ = hotkey::unregister(app, &old.hotkey);
    Ok(())
}

/// 开机自启变更；失败时回滚快捷键并报错。
fn apply_autostart_change(app: &AppHandle, old: &AppConfig, new: &AppConfig) -> Result<(), String> {
    if old.auto_start == new.auto_start {
        return Ok(());
    }
    if let Err(e) = set_autostart(app, new.auto_start) {
        rollback_hotkey(app, old, new);
        return Err(format!("开机自启设置失败：{e}"));
    }
    Ok(())
}

fn set_autostart(app: &AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    if enabled {
        manager.enable()
    } else {
        manager.disable()
    }
    .map_err(|e| e.to_string())
}

fn rollback(app: &AppHandle, old: &AppConfig, new: &AppConfig) {
    rollback_hotkey(app, old, new);
    if old.auto_start != new.auto_start {
        let _ = set_autostart(app, old.auto_start);
    }
}

fn rollback_hotkey(app: &AppHandle, old: &AppConfig, new: &AppConfig) {
    if old.hotkey != new.hotkey {
        let _ = hotkey::unregister(app, &new.hotkey);
        let _ = hotkey::register(app, &old.hotkey);
    }
}
