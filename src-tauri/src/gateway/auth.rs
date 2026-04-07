use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::state::AppState;

/// API Key 认证中间件
///
/// 验证请求头中的 API Key:
/// - Authorization: Bearer {api_key}
/// - x-api-key: {api_key}
pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 获取配置的 API Key
    let config = state.gateway_config.lock().unwrap();
    let expected_api_key = config.api_key.clone();
    drop(config);

    // 检查 Authorization: Bearer 头
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let provided_key = &auth_str[7..]; // 跳过 "Bearer "
                if provided_key == expected_api_key {
                    return Ok(next.run(request).await);
                }
            }
        }
    }

    // 检查 x-api-key 头 (Anthropic 格式)
    if let Some(api_key_header) = headers.get("x-api-key") {
        if let Ok(provided_key) = api_key_header.to_str() {
            if provided_key == expected_api_key {
                return Ok(next.run(request).await);
            }
        }
    }

    // 认证失败
    Err(StatusCode::UNAUTHORIZED)
}

/// 健康检查端点不需要认证
pub fn is_public_endpoint(path: &str) -> bool {
    matches!(path, "/health" | "/")
}
