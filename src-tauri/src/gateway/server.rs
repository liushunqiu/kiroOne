use axum::{
    extract::State,
    http::StatusCode,
    response::{Json, Response, IntoResponse},
    routing::{get, post},
    Router,
    body::Body,
    middleware,
};
use serde_json::json;
use std::sync::Arc;
use tokio::task::AbortHandle;
use tokio::time::{sleep, Duration};
use futures::stream::StreamExt;

use crate::state::AppState;
use super::streaming::{AwsEventStreamParser, SseFormatter};
use super::auth::auth_middleware;

/// 重试配置
struct RetryConfig {
    max_retries: u32,
    initial_delay_ms: u64,
    max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
        }
    }
}

/// 带指数退避的重试逻辑
async fn retry_with_backoff<F, Fut, T, E>(
    operation: F,
    config: RetryConfig,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay_ms;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;
                if attempt > config.max_retries {
                    eprintln!("重试失败,已达最大重试次数: {}", e);
                    return Err(e);
                }

                eprintln!("请求失败 (尝试 {}/{}): {}, {}ms 后重试",
                    attempt, config.max_retries, e, delay);

                sleep(Duration::from_millis(delay)).await;

                // 指数退避: 每次延迟翻倍,但不超过最大延迟
                delay = (delay * 2).min(config.max_delay_ms);
            }
        }
    }
}

pub struct GatewayServer {
    abort_handle: Option<AbortHandle>,
}

impl GatewayServer {
    pub fn new() -> Self {
        Self {
            abort_handle: None,
        }
    }

    pub async fn start(&mut self, state: AppState, port: u16) -> Result<(), String> {
        // 如果已经在运行,先停止
        if self.abort_handle.is_some() {
            return Err("网关已在运行".to_string());
        }

        // 创建需要认证的路由
        let protected_routes = Router::new()
            .route("/v1/models", get(list_models))
            .route("/v1/chat/completions", post(chat_completions))
            .route("/v1/messages", post(anthropic_messages))
            .route("/usage", get(get_usage))
            .route("/account", get(get_account))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ));

        // 创建公开路由 (不需要认证)
        let public_routes = Router::new()
            .route("/", get(root))
            .route("/health", get(health_check));

        // 合并路由
        let app = Router::new()
            .merge(public_routes)
            .merge(protected_routes)
            .with_state(state);

        let addr = format!("127.0.0.1:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("绑定端口 {} 失败: {}", port, e))?;

        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });

        self.abort_handle = Some(handle.abort_handle());
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(handle) = self.abort_handle.take() {
            handle.abort();
            Ok(())
        } else {
            Err("网关未运行".to_string())
        }
    }

    pub fn is_running(&self) -> bool {
        self.abort_handle.is_some()
    }
}

// ===== API 端点处理器 =====

async fn root() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "message": "Kiro One Gateway",
        "version": "0.1.0"
    }))
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "timestamp": chrono::Local::now().to_rfc3339()
    }))
}

async fn list_models(State(state): State<AppState>) -> Json<serde_json::Value> {
    let providers = state.providers.lock().unwrap();

    let models: Vec<serde_json::Value> = providers
        .values()
        .filter(|p| p.is_active)
        .map(|p| {
            json!({
                "id": format!("{}-model", p.name.to_lowercase()),
                "object": "model",
                "created": 1677610602,
                "owned_by": p.name,
                "permission": [],
                "root": format!("{}-model", p.name.to_lowercase()),
                "parent": null,
            })
        })
        .collect();

    Json(json!({
        "object": "list",
        "data": models
    }))
}

