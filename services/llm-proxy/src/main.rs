mod tool_executor;
mod tools;

use api_openapi::models::{AskProxyRequest, AskProxyResponse};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use llm_anthropic::AnthropicProvider;
use llm_core::{
    run_agent_loop, ChatMessage, GenerationConfig, LlmProvider, LlmRequest, MessageRole,
};
use llm_openai::OpenAIProvider;
use serde_json::Value;
use std::sync::OnceLock;

use crate::tool_executor::PortfolioToolExecutor;

// ─── Anthropic API key ────────────────────────────────────────────────────────
//
// Loaded once per cold start. Sources, in order:
//   1. ANTHROPIC_API_KEY_ARN env var → fetch from Secrets Manager (Lambda)
//   2. ANTHROPIC_API_KEY env var     → direct value (local dev)
//   3. Absent                        → handler returns an error

static ANTHROPIC_API_KEY: OnceLock<Option<String>> = OnceLock::new();

async fn init_anthropic_key() {
    if ANTHROPIC_API_KEY.get().is_some() {
        return;
    }

    let value = if let Ok(arn) = std::env::var("ANTHROPIC_API_KEY_ARN") {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_secretsmanager::Client::new(&config);
        match client.get_secret_value().secret_id(&arn).send().await {
            Ok(resp) => resp.secret_string().map(|s| {
                let s = s.trim().to_string();
                if s.starts_with('{') {
                    if let Ok(Value::Object(map)) = serde_json::from_str(&s) {
                        if let Some(v) = map.values().next().and_then(|v| v.as_str()) {
                            tracing::info!("→ ANTHROPIC_API_KEY unwrapped from JSON secret");
                            return v.to_string();
                        }
                    }
                }
                s
            }),
            Err(e) => {
                tracing::error!("Failed to fetch ANTHROPIC_API_KEY from Secrets Manager: {e}");
                None
            }
        }
    } else if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        tracing::info!("→ ANTHROPIC_API_KEY loaded from env (dev mode)");
        Some(key)
    } else {
        tracing::warn!("ANTHROPIC_API_KEY_ARN and ANTHROPIC_API_KEY not set");
        None
    };

    ANTHROPIC_API_KEY.set(value).ok();
}

// ─── OpenAI API key ───────────────────────────────────────────────────────────
//
// Loaded once per cold start. Sources, in order:
//   1. OPENAI_API_KEY_ARN env var → fetch from Secrets Manager (Lambda)
//   2. OPENAI_API_KEY env var     → direct value (local dev)
//   3. Absent                    → handler returns an error if provider is "openai"

static OPENAI_API_KEY: OnceLock<Option<String>> = OnceLock::new();

async fn init_openai_key() {
    if OPENAI_API_KEY.get().is_some() {
        return;
    }

    let value = if let Ok(arn) = std::env::var("OPENAI_API_KEY_ARN") {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_secretsmanager::Client::new(&config);
        match client.get_secret_value().secret_id(&arn).send().await {
            Ok(resp) => resp.secret_string().map(|s| {
                let s = s.trim().to_string();
                if s.starts_with('{') {
                    if let Ok(Value::Object(map)) = serde_json::from_str(&s) {
                        if let Some(v) = map.values().next().and_then(|v| v.as_str()) {
                            tracing::info!("→ OPENAI_API_KEY unwrapped from JSON secret");
                            return v.to_string();
                        }
                    }
                }
                s
            }),
            Err(e) => {
                tracing::error!("Failed to fetch OPENAI_API_KEY from Secrets Manager: {e}");
                None
            }
        }
    } else if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        tracing::info!("→ OPENAI_API_KEY loaded from env (dev mode)");
        Some(key)
    } else {
        tracing::warn!("OPENAI_API_KEY_ARN and OPENAI_API_KEY not set");
        None
    };

    OPENAI_API_KEY.set(value).ok();
}

// ─── Handler ──────────────────────────────────────────────────────────────────

async fn handler(event: LambdaEvent<AskProxyRequest>) -> Result<AskProxyResponse, Error> {
    let req = event.payload;
    let provider_id = req.provider.as_str();

    let (provider, provider_name) = match provider_id {
        "anthropic" => {
            let api_key = ANTHROPIC_API_KEY
                .get()
                .and_then(|v| v.as_deref())
                .ok_or("Anthropic API key not configured")?
                .to_owned();
            (
                Box::new(AnthropicProvider::new(api_key)) as Box<dyn LlmProvider>,
                "anthropic",
            )
        }
        "openai" => {
            let api_key = OPENAI_API_KEY
                .get()
                .and_then(|v| v.as_deref())
                .ok_or("OpenAI API key not configured")?
                .to_owned();
            (
                Box::new(OpenAIProvider::new(api_key)) as Box<dyn LlmProvider>,
                "openai",
            )
        }
        _ => {
            return Err(format!("Unknown provider: {provider_id}").into());
        }
    };

    let llm_req = LlmRequest {
        model: provider.default_model().to_owned(),
        messages: vec![ChatMessage::text(MessageRole::User, req.user_message)],
        system: Some(req.system_prompt),
        tools: vec![],
        grounding: None,
        config: GenerationConfig {
            max_tokens: req.max_tokens,
            temperature: req.temperature,
            prompt_version: "ask-v1",
        },
    };

    if !req.tools.is_empty() {
        let base_url = req
            .api_base_url
            .ok_or("api_base_url required when tools are provided")?;
        let executor = PortfolioToolExecutor::new(base_url);
        let result = run_agent_loop(&*provider, &executor, llm_req, 5, 4000)
            .await
            .map_err(|e| format!("Agent loop error: {e}"))?;

        return Ok(AskProxyResponse {
            content: result.final_content,
            model: result.model,
            input_tokens: result.total_input_tokens,
            output_tokens: result.total_output_tokens,
            tools_used: result
                .tool_calls_made
                .iter()
                .map(|(c, _)| c.name.clone())
                .collect(),
            turns: result.turns as u32,
            provider: provider_name.to_string(),
        });
    }

    let resp = provider
        .generate(llm_req)
        .await
        .map_err(|e| format!("LLM error: {e}"))?;

    Ok(AskProxyResponse {
        content: resp.content,
        model: resp.model,
        input_tokens: resp.input_tokens,
        output_tokens: resp.output_tokens,
        tools_used: vec![],
        turns: 1,
        provider: provider_name.to_string(),
    })
}

// ─── Entry point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("RUST_LOG"))
        .json()
        .init();

    init_anthropic_key().await;
    init_openai_key().await;
    tracing::info!(
        "→ Anthropic key ready (present={})",
        ANTHROPIC_API_KEY.get().and_then(|v| v.as_ref()).is_some()
    );
    tracing::info!(
        "→ OpenAI key ready (present={})",
        OPENAI_API_KEY.get().and_then(|v| v.as_ref()).is_some()
    );

    run(service_fn(handler)).await
}
