use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::AppHandle;

pub mod php_manager;
pub mod provisioner;
pub mod service;

#[cfg(unix)]
pub mod unix;

#[cfg(windows)]
pub mod windows;

pub use provisioner::RuntimeProvisioner;
pub use service::{ServiceInfo, ServiceKind, ServiceState};

/// Global config held in ServiceManager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaradocConfig {
    pub active_php: String,
    pub www_directory: String,
    pub data_directory: String,
    pub bin_directory: String,
}

pub fn resolve_default_www_path() -> std::path::PathBuf {
    #[cfg(windows)]
    {
        let c_www = std::path::PathBuf::from(r"C:\www");
        if c_www.exists() || std::fs::create_dir_all(&c_www).is_ok() {
            return c_www;
        }
        if let Some(home) = dirs::home_dir() {
            let user_www = home.join("www");
            let _ = std::fs::create_dir_all(&user_www);
            return user_www;
        }
        c_www
    }

    #[cfg(not(windows))]
    {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        let user_www = home.join("www");
        let _ = std::fs::create_dir_all(&user_www);
        user_www
    }
}

impl Default for LaradocConfig {
    fn default() -> Self {
        let base = dirs::data_local_dir()
            .unwrap_or_default()
            .join("laradoc-studio");
        let www_path = resolve_default_www_path();
        Self {
            active_php: "8.3".into(),
            www_directory: www_path.to_string_lossy().into_owned(),
            data_directory: base.join("data").to_string_lossy().into_owned(),
            bin_directory: base.join("bin").to_string_lossy().into_owned(),
        }
    }
}


/// Central service orchestrator — manages isolated services and on-demand Herd-style provisioning
pub struct ServiceManager {
    pub config: LaradocConfig,
    pub services: HashMap<ServiceKind, ServiceInfo>,
}

impl ServiceManager {
    pub fn new() -> Self {
        let config = LaradocConfig::default();
        let mut services = HashMap::new();

        let definitions = vec![
            ServiceInfo::new(ServiceKind::Nginx, "Nginx Web Server", 80),
            ServiceInfo::new(ServiceKind::Php82, "PHP 8.2 FPM", 9082),
            ServiceInfo::new(ServiceKind::Php83, "PHP 8.3 FPM (Active)", 9083),
            ServiceInfo::new(ServiceKind::Php84, "PHP 8.4 FPM", 9084),
            ServiceInfo::new(ServiceKind::Php85, "PHP 8.5 FPM", 9085),
            ServiceInfo::new(ServiceKind::Mysql, "MySQL Database", 3306),
            ServiceInfo::new(ServiceKind::Redis, "Redis Cache", 6379),
            ServiceInfo::new(ServiceKind::Mailpit, "Mail Catcher (SMTP)", 1025),
        ];

        for svc in definitions {
            services.insert(svc.kind.clone(), svc);
        }

        let mut sm = Self { config, services };
        sm.refresh_installed_states();
        sm
    }

    /// Refresh installed states based on physical disk presence
    pub fn refresh_installed_states(&mut self) {
        for (kind, svc) in self.services.iter_mut() {
            if svc.state == ServiceState::Running || svc.state == ServiceState::Installing {
                continue;
            }
            if *kind == ServiceKind::Mailpit {
                svc.state = ServiceState::Stopped;
                continue;
            }
            if RuntimeProvisioner::is_installed(kind, &self.config) {
                if svc.state == ServiceState::NotInstalled {
                    svc.state = ServiceState::Stopped;
                    svc.error_message = None;
                }
            } else {
                svc.state = ServiceState::NotInstalled;
            }
        }
    }

    /// Provision a runtime binary on-demand (Herd pattern)
    pub async fn install_service(&mut self, kind: ServiceKind, app: AppHandle) -> Result<(), String> {
        if let Some(svc) = self.services.get_mut(&kind) {
            svc.state = ServiceState::Installing;
            svc.progress_percent = Some(10);
            svc.error_message = None;
        }

        let res = RuntimeProvisioner::provision(app, &kind, &self.config).await;

        if let Some(svc) = self.services.get_mut(&kind) {
            match res {
                Ok(_) => {
                    svc.state = ServiceState::Stopped;
                    svc.progress_percent = None;
                    svc.error_message = None;
                }
                Err(ref e) => {
                    svc.state = ServiceState::Error;
                    svc.progress_percent = None;
                    svc.error_message = Some(e.clone());
                }
            }
        }

        res
    }

