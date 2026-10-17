use serde::{Deserialize, Serialize};
use std::time::Instant;

/// All managed service kinds in Laradoc Studio
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceKind {
    Nginx,
    Php82,
    Php83,
    Php84,
    Php85,
    Mysql,
    Redis,
    Mailpit,
}

impl ServiceKind {
    pub fn all() -> Vec<ServiceKind> {
        vec![
            ServiceKind::Nginx,
            ServiceKind::Php82,
            ServiceKind::Php83,
            ServiceKind::Php84,
            ServiceKind::Php85,
            ServiceKind::Mysql,
            ServiceKind::Redis,
            ServiceKind::Mailpit,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceKind::Nginx => "nginx",
            ServiceKind::Php82 => "php82",
            ServiceKind::Php83 => "php83",
            ServiceKind::Php84 => "php84",
            ServiceKind::Php85 => "php85",
            ServiceKind::Mysql => "mysql",
            ServiceKind::Redis => "redis",
            ServiceKind::Mailpit => "mailpit",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "nginx" => Some(ServiceKind::Nginx),
            "php82" => Some(ServiceKind::Php82),
            "php83" => Some(ServiceKind::Php83),
            "php84" => Some(ServiceKind::Php84),
            "php85" => Some(ServiceKind::Php85),
            "mysql" => Some(ServiceKind::Mysql),
            "redis" => Some(ServiceKind::Redis),
            "mailpit" => Some(ServiceKind::Mailpit),
            _ => None,
        }
    }

    /// Binary name for this service within the bundled bin/ directory
    pub fn binary_name(&self) -> &'static str {
        match self {
            ServiceKind::Nginx => "nginx/sbin/nginx",
            ServiceKind::Php82 => "php/8.2/sbin/php-fpm",
            ServiceKind::Php83 => "php/8.3/sbin/php-fpm",
            ServiceKind::Php84 => "php/8.4/sbin/php-fpm",
            ServiceKind::Php85 => "php/8.5/sbin/php-fpm",
            ServiceKind::Mysql => "mysql/bin/mysqld_safe",
            ServiceKind::Redis => "redis/bin/redis-server",
            ServiceKind::Mailpit => "mailpit/mailpit",
        }
    }
}

/// Lifecycle state of a service
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceState {
    Stopped,
    Starting,
    Running,
    Error,
    Conflict,
}

/// Internal tracked state for a managed service
pub struct ServiceInfo {
    pub kind: ServiceKind,
    pub display_name: String,
    pub port: u16,
    pub state: ServiceState,
    pub pid: Option<u32>,
    pub started_at: Option<Instant>,
    pub memory_bytes: u64,
    pub cpu_percent: f32,
    pub error_message: Option<String>,
    pub conflict_process: Option<String>,
}

impl ServiceInfo {
    pub fn new(kind: ServiceKind, display_name: &str, port: u16) -> Self {
        Self {
            kind,
            display_name: display_name.to_string(),
            port,
            state: ServiceState::Stopped,
            pid: None,
            started_at: None,
            memory_bytes: 0,
            cpu_percent: 0.0,
            error_message: None,
            conflict_process: None,
        }
    }

    pub fn uptime_seconds(&self) -> u64 {
        self.started_at
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0)
    }

    pub fn to_status(&self) -> ServiceStatus {
        ServiceStatus {
            name: self.display_name.clone(),
            kind: self.kind.as_str().to_string(),
            state: match self.state {
                ServiceState::Stopped => "stopped",
                ServiceState::Starting => "starting",
                ServiceState::Running => "running",
                ServiceState::Error => "error",
                ServiceState::Conflict => "conflict",
            }
            .to_string(),
            port: self.port,
            pid: self.pid,
            uptime_seconds: self.uptime_seconds(),
            memory_bytes: self.memory_bytes,
            cpu_percent: self.cpu_percent,
            error_message: self.error_message.clone(),
            conflict_process: self.conflict_process.clone(),
        }
    }
}

/// Serializable status DTO sent to frontend via IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub kind: String,
    pub state: String,
    pub port: u16,
    pub pid: Option<u32>,
    pub uptime_seconds: u64,
    pub memory_bytes: u64,
    pub cpu_percent: f32,
    pub error_message: Option<String>,
    pub conflict_process: Option<String>,
}
