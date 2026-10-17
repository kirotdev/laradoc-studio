use tauri::{AppHandle, State};
use crate::AppState;
use crate::supervisor::service::ServiceStatus;
use crate::supervisor::ServiceKind;

#[tauri::command]
pub async fn get_services_status(state: State<'_, AppState>) -> Result<Vec<ServiceStatus>, String> {
    let mgr = state.service_manager.lock().await;
    Ok(mgr.get_all_statuses())
}

#[tauri::command]
pub async fn start_all(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let mut mgr = state.service_manager.lock().await;
    mgr.start_all(app).await;
    Ok(())
}

#[tauri::command]
pub async fn stop_all(state: State<'_, AppState>) -> Result<(), String> {
    let mut mgr = state.service_manager.lock().await;
    mgr.stop_all().await;
    Ok(())
}

#[tauri::command]
pub async fn start_service(
    app: AppHandle,
    state: State<'_, AppState>,
    service: String,
) -> Result<(), String> {
    let kind = ServiceKind::from_str(&service)
        .ok_or_else(|| format!("Unknown service: {}", service))?;
    let mut mgr = state.service_manager.lock().await;
    mgr.start_service(kind, app).await;
    Ok(())
}

#[tauri::command]
pub async fn stop_service(
    state: State<'_, AppState>,
    service: String,
) -> Result<(), String> {
    let kind = ServiceKind::from_str(&service)
        .ok_or_else(|| format!("Unknown service: {}", service))?;
    let mut mgr = state.service_manager.lock().await;
    mgr.stop_service(&kind).await;
    Ok(())
}

#[tauri::command]
pub async fn restart_service(
    app: AppHandle,
    state: State<'_, AppState>,
    service: String,
) -> Result<(), String> {
    let kind = ServiceKind::from_str(&service)
        .ok_or_else(|| format!("Unknown service: {}", service))?;
    let mut mgr = state.service_manager.lock().await;
    mgr.restart_service(kind, app).await;
    Ok(())
}

#[tauri::command]
pub async fn set_active_php(
    state: State<'_, AppState>,
    version: String,
) -> Result<(), String> {
    let valid = ["8.2", "8.3", "8.4", "8.5"];
    if !valid.contains(&version.as_str()) {
        return Err(format!("Invalid PHP version: {}", version));
    }
    let mut mgr = state.service_manager.lock().await;
    mgr.config.active_php = version.clone();
    log::info!("Active PHP version set to {}", version);
    Ok(())
}
