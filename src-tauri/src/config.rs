use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    pub buttons: Vec<TransformButton>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
        button("underscore-to-hyphen", "下划线→中横线", "underscore-to-hyphen"),
        button("hyphen-to-underscore", "中横线→下划线", "hyphen-to-underscore"),
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
