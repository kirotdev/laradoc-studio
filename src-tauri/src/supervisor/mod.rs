use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::AppHandle;

pub mod php_manager;
pub mod service;

#[cfg(unix)]
pub mod unix;

#[cfg(windows)]
pub mod windows;

pub use service::{ServiceInfo, ServiceKind, ServiceState};

/// Global config held in ServiceManager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaradocConfig {
    pub active_php: String,
    pub www_directory: String,
    pub data_directory: String,
    pub bin_directory: String,
}

impl Default for LaradocConfig {
    fn default() -> Self {
        let base = dirs::data_local_dir()
            .unwrap_or_default()
            .join("laradoc-studio");
        Self {
            active_php: "8.3".into(),
            www_directory: base.join("www").to_string_lossy().into_owned(),
            data_directory: base.join("data").to_string_lossy().into_owned(),
            bin_directory: base.join("bin").to_string_lossy().into_owned(),
        }
    }
}

/// Central service orchestrator — all services start in Stopped state (cold start)
pub struct ServiceManager {
    pub config: LaradocConfig,
    pub services: HashMap<ServiceKind, ServiceInfo>,
}

impl ServiceManager {
    pub fn new() -> Self {
        let config = LaradocConfig::default();
        let mut services = HashMap::new();

        // Register all managed services in Stopped state
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

        Self { config, services }
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

    pub async fn start_service(&mut self, kind: ServiceKind, _app: AppHandle) {
        // Pre-flight: check port availability
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

        if let Some(svc) = self.services.get_mut(&kind) {
            svc.state = ServiceState::Starting;
        }

        // Platform-specific process spawn
        #[cfg(unix)]
        let result = unix::spawn_service(&kind, &self.config).await;

        #[cfg(windows)]
        let result = windows::spawn_service(&kind, &self.config).await;

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
        std::net::TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
    }

    fn find_port_owner(&self, port: u16) -> Option<String> {
        let sys = sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::new().with_processes(sysinfo::ProcessRefreshKind::new()),
        );
        for (_pid, process) in sys.processes() {
            for network in process.connections() {
                if network.local_addr.port() == port {
                    return Some(format!(
                        "{} (PID {})",
                        process.name().to_string_lossy(),
                        process.pid()
                    ));
                }
            }
        }
        None
    }
}
