#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub email: Option<String>,
    pub label: String,
    pub status: String,
    pub provider: Option<String>,
    pub auth_method: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<String>,
    pub user_id: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub region: Option<String>,
    pub profile_arn: Option<String>,
    pub usage_data: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub api_base_url: String,
    pub api_key: String,
    pub api_format: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayConfig {
    pub port: u16,
    pub api_key: String,
    pub is_running: bool,
    pub default_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiroLocalToken {
    #[serde(rename = "accessToken")]
    pub access_token: Option<String>,
    #[serde(rename = "refreshToken")]
    pub refresh_token: Option<String>,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<String>,
    #[serde(rename = "authMethod")]
    pub auth_method: Option<String>,
    pub provider: Option<String>,
    #[serde(rename = "profileArn")]
    pub profile_arn: Option<String>,
    #[serde(rename = "clientIdHash")]
    pub client_id_hash: Option<String>,
    pub region: Option<String>,
}

pub struct AppState {
    pub accounts: Mutex<HashMap<String, Account>>,
    pub providers: Mutex<HashMap<String, Provider>>,
    pub gateway_config: Mutex<GatewayConfig>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            accounts: Mutex::new(HashMap::new()),
            providers: Mutex::new(HashMap::new()),
            gateway_config: Mutex::new(GatewayConfig {
                port: 8710,
                api_key: uuid::Uuid::new_v4().to_string(),
                is_running: false,
                default_model: Some("claude-sonnet-4".to_string()),
            }),
        }
    }
}

#[tauri::command]
fn get_accounts(state: tauri::State<AppState>) -> Vec<Account> {
    let accounts = state.accounts.lock().unwrap();
    accounts.values().cloned().collect()
}

#[tauri::command]
fn add_account(state: tauri::State<AppState>, label: String, email: Option<String>) -> String {
    let now = chrono::Local::now().format("%Y/%m/%d %H:%M:%S").to_string();
    let account = Account {
        id: Uuid::new_v4().to_string(),
        email,
        label,
        status: "active".to_string(),
        provider: None,
        auth_method: None,
        access_token: None,
        refresh_token: None,
        expires_at: None,
        user_id: None,
        client_id: None,
        client_secret: None,
        region: None,
        profile_arn: None,
        usage_data: None,
        created_at: now.clone(),
        updated_at: now,
    };
    let mut accounts = state.accounts.lock().unwrap();
    accounts.insert(account.id.clone(), account.clone());
    account.id
}

#[tauri::command]
fn update_account(state: tauri::State<AppState>, id: String, label: Option<String>, status: Option<String>) -> bool {
    let mut accounts = state.accounts.lock().unwrap();
    if let Some(account) = accounts.get_mut(&id) {
        if let Some(l) = label { account.label = l; }
        if let Some(s) = status { account.status = s; }
        account.updated_at = chrono::Local::now().format("%Y/%m/%d %H:%M:%S").to_string();
        true
    } else { false }
}

#[tauri::command]
fn delete_account(state: tauri::State<AppState>, id: String) -> bool {
    state.accounts.lock().unwrap().remove(&id).is_some()
}

#[tauri::command]
async fn sync_account(state: tauri::State<'_, AppState>, id: String) -> Result<String, String> {
    let account = {
        let accounts = state.accounts.lock().unwrap();
        accounts.get(&id).cloned().ok_or("账号不存在")?
    };
    
    let _refresh_token = account.refresh_token.clone().ok_or("账号缺少 refresh_token")?;
    let provider = account.provider.clone().unwrap_or_else(|| "Google".to_string());
    
    // 模拟额度数据 (实际应该调用 Kiro API)
    // 这里生成一些示例数据来演示功能
    let usage_data = serde_json::json!({
        "usageBreakdownList": [{
            "currentUsageWithPrecision": 150.5,
            "usageLimitWithPrecision": 1000.0,
            "currentUsage": 150,
            "usageLimit": 1000,
            "percentage": 15.05
        }],
        "provider": provider,
        "syncedAt": chrono::Local::now().format("%Y/%m/%d %H:%M:%S").to_string()
    });
    
    let mut accounts = state.accounts.lock().unwrap();
    if let Some(acc) = accounts.get_mut(&id) {
        acc.usage_data = Some(usage_data.to_string());
        acc.status = "active".to_string();
        acc.updated_at = chrono::Local::now().format("%Y/%m/%d %H:%M:%S").to_string();
    }
    
    Ok(format!("账号 {} 同步成功", id))
}

#[tauri::command]
fn export_accounts_command(state: tauri::State<AppState>, ids: Vec<String>) -> String {
    let accounts = state.accounts.lock().unwrap();
    let selected: Vec<&Account> = ids.iter().filter_map(|id| accounts.get(id)).collect();
    serde_json::to_string_pretty(&selected).unwrap_or_else(|_| "[]".to_string())
}

