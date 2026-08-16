//! 自动替换链路（需求 3.1 / 4.6）：保存剪贴板 → 模拟复制 → 读取 → 转换 →
//! 写入剪贴板 → 模拟粘贴 → 恢复剪贴板。
//!
//! 隐私：转换文本仅在内存中处理，不写入日志、配置或磁盘。

use crate::{config::ConfigState, transform};
use enigo::{Key, Keyboard};
use std::time::Duration;
use tauri::{AppHandle, Manager};

/// 选中文本过大阈值（字节），超过后不自动替换，仅复制结果到剪贴板。
const MAX_SELECTION_BYTES: usize = 1024 * 1024;

/// 执行完整替换流程，返回面向用户的中文提示。
pub fn replace_selection(
    app: &AppHandle,
    transform_id: &str,
    custom_type: Option<String>,
    param1: Option<String>,
    param2: Option<String>,
) -> Result<String, String> {
    let (restore_enabled, delay_ms) = {
        let state = app.state::<ConfigState>();
        let config = state.0.lock().map_err(|e| e.to_string())?;
        (config.restore_clipboard, config.replace_delay_ms as u64)
    };

    // macOS：未授权辅助功能时无法模拟按键，先给出明确引导。
    if !crate::macos::accessibility_trusted() {
        return Err("需要开启“辅助功能”权限，请在设置页开启".to_string());
    }

    // 1. 保存当前剪贴板文本；非文本内容直接放弃，避免破坏剪贴板
    let mut clipboard = arboard::Clipboard::new().map_err(|e| format!("无法访问剪贴板：{e}"))?;
    let backup = match clipboard.get_text() {
        Ok(text) => text,
        Err(_) => return Err("剪贴板包含图片/文件等非文本内容，已放弃自动替换".to_string()),
    };

    // 2. 模拟 Ctrl+C 复制选中文本
    send_shortcut(copy_key(), letter_key('c'))?;
    std::thread::sleep(Duration::from_millis(delay_ms));

    // 3. 读取复制结果；与备份一致判定为未选中文本
    let selected = match clipboard.get_text() {
        Ok(text) => text,
        Err(_) => {
            let _ = restore_clipboard(&mut clipboard, backup.as_str());
            return Err("复制到的内容不是文本，已放弃自动替换".to_string());
        }
    };
    if selected == backup {
        let _ = restore_clipboard(&mut clipboard, backup.as_str());
        return Err("未检测到选中文本".to_string());
    }

    // 4. 转换（自定义按钮走自定义调度）
    let result = match custom_type.as_deref() {
        Some(custom_type) => transform::transform_custom(
            &selected,
            custom_type,
            param1.as_deref(),
            param2.as_deref(),
        ),
        None => transform::transform(&selected, transform_id),
    };

    // 选中文本过大：不自动替换，仅将结果放入剪贴板并提示
    if selected.len() > MAX_SELECTION_BYTES {
        clipboard
            .set_text(result)
            .map_err(|e| format!("写入剪贴板失败：{e}"))?;
        return Ok("文本过大，已复制到剪贴板，请手动粘贴".to_string());
    }

    // 5. 写入转换结果并模拟 Ctrl+V 粘贴
    clipboard
        .set_text(&result)
        .map_err(|e| format!("写入剪贴板失败：{e}"))?;
    if let Err(e) = send_shortcut(copy_key(), letter_key('v')) {
        let restore_note = match restore_clipboard(&mut clipboard, backup.as_str()) {
            Ok(()) => "原剪贴板已恢复".to_string(),
            Err(_) => "且原剪贴板恢复失败，结果保留在剪贴板".to_string(),
        };
        return Err(format!("粘贴失败：{e}，{restore_note}"));
    }
    std::thread::sleep(Duration::from_millis(delay_ms));

    // 6. 恢复原剪贴板（可关闭，默认开启）
    if restore_enabled {
        if let Err(e) = restore_clipboard(&mut clipboard, backup.as_str()) {
            return Ok(format!(
                "转换完成，但原剪贴板恢复失败：{e}，结果保留在剪贴板"
            ));
        }
    }

    Ok("转换完成".to_string())
}

/// 恢复剪贴板文本。
fn restore_clipboard(clipboard: &mut arboard::Clipboard, backup: &str) -> Result<(), String> {
    clipboard.set_text(backup).map_err(|e| e.to_string())
}

/// 复制/粘贴使用的修饰键：Windows 为 Ctrl，macOS 为 Cmd（M5 接入）。
#[cfg(target_os = "macos")]
fn copy_key() -> Key {
    Key::Meta
}

#[cfg(not(target_os = "macos"))]
fn copy_key() -> Key {
    Key::Control
}

/// 字母按键：macOS 的 `enigo::Key` 没有 `C`/`V` 变体，统一用 `Unicode` 表示。
#[cfg(target_os = "macos")]
fn letter_key(letter: char) -> Key {
    Key::Unicode(letter)
}

#[cfg(not(target_os = "macos"))]
fn letter_key(letter: char) -> Key {
    match letter {
        'c' => Key::C,
        'v' => Key::V,
        _ => Key::Unicode(letter),
    }
}

/// 模拟一次「修饰键 + 字母」快捷键。
fn send_shortcut(modifier: Key, letter: Key) -> Result<(), String> {
    let mut enigo = enigo::Enigo::new(&enigo::Settings::default())
        .map_err(|e| format!("键盘模拟初始化失败：{e}"))?;
    enigo
        .key(modifier, enigo::Direction::Press)
        .map_err(|e| e.to_string())?;
    enigo
        .key(letter, enigo::Direction::Click)
        .map_err(|e| e.to_string())?;
    enigo
        .key(modifier, enigo::Direction::Release)
        .map_err(|e| e.to_string())?;
    Ok(())
}
