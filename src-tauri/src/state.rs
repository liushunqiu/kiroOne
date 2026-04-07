use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use crate::persistence::DataStore;
use reqwest::Client;
use std::time::Duration;

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
    pub proxy_enabled: bool,
    pub proxy_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
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
    pub accounts: Arc<Mutex<HashMap<String, Account>>>,
    pub providers: Arc<Mutex<HashMap<String, Provider>>>,
    pub gateway_config: Arc<Mutex<GatewayConfig>>,
    pub data_store: Arc<DataStore>,
    pub http_client: Arc<Mutex<Client>>,
}

impl AppState {
    /// 根据网关配置构建 HTTP 客户端
    fn build_http_client(config: &GatewayConfig) -> Client {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10));

        // 应用代理配置
        if config.proxy_enabled {
            if let Some(proxy_url) = &config.proxy_url {
                if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                    builder = builder.proxy(proxy);
                }
            }
        }

        builder.build().unwrap_or_else(|_| Client::new())
    }

    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let data_store = Arc::new(DataStore::new(app)?);

        // 加载账号数据,失败时使用空 HashMap
        let accounts = Arc::new(Mutex::new(
            data_store
                .load::<HashMap<String, Account>>("accounts.json")
                .unwrap_or_default(),
        ));

        // 加载供应商数据
        let providers = Arc::new(Mutex::new(
            data_store
                .load::<HashMap<String, Provider>>("providers.json")
                .unwrap_or_default(),
        ));

        // 加载网关配置,失败时使用默认值
        let gateway_config = Arc::new(Mutex::new(
            data_store
                .load::<GatewayConfig>("gateway_config.json")
                .unwrap_or_else(|_| {
                    let now = chrono::Local::now()
                        .format("%Y/%m/%d %H:%M:%S")
                        .to_string();
                    GatewayConfig {
                        port: 8710,
                        api_key: uuid::Uuid::new_v4().to_string(),
                        is_running: false,
                        default_model: Some("claude-sonnet-4".to_string()),
                        proxy_enabled: false,
                        proxy_url: None,
                        created_at: now.clone(),
                        updated_at: now,
                    }
                }),
        ));

        // 创建共享 HTTP 客户端
        let http_client = {
            let config = gateway_config.lock().unwrap();
            Arc::new(Mutex::new(Self::build_http_client(&config)))
        };

        Ok(Self {
            accounts,
            providers,
            gateway_config,
            data_store,
            http_client,
        })
    }

    pub fn save_accounts(&self) -> Result<(), String> {
        let accounts = self.accounts.lock().unwrap();
        self.data_store.save("accounts.json", &*accounts)
    }

    pub fn save_providers(&self) -> Result<(), String> {
        let providers = self.providers.lock().unwrap();
        self.data_store.save("providers.json", &*providers)
    }

    pub fn save_gateway_config(&self) -> Result<(), String> {
        let config = self.gateway_config.lock().unwrap();
        self.data_store.save("gateway_config.json", &*config)
    }

    /// 重建 HTTP 客户端(当代理配置改变时调用)
    pub fn rebuild_http_client(&self) {
        let config = self.gateway_config.lock().unwrap();
        let new_client = Self::build_http_client(&config);
        let mut client = self.http_client.lock().unwrap();
        *client = new_client;
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            accounts: Arc::clone(&self.accounts),
            providers: Arc::clone(&self.providers),
            gateway_config: Arc::clone(&self.gateway_config),
            data_store: Arc::clone(&self.data_store),
            http_client: Arc::clone(&self.http_client),
        }
    }
}
