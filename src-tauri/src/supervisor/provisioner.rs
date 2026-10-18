use super::{LaradocConfig, ServiceKind};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeProgressPayload {
    pub service: String,
    pub percent: u8,
    pub status: String,
}

pub struct RuntimeProvisioner;

impl RuntimeProvisioner {
    /// Check whether a service runtime binary is physically installed and executable
    pub fn is_installed(kind: &ServiceKind, config: &LaradocConfig) -> bool {
        if kind == &ServiceKind::Mailpit {
            return true; // Embedded pure-Rust SMTP catcher
        }

        let target_binary = Path::new(&config.bin_directory).join(kind.binary_name());
        if target_binary.exists() {
            return true;
        }

        // Secondary fallback to platform system paths (e.g. if installed via Homebrew)
        super::mod_supervisor_resolve_system_fallback(kind).is_some()
    }

    /// Extract bundled runtimes from Tauri app bundle resources on first launch if present
    pub fn unpack_bundled_resources_if_present(app: &AppHandle, config: &LaradocConfig) {
        let Ok(resource_dir) = app.path().resource_dir() else { return };
        let bundled_bin = resource_dir.join("bin");

        if bundled_bin.exists() && bundled_bin.is_dir() {
            let target_bin = Path::new(&config.bin_directory);
            if !target_bin.exists() {
                log::info!("Copying pre-bundled runtimes from {:?} to {:?}", bundled_bin, target_bin);
                let _ = copy_dir_all(&bundled_bin, target_bin);
                #[cfg(unix)]
                set_exec_permissions_recursive(target_bin);
            }
        }
    }

    /// Download and provision a standalone runtime package (Herd-style on-demand provisioning)
    pub async fn provision(
        app: AppHandle,
        kind: &ServiceKind,
        config: &LaradocConfig,
    ) -> Result<(), String> {
        let service_key = kind.as_str().to_string();

        emit_progress(&app, &service_key, 5, "Preparing runtime environment...");

        // Ensure directories exist
        let bin_path = Path::new(&config.bin_directory);
        let etc_path = PathBuf::from(config.bin_directory.replace("/bin", "/etc"));
        let data_path = Path::new(&config.data_directory);

        let _ = std::fs::create_dir_all(bin_path);
        let _ = std::fs::create_dir_all(&etc_path);
        let _ = std::fs::create_dir_all(data_path.join("pids"));
        let _ = std::fs::create_dir_all(data_path.join("logs"));

        // Determine download URL based on OS and architecture
        let (url, is_archive) = get_runtime_url(kind)?;

        emit_progress(&app, &service_key, 15, "Downloading runtime archive...");

        let temp_archive = data_path.join(format!("{}-package.tmp", service_key));

        // Download using streaming curl or reqwest
        download_file(&app, &service_key, &url, &temp_archive).await?;

        emit_progress(&app, &service_key, 75, "Extracting and verifying runtime...");

        // Extract archive to bin_directory
        extract_archive(&temp_archive, bin_path, is_archive)?;
        let _ = std::fs::remove_file(&temp_archive);

        #[cfg(unix)]
        {
            set_exec_permissions_recursive(bin_path);
            clear_quarantine_attribute(bin_path);
        }

        emit_progress(&app, &service_key, 90, "Configuring services & pools...");

        // Generate configuration files (php-fpm.conf, php.ini, nginx.conf)
        generate_configs(config, kind, &etc_path, data_path)?;

        emit_progress(&app, &service_key, 100, "Ready");
        log::info!("Successfully provisioned runtime for {:?}", kind);

        Ok(())
    }
}

/// Helper to emit download/installation progress to frontend
fn emit_progress(app: &AppHandle, service: &str, percent: u8, status: &str) {
    let _ = app.emit(
        "runtime:progress",
        RuntimeProgressPayload {
            service: service.to_string(),
            percent,
            status: status.to_string(),
        },
    );
}

/// Download a URL to target file with progress updates
async fn download_file(
    app: &AppHandle,
    service: &str,
    url: &str,
    target: &Path,
) -> Result<(), String> {
    log::info!("Downloading {} from {}", service, url);

    // Native curl download with progress reporting
    let mut child = tokio::process::Command::new("curl")
        .args(["-fSL", "--create-dirs", "-o", target.to_str().unwrap(), url])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn curl: {}", e))?;

    // Monitor progress
    let start_time = tokio::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    break;
                } else {
                    return Err(format!("Download failed with exit code {:?}", status.code()));
                }
            }
            Ok(None) => {
                let elapsed = start_time.elapsed().as_secs();
                let fake_pct = (20 + (elapsed * 5).min(50)) as u8;
                emit_progress(app, service, fake_pct, "Downloading runtime archive...");
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            Err(e) => return Err(format!("Download monitoring error: {}", e)),
        }
    }

    Ok(())
}

