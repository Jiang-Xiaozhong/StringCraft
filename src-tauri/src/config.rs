use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

/// 全局配置状态，所有命令共享。
pub struct ConfigState(pub Mutex<AppConfig>);

const DEFAULT_BACKGROUND_COLOR: &str = "#DCEBFA";
const DEFAULT_BACKGROUND_COLOR_DARK: &str = "#27384A";
const TOOLBAR_CHROME_WIDTH: u32 = 56;
const BUTTON_GAP: u32 = 4;
const ACTION_GAP: u32 = 4;
const BODY_GAP: u32 = 6;
const BAR_BORDER: u32 = 2;
const BAR_PADDING_TOTAL: u32 = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub version: u32,
    pub hotkey: String,
    pub toolbar_width: u32,
    pub button_width: u32,
    pub button_height: u32,
    pub font_size: u32,
    pub opacity: u32,
    pub theme: String,
    pub background_color: String,
    pub background_color_dark: String,
    pub position: Option<WindowPosition>,
    pub auto_start: bool,
    pub restore_clipboard: bool,
    pub replace_delay_ms: u32,
    pub debug_log: bool,
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
    #[serde(alias = "label")]
    pub name: String,
    pub transform: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_true")]
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

fn button(id: &str, name: &str, transform: &str, description: &str) -> TransformButton {
    TransformButton {
        id: id.to_string(),
        name: name.to_string(),
        transform: transform.to_string(),
        description: description.to_string(),
        visible: true,
    }
}

fn default_true() -> bool {
    true
}

pub fn default_config() -> AppConfig {
    AppConfig {
        version: 1,
        hotkey: crate::hotkey::default_hotkey().to_string(),
        toolbar_width: default_toolbar_width(),
        button_width: 72,
        button_height: 32,
        font_size: 13,
        opacity: 100,
        theme: "system".to_string(),
        background_color: DEFAULT_BACKGROUND_COLOR.to_string(),
        background_color_dark: DEFAULT_BACKGROUND_COLOR_DARK.to_string(),
        position: None,
        auto_start: false,
        restore_clipboard: true,
        replace_delay_ms: 80,
        debug_log: false,
        buttons: default_buttons(),
    }
}

/// 默认工具条宽度：恰好容纳 10 个默认宽度按钮（72px）。
fn default_toolbar_width() -> u32 {
    TOOLBAR_CHROME_WIDTH + 10 * 72 + 9 * BUTTON_GAP
}

/// 工具条最小宽度：至少容纳 1 个按钮和设置按钮。
fn min_toolbar_width(button_width: u32, button_height: u32) -> u32 {
    // 设置/隐藏按钮为正方形，横排时操作区宽度 = 2 * 按钮高度 + 间距。
    let action_width_horizontal = 2 * button_height + ACTION_GAP;
    BAR_PADDING_TOTAL + action_width_horizontal + BODY_GAP + BAR_BORDER + button_width
}

/// 20 个内置转换按钮：顺序固定，id 与显示文字解耦（对应需求 4.3）。
pub fn default_buttons() -> Vec<TransformButton> {
    vec![
        button("upper", "AB", "upper", "所有字母转为大写"),
        button("lower", "ab", "lower", "所有字母转为小写"),
        button(
            "capitalize-words",
            "Ab",
            "capitalize-words",
            "每个单词首字母大写，其余小写",
        ),
        button(
            "uncapitalize-words",
            "aB",
            "uncapitalize-words",
            "每个单词首字母小写",
        ),
        button("sentence-case", "Aa", "sentence-case", "每个句子首字母大写"),
        button(
            "space-to-underscore",
            "s_",
            "space-to-underscore",
            "空格替换为下划线",
        ),
        button(
            "to-camel",
            "camel",
            "to-camel",
            "下划线或空格分词并转为驼峰",
        ),
        button(
            "camel-to-underscore",
            "c_",
            "camel-to-underscore",
            "驼峰分词并以下划线连接",
        ),
        button(
            "camel-to-space",
            "c sp",
            "camel-to-space",
            "驼峰分词并以空格连接",
        ),
        button(
            "space-to-hyphen",
            "s-",
            "space-to-hyphen",
            "空格替换为中横线",
        ),
        button(
            "underscore-to-hyphen",
            "_-",
            "underscore-to-hyphen",
            "下划线替换为中横线",
        ),
        button(
            "hyphen-to-underscore",
            "-_",
            "hyphen-to-underscore",
            "中横线替换为下划线",
        ),
        button(
            "underscore-to-space",
            "_s",
            "underscore-to-space",
            "下划线替换为空格",
        ),
        button(
            "underscore-to-dot",
            "_.",
            "underscore-to-dot",
            "下划线替换为小数点",
        ),
        button(
            "dot-to-underscore",
            "._",
            "dot-to-underscore",
            "小数点替换为下划线",
        ),
        button(
            "space-to-newline",
            "s↵",
            "space-to-newline",
            "空格替换为换行",
        ),
        button(
            "newline-to-space",
            "↵s",
            "newline-to-space",
            "换行替换为空格",
        ),
        button(
            "remove-symbols",
            "NoSym",
            "remove-symbols",
            "删除除 Unicode 字母、数字、空白外的所有字符",
        ),
        button("remove-spaces", "NoSp", "remove-spaces", "删除所有空格字符"),
        button(
            "remove-newlines",
            "NoNl",
            "remove-newlines",
            "删除所有换行符",
        ),
    ]
}

