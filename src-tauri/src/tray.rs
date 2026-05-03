use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Manager, Runtime,
};

pub fn setup<R: Runtime>(app: &mut App<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let pause_all = MenuItem::with_id(app, "pause_all", "Pause All", true, None::<&str>)?;
    let resume_all = MenuItem::with_id(app, "resume_all", "Resume All", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show, &pause_all, &resume_all, &quit])?;

    TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Super Download Manager")
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("main") {
                    if win.is_visible().unwrap_or(false) {
                        let _ = win.hide();
                    } else {
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                }
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}