#[tauri::command]
fn import_accounts_command(state: tauri::State<AppState>, json: String) -> Result<usize, String> {
    let accounts_data: Vec<serde_json::Value> = serde_json::from_str(&json).map_err(|e| format!("JSON 解析失败: {}", e))?;
    let mut map = state.accounts.lock().unwrap();
    let mut count = 0;
    for data in accounts_data {
        // 支持多种字段名格式
        let refresh_token = data.get("refreshToken")
            .or_else(|| data.get("refresh_token"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if refresh_token.is_none() { continue; }
        
        let provider = data.get("provider")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let email = data.get("email")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let label = data.get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("Imported Account");
        
        let now = chrono::Local::now().format("%Y/%m/%d %H:%M:%S").to_string();
        let account = Account {
            id: Uuid::new_v4().to_string(),
            email,
            label: label.to_string(),
            status: "active".to_string(),
            provider,
            auth_method: if data.get("clientId").is_some() && data.get("clientSecret").is_some() {
                Some("IdC".to_string())
            } else { Some("social".to_string()) },
            access_token: None,
            refresh_token,
            expires_at: None,
            user_id: None,
            client_id: data.get("clientId").and_then(|v| v.as_str()).map(|s| s.to_string()),
            client_secret: data.get("clientSecret").and_then(|v| v.as_str()).map(|s| s.to_string()),
            region: data.get("region").and_then(|v| v.as_str()).map(|s| s.to_string()),
            profile_arn: None,
            usage_data: None,
            created_at: now.clone(),
            updated_at: now,
        };
        map.insert(account.id.clone(), account);
        count += 1;
    }
    Ok(count)
}

// ===== Kiro IDE 导入 =====

#[tauri::command]
async fn get_kiro_local_token() -> Option<KiroLocalToken> {
    tokio::task::spawn_blocking(|| {
        let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).ok()?;
        let path = std::path::Path::new(&home).join(".aws").join("sso").join("cache").join("kiro-auth-token.json");
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }).await.ok().flatten()
}

#[tauri::command]
async fn add_account_by_social(
    state: tauri::State<'_, AppState>,
    refresh_token: String,
    provider: Option<String>,
    access_token: Option<String>,
) -> Result<serde_json::Value, String> {
    let idp = provider.as_deref().unwrap_or("Google").to_string();
    let email = format!("user@{}.com", idp.to_lowercase());
    let now = chrono::Local::now().format("%Y/%m/%d %H:%M:%S").to_string();
    let account = Account {
        id: Uuid::new_v4().to_string(),
        email: Some(email),
        label: format!("Kiro {} Account", idp),
        status: "active".to_string(),
        provider: Some(idp.clone()),
        auth_method: Some("social".to_string()),
        access_token,
        refresh_token: Some(refresh_token),
        expires_at: None,
        user_id: None,
        client_id: None,
        client_secret: None,
        region: None,
        profile_arn: None,
        usage_data: None,
        created_at: now.clone(),
        updated_at: now,
    };
    let mut accounts = state.accounts.lock().unwrap();
    accounts.insert(account.id.clone(), account.clone());
    Ok(serde_json::json!({ "account": account, "isNew": true }))
}

#[tauri::command]
async fn import_from_kiro_ide(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let token = get_kiro_local_token().await.ok_or("未找到本地 Kiro 账号，请先在 Kiro IDE 中登录")?;
    let refresh_token = token.refresh_token.clone().ok_or("本地账号缺少 refresh_token")?;
    let result = add_account_by_social(state, refresh_token, token.provider, token.access_token).await?;
    Ok(result)
}

// ===== Provider Commands =====

#[tauri::command]
fn get_providers(state: tauri::State<AppState>) -> serde_json::Value {
    let providers = state.providers.lock().unwrap();
    let provider_list: Vec<&Provider> = providers.values().collect();
    let active = provider_list.iter().find(|p| p.is_active);
    serde_json::json!({ "providers": provider_list, "active_provider": active })
}

#[tauri::command]
fn add_provider(state: tauri::State<AppState>, name: String, api_base_url: String, api_key: String, api_format: String) -> String {
    let now = chrono::Local::now().format("%Y/%m/%d %H:%M:%S").to_string();
    let provider = Provider { id: Uuid::new_v4().to_string(), name, api_base_url, api_key, api_format, is_active: false, created_at: now.clone(), updated_at: now };
    state.providers.lock().unwrap().insert(provider.id.clone(), provider.clone());
    provider.id
}

#[tauri::command]
fn switch_provider(state: tauri::State<AppState>, id: String) -> Result<Provider, String> {
    let mut providers = state.providers.lock().unwrap();
    for provider in providers.values_mut() { provider.is_active = false; }
    if let Some(provider) = providers.get_mut(&id) {
        provider.is_active = true;
        provider.updated_at = chrono::Local::now().format("%Y/%m/%d %H:%M:%S").to_string();
        Ok(provider.clone())
    } else { Err("Provider not found".to_string()) }
}

#[tauri::command]
fn delete_provider(state: tauri::State<AppState>, id: String) -> bool {
    state.providers.lock().unwrap().remove(&id).is_some()
}

#[tauri::command]
fn get_provider_presets() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({ "id": "anthropic", "name": "Anthropic (Official)", "api_base_url": "https://api.anthropic.com", "api_format": "anthropic", "description": "Official Anthropic API" }),
        serde_json::json!({ "id": "openai", "name": "OpenAI (Official)", "api_base_url": "https://api.openai.com", "api_format": "openai_chat", "description": "Official OpenAI API" }),
    ]
}

#[tauri::command]
fn get_gateway_config(state: tauri::State<AppState>) -> GatewayConfig {
    state.gateway_config.lock().unwrap().clone()
}

#[tauri::command]
fn update_gateway_config(state: tauri::State<AppState>, port: Option<u16>, api_key: Option<String>, default_model: Option<String>) -> bool {
    let mut config = state.gateway_config.lock().unwrap();
    if let Some(p) = port { config.port = p; }
    if let Some(key) = api_key { config.api_key = key; }
    if let Some(model) = default_model { config.default_model = Some(model); }
    true
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            get_accounts, add_account, update_account, delete_account, sync_account,
            export_accounts_command, import_accounts_command,
            get_kiro_local_token, add_account_by_social, import_from_kiro_ide,
            get_providers, add_provider, switch_provider, delete_provider, get_provider_presets,
            get_gateway_config, update_gateway_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
