use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{Manager, WindowEvent};

pub mod commands;
pub mod hosts;
pub mod mail;
pub mod pki;
pub mod supervisor;
pub mod tray;
pub mod vhost;

use supervisor::ServiceManager;
use mail::MailStore;

/// Shared application state injected into Tauri's state container
pub struct AppState {
    pub service_manager: Arc<Mutex<ServiceManager>>,
    pub mail_store: Arc<Mutex<MailStore>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Focus the already-running window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Initialize shared state with cold-start (all services stopped)
            let service_manager = Arc::new(Mutex::new(ServiceManager::new()));
            let mail_store = Arc::new(Mutex::new(MailStore::new()));

            app.manage(AppState {
                service_manager: Arc::clone(&service_manager),
                mail_store: Arc::clone(&mail_store),
            });

            // Set up system tray
            tray::setup_tray(app.handle())?;

            // Spawn the mail SMTP server
            let mail_handle = app_handle.clone();
            let mail_store_bg = Arc::clone(&mail_store);
            tokio::spawn(async move {
                if let Err(e) = mail::smtp::run_smtp_server(
                    "127.0.0.1:1025",
                    mail_store_bg,
                    mail_handle,
                )
                .await
                {
                    log::error!("SMTP server error: {}", e);
                }
            });

            // Spawn vhost directory watcher on www/
            let vhost_handle = app_handle.clone();
            let data_dir = app.path().app_data_dir().unwrap_or_default();
            let www_dir = data_dir.join("www");
            tokio::spawn(async move {
                if let Err(e) = vhost::watcher::watch_www_directory(www_dir, vhost_handle).await {
                    log::error!("Directory watcher error: {}", e);
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // Close-to-tray: minimize instead of quitting when services are running
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::services::start_all,
            commands::services::stop_all,
            commands::services::start_service,
            commands::services::stop_service,
            commands::services::restart_service,
            commands::services::get_services_status,
            commands::services::set_active_php,
            commands::sites::list_sites,
            commands::sites::open_site_in_browser,
            commands::sites::open_site_folder,
            commands::sites::set_site_php_version,
            commands::sites::rescan_sites,
            commands::logs::get_recent_logs,
            commands::logs::clear_logs,
            commands::mail::get_emails,
            commands::mail::clear_emails,
            commands::settings::get_settings,
            commands::settings::install_root_ca,
            commands::settings::sync_hosts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Laradoc Studio");
}
