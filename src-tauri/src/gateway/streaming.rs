use bytes::Bytes;
use futures::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

/// AWS EventStream 事件
#[derive(Debug, Clone)]
pub struct AwsEvent {
    pub event_type: String,
    pub data: String,
}

/// AWS EventStream 解析器
///
/// Kiro API 使用 AWS EventStream 格式返回流式响应
/// 格式: application/vnd.amazon.eventstream
pub struct AwsEventStreamParser {
    buffer: Vec<u8>,
}

impl AwsEventStreamParser {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    /// 解析 AWS EventStream 格式的数据
    ///
    /// AWS EventStream 是一个二进制协议,每个消息包含:
    /// - 4 bytes: total length
    /// - 4 bytes: headers length
    /// - headers
    /// - payload
    /// - 4 bytes: CRC
    pub fn parse(&mut self, chunk: &[u8]) -> Vec<AwsEvent> {
        self.buffer.extend_from_slice(chunk);
        let mut events = Vec::new();

        // 简化实现: 假设每个 chunk 包含完整的 JSON 行
        // 实际的 AWS EventStream 需要完整的二进制解析
        let text = String::from_utf8_lossy(&self.buffer);

        for line in text.lines() {
            if line.is_empty() {
                continue;
            }

            // 尝试解析为 JSON
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                // 提取事件类型和数据
                if let Some(content) = json.get("content").and_then(|v| v.as_str()) {
                    events.push(AwsEvent {
                        event_type: "content".to_string(),
                        data: content.to_string(),
                    });
                }

                if let Some(usage) = json.get("usage") {
                    events.push(AwsEvent {
                        event_type: "usage".to_string(),
                        data: usage.to_string(),
                    });
                }
            }
        }

        // 清空缓冲区
        self.buffer.clear();
        events
    }
}

/// SSE (Server-Sent Events) 格式化器
pub struct SseFormatter;

impl SseFormatter {
    /// 格式化为 SSE 事件
    ///
    /// SSE 格式:
    /// event: {event_type}
    /// data: {json_data}
    ///
    pub fn format(event_type: &str, data: &serde_json::Value) -> String {
        format!(
            "event: {}\ndata: {}\n\n",
            event_type,
            serde_json::to_string(data).unwrap_or_default()
        )
    }

    /// 格式化 OpenAI 流式响应
    pub fn format_openai_chunk(
        content: &str,
        model: &str,
        finish_reason: Option<&str>,
    ) -> String {
        let data = serde_json::json!({
            "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
            "object": "chat.completion.chunk",
            "created": chrono::Local::now().timestamp(),
            "model": model,
            "choices": [{
                "index": 0,
                "delta": {
                    "content": content
                },
                "finish_reason": finish_reason
            }]
        });

        format!("data: {}\n\n", serde_json::to_string(&data).unwrap())
    }

    /// 格式化 Anthropic 流式响应
    pub fn format_anthropic_chunk(
        event_type: &str,
        content: Option<&str>,
        index: Option<usize>,
    ) -> String {
        let data = match event_type {
            "message_start" => {
                serde_json::json!({
                    "type": "message_start",
                    "message": {
                        "id": format!("msg_{}", uuid::Uuid::new_v4()),
                        "type": "message",
                        "role": "assistant",
                        "content": [],
                        "model": "claude-sonnet-4",
                        "stop_reason": null,
                        "usage": {
                            "input_tokens": 0,
                            "output_tokens": 0
                        }
                    }
                })
            }
            "content_block_start" => {
                serde_json::json!({
                    "type": "content_block_start",
                    "index": index.unwrap_or(0),
                    "content_block": {
                        "type": "text",
                        "text": ""
                    }
                })
            }
            "content_block_delta" => {
                serde_json::json!({
                    "type": "content_block_delta",
                    "index": index.unwrap_or(0),
                    "delta": {
                        "type": "text_delta",
                        "text": content.unwrap_or("")
                    }
                })
            }
            "content_block_stop" => {
                serde_json::json!({
                    "type": "content_block_stop",
                    "index": index.unwrap_or(0)
                })
            }
            "message_delta" => {
                serde_json::json!({
                    "type": "message_delta",
                    "delta": {
                        "stop_reason": "end_turn"
                    },
                    "usage": {
                        "output_tokens": 0
                    }
                })
            }
            "message_stop" => {
                serde_json::json!({
                    "type": "message_stop"
                })
            }
            _ => serde_json::json!({}),
        };

        Self::format(event_type, &data)
    }
}
