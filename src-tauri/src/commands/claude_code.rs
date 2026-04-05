use tauri::State;
use crate::AppState;

#[tauri::command]
pub async fn configure_claude_code(state: State<'_, AppState>) -> Result<String, String> {
    let port = state.config.gateway.port;
    crate::services::claude_config::apply_claude_config(port)
        .map_err(|e: anyhow::Error| e.to_string())
}