    pub async fn start_all(&mut self, app: AppHandle) {
        for kind in ServiceKind::all() {
            self.start_service(kind, app.clone()).await;
        }
    }

    pub async fn stop_all(&mut self) {
        for kind in ServiceKind::all() {
            self.stop_service(&kind).await;
        }
    }

    pub async fn start_service(&mut self, kind: ServiceKind, app: AppHandle) {
        // Special case: Mailpit is our built-in embedded Rust SMTP server on 1025
        if kind == ServiceKind::Mailpit {
            if let Some(svc) = self.services.get_mut(&kind) {
                svc.state = ServiceState::Running;
                svc.started_at = Some(std::time::Instant::now());
                svc.error_message = None;
                svc.conflict_process = None;
            }
            log::info!("Embedded Mail Catcher (SMTP 1025) is active");
            return;
        }

        // On-demand provisioning if binary is not yet downloaded
        if !RuntimeProvisioner::is_installed(&kind, &self.config) {
            log::info!("Service {:?} not installed yet, auto-provisioning...", kind);
            if let Err(e) = self.install_service(kind.clone(), app.clone()).await {
                log::error!("Auto-provisioning failed for {:?}: {}", kind, e);
                return;
            }
        }

        // Pre-flight 1: Port availability check
        if let Some(svc) = self.services.get(&kind) {
            let port = svc.port;
            if !self.is_port_free(port) {
                let conflicting = self.find_port_owner(port);
                if let Some(svc_mut) = self.services.get_mut(&kind) {
                    svc_mut.state = ServiceState::Conflict;
                    svc_mut.conflict_process = conflicting;
                }
                log::warn!("Port {} is occupied, cannot start {:?}", port, kind);
                return;
            }
        }

        // Pre-flight 2: Binary availability resolution
        let binary = match self.resolve_binary(&kind) {
            Some(b) => b,
            None => {
                if let Some(svc_mut) = self.services.get_mut(&kind) {
                    svc_mut.state = ServiceState::Error;
                    svc_mut.error_message = Some(format!(
                        "Binary for {} could not be located after provisioning.",
                        kind.as_str()
                    ));
                }
                log::warn!("Binary for {:?} not found", kind);
                return;
            }
        };

        if let Some(svc) = self.services.get_mut(&kind) {
            svc.state = ServiceState::Starting;

            svc.error_message = None;
            svc.conflict_process = None;
        }

        // Platform-specific process spawn
        #[cfg(unix)]
        let result = unix::spawn_service(&kind, &self.config, &binary).await;

        #[cfg(windows)]
        let result = windows::spawn_service(&kind, &self.config, &binary).await;

        #[cfg(not(any(unix, windows)))]
        let result: Result<u32, String> = Err("Unsupported platform".into());

        if let Some(svc) = self.services.get_mut(&kind) {
            match result {
                Ok(pid) => {
                    svc.state = ServiceState::Running;
                    svc.pid = Some(pid);
                    svc.started_at = Some(std::time::Instant::now());
                    log::info!("Started {:?} (PID {})", kind, pid);
                }
                Err(e) => {
                    svc.state = ServiceState::Error;
                    svc.error_message = Some(e.clone());
                    log::error!("Failed to start {:?}: {}", kind, e);
                }
            }
        }
    }

    pub async fn stop_service(&mut self, kind: &ServiceKind) {
        if kind == &ServiceKind::Mailpit {
            if let Some(svc) = self.services.get_mut(kind) {
                svc.state = ServiceState::Stopped;
                svc.started_at = None;
            }
            return;
        }

        let pid = self.services.get(kind).and_then(|s| s.pid);

        if let Some(pid) = pid {
            #[cfg(unix)]
            unix::kill_service(pid).await;

            #[cfg(windows)]
            windows::kill_service(pid).await;
        }

        if let Some(svc) = self.services.get_mut(kind) {
            svc.state = ServiceState::Stopped;
            svc.pid = None;
            svc.started_at = None;
            svc.error_message = None;
            svc.conflict_process = None;
        }
    }

