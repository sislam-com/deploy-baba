//! Anthropic Claude adapter for `llm-core`.
//!
//! Implements [`LlmProvider`] against the Anthropic Messages API using
//! `reqwest` as the HTTP client. No Anthropic-specific SDK dependency —
//! direct HTTP keeps the dependency surface minimal and gives us full control
//! over serialisation.
//!
//! # Usage
//!
//! ```rust,no_run
//! use llm_anthropic::AnthropicProvider;
//! use llm_core::{LlmProvider, LlmRequest, GenerationConfig, ChatMessage, MessageRole};
//!
//! #[tokio::main]
//! async fn main() {
//!     let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap();
//!     let provider = AnthropicProvider::new(api_key);
//!
//!     let req = LlmRequest {
//!         model: provider.default_model().to_owned(),
//!         messages: vec![ChatMessage::text(MessageRole::User, "Hello!")],
//!         system: None,
//!         tools: vec![],
//!         grounding: None,
//!         config: GenerationConfig { max_tokens: 50, temperature: 0.5, prompt_version: "demo-v1" },
//!     };
//!     let resp = provider.generate(req).await.unwrap();
//!     println!("{}", resp.content);
//! }
//! ```

use async_trait::async_trait;
use llm_core::{
    grounding::assemble_grounded_prompt, LlmError, LlmProvider, LlmRequest, LlmResponse,
    MessageContent, StopReason, ToolCall,
};
use serde::{Deserialize, Serialize};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Default model for cost-efficient generation (fast, cheap).
pub const DEFAULT_MODEL: &str = "claude-haiku-4-5-20251001";
/// Upgrade model for higher-quality generation.
pub const UPGRADE_MODEL: &str = "claude-sonnet-4-6";

// ── Wire types for the Anthropic Messages API ─────────────────────────────

#[derive(Serialize)]
struct ApiRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<ApiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<ApiTool<'a>>,
    temperature: f32,
}

#[derive(Serialize)]
struct ApiMessage {
    role: String,
    content: serde_json::Value,
}

#[derive(Serialize)]
struct ApiTool<'a> {
    name: &'a str,
    description: &'a str,
    input_schema: &'a serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    content: Vec<ContentBlock>,
    #[serde(default)]
    stop_reason: Option<String>,
    model: String,
    usage: Usage,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

#[derive(Deserialize, Debug)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Deserialize, Debug)]
struct ApiError {
    error: ApiErrorBody,
}

#[derive(Deserialize, Debug)]
struct ApiErrorBody {
    #[serde(rename = "type")]
    kind: String,
    message: String,
}

// ── Provider ─────────────────────────────────────────────────────────────

/// Anthropic Claude implementation of [`LlmProvider`].
///
/// Constructed with a plain API key string. The key is loaded by the caller
/// (e.g. from AWS Secrets Manager via `init_api_key()` in `services/ui`) and
/// injected here — the adapter never reads environment variables or secrets
/// directly.
pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    /// Create a new provider with the given Anthropic API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: reqwest::Client::new(),
        }
    }
}

