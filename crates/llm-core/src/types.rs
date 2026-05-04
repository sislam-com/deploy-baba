use crate::grounding::GroundingContract;
use serde::{Deserialize, Serialize};

/// Content of a chat message — either plain text or a tool result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    Text {
        text: String,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(default)]
        is_error: bool,
    },
}

/// A single message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: MessageContent,
}

impl ChatMessage {
    pub fn text(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: MessageContent::Text {
                text: content.into(),
            },
        }
    }

    pub fn tool_result(
        tool_use_id: impl Into<String>,
        content: impl Into<String>,
        is_error: bool,
    ) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::ToolResult {
                tool_use_id: tool_use_id.into(),
                content: content.into(),
                is_error,
            },
        }
    }

    pub fn text_content(&self) -> &str {
        match &self.content {
            MessageContent::Text { text } => text,
            MessageContent::ToolResult { content, .. } => content,
        }
    }
}

/// Role of a message participant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Definition of a tool the model may call (structured output / tool-use).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    /// JSON Schema describing the tool's input parameters.
    pub input_schema: serde_json::Value,
}

/// A tool call produced by the model in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    /// Parsed arguments as returned by the model.
    pub arguments: serde_json::Value,
}

/// Parameters controlling generation behaviour.
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    /// Hard upper bound on output tokens.
    pub max_tokens: u32,
    /// Sampling temperature (0.0 = deterministic, 1.0 = max randomness).
    pub temperature: f32,
    /// Semantic version of the prompt. Must be bumped when prompt intent changes.
    /// Used as part of the `tailor_cache` key — see `plans/cross-cutting/llm-policy.md`.
    pub prompt_version: &'static str,
}

/// A fully assembled generation request.
#[derive(Debug, Clone)]
pub struct LlmRequest {
    /// Model name, e.g. `"claude-haiku-4-5-20251001"`. If empty, the provider's
    /// `default_model()` is used.
    pub model: String,
    /// Conversation turn(s). Must contain at least one `User` turn.
    pub messages: Vec<ChatMessage>,
    /// Optional system prompt injected before the conversation.
    pub system: Option<String>,
    /// Tools the model may invoke. Empty means text-only generation.
    pub tools: Vec<ToolDef>,
    /// If set, the prompt-assembly layer enforces the grounding contract before
    /// constructing the final prompt for the adapter.
    pub grounding: Option<GroundingContract>,
    pub config: GenerationConfig,
}

/// The model's response to a generation request.
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// Primary text content of the response.
    pub content: String,
    /// Any tool calls the model requested (empty for text-only responses).
    pub tool_calls: Vec<ToolCall>,
    /// Tokens consumed by the prompt.
    pub input_tokens: u32,
    /// Tokens in the generated response.
    pub output_tokens: u32,
    /// Actual model name used (may differ from requested if the provider remaps).
    pub model: String,
    pub stop_reason: StopReason,
}

/// Why the model stopped generating.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopReason {
    /// The model reached a natural stopping point.
    EndTurn,
    /// The `max_tokens` limit was hit before the model finished.
    MaxTokens,
    /// The model invoked a tool.
    ToolUse,
    /// A provider-specific stop sequence was matched.
    StopSequence,
    /// Unrecognised stop reason returned by the provider.
    Other(String),
}
