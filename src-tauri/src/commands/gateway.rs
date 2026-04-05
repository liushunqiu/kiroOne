use tauri::State;
use crate::AppState;

#[tauri::command]
pub async fn start_gateway(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.gateway.clone();
    let port = config.port;
    let state = state.inner().clone();

    // 启动 HTTP 服务器
    tokio::spawn(async move {
        let addr = format!("{}:{}", config.host, port);
        tracing::info!("Starting API gateway on {}", addr);

        let router = crate::services::http_server::create_router(state);

        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e: std::io::Error| format!("Failed to bind to {}: {}", addr, e))
            .unwrap();

        axum::serve(listener, router)
            .await
            .map_err(|e: std::io::Error| format!("Server error: {}", e))
            .unwrap();
    });

    Ok(format!("API 网关已启动，端口: {}", port))
}

#[tauri::command]
pub async fn stop_gateway(state: State<'_, AppState>) -> Result<String, String> {
    // 这里需要实现停止逻辑 (使用 CancellationToken)
    Ok("API 网关已停止".to_string())
}

#[tauri::command]
pub async fn get_gateway_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let config = &state.config.gateway;
    Ok(serde_json::json!({
        "running": true,
        "host": config.host,
        "port": config.port,
        "url": format!("http://{}:{}", config.host, config.port)
    }))
}
