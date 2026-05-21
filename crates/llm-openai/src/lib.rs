//! OpenAI adapter for `llm-core`.
//!
//! Implements [`LlmProvider`] against the OpenAI Chat Completions API using
//! `reqwest` as the HTTP client. No OpenAI-specific SDK dependency —
//! direct HTTP keeps the dependency surface minimal and gives us full control
//! over serialisation.
//!
//! # Usage
//!
//! ```rust,no_run
//! use llm_openai::OpenAIProvider;
//! use llm_core::{LlmProvider, LlmRequest, GenerationConfig, ChatMessage, MessageRole};
//!
//! #[tokio::main]
//! async fn main() {
//!     let api_key = std::env::var("OPENAI_API_KEY").unwrap();
//!     let provider = OpenAIProvider::new(api_key);
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
    grounding::assemble_grounded_prompt, EmbeddingProvider, LlmError, LlmProvider, LlmRequest,
    LlmResponse, MessageContent, StopReason, ToolCall,
};
use serde::{Deserialize, Serialize};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const OPENAI_EMBEDDINGS_URL: &str = "https://api.openai.com/v1/embeddings";

/// Default model for cost-efficient generation (fast, cheap).
pub const DEFAULT_MODEL: &str = "gpt-4o-mini";
/// Upgrade model for higher-quality generation.
pub const UPGRADE_MODEL: &str = "gpt-4o";
/// Default embedding model (1536 dimensions, $0.02/1M tokens).
pub const DEFAULT_EMBEDDING_MODEL: &str = "text-embedding-3-small";
/// Dimension of vectors produced by `text-embedding-3-small`.
pub const EMBEDDING_DIM: usize = 1536;

// ── Wire types for the OpenAI Chat Completions API ─────────────────────────────

#[derive(Serialize)]
struct ApiRequest<'a> {
    model: &'a str,
    messages: Vec<ApiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<ApiTool<'a>>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct ApiMessage {
    role: String,
    content: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tool_calls: Vec<ApiToolCall>,
}

#[derive(Serialize)]
struct ApiTool<'a> {
    #[serde(rename = "type")]
    tool_type: &'static str,
    function: ApiFunction<'a>,
}

#[derive(Serialize)]
struct ApiFunction<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a serde_json::Value,
}

#[derive(Serialize)]
struct ApiToolCall {
    id: String,
    #[serde(rename = "type")]
    tool_type: &'static str,
    function: ApiFunctionCall,
}

#[derive(Serialize)]
struct ApiFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    choices: Vec<Choice>,
    usage: Usage,
    model: String,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Message {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<ApiToolCallResponse>>,
}

#[derive(Deserialize, Debug)]
struct ApiToolCallResponse {
    id: String,
    function: ApiFunctionResponse,
}

#[derive(Deserialize, Debug)]
struct ApiFunctionResponse {
    name: String,
    arguments: String,
}

#[derive(Deserialize, Debug)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[derive(Deserialize, Debug)]
struct ApiError {
    error: ApiErrorBody,
}

#[derive(Deserialize, Debug)]
struct ApiErrorBody {
    message: String,
    #[serde(rename = "type")]
    kind: String,
}

// ── Provider ─────────────────────────────────────────────────────────────

/// OpenAI implementation of [`LlmProvider`].
///
/// Constructed with a plain API key string. The key is loaded by the caller
/// (e.g. from AWS Secrets Manager via `init_api_key()` in `services/ui`) and
/// injected here — the adapter never reads environment variables or secrets
/// directly.
pub struct OpenAIProvider {
    api_key: String,
    client: reqwest::Client,
}

