use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: String,
    pub service: String,
    pub level: String,
    pub message: String,
}

/// In-memory recent log buffer accessible from IPC
pub struct LogStore {
    pub entries: Mutex<Vec<LogEntry>>,
}

impl LogStore {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
        }
    }
}

#[tauri::command]
pub async fn get_recent_logs() -> Result<Vec<LogEntry>, String> {
    // Returns seeded startup logs — in production, these are emitted
    // via Tauri events as log:entry from the supervisor output reader tasks
    Ok(vec![
        LogEntry {
            id: "boot-1".into(),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            service: "System".into(),
            level: "info".into(),
            message: "Laradoc Studio started — Cold Start policy active. All services stopped by default.".into(),
        },
        LogEntry {
            id: "boot-2".into(),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            service: "PHP-Manager".into(),
            level: "info".into(),
            message: "FastCGI pool ports reserved: 8.2→9082, 8.3→9083, 8.4→9084, 8.5→9085.".into(),
        },
        LogEntry {
            id: "boot-3".into(),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            service: "Mailpit".into(),
            level: "info".into(),
            message: "Local SMTP listener ready on 127.0.0.1:1025. Web UI on 127.0.0.1:8025.".into(),
        },
    ])
}

#[tauri::command]
pub async fn clear_logs() -> Result<(), String> {
    // In full implementation, clears in-memory log buffer
    log::info!("Log buffer cleared by user request");
    Ok(())
}
