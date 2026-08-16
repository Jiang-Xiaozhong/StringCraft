use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Manager,
};

pub fn create_tray(app: &App) -> tauri::Result<()> {
    let language = app
        .state::<crate::config::ConfigState>()
        .0
        .lock()
        .map(|config| config.language.clone())
        .unwrap_or_else(|_| "zh-CN".to_string());
    let (show_text, hide_text, settings_text, quit_text) = if language == "en-US" {
        ("Show Float Bar", "Hide Float Bar", "Settings", "Quit")
    } else {
        ("呼出悬浮条", "隐藏悬浮条", "打开设置", "退出")
    };

    let show = MenuItem::with_id(app, "show", show_text, true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", hide_text, true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", settings_text, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", quit_text, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &settings, &quit])?;

    let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;

    let tray_builder = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .tooltip("StringCraft")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => crate::show_float_bar(app),
            "hide" => crate::hide_float_bar(app),
            "settings" => crate::open_settings(app),
            "quit" => app.exit(0),
            _ => {}
        });

    // Windows：左键单击呼出悬浮条，右键弹出菜单；macOS：点击弹出菜单。
    #[cfg(target_os = "macos")]
    let tray_builder = tray_builder.show_menu_on_left_click(true);
    #[cfg(not(target_os = "macos"))]
    let tray_builder = tray_builder.show_menu_on_left_click(false);

    tray_builder
        .on_tray_icon_event(|tray, event| {
            #[cfg(target_os = "windows")]
            {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    crate::show_float_bar(tray.app_handle());
                }
            }
            #[cfg(not(target_os = "windows"))]
            let _ = (tray, event);
        })
        .build(app)?;

    Ok(())
}
