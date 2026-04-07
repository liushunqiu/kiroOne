use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageBreakdown {
    #[serde(rename = "currentUsageWithPrecision")]
    pub current_usage_with_precision: Option<f64>,
    #[serde(rename = "usageLimitWithPrecision")]
    pub usage_limit_with_precision: Option<f64>,
    #[serde(rename = "currentUsage")]
    pub current_usage: Option<i64>,
    #[serde(rename = "usageLimit")]
    pub usage_limit: Option<i64>,
    pub percentage: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageData {
    #[serde(rename = "usageBreakdownList")]
    pub usage_breakdown_list: Vec<UsageBreakdown>,
    pub provider: Option<String>,
    #[serde(rename = "syncedAt")]
    pub synced_at: String,
}

pub struct KiroApiClient {
    client: Client,
    api_host: String,
}

impl KiroApiClient {
    pub fn new(api_host: Option<String>) -> Self {
        // 使用真实的 Kiro API 端点
        let api_host = api_host.unwrap_or_else(|| {
            "https://prod.us-east-1.codewhisperer.aws.dev".to_string()
        });

        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            api_host,
        }
    }

    /// 同步账号额度信息 (调用真实的 Kiro API)
    pub async fn sync_account_usage(
        &self,
        access_token: &str,
        provider: Option<&str>,
    ) -> Result<UsageData, String> {
        // 构建请求头 (参考 KiroaaS 的实现)
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-amz-target",
            "com.amazon.aws.codewhisperer.runtime.AmazonCodeWhispererService.GetUsageLimits"
                .parse()
                .unwrap(),
        );
        headers.insert(
            "Content-Type",
            "application/x-amz-json-1.0".parse().unwrap(),
        );
        headers.insert(
            "Authorization",
            format!("Bearer {}", access_token).parse().unwrap(),
        );

        // 构建请求体
        let body = serde_json::json!({
            "origin": "AI_EDITOR",
            "isEmailRequired": true
        });

        // 发送请求
        let response = self
            .client
            .post(&self.api_host)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

        // 检查响应状态
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "无法读取错误信息".to_string());
            return Err(format!("API 错误 {}: {}", status, error_text));
        }

        // 解析响应
        let api_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))?;

        // 提取 usageBreakdownList
        let usage_list = api_response
            .get("usageBreakdownList")
            .and_then(|v| v.as_array())
            .ok_or("响应中缺少 usageBreakdownList")?;

        // 转换为我们的数据结构
        let mut breakdown_list = Vec::new();
        for item in usage_list {
            let breakdown = UsageBreakdown {
                current_usage_with_precision: item
                    .get("currentUsageWithPrecision")
                    .and_then(|v| v.as_f64()),
                usage_limit_with_precision: item
                    .get("usageLimitWithPrecision")
                    .and_then(|v| v.as_f64()),
                current_usage: item.get("currentUsage").and_then(|v| v.as_i64()),
                usage_limit: item.get("usageLimit").and_then(|v| v.as_i64()),
                percentage: item.get("percentage").and_then(|v| v.as_f64()),
            };
            breakdown_list.push(breakdown);
        }

        Ok(UsageData {
            usage_breakdown_list: breakdown_list,
            provider: provider.map(|s| s.to_string()),
            synced_at: chrono::Local::now()
                .format("%Y/%m/%d %H:%M:%S")
                .to_string(),
        })
    }

    /// 刷新访问令牌
    pub async fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<RefreshTokenResponse, String> {
        let auth_endpoint = "https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken";

        let body = serde_json::json!({
            "refreshToken": refresh_token
        });

        let response = self
            .client
            .post(auth_endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("刷新令牌失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "无法读取错误信息".to_string());
            return Err(format!("刷新令牌失败 {}: {}", status, error_text));
        }

        let result: RefreshTokenResponse = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))?;

        Ok(result)
    }

    /// 生成模拟数据 (用于 API 不可用时的降级)
    pub fn generate_mock_usage(provider: Option<&str>) -> UsageData {
        UsageData {
            usage_breakdown_list: vec![UsageBreakdown {
                current_usage_with_precision: Some(150.5),
                usage_limit_with_precision: Some(1000.0),
                current_usage: Some(150),
                usage_limit: Some(1000),
                percentage: Some(15.05),
            }],
            provider: provider.map(|s| s.to_string()),
            synced_at: chrono::Local::now()
                .format("%Y/%m/%d %H:%M:%S")
                .to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<String>,
}
