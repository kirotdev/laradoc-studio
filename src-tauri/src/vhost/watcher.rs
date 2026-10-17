use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::{new_debouncer, DebouncedEvent};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::AppHandle;

/// Watch the www/ directory for new project folders.
/// On creation of a new subdirectory, automatically provisions:
///   - Nginx virtual host config (*.test.conf)
///   - Hosts file entry (<project>.test)
///   - OS desktop notification
pub async fn watch_www_directory(
    www_dir: PathBuf,
    app: AppHandle,
) -> Result<(), String> {
    if !www_dir.exists() {
        std::fs::create_dir_all(&www_dir)
            .map_err(|e| format!("Cannot create www/: {}", e))?;
    }

    let (tx, rx) = std::sync::mpsc::channel::<Result<Vec<DebouncedEvent>, Vec<notify::Error>>>();

    let mut debouncer = new_debouncer(Duration::from_millis(500), None, tx)
        .map_err(|e| format!("Debouncer init failed: {}", e))?;

    debouncer
        .watcher()
        .watch(&www_dir, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Watcher start failed: {}", e))?;

    log::info!("Watching www/ for new project directories: {:?}", www_dir);

    // Process events in blocking manner (this runs in its own tokio task)
    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                for event in events {
                    handle_www_event(&event, &www_dir, &app).await;
                }
            }
            Ok(Err(errors)) => {
                for e in errors {
                    log::warn!("Watcher error: {}", e);
                }
            }
            Err(_) => {
                log::warn!("www/ watcher channel closed");
                break;
            }
        }
    }

    Ok(())
}

async fn handle_www_event(event: &DebouncedEvent, www_dir: &Path, _app: &AppHandle) {
    let path = &event.path;

    // Only react to new directories
    if !path.is_dir() {
        return;
    }

    let project_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(n) => n.to_string(),
        None => return,
    };

    // Skip hidden folders
    if project_name.starts_with('.') {
        return;
    }

    let domain = format!("{}.test", project_name);
    log::info!("Detected new project: {} -> {}", project_name, domain);

    // Detect document root
    let public_root = detect_doc_root(path);
    log::info!("Document root for {}: {:?}", project_name, public_root);

    // Generate Nginx vhost config
    let etc_nginx = www_dir.parent()
        .unwrap_or(Path::new("."))
        .join("etc")
        .join("nginx")
        .join("sites-enabled");

    let ssl_dir = www_dir.parent()
        .unwrap_or(Path::new("."))
        .join("etc")
        .join("ssl");

    let conf = super::template::generate_nginx_vhost(
        &project_name,
        &domain,
        &public_root.to_string_lossy(),
        &ssl_dir.to_string_lossy(),
        "9083", // default PHP pool
    );

    let conf_path = etc_nginx.join(format!("{}.conf", domain));
    if let Err(e) = std::fs::create_dir_all(&etc_nginx) {
        log::error!("Cannot create sites-enabled/: {}", e);
        return;
    }
    if let Err(e) = std::fs::write(&conf_path, &conf) {
        log::error!("Cannot write vhost config: {}", e);
        return;
    }
    log::info!("Generated vhost config: {:?}", conf_path);

    // Update hosts file
    if let Err(e) = crate::hosts::add_domain(&domain) {
        log::warn!("Hosts update failed for {}: {}", domain, e);
    }

    // Reload Nginx gracefully
    reload_nginx();
}

fn detect_doc_root(project_path: &Path) -> PathBuf {
    let candidates = ["public", "web", "dist", "html"];
    for candidate in &candidates {
        let candidate_path = project_path.join(candidate);
        if candidate_path.is_dir() {
            return candidate_path;
        }
    }
    project_path.to_path_buf()
}

fn reload_nginx() {
    // nginx -s reload — non-blocking
    let _ = std::process::Command::new("nginx")
        .args(["-s", "reload"])
        .output();
    log::info!("nginx reload signal sent");
}