async fn chat_completions(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    // 检查是否为流式请求
    let is_streaming = payload
        .get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // 获取活跃的供应商
    let providers = state.providers.lock().unwrap();
    let active_provider = providers.values().find(|p| p.is_active);

    if active_provider.is_none() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    let provider = active_provider.unwrap().clone();
    drop(providers);

    // 获取一个有效的账号
    let accounts = state.accounts.lock().unwrap();
    let active_account = accounts
        .values()
        .find(|a| a.status == "active" && a.access_token.is_some());

    if active_account.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let account = active_account.unwrap().clone();
    drop(accounts);

    // 转换 OpenAI 格式到 Kiro 格式
    let kiro_payload = convert_openai_to_kiro(&payload);

    // 调用 Kiro API
    let kiro_api_host = "https://prod.us-east-1.codewhisperer.aws.dev";
    let access_token = account.access_token.as_ref().unwrap().clone();
    let model = payload
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // 使用重试逻辑包装 HTTP 请求
    let kiro_payload_clone = kiro_payload.clone();
    let state_clone = state.clone();
    let retry_config = RetryConfig::default();

    let response = retry_with_backoff(
        || async {
            let client = state_clone.http_client.lock().unwrap();
            client
                .post(kiro_api_host)
                .header("Authorization", format!("Bearer {}", access_token))
                .header(
                    "x-amz-target",
                    "com.amazon.aws.codewhisperer.runtime.AmazonCodeWhispererService.GenerateAssistantResponse",
                )
                .header("Content-Type", "application/x-amz-json-1.0")
                .json(&kiro_payload_clone)
                .send()
                .await
                .map_err(|e| e.to_string())
        },
        retry_config,
    )
    .await
    .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }

    if is_streaming {
        // 流式响应
        let stream = response.bytes_stream();
        let model_clone = model.clone();

        let sse_stream = stream.map(move |chunk_result| {
            match chunk_result {
                Ok(chunk) => {
                    // 解析 Kiro 流式响应
                    let mut parser = AwsEventStreamParser::new();
                    let events = parser.parse(&chunk);

                    let mut output = String::new();
                    let mut has_usage = false;

                    for event in events {
                        if event.event_type == "content" {
                            // 转换为 OpenAI SSE 格式
                            output.push_str(&SseFormatter::format_openai_chunk(
                                &event.data,
                                &model_clone,
                                None,
                            ));
                        } else if event.event_type == "usage" {
                            // 流结束,发送最后一个 chunk 和 [DONE]
                            has_usage = true;
                        }
                    }

                    // 如果收到 usage 事件,发送结束标记
                    if has_usage {
                        output.push_str(&SseFormatter::format_openai_chunk(
                            "",
                            &model_clone,
                            Some("stop"),
                        ));
                        output.push_str("data: [DONE]\n\n");
                    }

                    Ok::<_, std::io::Error>(output)
                }
                Err(e) => {
                    eprintln!("Stream error: {}", e);
                    Err(std::io::Error::new(std::io::ErrorKind::Other, e))
                }
            }
        });

        // 返回 SSE 响应
        let body = Body::from_stream(sse_stream);
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .body(body)
            .unwrap())
    } else {
        // 非流式响应
        let kiro_response: serde_json::Value = response
            .json()
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?;

        // 转换 Kiro 响应到 OpenAI 格式
        let openai_response = convert_kiro_to_openai(&kiro_response, &payload);

        Ok(Json(openai_response).into_response())
    }
}

async fn anthropic_messages(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Response, StatusCode> {
    // 检查是否为流式请求
    let is_streaming = payload
        .get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let providers = state.providers.lock().unwrap();
    let active_provider = providers.values().find(|p| p.is_active);

    if active_provider.is_none() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    let provider = active_provider.unwrap().clone();
    drop(providers);

    // 获取一个有效的账号
    let accounts = state.accounts.lock().unwrap();
    let active_account = accounts
        .values()
        .find(|a| a.status == "active" && a.access_token.is_some());

    if active_account.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let account = active_account.unwrap().clone();
    drop(accounts);

    // 转换 Anthropic 格式到 Kiro 格式
    let kiro_payload = convert_anthropic_to_kiro(&payload);

    // 调用 Kiro API
    let kiro_api_host = "https://prod.us-east-1.codewhisperer.aws.dev";
    let access_token = account.access_token.as_ref().unwrap().clone();

    // 使用重试逻辑包装 HTTP 请求
    let kiro_payload_clone = kiro_payload.clone();
    let state_clone = state.clone();
    let retry_config = RetryConfig::default();

    let response = retry_with_backoff(
        || async {
            let client = state_clone.http_client.lock().unwrap();
            client
                .post(kiro_api_host)
                .header("Authorization", format!("Bearer {}", access_token))
                .header(
                    "x-amz-target",
                    "com.amazon.aws.codewhisperer.runtime.AmazonCodeWhispererService.GenerateAssistantResponse",
                )
                .header("Content-Type", "application/x-amz-json-1.0")
                .json(&kiro_payload_clone)
                .send()
                .await
                .map_err(|e| e.to_string())
        },
        retry_config,
    )
    .await
    .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }

    if is_streaming {
        // 流式响应
        let stream = response.bytes_stream();

        let sse_stream = stream.enumerate().map(|(index, chunk_result)| {
            match chunk_result {
                Ok(chunk) => {
                    // 解析 Kiro 流式响应
                    let mut parser = AwsEventStreamParser::new();
                    let events = parser.parse(&chunk);

                    let mut output = String::new();

                    // 发送 message_start (仅第一次)
                    if index == 0 {
                        output.push_str(&SseFormatter::format_anthropic_chunk(
                            "message_start",
                            None,
                            None,
                        ));
                        output.push_str(&SseFormatter::format_anthropic_chunk(
                            "content_block_start",
                            None,
                            Some(0),
                        ));
                    }

                    let mut has_usage = false;
                    for event in events {
                        if event.event_type == "content" {
                            // 转换为 Anthropic SSE 格式
                            output.push_str(&SseFormatter::format_anthropic_chunk(
                                "content_block_delta",
                                Some(&event.data),
                                Some(0),
                            ));
                        } else if event.event_type == "usage" {
                            // 流结束
                            has_usage = true;
                        }
                    }

                    // 如果收到 usage 事件,发送结束标记
                    if has_usage {
                        output.push_str(&SseFormatter::format_anthropic_chunk(
                            "content_block_stop",
                            None,
                            Some(0),
                        ));
                        output.push_str(&SseFormatter::format_anthropic_chunk(
                            "message_delta",
                            None,
                            None,
                        ));
                        output.push_str(&SseFormatter::format_anthropic_chunk(
                            "message_stop",
                            None,
                            None,
                        ));
                    }

                    Ok::<_, std::io::Error>(output)
                }
                Err(e) => {
                    eprintln!("Stream error: {}", e);
                    Err(std::io::Error::new(std::io::ErrorKind::Other, e))
                }
            }
        });

        // 返回 SSE 响应
        let body = Body::from_stream(sse_stream);
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/event-stream")
            .header("Cache-Control", "no-cache")
            .header("Connection", "keep-alive")
            .body(body)
            .unwrap())
    } else {
        // 非流式响应
        let kiro_response: serde_json::Value = response
            .json()
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?;

        // 转换 Kiro 响应到 Anthropic 格式
        let anthropic_response = convert_kiro_to_anthropic(&kiro_response, &payload);

        Ok(Json(anthropic_response).into_response())
    }
}

