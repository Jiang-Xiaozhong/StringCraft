use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::AppHandle;
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
use tauri::Manager;

/// 全局配置状态，所有命令共享。
pub struct ConfigState(pub Mutex<AppConfig>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub version: u32,
    pub hotkey: String,
    pub rows: u32,
    pub button_width: u32,
    pub button_height: u32,
    pub font_size: u32,
    pub opacity: u32,
    pub theme: String,
    pub auto_start: bool,
    pub restore_clipboard: bool,
    pub replace_delay_ms: u32,
    pub buttons: Vec<TransformButton>,
}

impl Default for AppConfig {
    fn default() -> Self {
        default_config()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TransformButton {
    pub id: String,
    pub label: String,
    pub transform: String,
}

fn button(id: &str, label: &str, transform: &str) -> TransformButton {
    TransformButton {
        id: id.to_string(),
        label: label.to_string(),
        transform: transform.to_string(),
    }
}

pub fn default_config() -> AppConfig {
    AppConfig {
        version: 1,
        hotkey: crate::hotkey::default_hotkey().to_string(),
        rows: 2,
        button_width: 72,
        button_height: 32,
        font_size: 13,
        opacity: 100,
        theme: "system".to_string(),
        auto_start: false,
        restore_clipboard: true,
        replace_delay_ms: 80,
        buttons: default_buttons(),
    }
}

/// 20 个内置转换按钮：顺序固定，id 与显示文字解耦（对应需求 4.3）。
pub fn default_buttons() -> Vec<TransformButton> {
    vec![
        button("upper", "全大写", "upper"),
        button("lower", "全小写", "lower"),
        button("capitalize-words", "首字母大写", "capitalize-words"),
        button("uncapitalize-words", "首字母小写", "uncapitalize-words"),
        button("sentence-case", "句子首字母大写", "sentence-case"),
        button("space-to-underscore", "空格→下划线", "space-to-underscore"),
        button("to-camel", "下划线&空格→驼峰", "to-camel"),
        button("camel-to-underscore", "驼峰→下划线", "camel-to-underscore"),
        button("camel-to-space", "驼峰→空格", "camel-to-space"),
        button("space-to-hyphen", "空格→中横线", "space-to-hyphen"),
        button(
            "underscore-to-hyphen",
            "下划线→中横线",
            "underscore-to-hyphen",
        ),
        button(
            "hyphen-to-underscore",
            "中横线→下划线",
            "hyphen-to-underscore",
        ),
        button("underscore-to-space", "下划线→空格", "underscore-to-space"),
        button("underscore-to-dot", "下划线→小数点", "underscore-to-dot"),
        button("dot-to-underscore", "小数点→下划线", "dot-to-underscore"),
        button("space-to-newline", "空格→换行", "space-to-newline"),
        button("newline-to-space", "换行→空格", "newline-to-space"),
        button("remove-symbols", "清除符号", "remove-symbols"),
        button("remove-spaces", "清除空格", "remove-spaces"),
        button("remove-newlines", "清除换行", "remove-newlines"),
    ]
}

/// 配置文件的绝对路径：
/// Windows `%APPDATA%\StringCraft\config.json`；
/// macOS `~/Library/Application Support/StringCraft/config.json`（需求 6.3）。
pub fn config_path(_app: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "无法定位 APPDATA 目录".to_string())?;

    #[cfg(target_os = "macos")]
    let base = dirs::home_dir()
        .map(|home| home.join("Library").join("Application Support"))
        .ok_or_else(|| "无法定位用户主目录".to_string())?;

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = _app
        .path()
        .app_config_dir()
        .map_err(|e| format!("无法定位配置目录：{e}"))?;

    Ok(base.join("StringCraft").join("config.json"))
}

/// 加载配置；文件不存在时创建默认配置，损坏时备份并恢复默认。
pub fn load_or_default(app: &AppHandle) -> AppConfig {
    let path = match config_path(app) {
        Ok(path) => path,
        Err(_) => return default_config(),
    };

    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
            Ok(mut config) => {
                normalize(&mut config);
                config
            }
            Err(_) => {
                backup_corrupted(&path);
                let config = default_config();
                let _ = save_to_path(&config, &path);
                config
            }
        },
        Err(_) => {
            let config = default_config();
            let _ = save_to_path(&config, &path);
            config
        }
    }
}

/// 持久化配置（先写临时文件再重命名，避免写一半损坏）。
pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_path(app)?;
    save_to_path(config, &path)
}

fn save_to_path(config: &AppConfig, path: &Path) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("创建配置目录失败：{e}"))?;
    }
    let json = serde_json::to_string_pretty(config).map_err(|e| format!("序列化配置失败：{e}"))?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, json).map_err(|e| format!("写入配置失败：{e}"))?;
    fs::rename(&tmp, path).map_err(|e| format!("保存配置失败：{e}"))?;
    Ok(())
}

/// 数值取边界、非法枚举回退默认。
pub fn normalize(config: &mut AppConfig) {
    config.rows = config.rows.clamp(1, 3);
    config.button_width = config.button_width.clamp(40, 200);
    config.button_height = config.button_height.clamp(28, 80);
    config.font_size = config.font_size.clamp(10, 24);
    config.opacity = config.opacity.clamp(0, 100);
    config.replace_delay_ms = config.replace_delay_ms.clamp(20, 1000);
    if !matches!(config.theme.as_str(), "system" | "light" | "dark") {
        config.theme = "system".to_string();
    }
}

/// 逻辑校验：行数/按钮数量/按钮内容/快捷键格式。
pub fn validate(config: &AppConfig) -> Result<(), String> {
    if !(1..=3).contains(&config.rows) {
        return Err("行数需在 1~3 之间".to_string());
    }
    if config.buttons.len() > 90 {
        return Err("按钮总数不能超过 90 个".to_string());
    }
    if config.buttons.len() > (config.rows as usize) * 30 {
        return Err("每行最多 30 个按钮".to_string());
    }
    for item in &config.buttons {
        if item.label.trim().is_empty() {
            return Err("按钮文字不能为空".to_string());
        }
        if !crate::transform::is_known_transform(&item.transform) {
            return Err(format!("未知转换功能：{}", item.transform));
        }
    }
    if !crate::hotkey::is_valid_hotkey(&config.hotkey) {
        return Err("快捷键需包含至少一个修饰键（Ctrl/Alt/Shift/Cmd）".to_string());
    }
    Ok(())
}

/// 配置文件损坏时重命名备份，保留现场供排查（不含任何用户文本）。
fn backup_corrupted(path: &Path) {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup = path.with_file_name(format!("config.json.corrupted-{stamp}"));
    let _ = fs::rename(path, backup);
}
