use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App,
};

pub fn create_tray(app: &App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "呼出悬浮条", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "隐藏悬浮条", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "打开设置", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
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
