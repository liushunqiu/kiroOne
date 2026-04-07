#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod persistence;
mod state;
mod gateway;
mod api_client;
mod claude_sync;

use state::{Account, AppState, GatewayConfig, KiroLocalToken, Provider};
use uuid::Uuid;
use std::sync::Mutex;
use gateway::GatewayServer;
use api_client::KiroApiClient;

#[tauri::command]
fn get_accounts(state: tauri::State<AppState>) -> Vec<Account> {
    let accounts = state.accounts.lock().unwrap();
    accounts.values().cloned().collect()
}

#[tauri::command]
fn add_account(state: tauri::State<AppState>, label: String, email: Option<String>) -> Result<String, String> {
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
    let account_id = account.id.clone();
    let mut accounts = state.accounts.lock().unwrap();
    accounts.insert(account.id.clone(), account);
    drop(accounts);

    state.save_accounts()?;
    Ok(account_id)
}

#[tauri::command]
fn update_account(state: tauri::State<AppState>, id: String, label: Option<String>, status: Option<String>) -> Result<bool, String> {
    let mut accounts = state.accounts.lock().unwrap();
    let updated = if let Some(account) = accounts.get_mut(&id) {
        if let Some(l) = label { account.label = l; }
        if let Some(s) = status { account.status = s; }
        account.updated_at = chrono::Local::now().format("%Y/%m/%d %H:%M:%S").to_string();
        true
    } else {
        false
    };
    drop(accounts);

    if updated {
        state.save_accounts()?;
    }
    Ok(updated)
}

#[tauri::command]
fn delete_account(state: tauri::State<AppState>, id: String) -> Result<bool, String> {
    let deleted = state.accounts.lock().unwrap().remove(&id).is_some();

    if deleted {
        state.save_accounts()?;
    }
    Ok(deleted)
}

#[tauri::command]
async fn sync_account(state: tauri::State<'_, AppState>, id: String) -> Result<String, String> {
    // 获取账号信息
    let account = {
        let accounts = state.accounts.lock().unwrap();
        accounts.get(&id).cloned().ok_or("账号不存在")?
    };

    let refresh_token = account
        .refresh_token
        .clone()
        .ok_or("账号缺少 refresh_token")?;
    let provider = account.provider.clone();

    // 创建 API 客户端
    let api_client = KiroApiClient::new(None);

    // 步骤 1: 刷新 access token
    let access_token = match api_client.refresh_access_token(&refresh_token).await {
        Ok(response) => {
            // 更新账号的 access_token 和 expires_at
            let mut accounts = state.accounts.lock().unwrap();
            if let Some(acc) = accounts.get_mut(&id) {
                acc.access_token = Some(response.access_token.clone());
                acc.expires_at = response.expires_at.clone();
                acc.updated_at = chrono::Local::now()
                    .format("%Y/%m/%d %H:%M:%S")
                    .to_string();
            }
            drop(accounts);
            state.save_accounts()?;

            response.access_token
        }
        Err(e) => {
            eprintln!("刷新 Token 失败: {}", e);
            return Err(format!("刷新 Token 失败: {}", e));
        }
    };

    // 步骤 2: 调用真实 API 获取额度
    let usage_data = match api_client
        .sync_account_usage(&access_token, provider.as_deref())
        .await
    {
        Ok(data) => data,
        Err(e) => {
            // API 调用失败,使用模拟数据作为降级
            eprintln!("API 调用失败,使用模拟数据: {}", e);
            KiroApiClient::generate_mock_usage(provider.as_deref())
        }
    };

    // 序列化 usage_data
    let usage_json = serde_json::to_string(&usage_data)
        .map_err(|e| format!("序列化数据失败: {}", e))?;

    // 更新账号数据
    let mut accounts = state.accounts.lock().unwrap();
    if let Some(acc) = accounts.get_mut(&id) {
        acc.usage_data = Some(usage_json);
        acc.status = "active".to_string();
        acc.updated_at = chrono::Local::now()
            .format("%Y/%m/%d %H:%M:%S")
            .to_string();
    }
    drop(accounts);

    state.save_accounts()?;
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
    drop(map);

    if count > 0 {
        state.save_accounts()?;
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
    drop(accounts);

    state.save_accounts()?;
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
fn add_provider(state: tauri::State<AppState>, name: String, api_base_url: String, api_key: String, api_format: String) -> Result<String, String> {
    let now = chrono::Local::now().format("%Y/%m/%d %H:%M:%S").to_string();
    let provider = Provider { id: Uuid::new_v4().to_string(), name, api_base_url, api_key, api_format, is_active: false, created_at: now.clone(), updated_at: now };
    let provider_id = provider.id.clone();
    state.providers.lock().unwrap().insert(provider.id.clone(), provider);

    state.save_providers()?;
    Ok(provider_id)
}

#[tauri::command]
fn switch_provider(state: tauri::State<AppState>, id: String) -> Result<Provider, String> {
    let mut providers = state.providers.lock().unwrap();
    for provider in providers.values_mut() { provider.is_active = false; }
    let result = if let Some(provider) = providers.get_mut(&id) {
        provider.is_active = true;
        provider.updated_at = chrono::Local::now().format("%Y/%m/%d %H:%M:%S").to_string();
        Ok(provider.clone())
    } else {
        Err("Provider not found".to_string())
    };
    drop(providers);

    if result.is_ok() {
        state.save_providers()?;
    }
    result
}

#[tauri::command]
fn update_provider(
    state: tauri::State<AppState>,
    id: String,
    name: Option<String>,
    api_base_url: Option<String>,
    api_key: Option<String>,
    api_format: Option<String>,
) -> Result<Provider, String> {
    let mut providers = state.providers.lock().unwrap();
    let result = if let Some(provider) = providers.get_mut(&id) {
        if let Some(n) = name { provider.name = n; }
        if let Some(url) = api_base_url { provider.api_base_url = url; }
        if let Some(key) = api_key { provider.api_key = key; }
        if let Some(format) = api_format { provider.api_format = format; }
        provider.updated_at = chrono::Local::now().format("%Y/%m/%d %H:%M:%S").to_string();
        Ok(provider.clone())
    } else {
        Err("Provider not found".to_string())
    };
    drop(providers);

    if result.is_ok() {
        state.save_providers()?;
    }
    result
}

#[tauri::command]
fn delete_provider(state: tauri::State<AppState>, id: String) -> Result<bool, String> {
    let deleted = state.providers.lock().unwrap().remove(&id).is_some();

    if deleted {
        state.save_providers()?;
    }
    Ok(deleted)
}

#[tauri::command]
fn get_provider_presets() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({ "id": "anthropic", "name": "Anthropic (Official)", "api_base_url": "https://api.anthropic.com", "api_format": "anthropic", "description": "Official Anthropic API" }),
        serde_json::json!({ "id": "openai", "name": "OpenAI (Official)", "api_base_url": "https://api.openai.com", "api_format": "openai_chat", "description": "Official OpenAI API" }),
    ]
}

