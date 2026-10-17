use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, "quit", "Quit Laradoc Studio", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Open Dashboard", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;

    let start_all = MenuItem::with_id(app, "start_all", "▶  Start All Services", true, None::<&str>)?;
    let stop_all = MenuItem::with_id(app, "stop_all", "⏹  Stop All Services", true, None::<&str>)?;
    let separator2 = PredefinedMenuItem::separator(app)?;

    // PHP version submenu
    let php82 = MenuItem::with_id(app, "php_82", "PHP 8.2", true, None::<&str>)?;
    let php83 = MenuItem::with_id(app, "php_83", "PHP 8.3 (Default)", true, None::<&str>)?;
    let php84 = MenuItem::with_id(app, "php_84", "PHP 8.4", true, None::<&str>)?;
    let php85 = MenuItem::with_id(app, "php_85", "PHP 8.5 (Preview)", true, None::<&str>)?;
    let php_submenu = Submenu::with_id_and_items(
        app,
        "php_version",
        "Active PHP Version",
        true,
        &[&php82, &php83, &php84, &php85],
    )?;

    let separator3 = PredefinedMenuItem::separator(app)?;
    let open_www = MenuItem::with_id(app, "open_www", "📁  Open www/ Folder", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show,
            &separator,
            &start_all,
            &stop_all,
            &separator2,
            &php_submenu,
            &separator3,
            &open_www,
            &separator,
            &quit,
        ],
    )?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                log::info!("Quit requested from tray — stopping all services");
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    let state = app_clone.state::<crate::AppState>();
                    let mut mgr = state.service_manager.lock().await;
                    mgr.stop_all().await;
                });
                app.exit(0);
            }
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "start_all" => {
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    let state = app_clone.state::<crate::AppState>();
                    let mut mgr = state.service_manager.lock().await;
                    mgr.start_all(app_clone.clone()).await;
                });
            }
            "stop_all" => {
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    let state = app_clone.state::<crate::AppState>();
                    let mut mgr = state.service_manager.lock().await;
                    mgr.stop_all().await;
                });
            }
            "php_82" => set_active_php_tray(app.clone(), "8.2"),
            "php_83" => set_active_php_tray(app.clone(), "8.3"),
            "php_84" => set_active_php_tray(app.clone(), "8.4"),
            "php_85" => set_active_php_tray(app.clone(), "8.5"),
            "open_www" => {
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    let state = app_clone.state::<crate::AppState>();
                    let mgr = state.service_manager.lock().await;
                    let www = mgr.config.www_directory.clone();
                    drop(mgr);
                    let _ = open::that(www);
                });
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn set_active_php_tray(app: AppHandle, version: &str) {
    let version = version.to_string();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<crate::AppState>();
        let mut mgr = state.service_manager.lock().await;
        mgr.config.active_php = version;
        log::info!("Active PHP version changed via tray");
    });
}
