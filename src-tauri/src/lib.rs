// Library entry point for ai-unified-manager
// This file is needed because Cargo.toml specifies lib.name = "ai_unified_manager_lib"

pub mod models;
pub mod db;
pub mod services;
pub mod utils;
pub mod commands;

use std::sync::Arc;
use crate::services::account_mgr::AccountManager;
use crate::models::config::AppConfig;
use crate::services::kiro_client::KiroClient;

#[derive(Clone)]
pub struct AppState {
    pub account_manager: Arc<AccountManager>,
    pub kiro_client: Arc<KiroClient>,
    pub config: Arc<AppConfig>,
}
