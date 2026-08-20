use crate::config::{self, AppConfig, ConfigState, StartupNotice, WindowPosition};
use crate::{hotkey, logging, selection, transform, update, FLOAT_BAR_LABEL};
use std::fs;
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

/// 读取并清除启动时的一次性提示。
#[tauri::command]
pub fn take_startup_notice(app: AppHandle) -> Option<String> {
    let state = app.state::<StartupNotice>();
    state.0.lock().ok().and_then(|mut guard| guard.take())
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

    if let Err(e) = config::save_config(&config) {
        rollback(&app, &old, &config);
        return Err(format!("保存配置失败：{e}"));
    }

    *guard = config.clone();
    drop(guard);
    if old.language != config.language {
        let _ = crate::tray::rebuild_tray(&app);
    }
    logging::log_event(&app, "配置已保存");
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
    custom_type: Option<String>,
    param1: Option<String>,
    param2: Option<String>,
) -> Result<String, String> {
    logging::log_event(&app, &format!("执行转换：{transform_id}"));
    if let Some(text) = text {
        return Ok(match custom_type.as_deref() {
            Some(custom_type) => transform::transform_custom(
                &text,
                custom_type,
                param1.as_deref(),
                param2.as_deref(),
            ),
            None => transform::transform(&text, &transform_id),
        });
    }
    tauri::async_runtime::spawn_blocking(move || {
        selection::replace_selection(&app, &transform_id, custom_type, param1, param2)
    })
    .await
    .map_err(|e| format!("任务执行失败：{e}"))?
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), String> {
    crate::open_settings(&app);
    Ok(())
}

/// 隐藏悬浮条（Windows 下由后端移出屏幕保持 WebView2 存活）。
#[tauri::command]
pub fn hide_float_bar(app: AppHandle) -> Result<(), String> {
    crate::hide_float_bar(&app);
    Ok(())
}

/// 用系统默认浏览器打开 URL（下载更新跳转 GitHub Releases 发布页面）。
#[tauri::command]
pub fn open_in_browser(url: String) -> Result<(), String> {
    crate::open_in_browser(&url)
}

/// 悬浮条加载完成后调用：设置不抢焦点（位置记忆由前端负责）。
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

/// 悬浮条拖动结束后保存位置，不触发 config-changed 以避免位置回跳。
#[tauri::command]
pub fn update_float_bar_position(app: AppHandle, x: i32, y: i32) -> Result<(), String> {
    let next = {
        let state = app.state::<ConfigState>();
        let guard = state.0.lock().map_err(|e| e.to_string())?;
        let mut next = guard.clone();
        next.position = Some(WindowPosition { x, y });
        next
    };
    config::save_config(&next)?;
    {
        let state = app.state::<ConfigState>();
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;
        *guard = next;
    }
    logging::log_event(&app, "悬浮条位置已保存");
    Ok(())
}

/// 悬浮条边缘拖动结束后保存宽度，不触发 config-changed 以避免布局回跳。
#[tauri::command]
pub fn update_float_bar_width(app: AppHandle, width: u32) -> Result<(), String> {
    let mut next = {
        let state = app.state::<ConfigState>();
        let guard = state.0.lock().map_err(|e| e.to_string())?;
        let mut next = guard.clone();
        next.toolbar_width = width;
        next
    };
    config::normalize(&mut next);
    config::save_config(&next)?;
    {
        let state = app.state::<ConfigState>();
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;
        *guard = next;
    }
    logging::log_event(&app, "悬浮条宽度已保存");
    Ok(())
}

/// 导出当前配置到指定路径（路径由前端文件对话框选择）。
#[tauri::command]
pub fn export_config_to(app: AppHandle, path: String) -> Result<String, String> {
    let state = app.state::<ConfigState>();
    let config = state.0.lock().map_err(|e| e.to_string())?.clone();
    let json = serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败：{e}"))?;
    fs::write(&path, json).map_err(|e| format!("写入配置文件失败：{e}"))?;
    logging::log_event(&app, "配置已导出");
    Ok("配置已导出".to_string())
}

/// 从指定路径导入配置，校验后覆盖当前配置并持久化。
#[tauri::command]
pub fn import_config_from(app: AppHandle, path: String) -> Result<String, String> {
    let content = fs::read_to_string(&path).map_err(|e| format!("读取配置文件失败：{e}"))?;
    let mut config: AppConfig =
        serde_json::from_str(&content).map_err(|e| format!("配置格式不正确：{e}"))?;
    config::normalize(&mut config);
    config::validate(&config)?;

    let state = app.state::<ConfigState>();
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    let old = guard.clone();
    apply_hotkey_change(&app, &old, &config)?;
    apply_autostart_change(&app, &old, &config)?;

    if let Err(e) = config::save_config(&config) {
        rollback(&app, &old, &config);
        return Err(format!("保存配置失败：{e}"));
    }
    *guard = config;
    drop(guard);
    logging::log_event(&app, "配置已导入");
    let _ = app.emit_to(FLOAT_BAR_LABEL, "config-changed", ());
    Ok("配置已导入并生效".to_string())
}

/// 手动检查更新。
#[tauri::command]
pub fn check_for_update() -> Result<update::UpdateInfo, String> {
    update::check_for_update()
}

/// 下载更新安装包，返回本地路径。
#[tauri::command]
pub fn download_update(asset_url: String) -> Result<String, String> {
    update::download_update(&asset_url)
}

/// 启动已下载的安装程序。
#[tauri::command]
pub fn install_update(path: String) -> Result<String, String> {
    update::launch_installer(&path)
}

/// 当前是否为 macOS 平台。
#[tauri::command]
pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

/// 查询 macOS 辅助功能权限状态。
#[tauri::command]
pub fn macos_accessibility_trusted() -> bool {
    crate::macos::accessibility_trusted()
}

/// 打开 macOS 辅助功能权限设置页。
#[tauri::command]
pub fn open_macos_accessibility_settings() -> Result<(), String> {
    crate::macos::open_accessibility_settings()
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
