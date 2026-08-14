#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::io::Write;

fn main() {
    install_panic_hook();
    stringcraft_lib::run()
}

/// 把 panic 信息（含调用栈）写入临时目录，方便无控制台的 GUI 版排查崩溃。
fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        let message = format!("panic: {info}\n{backtrace}");

        let path = std::env::temp_dir().join("stringcraft-panic.log");
        if let Ok(mut file) = std::fs::File::create(&path) {
            let _ = file.write_all(message.as_bytes());
        }
        eprintln!("{message}");
        default_hook(info);
    }));
}