fn message_to_api(msg: &llm_core::ChatMessage) -> ApiMessage {
    let role = match msg.role {
        llm_core::MessageRole::User => "user",
        llm_core::MessageRole::Assistant => "assistant",
        llm_core::MessageRole::System => "user",
    };

    let content = match &msg.content {
        MessageContent::Text { text } => serde_json::Value::String(text.clone()),
        MessageContent::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => serde_json::json!([{
            "type": "tool_result",
            "tool_use_id": tool_use_id,
            "content": content,
            "is_error": is_error,
        }]),
    };

    ApiMessage {
        role: role.to_string(),
        content,
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn provider_id(&self) -> &'static str {
        "anthropic"
    }

    fn default_model(&self) -> &str {
        DEFAULT_MODEL
    }

    async fn generate(&self, req: LlmRequest) -> Result<LlmResponse, LlmError> {
        let req = assemble_grounded_prompt(req)?;

        let model = if req.model.is_empty() {
            DEFAULT_MODEL
        } else {
            req.model.as_str()
        };

        let messages: Vec<ApiMessage> = req.messages.iter().map(message_to_api).collect();

        let tools: Vec<ApiTool<'_>> = req
            .tools
            .iter()
            .map(|t| ApiTool {
                name: &t.name,
                description: &t.description,
                input_schema: &t.input_schema,
            })
            .collect();

        let body = ApiRequest {
            model,
            max_tokens: req.config.max_tokens,
            messages,
            system: req.system.as_deref(),
            tools,
            temperature: req.config.temperature,
        };

        let http_resp = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        let status = http_resp.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry = http_resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(5);
            return Err(LlmError::RateLimited {
                retry_after_secs: retry,
            });
        }

        if !status.is_success() {
            let text = http_resp
                .text()
                .await
                .unwrap_or_else(|_| status.to_string());
            if let Ok(api_err) = serde_json::from_str::<ApiError>(&text) {
                return Err(LlmError::Upstream {
                    message: format!("[{}] {}", api_err.error.kind, api_err.error.message),
                });
            }
            return Err(LlmError::Upstream { message: text });
        }

        let api: ApiResponse = http_resp
            .json()
            .await
            .map_err(|e| LlmError::Parse(e.to_string()))?;

        let mut text_content = String::new();
        let mut tool_calls = Vec::new();

        for block in api.content {
            match block {
                ContentBlock::Text { text } => {
                    if !text_content.is_empty() {
                        text_content.push('\n');
                    }
                    text_content.push_str(&text);
                }
                ContentBlock::ToolUse { id, name, input } => {
                    tool_calls.push(ToolCall {
                        id,
                        name,
                        arguments: input,
                    });
                }
            }
        }

        let stop_reason = match api.stop_reason.as_deref() {
            Some("end_turn") => StopReason::EndTurn,
            Some("max_tokens") => StopReason::MaxTokens,
            Some("tool_use") => StopReason::ToolUse,
            Some("stop_sequence") => StopReason::StopSequence,
            Some(other) => StopReason::Other(other.to_owned()),
            None => StopReason::EndTurn,
        };

        Ok(LlmResponse {
            content: text_content,
            tool_calls,
            input_tokens: api.usage.input_tokens,
            output_tokens: api.usage.output_tokens,
            model: api.model,
            stop_reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_core::{ChatMessage, LlmProvider, MessageRole};

    #[test]
    fn provider_id_is_anthropic() {
        let p = AnthropicProvider::new("test-key");
        assert_eq!(p.provider_id(), "anthropic");
    }

    #[test]
    fn default_model_is_haiku() {
        let p = AnthropicProvider::new("test-key");
        assert_eq!(p.default_model(), DEFAULT_MODEL);
        assert!(p.default_model().contains("haiku"));
    }

    #[test]
    fn message_to_api_text() {
        let msg = ChatMessage::text(MessageRole::User, "Hello");
        let api = message_to_api(&msg);
        assert_eq!(api.role, "user");
        assert_eq!(api.content, serde_json::Value::String("Hello".into()));
    }

    #[test]
    fn message_to_api_assistant() {
        let msg = ChatMessage::text(MessageRole::Assistant, "Hi");
        let api = message_to_api(&msg);
        assert_eq!(api.role, "assistant");
    }

    #[test]
    fn message_to_api_system_maps_to_user() {
        let msg = ChatMessage::text(MessageRole::System, "You are helpful");
        let api = message_to_api(&msg);
        assert_eq!(api.role, "user");
    }

    #[test]
    fn message_to_api_tool_result() {
        let msg = ChatMessage::tool_result("call-1", "result text", false);
        let api = message_to_api(&msg);
        assert_eq!(api.role, "user");
        let arr = api.content.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["type"], "tool_result");
        assert_eq!(arr[0]["tool_use_id"], "call-1");
        assert_eq!(arr[0]["content"], "result text");
        assert_eq!(arr[0]["is_error"], false);
    }

    #[test]
    fn message_to_api_error_tool_result() {
        let msg = ChatMessage::tool_result("call-2", "error msg", true);
        let api = message_to_api(&msg);
        let arr = api.content.as_array().unwrap();
        assert_eq!(arr[0]["is_error"], true);
    }

    #[test]
    fn api_response_parse_text() {
        let json = r#"{
            "content": [{"type": "text", "text": "Hello world"}],
            "stop_reason": "end_turn",
            "model": "claude-haiku-4-5-20251001",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;
        let resp: ApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.model, "claude-haiku-4-5-20251001");
        assert_eq!(resp.usage.input_tokens, 10);
        assert_eq!(resp.usage.output_tokens, 5);
        assert!(matches!(resp.content[0], ContentBlock::Text { .. }));
    }

    #[test]
    fn api_response_parse_tool_use() {
        let json = r#"{
            "content": [
                {"type": "text", "text": "Let me check."},
                {"type": "tool_use", "id": "tu_1", "name": "get_weather", "input": {"city": "NYC"}}
            ],
            "stop_reason": "tool_use",
            "model": "claude-haiku-4-5-20251001",
            "usage": {"input_tokens": 20, "output_tokens": 15}
        }"#;
        let resp: ApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.content.len(), 2);
        assert!(matches!(resp.content[1], ContentBlock::ToolUse { .. }));
        if let ContentBlock::ToolUse { id, name, input } = &resp.content[1] {
            assert_eq!(id, "tu_1");
            assert_eq!(name, "get_weather");
            assert_eq!(input["city"], "NYC");
        }
    }

    #[test]
    fn api_error_parse() {
        let json = r#"{
            "error": {"type": "invalid_request_error", "message": "Invalid API key"}
        }"#;
        let err: ApiError = serde_json::from_str(json).unwrap();
        assert_eq!(err.error.kind, "invalid_request_error");
        assert_eq!(err.error.message, "Invalid API key");
    }

    #[test]
    fn stop_reason_mapping() {
        assert!(matches!(
            match Some("end_turn") {
                Some("end_turn") => StopReason::EndTurn,
                Some("max_tokens") => StopReason::MaxTokens,
                Some("tool_use") => StopReason::ToolUse,
                Some("stop_sequence") => StopReason::StopSequence,
                Some(other) => StopReason::Other(other.to_owned()),
                None => StopReason::EndTurn,
            },
            StopReason::EndTurn
        ));
        assert!(matches!(
            match Some("tool_use") {
                Some("end_turn") => StopReason::EndTurn,
                Some("max_tokens") => StopReason::MaxTokens,
                Some("tool_use") => StopReason::ToolUse,
                Some("stop_sequence") => StopReason::StopSequence,
                Some(other) => StopReason::Other(other.to_owned()),
                None => StopReason::EndTurn,
            },
            StopReason::ToolUse
        ));
    }
}
