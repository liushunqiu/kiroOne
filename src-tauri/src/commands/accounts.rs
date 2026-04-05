use tauri::State;
use crate::models::account::Account;
use crate::AppState;

#[tauri::command]
pub async fn get_accounts(state: State<'_, AppState>) -> Result<Vec<Account>, String> {
    state.account_manager
        .get_all_accounts()
        .await
        .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub async fn import_account(
    state: State<'_, AppState>,
    account: Account,
) -> Result<(), String> {
    state.account_manager
        .import_account(account)
        .await
        .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub async fn switch_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<(), String> {
    state.account_manager
        .switch_account(&account_id)
        .await
        .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub async fn delete_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<(), String> {
    state.account_manager
        .delete_account(&account_id)
        .await
        .map_err(|e: anyhow::Error| e.to_string())
}
