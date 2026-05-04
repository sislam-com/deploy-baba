//! Test doubles for LLM providers.
//!
//! Use [`StubLlmProvider`] in integration tests to avoid real network calls.
//! Configure it with canned responses keyed on a substring of the user message.
//!
//! # Example
//!
//! ```rust
//! use llm_core::testing::StubLlmProvider;
//! use llm_core::{LlmProvider, LlmRequest, GenerationConfig, ChatMessage, MessageRole};
//!
//! #[tokio::test]
//! async fn test_with_stub() {
//!     let stub = StubLlmProvider::new()
//!         .with_response("rewrite", "Improved bullet text.");
//!
//!     let req = LlmRequest {
//!         model: String::new(),
//!         messages: vec![ChatMessage::text(MessageRole::User, "Please rewrite this.")],
//!         system: None,
//!         tools: vec![],
//!         grounding: None,
//!         config: GenerationConfig { max_tokens: 100, temperature: 0.0, prompt_version: "test-v1" },
//!     };
//!     let resp = stub.generate(req).await.unwrap();
//!     assert_eq!(resp.content, "Improved bullet text.");
//! }
//! ```

use crate::error::LlmError;
use crate::types::{LlmRequest, LlmResponse, StopReason, ToolCall};
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A deterministic stub that returns canned responses without network calls.
///
/// Match rules are evaluated in insertion order; the first rule whose key is
/// a substring of the concatenated user message content wins. Falls back to
/// `default_response` if no rule matches.
#[derive(Debug, Default)]
pub struct StubLlmProvider {
    rules: Vec<(String, String)>,
    default_response: String,
    tool_rules: Vec<(String, Vec<ToolCall>)>,
    always_tool_calls: Option<Vec<ToolCall>>,
    call_count: AtomicUsize,
}

impl StubLlmProvider {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            default_response: "stub response".to_owned(),
            tool_rules: Vec::new(),
            always_tool_calls: None,
            call_count: AtomicUsize::new(0),
        }
    }

    /// Add a match rule: if any user message contains `key`, return `response`.
    #[must_use]
    pub fn with_response(mut self, key: impl Into<String>, response: impl Into<String>) -> Self {
        self.rules.push((key.into(), response.into()));
        self
    }

    /// Set the response returned when no rule matches.
    #[must_use]
    pub fn with_default(mut self, response: impl Into<String>) -> Self {
        self.default_response = response.into();
        self
    }

    /// On the first call matching `key`, return a ToolUse stop reason with these tool calls.
    /// Subsequent calls fall through to text rules.
    #[must_use]
    pub fn with_tool_response(mut self, key: impl Into<String>, calls: Vec<ToolCall>) -> Self {
        self.tool_rules.push((key.into(), calls));
        self
    }

    /// Always return ToolUse stop reason (for testing max_turns exhaustion).
    #[must_use]
    pub fn with_tool_response_always(mut self, calls: Vec<ToolCall>) -> Self {
        self.always_tool_calls = Some(calls);
        self
    }
}

#[async_trait]
impl crate::LlmProvider for StubLlmProvider {
    fn provider_id(&self) -> &'static str {
        "stub"
    }

    fn default_model(&self) -> &str {
        "stub-model"
    }

    async fn generate(&self, req: LlmRequest) -> Result<LlmResponse, LlmError> {
        let call_num = self.call_count.fetch_add(1, Ordering::SeqCst);

        let user_content: String = req
            .messages
            .iter()
            .filter(|m| matches!(m.role, crate::types::MessageRole::User))
            .map(|m| m.text_content())
            .collect::<Vec<_>>()
            .join(" ");

        // Always-tool mode (for exhaustion testing)
        if let Some(calls) = &self.always_tool_calls {
            return Ok(LlmResponse {
                content: String::new(),
                tool_calls: calls.clone(),
                input_tokens: 0,
                output_tokens: 0,
                model: "stub-model".to_owned(),
                stop_reason: StopReason::ToolUse,
            });
        }

        // Tool rules: fire once (first call matching key)
        if call_num == 0 {
            for (key, calls) in &self.tool_rules {
                if user_content.contains(key.as_str()) {
                    return Ok(LlmResponse {
                        content: String::new(),
                        tool_calls: calls.clone(),
                        input_tokens: 0,
                        output_tokens: 0,
                        model: "stub-model".to_owned(),
                        stop_reason: StopReason::ToolUse,
                    });
                }
            }
        }

        // Text rules
        let content = self
            .rules
            .iter()
            .find(|(key, _)| user_content.contains(key.as_str()))
            .map(|(_, resp)| resp.clone())
            .unwrap_or_else(|| self.default_response.clone());

        Ok(LlmResponse {
            content,
            tool_calls: vec![],
            input_tokens: 0,
            output_tokens: 0,
            model: "stub-model".to_owned(),
            stop_reason: StopReason::EndTurn,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChatMessage, GenerationConfig, LlmProvider, LlmRequest, MessageRole};

    fn req(content: &str) -> LlmRequest {
        LlmRequest {
            model: String::new(),
            messages: vec![ChatMessage::text(MessageRole::User, content)],
            system: None,
            tools: vec![],
            grounding: None,
            config: GenerationConfig {
                max_tokens: 100,
                temperature: 0.0,
                prompt_version: "test-v1",
            },
        }
    }

    #[tokio::test]
    async fn matches_first_rule() {
        let stub = StubLlmProvider::new()
            .with_response("rewrite", "Rewritten text.")
            .with_response("summarise", "Summary text.");
        let resp = stub
            .generate(req("Please rewrite this bullet."))
            .await
            .unwrap();
        assert_eq!(resp.content, "Rewritten text.");
    }

    #[tokio::test]
    async fn falls_back_to_default() {
        let stub = StubLlmProvider::new().with_default("fallback");
        let resp = stub.generate(req("something unmatched")).await.unwrap();
        assert_eq!(resp.content, "fallback");
    }

    #[tokio::test]
    async fn provider_id_is_stub() {
        let stub = StubLlmProvider::new();
        assert_eq!(stub.provider_id(), "stub");
    }
}
