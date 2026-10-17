use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteInfo {
    pub name: String,
    pub domain: String,
    pub path: String,
    pub public_path: String,
    pub php_version: String,
    pub has_ssl: bool,
    pub is_secured: bool,
}

#[tauri::command]
pub async fn list_sites(state: State<'_, AppState>) -> Result<Vec<SiteInfo>, String> {
    let mgr = state.service_manager.lock().await;
    let www = PathBuf::from(&mgr.config.www_directory);
    drop(mgr);

    if !www.exists() {
        return Ok(vec![]);
    }

    let mut sites = Vec::new();

    let entries = std::fs::read_dir(&www)
        .map_err(|e| format!("Cannot read www/: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) if !n.starts_with('.') => n.to_string(),
            _ => continue,
        };

        let domain = format!("{}.test", name);
        let public_path = detect_public_root(&path);
        let php_version = detect_project_php(&path);

        sites.push(SiteInfo {
            name: name.clone(),
            domain,
            path: path.to_string_lossy().into_owned(),
            public_path: public_path.to_string_lossy().into_owned(),
            php_version,
            has_ssl: true,
            is_secured: true,
        });
    }

    sites.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(sites)
}

#[tauri::command]
pub async fn open_site_in_browser(domain: String) -> Result<(), String> {
    let url = format!("https://{}", domain);
    open::that(&url).map_err(|e| format!("Cannot open browser: {}", e))
}

#[tauri::command]
pub async fn open_site_folder(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let target = if path.is_empty() {
        let mgr = state.service_manager.lock().await;
        mgr.config.www_directory.clone()
    } else {
        path
    };
    open::that(&target).map_err(|e| format!("Cannot open folder: {}", e))
}

#[tauri::command]
pub async fn set_site_php_version(
    state: State<'_, AppState>,
    site: String,
    version: String,
) -> Result<(), String> {
    let mgr = state.service_manager.lock().await;
    let www = PathBuf::from(&mgr.config.www_directory);
    drop(mgr);

    let php_version_file = www.join(&site).join(".php-version");
    if version == "default" {
        let _ = std::fs::remove_file(&php_version_file);
    } else {
        std::fs::write(&php_version_file, &version)
            .map_err(|e| format!("Cannot write .php-version: {}", e))?;
    }

    // Regenerate the vhost conf with the new PHP pool port
    let etc_nginx = www.parent()
        .unwrap_or(&www)
        .join("etc").join("nginx").join("sites-enabled");
    let ssl_dir = www.parent()
        .unwrap_or(&www)
        .join("etc").join("ssl");
    let project_path = www.join(&site);
    let public_root = detect_public_root(&project_path);
    let domain = format!("{}.test", site);

    let pool_port = version_to_pool_port(&version);
    let conf = crate::vhost::template::generate_nginx_vhost(
        &site,
        &domain,
        &public_root.to_string_lossy(),
        &ssl_dir.to_string_lossy(),
        pool_port,
    );

    let conf_path = etc_nginx.join(format!("{}.conf", domain));
    if let Err(e) = std::fs::write(&conf_path, conf) {
        log::warn!("Cannot update vhost for {}: {}", site, e);
    }

    Ok(())
}

#[tauri::command]
pub async fn rescan_sites(state: State<'_, AppState>) -> Result<(), String> {
    let mgr = state.service_manager.lock().await;
    let www = mgr.config.www_directory.clone();
    drop(mgr);
    log::info!("Manual rescan requested for: {}", www);
    Ok(())
}

fn detect_public_root(project_path: &PathBuf) -> PathBuf {
    let candidates = ["public", "web", "dist", "html"];
    for c in &candidates {
        let p = project_path.join(c);
        if p.is_dir() {
            return p;
        }
    }
    project_path.clone()
}

fn detect_project_php(project_path: &PathBuf) -> String {
    // Check .php-version first
    let php_ver_file = project_path.join(".php-version");
    if let Ok(v) = std::fs::read_to_string(&php_ver_file) {
        let trimmed = v.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }
    // Then composer.json
    if let Some(v) = crate::supervisor::php_manager::detect_project_php_version(
        &project_path.to_string_lossy(),
    ) {
        return v;
    }
    "default".to_string()
}

fn version_to_pool_port(version: &str) -> &'static str {
    match version {
        "8.2" => "9082",
        "8.3" => "9083",
        "8.4" => "9084",
        "8.5" => "9085",
        _ => "9083",
    }
}
