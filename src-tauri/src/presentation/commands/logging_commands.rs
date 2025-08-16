use tracing::info;

#[tauri::command]
pub fn log_to_file(message: String) {
    info!("[FRONTEND] {}", message);
}