    pub fn resolve_binary(&self, kind: &ServiceKind) -> Option<String> {
        let bundled = format!("{}/{}", self.config.bin_directory, kind.binary_name());
        if std::path::Path::new(&bundled).exists() {
            return Some(bundled);
        }

        mod_supervisor_resolve_system_fallback(kind)
    }


    pub async fn restart_service(&mut self, kind: ServiceKind, app: AppHandle) {
        self.stop_service(&kind).await;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        self.start_service(kind, app).await;
    }

    pub fn get_all_statuses(&self) -> Vec<service::ServiceStatus> {
        let mut statuses: Vec<_> = self.services.values().map(|s| s.to_status()).collect();
        // Sort by a canonical order
        let order = ServiceKind::all();
        statuses.sort_by_key(|s| order.iter().position(|k| k.as_str() == s.kind).unwrap_or(99));
        statuses
    }

    fn is_port_free(&self, port: u16) -> bool {
        match std::net::TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(_) => true,
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => false,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                #[cfg(unix)]
                {
                    if let Ok(output) = std::process::Command::new("lsof")
                        .args(["-i", &format!(":{}", port), "-sTCP:LISTEN", "-n", "-P"])
                        .output()
                    {
                        return output.stdout.is_empty();
                    }
                }
                true
            }
            Err(_) => false,
        }
    }

    fn find_port_owner(&self, port: u16) -> Option<String> {
        #[cfg(unix)]
        {
            if let Ok(output) = std::process::Command::new("lsof")
                .args(["-i", &format!(":{}", port), "-sTCP:LISTEN", "-n", "-P"])
                .output()
            {
                if output.status.success() {
                    let text = String::from_utf8_lossy(&output.stdout);
                    for line in text.lines().skip(1) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            return Some(format!("{} (PID {})", parts[0], parts[1]));
                        }
                    }
                }
            }
        }
        Some(format!("Occupied by external service on port {}", port))
    }
}

pub fn mod_supervisor_resolve_system_fallback(kind: &ServiceKind) -> Option<String> {
    let candidates = match kind {
        ServiceKind::Nginx => vec![
            "/opt/homebrew/bin/nginx",
            "/usr/local/bin/nginx",
            "/usr/bin/nginx",
        ],
        ServiceKind::Php82 => vec![
            "/opt/homebrew/opt/php@8.2/sbin/php-fpm",
            "/usr/local/opt/php@8.2/sbin/php-fpm",
        ],
        ServiceKind::Php83 => vec![
            "/opt/homebrew/opt/php@8.3/sbin/php-fpm",
            "/usr/local/opt/php@8.3/sbin/php-fpm",
            "/opt/homebrew/sbin/php-fpm",
            "/usr/local/sbin/php-fpm",
        ],
        ServiceKind::Php84 => vec![
            "/opt/homebrew/opt/php@8.4/sbin/php-fpm",
            "/usr/local/opt/php@8.4/sbin/php-fpm",
        ],
        ServiceKind::Php85 => vec![
            "/opt/homebrew/opt/php@8.5/sbin/php-fpm",
            "/usr/local/opt/php@8.5/sbin/php-fpm",
        ],
        ServiceKind::Mysql => vec![
            "/opt/homebrew/bin/mysqld_safe",
            "/opt/homebrew/bin/mariadbd-safe",
            "/usr/local/bin/mysqld_safe",
        ],
        ServiceKind::Redis => vec![
            "/opt/homebrew/bin/redis-server",
            "/usr/local/bin/redis-server",
            "/usr/bin/redis-server",
        ],
        ServiceKind::Mailpit => vec![],
    };

    for c in candidates {
        if std::path::Path::new(c).exists() {
            return Some(c.to_string());
        }
    }

    None
}