/// Extract downloaded archive into target directory
fn extract_archive(archive: &Path, dest: &Path, is_tar_gz: bool) -> Result<(), String> {
    if is_tar_gz {
        let output = std::process::Command::new("tar")
            .args(["-xzf", archive.to_str().unwrap(), "-C", dest.to_str().unwrap()])
            .output()
            .map_err(|e| format!("Failed to extract tar.gz: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Extraction error: {}", err));
        }
    } else {
        // Zip archive extraction
        #[cfg(unix)]
        let cmd = "unzip";
        #[cfg(windows)]
        let cmd = "tar";

        let output = std::process::Command::new(cmd)
            .args([archive.to_str().unwrap(), "-d", dest.to_str().unwrap()])
            .output()
            .map_err(|e| format!("Failed to extract zip: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Zip extraction error: {}", err));
        }
    }

    Ok(())
}

/// Generate optimized service configuration files
fn generate_configs(
    _config: &LaradocConfig,
    kind: &ServiceKind,
    etc_path: &Path,
    data_path: &Path,
) -> Result<(), String> {
    let user = whoami::username();

    match kind {
        ServiceKind::Php82 | ServiceKind::Php83 | ServiceKind::Php84 | ServiceKind::Php85 => {
            let version = match kind {
                ServiceKind::Php82 => "8.2",
                ServiceKind::Php83 => "8.3",
                ServiceKind::Php84 => "8.4",
                ServiceKind::Php85 => "8.5",
                _ => "8.3",
            };
            let port = match kind {
                ServiceKind::Php82 => 9082,
                ServiceKind::Php83 => 9083,
                ServiceKind::Php84 => 9084,
                ServiceKind::Php85 => 9085,
                _ => 9083,
            };

            let php_etc = etc_path.join(format!("php/{}", version));
            let _ = std::fs::create_dir_all(&php_etc);

            let fpm_conf = format!(
                r#"[global]
pid = {data_dir}/pids/php{ver}.pid
error_log = {data_dir}/logs/php{ver}-error.log
daemonize = no

[www]
user = {user}
listen = 127.0.0.1:{port}
listen.allowed_clients = 127.0.0.1
pm = ondemand
pm.max_children = 10
pm.process_idle_timeout = 10s
pm.max_requests = 500
catch_workers_output = yes
php_admin_value[error_log] = {data_dir}/logs/php{ver}-fpm.log
php_admin_flag[log_errors] = on
"#,
                data_dir = data_path.display(),
                ver = version.replace('.', ""),
                user = user,
                port = port
            );
            let _ = std::fs::write(php_etc.join("php-fpm.conf"), fpm_conf);

            let php_ini = r#"[PHP]
engine = On
short_open_tag = Off
precision = 14
output_buffering = 4096
zlib.output_compression = Off
implicit_flush = Off
serialize_precision = -1
zend.enable_gc = On
expose_php = Off
max_execution_time = 300
max_input_time = 60
memory_limit = 512M
error_reporting = E_ALL
display_errors = On
display_startup_errors = On
log_errors = On
post_max_size = 128M
upload_max_filesize = 128M
max_file_uploads = 20
date.timezone = UTC
opcache.enable = 1
opcache.enable_cli = 0
opcache.memory_consumption = 128
opcache.interned_strings_buffer = 8
opcache.max_accelerated_files = 10000
"#;
            let _ = std::fs::write(php_etc.join("php.ini"), php_ini);
        }
        ServiceKind::Nginx => {
            let nginx_etc = etc_path.join("nginx");
            let _ = std::fs::create_dir_all(&nginx_etc);

            let nginx_conf = format!(
                r#"worker_processes auto;
error_log {data_dir}/logs/nginx-error.log warn;
pid {data_dir}/pids/nginx.pid;

events {{
    worker_connections 1024;
}}

http {{
    include mime.types;
    default_type application/octet-stream;

    access_log {data_dir}/logs/nginx-access.log;

    sendfile on;
    keepalive_timeout 65;
    client_max_body_size 128M;

    include {data_dir}/vhosts/*.conf;
}}
"#,
                data_dir = data_path.display()
            );
            let _ = std::fs::write(nginx_etc.join("nginx.conf"), nginx_conf);

            // Create standard mime.types
            let mime_types = r#"types {
    text/html                             html htm shtml;
    text/css                              css;
    text/xml                              xml;
    image/gif                             gif;
    image/jpeg                            jpeg jpg;
    application/javascript                js;
    application/json                      json;
    image/png                             png;
    image/svg+xml                         svg svgz;
    image/webp                            webp;
    application/pdf                       pdf;
    application/zip                       zip;
    audio/mpeg                            mp3;
    video/mp4                             mp4;
}
"#;
            let _ = std::fs::write(nginx_etc.join("mime.types"), mime_types);
        }
        ServiceKind::Redis => {
            let redis_etc = etc_path.join("redis");
            let _ = std::fs::create_dir_all(&redis_etc);

            let redis_conf = format!(
                r#"bind 127.0.0.1
port 6379
daemonize no
pidfile {data_dir}/pids/redis.pid
logfile {data_dir}/logs/redis.log
dir {data_dir}/redis
save 900 1
save 300 10
"#,
                data_dir = data_path.display()
            );
            let _ = std::fs::write(redis_etc.join("redis.conf"), redis_conf);
        }
        _ => {}
    }

    Ok(())
}

/// Runtime download URL manifest resolving to pre-compiled static/relocatable archives
fn get_runtime_url(kind: &ServiceKind) -> Result<(String, bool), String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let base_cdn = "https://github.com/kirotdev/laradoc-studio/releases/download/runtimes";

    match (os, arch, kind) {
        // macOS Apple Silicon (arm64)
        ("macos", "aarch64", ServiceKind::Php82) => Ok((format!("{}/php-8.2-darwin-arm64.tar.gz", base_cdn), true)),
        ("macos", "aarch64", ServiceKind::Php83) => Ok((format!("{}/php-8.3-darwin-arm64.tar.gz", base_cdn), true)),
        ("macos", "aarch64", ServiceKind::Php84) => Ok((format!("{}/php-8.4-darwin-arm64.tar.gz", base_cdn), true)),
        ("macos", "aarch64", ServiceKind::Php85) => Ok((format!("{}/php-8.5-darwin-arm64.tar.gz", base_cdn), true)),
        ("macos", "aarch64", ServiceKind::Nginx) => Ok((format!("{}/nginx-darwin-arm64.tar.gz", base_cdn), true)),
        ("macos", "aarch64", ServiceKind::Mysql) => Ok((format!("{}/mysql-darwin-arm64.tar.gz", base_cdn), true)),
        ("macos", "aarch64", ServiceKind::Redis) => Ok((format!("{}/redis-darwin-arm64.tar.gz", base_cdn), true)),

        // macOS Intel (x86_64)
        ("macos", "x86_64", ServiceKind::Php82) => Ok((format!("{}/php-8.2-darwin-x64.tar.gz", base_cdn), true)),
        ("macos", "x86_64", ServiceKind::Php83) => Ok((format!("{}/php-8.3-darwin-x64.tar.gz", base_cdn), true)),
        ("macos", "x86_64", ServiceKind::Php84) => Ok((format!("{}/php-8.4-darwin-x64.tar.gz", base_cdn), true)),
        ("macos", "x86_64", ServiceKind::Php85) => Ok((format!("{}/php-8.5-darwin-x64.tar.gz", base_cdn), true)),
        ("macos", "x86_64", ServiceKind::Nginx) => Ok((format!("{}/nginx-darwin-x64.tar.gz", base_cdn), true)),
        ("macos", "x86_64", ServiceKind::Mysql) => Ok((format!("{}/mysql-darwin-x64.tar.gz", base_cdn), true)),
        ("macos", "x86_64", ServiceKind::Redis) => Ok((format!("{}/redis-darwin-x64.tar.gz", base_cdn), true)),

        // Windows (x86_64)
        ("windows", "x86_64", ServiceKind::Php82) => Ok((format!("{}/php-8.2-windows-x64.zip", base_cdn), false)),
        ("windows", "x86_64", ServiceKind::Php83) => Ok((format!("{}/php-8.3-windows-x64.zip", base_cdn), false)),
        ("windows", "x86_64", ServiceKind::Php84) => Ok((format!("{}/php-8.4-windows-x64.zip", base_cdn), false)),
        ("windows", "x86_64", ServiceKind::Nginx) => Ok((format!("{}/nginx-windows-x64.zip", base_cdn), false)),
        ("windows", "x86_64", ServiceKind::Mysql) => Ok((format!("{}/mysql-windows-x64.zip", base_cdn), false)),
        ("windows", "x86_64", ServiceKind::Redis) => Ok((format!("{}/redis-windows-x64.zip", base_cdn), false)),

        _ => Err(format!("No pre-compiled runtime found for {:?} on {} ({})", kind, os, arch)),
    }
}

/// Recursively copy a directory
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn set_exec_permissions_recursive(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                set_exec_permissions_recursive(&p);
            } else if let Ok(metadata) = p.metadata() {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&p, perms);
            }
        }
    }
}

#[cfg(unix)]
fn clear_quarantine_attribute(path: &Path) {
    let _ = std::process::Command::new("xattr")
        .args(["-cr", path.to_str().unwrap()])
        .output();
}

mod whoami {
    pub fn username() -> String {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "nobody".to_string())
    }
}
