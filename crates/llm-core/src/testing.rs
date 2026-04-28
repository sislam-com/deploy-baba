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
//!         messages: vec![ChatMessage { role: MessageRole::User, content: "Please rewrite this.".to_owned() }],
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
use crate::types::{LlmRequest, LlmResponse, StopReason};
use async_trait::async_trait;

/// A deterministic stub that returns canned responses without network calls.
///
/// Match rules are evaluated in insertion order; the first rule whose key is
/// a substring of the concatenated user message content wins. Falls back to
/// `default_response` if no rule matches.
#[derive(Debug, Clone, Default)]
pub struct StubLlmProvider {
    rules: Vec<(String, String)>,
    default_response: String,
}

impl StubLlmProvider {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            default_response: "stub response".to_owned(),
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
        let user_content: String = req
            .messages
            .iter()
            .filter(|m| matches!(m.role, crate::types::MessageRole::User))
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");

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
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: content.to_owned(),
            }],
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
