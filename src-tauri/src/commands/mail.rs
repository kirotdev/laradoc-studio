use tauri::State;
use crate::AppState;
use crate::mail::CapturedEmail;

#[tauri::command]
pub async fn get_emails(state: State<'_, AppState>) -> Result<Vec<CapturedEmail>, String> {
    let store = state.mail_store.lock().await;
    Ok(store.all())
}

#[tauri::command]
pub async fn clear_emails(state: State<'_, AppState>) -> Result<(), String> {
    let mut store = state.mail_store.lock().await;
    store.clear();
    log::info!("Captured emails buffer cleared");
    Ok(())
}
