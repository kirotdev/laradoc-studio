use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub active_php: String,
    pub www_directory: String,
    pub root_ca_trusted: bool,
    pub hosts_synced: bool,
    pub auto_start_disabled: bool,
}

#[tauri::command]
pub async fn get_settings(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AppSettings, String> {
    let mgr = state.service_manager.lock().await;
    let active_php = mgr.config.active_php.clone();
    let www_directory = mgr.config.www_directory.clone();
    drop(mgr);

    let data_dir = app.path().app_data_dir().unwrap_or_default();
    let ca_path = data_dir.join("etc").join("ssl").join("ca.crt");
    let root_ca_trusted = crate::pki::store::is_ca_trusted(&ca_path.to_string_lossy());

    let hosts_content = crate::hosts::read_hosts().unwrap_or_default();
    let hosts_synced = hosts_content.contains("# BEGIN LARADOC-STUDIO");

    Ok(AppSettings {
        active_php,
        www_directory,
        root_ca_trusted,
        hosts_synced,
        auto_start_disabled: true,
    })
}

#[tauri::command]
pub async fn install_root_ca(
    app: AppHandle,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().unwrap_or_default();
    let ssl_dir = data_dir.join("etc").join("ssl");
    let ssl_dir_str = ssl_dir.to_string_lossy().to_string();

    // Ensure Local CA exists on disk first
    let _ca = crate::pki::LocalCa::init(&ssl_dir_str)?;

    let ca_cert_path = format!("{}/ca.crt", ssl_dir_str);
    crate::pki::store::install_root_ca(&ca_cert_path)?;
    log::info!("Root CA installed into OS trust store from {}", ca_cert_path);
    Ok(())
}

#[tauri::command]
pub async fn sync_hosts(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.service_manager.lock().await;
    let www_dir = PathBuf::from(&mgr.config.www_directory);
    drop(mgr);

    let mut entries = Vec::new();
    if www_dir.exists() {
        if let Ok(dir_entries) = std::fs::read_dir(&www_dir) {
            for entry in dir_entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                        if !name.starts_with('.') {
                            let domain = format!("{}.test", name);
                            entries.push((domain, "127.0.0.1".to_string()));
                        }
                    }
                }
            }
        }
    }

    crate::hosts::sync_hosts(&entries)?;
    log::info!("Hosts file synced with {} project domains", entries.len());
    Ok(())
}
