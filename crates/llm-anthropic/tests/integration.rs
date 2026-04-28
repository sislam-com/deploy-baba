//! Integration tests for `llm-anthropic` — require a real Anthropic API key.
//!
//! All tests are marked `#[ignore]` and are skipped in CI unless the caller
//! explicitly opts in with `cargo test -- --ignored`.
//!
//! # Running locally
//!
//! ```bash
//! # With key from env:
//! ANTHROPIC_API_KEY=sk-ant-... cargo test -p llm-anthropic -- --ignored --nocapture
//!
//! # With key pulled from Secrets Manager (recommended):
//! just test-llm PROFILE
//! ```
//!
//! These tests use `max_tokens: 10` to minimise cost and latency.

use llm_anthropic::AnthropicProvider;
use llm_core::{ChatMessage, GenerationConfig, LlmProvider, LlmRequest, MessageRole};

/// Resolve the API key from env, returning `None` if not present.
/// Tests call `skip_without_key!()` to bail out gracefully.
fn api_key() -> Option<String> {
    std::env::var("ANTHROPIC_API_KEY").ok()
}

macro_rules! skip_without_key {
    () => {
        match api_key() {
            Some(k) => k,
            None => {
                eprintln!("ANTHROPIC_API_KEY not set — skipping live test");
                return;
            }
        }
    };
}

fn minimal_req(provider: &AnthropicProvider, content: &str) -> LlmRequest {
    LlmRequest {
        model: provider.default_model().to_owned(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: content.to_owned(),
        }],
        system: None,
        tools: vec![],
        grounding: None,
        config: GenerationConfig {
            max_tokens: 10,
            temperature: 0.0,
            prompt_version: "test-v1",
        },
    }
}

// ── provider_id / default_model ───────────────────────────────────────────

#[test]
fn provider_id_is_anthropic() {
    // No API call — safe to run in CI
    let provider = AnthropicProvider::new("dummy");
    assert_eq!(provider.provider_id(), "anthropic");
}

#[test]
fn default_model_is_haiku() {
    let provider = AnthropicProvider::new("dummy");
    assert!(
        provider.default_model().contains("haiku"),
        "default model should be haiku, got {}",
        provider.default_model()
    );
}

// ── live generate ─────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY — run with: just test-llm PROFILE"]
async fn live_generate_returns_non_empty_content() {
    let key = skip_without_key!();
    let provider = AnthropicProvider::new(key);

    let resp = provider
        .generate(minimal_req(&provider, "Reply with the single word: pong"))
        .await
        .expect("generate should succeed");

    assert!(
        !resp.content.is_empty(),
        "response content must not be empty"
    );
    assert!(resp.input_tokens > 0, "should report input token usage");
    assert!(resp.output_tokens > 0, "should report output token usage");
    assert_eq!(resp.model.as_str(), provider.default_model());
}

#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY — run with: just test-llm PROFILE"]
async fn live_generate_respects_max_tokens() {
    let key = skip_without_key!();
    let provider = AnthropicProvider::new(key);

    let req = LlmRequest {
        config: GenerationConfig {
            max_tokens: 5,
            temperature: 0.0,
            prompt_version: "test-v1",
        },
        ..minimal_req(&provider, "Count to one hundred.")
    };

    let resp = provider
        .generate(req)
        .await
        .expect("generate should succeed");

    assert!(
        resp.output_tokens <= 5,
        "output should be capped at max_tokens=5, got {}",
        resp.output_tokens
    );
}

#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY — run with: just test-llm PROFILE"]
async fn live_generate_invalid_key_returns_upstream_error() {
    use llm_core::LlmError;

    let provider = AnthropicProvider::new("sk-ant-invalid-key");
    let result = provider.generate(minimal_req(&provider, "ping")).await;

    match result {
        Err(LlmError::Upstream { .. }) => {} // expected
        Err(other) => panic!("expected Upstream error, got: {other:?}"),
        Ok(_) => panic!("expected error for invalid key, got success"),
    }
}