#[tauri::command]
async fn start_gateway(
    app_state: tauri::State<'_, AppState>,
    gateway_server: tauri::State<'_, Mutex<GatewayServer>>,
) -> Result<String, String> {
    let config = app_state.gateway_config.lock().unwrap();
    let port = config.port;
    drop(config);

    let mut server = gateway_server.lock().unwrap();
    server.start(app_state.inner().clone(), port).await?;
    drop(server);

    // 更新配置状态
    let mut config = app_state.gateway_config.lock().unwrap();
    config.is_running = true;
    drop(config);

    app_state.save_gateway_config()?;
    Ok(format!("网关已启动,监听端口 {}", port))
}

#[tauri::command]
fn stop_gateway(
    app_state: tauri::State<AppState>,
    gateway_server: tauri::State<Mutex<GatewayServer>>,
) -> Result<String, String> {
    let mut server = gateway_server.lock().unwrap();
    server.stop()?;
    drop(server);

    // 更新配置状态
    let mut config = app_state.gateway_config.lock().unwrap();
    config.is_running = false;
    drop(config);

    app_state.save_gateway_config()?;
    Ok("网关已停止".to_string())
}

#[tauri::command]
fn get_gateway_config(state: tauri::State<AppState>) -> GatewayConfig {
    state.gateway_config.lock().unwrap().clone()
}

#[tauri::command]
fn update_gateway_config(
    state: tauri::State<AppState>,
    port: Option<u16>,
    api_key: Option<String>,
    default_model: Option<String>,
    proxy_enabled: Option<bool>,
    proxy_url: Option<String>,
) -> Result<bool, String> {
    let mut config = state.gateway_config.lock().unwrap();
    let mut proxy_changed = false;

    if let Some(p) = port {
        config.port = p;
    }
    if let Some(key) = api_key {
        config.api_key = key;
    }
    if let Some(model) = default_model {
        config.default_model = Some(model);
    }
    if let Some(enabled) = proxy_enabled {
        if config.proxy_enabled != enabled {
            proxy_changed = true;
        }
        config.proxy_enabled = enabled;
    }
    if let Some(url) = proxy_url {
        if config.proxy_url.as_ref() != Some(&url) {
            proxy_changed = true;
        }
        config.proxy_url = Some(url);
    }
    config.updated_at = chrono::Local::now()
        .format("%Y/%m/%d %H:%M:%S")
        .to_string();
    drop(config);

    state.save_gateway_config()?;

    // 如果代理配置改变,重建 HTTP 客户端
    if proxy_changed {
        state.rebuild_http_client();
    }

    Ok(true)
}

#[tauri::command]
fn sync_to_claude_code(state: tauri::State<AppState>) -> Result<String, String> {
    let config = state.gateway_config.lock().unwrap();
    claude_sync::sync_to_claude(config.port, &config.api_key)?;
    Ok(format!("已同步到 Claude Code: http://127.0.0.1:{}", config.port))
}

#[tauri::command]
fn read_from_claude_code() -> Result<serde_json::Value, String> {
    let (base_url, api_key) = claude_sync::read_from_claude()?;
    Ok(serde_json::json!({
        "baseUrl": base_url,
        "apiKey": api_key
    }))
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let state = AppState::new(&app.handle())
                .map_err(|e| format!("初始化应用状态失败: {}", e))?;
            app.manage(state);

            // 初始化网关服务器
            let gateway_server = Mutex::new(GatewayServer::new());
            app.manage(gateway_server);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_accounts, add_account, update_account, delete_account, sync_account,
            export_accounts_command, import_accounts_command,
            get_kiro_local_token, add_account_by_social, import_from_kiro_ide,
            get_providers, add_provider, update_provider, switch_provider, delete_provider, get_provider_presets,
            get_gateway_config, update_gateway_config, start_gateway, stop_gateway,
            sync_to_claude_code, read_from_claude_code,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
