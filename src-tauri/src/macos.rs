//! macOS 适配（M8）：辅助功能权限检测与引导。
//! Windows 下编译为桩实现，保证跨平台构建不受影响。

/// 是否已获得“辅助功能”权限。
#[cfg(target_os = "macos")]
pub fn accessibility_trusted() -> bool {
    unsafe { ffi::AXIsProcessTrusted() }
}

/// Windows 等其他平台视为已授权，便于统一调用。
#[cfg(not(target_os = "macos"))]
pub fn accessibility_trusted() -> bool {
    true
}

/// 一键打开 macOS“辅助功能”权限设置页。
#[cfg(target_os = "macos")]
pub fn open_accessibility_settings() -> Result<(), String> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .map_err(|e| format!("打开系统设置失败：{e}"))?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn open_accessibility_settings() -> Result<(), String> {
    Err("仅 macOS 支持该操作".to_string())
}

#[cfg(target_os = "macos")]
mod ffi {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        pub fn AXIsProcessTrusted() -> bool;
    }
}