/// 配置文件的绝对路径：
/// Windows `%APPDATA%\StringCraft\config.json`；
/// macOS `~/Library/Application Support/StringCraft/config.json`（需求 6.3）。
pub fn config_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| "无法定位 APPDATA 目录".to_string())?;

    #[cfg(target_os = "macos")]
    let base = dirs::home_dir()
        .map(|home| home.join("Library").join("Application Support"))
        .ok_or_else(|| "无法定位用户主目录".to_string())?;

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = dirs::config_dir().ok_or_else(|| "无法定位配置目录".to_string())?;

    Ok(base.join("StringCraft").join("config.json"))
}

/// 加载配置；文件不存在时创建默认配置，损坏时备份并恢复默认。
pub fn load_or_default() -> AppConfig {
    let path = match config_path() {
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
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path()?;
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
    config.button_width = config.button_width.clamp(40, 200);
    config.button_height = config.button_height.clamp(28, 80);
    config.font_size = config.font_size.clamp(10, 24);
    config.toolbar_width = config.toolbar_width.clamp(
        min_toolbar_width(config.button_width, config.button_height),
        4000,
    );
    config.opacity = config.opacity.clamp(20, 100);
    config.replace_delay_ms = config.replace_delay_ms.clamp(20, 1000);
    if !matches!(config.theme.as_str(), "system" | "light" | "dark") {
        config.theme = "system".to_string();
    }
    normalize_color(&mut config.background_color, DEFAULT_BACKGROUND_COLOR);
    normalize_color(
        &mut config.background_color_dark,
        DEFAULT_BACKGROUND_COLOR_DARK,
    );
}

/// 逻辑校验：按钮内容/快捷键格式。
pub fn validate(config: &AppConfig) -> Result<(), String> {
    if !(20..=100).contains(&config.opacity) {
        return Err("背景不透明度需在 20~100 之间".to_string());
    }
    if !is_hex_color(&config.background_color) {
        return Err("浅色背景颜色格式无效".to_string());
    }
    if !is_hex_color(&config.background_color_dark) {
        return Err("深色背景颜色格式无效".to_string());
    }
    for item in &config.buttons {
        if item.name.trim().is_empty() {
            return Err("按钮名称不能为空".to_string());
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

fn is_hex_color(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(|b| b.is_ascii_hexdigit())
}

fn normalize_color(value: &mut String, fallback: &str) {
    if !is_hex_color(value) {
        *value = fallback.to_string();
    }
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

#[cfg(test)]
mod tests {
    use super::TransformButton;

    #[test]
    fn migrates_legacy_label_to_name() {
        let json = r#"{"id":"upper","label":"全大写","transform":"upper"}"#;
        let button: TransformButton = serde_json::from_str(json).unwrap();
        assert_eq!(button.name, "全大写");
        assert_eq!(button.description, "");
    }

    #[test]
    fn accepts_name_and_description() {
        let json = r#"{"id":"upper","name":"全大写","transform":"upper","description":"所有字母转为大写"}"#;
        let button: TransformButton = serde_json::from_str(json).unwrap();
        assert_eq!(button.name, "全大写");
        assert_eq!(button.description, "所有字母转为大写");
    }
}