impl OpenAIProvider {
    /// Create a new provider with the given OpenAI API key.
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
        llm_core::MessageRole::System => "system",
    };

    let mut api_msg = ApiMessage {
        role: role.to_string(),
        content: serde_json::Value::Null,
        tool_call_id: None,
        tool_calls: vec![],
    };

    match &msg.content {
        MessageContent::Text { text } => {
            api_msg.content = serde_json::Value::String(text.clone());
        }
        MessageContent::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => {
            api_msg.content = serde_json::Value::String(content.clone());
            api_msg.tool_call_id = Some(tool_use_id.clone());
            // OpenAI doesn't have an explicit is_error field in tool results
            // We include it in the content string if needed
            if *is_error {
                api_msg.content = serde_json::Value::String(format!("[ERROR] {}", content));
            }
        }
    }

    api_msg
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    fn provider_id(&self) -> &'static str {
        "openai"
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

        // Add system prompt as a separate message if provided
        let mut all_messages = messages;
        if let Some(system) = &req.system {
            all_messages.insert(
                0,
                ApiMessage {
                    role: "system".to_string(),
                    content: serde_json::Value::String(system.clone()),
                    tool_call_id: None,
                    tool_calls: vec![],
                },
            );
        }

        let tools: Vec<ApiTool<'_>> = req
            .tools
            .iter()
            .map(|t| ApiTool {
                tool_type: "function",
                function: ApiFunction {
                    name: &t.name,
                    description: &t.description,
                    parameters: &t.input_schema,
                },
            })
            .collect();

        let body = ApiRequest {
            model,
            messages: all_messages,
            system: None, // System is already in messages array
            tools,
            max_tokens: req.config.max_tokens,
            temperature: req.config.temperature,
        };

        let http_resp = self
            .client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        let choice = api
            .choices
            .first()
            .ok_or_else(|| LlmError::Other("No choices in response".to_string()))?;

        let text_content = choice.message.content.clone().unwrap_or_default();

        let tool_calls = if let Some(calls) = &choice.message.tool_calls {
            calls
                .iter()
                .map(|c| ToolCall {
                    id: c.id.clone(),
                    name: c.function.name.clone(),
                    arguments: serde_json::from_str(&c.function.arguments)
                        .unwrap_or(serde_json::Value::Null),
                })
                .collect()
        } else {
            vec![]
        };

        let stop_reason = match choice.finish_reason.as_deref() {
            Some("stop") => StopReason::EndTurn,
            Some("length") => StopReason::MaxTokens,
            Some("tool_calls") => StopReason::ToolUse,
            Some("content_filter") => StopReason::StopSequence,
            Some(other) => StopReason::Other(other.to_string()),
            None => StopReason::EndTurn,
        };

        Ok(LlmResponse {
            content: text_content,
            tool_calls,
            input_tokens: api.usage.prompt_tokens,
            output_tokens: api.usage.completion_tokens,
            model: api.model,
            stop_reason,
        })
    }
}

// ── Wire types for the OpenAI Embeddings API ─────────────────────────────────

#[derive(Serialize)]
struct EmbeddingApiRequest<'a> {
    model: &'a str,
    input: &'a [String],
}

#[derive(Deserialize, Debug)]
struct EmbeddingApiResponse {
    data: Vec<EmbeddingData>,
    #[allow(dead_code)]
    usage: EmbeddingUsage,
}

#[derive(Deserialize, Debug)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Deserialize, Debug)]
struct EmbeddingUsage {
    #[allow(dead_code)]
    prompt_tokens: u32,
    #[allow(dead_code)]
    total_tokens: u32,
}

// ── EmbeddingProvider ────────────────────────────────────────────────────────

