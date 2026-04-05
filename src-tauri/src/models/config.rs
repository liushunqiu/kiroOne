use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub gateway: GatewayConfig,
    pub proxy: Option<ProxyConfig>,
    pub claude_code: ClaudeCodeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub port: u16,
    pub host: String,
    pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyType {
    HTTP,
    SOCKS5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeConfig {
    pub auto_configure: bool,
    pub config_path: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            gateway: GatewayConfig {
                port: 8000,
                host: "127.0.0.1".to_string(),
                auto_start: true,
            },
            proxy: None,
            claude_code: ClaudeCodeConfig {
                auto_configure: true,
                config_path: None,
            },
        }
    }
}
