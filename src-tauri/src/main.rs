pub mod commands;
pub mod services;
pub mod models;
pub mod db;
pub mod utils;

use std::sync::Arc;
use tauri::Manager;
use crate::services::account_mgr::AccountManager;
use crate::models::config::AppConfig;
use crate::services::kiro_client::KiroClient;

#[derive(Clone)]
pub struct AppState {
    pub account_manager: Arc<AccountManager>,
    pub kiro_client: Arc<KiroClient>,
    pub config: Arc<AppConfig>,
}

fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("ai_unified_manager=info".parse().unwrap())
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            // 初始化数据库
            let db_path = app.path().app_data_dir().unwrap().join("ai-unified-manager.db");
            std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
            let db_conn = db::connection::init_db(&db_path).unwrap();

            // 初始化配置
            let config = Arc::new(AppConfig::default());
            let config_for_gateway = config.clone();

            // 初始化服务
            let account_manager = Arc::new(AccountManager::new(db_conn));
            let kiro_client = Arc::new(KiroClient::new("https://api.kiro.dev".to_string()));

            let app_state = AppState {
                account_manager,
                kiro_client,
                config,
            };

            app.manage(app_state);

            // 自动启动 API 网关
            if config_for_gateway.gateway.auto_start {
                let port = config_for_gateway.gateway.port;
                let state = app.state::<AppState>();
                let router = crate::services::http_server::create_router(state.inner().clone());

                tokio::spawn(async move {
                    let addr = format!("127.0.0.1:{}", port);
                    tracing::info!("🚀 API Gateway starting on {}", addr);

                    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
                    axum::serve(listener, router).await.unwrap();
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::accounts::get_accounts,
            commands::accounts::import_account,
            commands::accounts::switch_account,
            commands::accounts::delete_account,
            commands::gateway::start_gateway,
            commands::gateway::stop_gateway,
            commands::gateway::get_gateway_status,
            commands::claude_code::configure_claude_code,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