#[async_trait]
impl EmbeddingProvider for OpenAIProvider {
    fn provider_id(&self) -> &'static str {
        "openai"
    }

    fn embedding_dim(&self) -> usize {
        EMBEDDING_DIM
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, LlmError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let body = EmbeddingApiRequest {
            model: DEFAULT_EMBEDDING_MODEL,
            input: texts,
        };

        let http_resp = self
            .client
            .post(OPENAI_EMBEDDINGS_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        let api: EmbeddingApiResponse = http_resp
            .json()
            .await
            .map_err(|e| LlmError::Parse(e.to_string()))?;

        // Return vectors sorted by input order (API may return out-of-order)
        let mut sorted: Vec<(usize, Vec<f32>)> = api
            .data
            .into_iter()
            .map(|d| (d.index, d.embedding))
            .collect();
        sorted.sort_by_key(|(idx, _)| *idx);

        Ok(sorted.into_iter().map(|(_, emb)| emb).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_core::{ChatMessage, EmbeddingProvider, LlmProvider, MessageRole};

    #[test]
    fn provider_id_is_openai() {
        let p = OpenAIProvider::new("test-key");
        assert_eq!(LlmProvider::provider_id(&p), "openai");
    }

    #[test]
    fn default_model_is_gpt4o_mini() {
        let p = OpenAIProvider::new("test-key");
        assert_eq!(p.default_model(), "gpt-4o-mini");
    }

    #[test]
    fn embedding_provider_id_is_openai() {
        let p = OpenAIProvider::new("test-key");
        assert_eq!(EmbeddingProvider::provider_id(&p), "openai");
    }

    #[test]
    fn embedding_dim_is_1536() {
        let p = OpenAIProvider::new("test-key");
        assert_eq!(p.embedding_dim(), 1536);
    }

    #[test]
    fn message_to_api_text() {
        let msg = ChatMessage::text(MessageRole::User, "hello");
        let api = message_to_api(&msg);
        assert_eq!(api.role, "user");
        assert_eq!(api.content, serde_json::Value::String("hello".into()));
        assert!(api.tool_call_id.is_none());
    }

    #[test]
    fn message_to_api_tool_result() {
        let msg = ChatMessage::tool_result("call-1", "result text", false);
        let api = message_to_api(&msg);
        assert_eq!(api.tool_call_id, Some("call-1".into()));
        assert_eq!(api.content, serde_json::Value::String("result text".into()));
    }

    #[test]
    fn message_to_api_error_tool_result() {
        let msg = ChatMessage::tool_result("call-2", "bad input", true);
        let api = message_to_api(&msg);
        assert_eq!(
            api.content,
            serde_json::Value::String("[ERROR] bad input".into())
        );
    }

    #[test]
    fn embedding_response_parse() {
        let json = r#"{
            "data": [
                {"embedding": [0.1, 0.2, 0.3], "index": 0},
                {"embedding": [0.4, 0.5, 0.6], "index": 1}
            ],
            "usage": {"prompt_tokens": 10, "total_tokens": 10}
        }"#;
        let resp: EmbeddingApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.data[0].embedding, vec![0.1, 0.2, 0.3]);
        assert_eq!(resp.data[1].index, 1);
        assert_eq!(resp.usage.prompt_tokens, 10);
    }

    #[test]
    fn embedding_response_reorder() {
        // Simulate API returning out of order
        let data = vec![
            EmbeddingData {
                embedding: vec![0.4, 0.5],
                index: 1,
            },
            EmbeddingData {
                embedding: vec![0.1, 0.2],
                index: 0,
            },
            EmbeddingData {
                embedding: vec![0.7, 0.8],
                index: 2,
            },
        ];
        let mut sorted: Vec<(usize, Vec<f32>)> =
            data.into_iter().map(|d| (d.index, d.embedding)).collect();
        sorted.sort_by_key(|(idx, _)| *idx);
        let result: Vec<Vec<f32>> = sorted.into_iter().map(|(_, emb)| emb).collect();

        assert_eq!(result[0], vec![0.1, 0.2]);
        assert_eq!(result[1], vec![0.4, 0.5]);
        assert_eq!(result[2], vec![0.7, 0.8]);
    }

    #[tokio::test]
    async fn embed_empty_returns_empty() {
        let p = OpenAIProvider::new("test-key");
        let result = p.embed(&[]).await.unwrap();
        assert!(result.is_empty());
    }
}