async fn get_usage(State(state): State<AppState>) -> Json<serde_json::Value> {
    let accounts = state.accounts.lock().unwrap();

    let total_accounts = accounts.len();
    let active_accounts = accounts.values().filter(|a| a.status == "active").count();

    Json(json!({
        "total_accounts": total_accounts,
        "active_accounts": active_accounts,
        "timestamp": chrono::Local::now().to_rfc3339()
    }))
}

async fn get_account(State(state): State<AppState>) -> Json<serde_json::Value> {
    let accounts = state.accounts.lock().unwrap();
    let providers = state.providers.lock().unwrap();

    Json(json!({
        "accounts_count": accounts.len(),
        "providers_count": providers.len(),
        "active_provider": providers.values().find(|p| p.is_active).map(|p| &p.name),
        "timestamp": chrono::Local::now().to_rfc3339()
    }))
}

// ===== 格式转换函数 =====

fn convert_openai_to_kiro(openai_payload: &serde_json::Value) -> serde_json::Value {
    // 提取 messages
    let messages = openai_payload
        .get("messages")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // 转换为 Kiro 格式
    let mut kiro_messages = Vec::new();
    for msg in messages {
        let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("user");
        let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");

        kiro_messages.push(json!({
            "role": role,
            "content": [{
                "text": content
            }]
        }));
    }

    json!({
        "conversationState": {
            "conversationId": uuid::Uuid::new_v4().to_string(),
            "history": kiro_messages,
            "currentMessage": {
                "userInputMessage": {
                    "content": kiro_messages.last()
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.get(0))
                        .and_then(|t| t.get("text"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                }
            }
        }
    })
}

fn convert_kiro_to_openai(
    kiro_response: &serde_json::Value,
    original_request: &serde_json::Value,
) -> serde_json::Value {
    // 提取 Kiro 响应中的文本
    let content = kiro_response
        .get("generateAssistantResponseResponse")
        .and_then(|r| r.get("content"))
        .and_then(|v| v.as_str())
        .unwrap_or("无响应内容");

    let model = original_request
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    json!({
        "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        "object": "chat.completion",
        "created": chrono::Local::now().timestamp(),
        "model": model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 20,
            "total_tokens": 30
        }
    })
}

fn convert_anthropic_to_kiro(anthropic_payload: &serde_json::Value) -> serde_json::Value {
    // 类似 OpenAI 转换,但处理 Anthropic 特定格式
    let messages = anthropic_payload
        .get("messages")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut kiro_messages = Vec::new();
    for msg in messages {
        let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("user");
        let content = msg
            .get("content")
            .and_then(|v| {
                if v.is_string() {
                    v.as_str()
                } else if v.is_array() {
                    v.get(0)
                        .and_then(|c| c.get("text"))
                        .and_then(|t| t.as_str())
                } else {
                    None
                }
            })
            .unwrap_or("");

        kiro_messages.push(json!({
            "role": role,
            "content": [{
                "text": content
            }]
        }));
    }

    json!({
        "conversationState": {
            "conversationId": uuid::Uuid::new_v4().to_string(),
            "history": kiro_messages,
            "currentMessage": {
                "userInputMessage": {
                    "content": kiro_messages.last()
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.get(0))
                        .and_then(|t| t.get("text"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                }
            }
        }
    })
}

fn convert_kiro_to_anthropic(
    kiro_response: &serde_json::Value,
    original_request: &serde_json::Value,
) -> serde_json::Value {
    let content = kiro_response
        .get("generateAssistantResponseResponse")
        .and_then(|r| r.get("content"))
        .and_then(|v| v.as_str())
        .unwrap_or("无响应内容");

    let model = original_request
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    json!({
        "id": format!("msg_{}", uuid::Uuid::new_v4()),
        "type": "message",
        "role": "assistant",
        "content": [{
            "type": "text",
            "text": content
        }],
        "model": model,
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 10,
            "output_tokens": 20
        }
    })
}
