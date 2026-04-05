use reqwest::Client;
use crate::models::message::{KiroRequest, KiroResponse, OpenAIRequest, AnthropicRequest, OpenAIResponse, AnthropicResponse};
use anyhow::Result;

pub struct KiroClient {
    client: Client,
    base_url: String,
}

impl KiroClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn chat(&self, request: KiroRequest, access_token: &str) -> Result<KiroResponse> {
        let response = self.client
            .post(format!("{}/v1/chat", self.base_url))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .json::<KiroResponse>()
            .await?;

        Ok(response)
    }

    // 将 OpenAI 格式转换为 Kiro 格式
    pub fn convert_openai_to_kiro(&self, request: &OpenAIRequest) -> KiroRequest {
        let prompt = request.messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        KiroRequest {
            prompt,
            model: request.model.clone(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
        }
    }

    // 将 Kiro 响应转换为 OpenAI 格式
    pub fn convert_kiro_to_openai(&self, kiro_response: &KiroResponse, original_request: &OpenAIRequest) -> OpenAIResponse {
        OpenAIResponse {
            id: format!("kiro-{}", uuid::Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: kiro_response.model.clone(),
            choices: vec![crate::models::message::Choice {
                index: 0,
                message: crate::models::message::Message {
                    role: "assistant".to_string(),
                    content: kiro_response.text.clone(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: kiro_response.usage.clone(),
        }
    }

    // 将 Anthropic 格式转换为 Kiro 格式
    pub fn convert_anthropic_to_kiro(&self, request: &AnthropicRequest) -> KiroRequest {
        let prompt = request.messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        KiroRequest {
            prompt: format!("{}\n\n{}", request.system.as_deref().unwrap_or(""), prompt),
            model: request.model.clone(),
            temperature: request.temperature,
            max_tokens: Some(request.max_tokens),
        }
    }

    // 将 Kiro 响应转换为 Anthropic 格式
    pub fn convert_kiro_to_anthropic(&self, kiro_response: &KiroResponse) -> AnthropicResponse {
        AnthropicResponse {
            id: format!("kiro-{}", uuid::Uuid::new_v4()),
            type_field: "message".to_string(),
            role: "assistant".to_string(),
            content: vec![crate::models::message::ContentBlock {
                type_field: "text".to_string(),
                text: kiro_response.text.clone(),
            }],
            model: kiro_response.model.clone(),
            stop_reason: Some("end_turn".to_string()),
            usage: kiro_response.usage.clone(),
        }
    }
}
