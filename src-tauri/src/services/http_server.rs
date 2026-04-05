use axum::{
    routing::{post, get},
    Router,
    Json,
    extract::State,
};
use crate::models::message::{OpenAIRequest, AnthropicRequest, OpenAIResponse, AnthropicResponse};
use crate::AppState;
use anyhow::Result;
use axum::http::StatusCode;

pub async fn chat_completions(
    State(state): State<AppState>,
    Json(request): Json<OpenAIRequest>,
) -> Result<Json<OpenAIResponse>, (StatusCode, String)> {
    // 获取当前活跃账号
    let account = state.account_manager
        .get_active_account()
        .await
        .map_err(|e: anyhow::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let account = account.ok_or((StatusCode::UNAUTHORIZED, "No active account".to_string()))?;

    // 检查 Token 是否过期
    if account.is_token_expired() {
        return Err((StatusCode::UNAUTHORIZED, "Token expired".to_string()));
    }

    // 转换为 Kiro 格式
    let kiro_request = state.kiro_client.convert_openai_to_kiro(&request);

    // 调用 Kiro API
    let access_token = account.access_token.ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "No access token".to_string()
    ))?;

    let kiro_response = state.kiro_client
        .chat(kiro_request, &access_token)
        .await
        .map_err(|e: anyhow::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 转换为 OpenAI 格式
    let response = state.kiro_client.convert_kiro_to_openai(&kiro_response, &request);

    Ok(Json(response))
}

pub async fn messages(
    State(state): State<AppState>,
    Json(request): Json<AnthropicRequest>,
) -> Result<Json<AnthropicResponse>, (StatusCode, String)> {
    // 获取当前活跃账号
    let account = state.account_manager
        .get_active_account()
        .await
        .map_err(|e: anyhow::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let account = account.ok_or((StatusCode::UNAUTHORIZED, "No active account".to_string()))?;

    // 检查 Token 是否过期
    if account.is_token_expired() {
        return Err((StatusCode::UNAUTHORIZED, "Token expired".to_string()));
    }

    // 转换为 Kiro 格式
    let kiro_request = state.kiro_client.convert_anthropic_to_kiro(&request);

    // 调用 Kiro API
    let access_token = account.access_token.ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "No access token".to_string()
    ))?;

    let kiro_response = state.kiro_client
        .chat(kiro_request, &access_token)
        .await
        .map_err(|e: anyhow::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 转换为 Anthropic 格式
    let response = state.kiro_client.convert_kiro_to_anthropic(&kiro_response);

    Ok(Json(response))
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/messages", post(messages))
        .route("/v1/models", get(list_models))
        .with_state(state)
}

pub async fn list_models() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "object": "list",
        "data": [
            {
                "id": "kiro-default",
                "object": "model",
                "created": chrono::Utc::now().timestamp(),
                "owned_by": "kiro"
            }
        ]
    }))
}
