//! `POST /api/ask` — RAG Q&A over the deploy-baba codebase (W-RAG.6.1).
//!
//! # Dual-mode (ADR-004)
//!
//! - **Lambda (production):** invokes the non-VPC llm-proxy Lambda via `LLM_PROXY_LAMBDA_NAME`.
//! - **Local dev:** calls the Anthropic API directly when `ANTHROPIC_API_KEY` is available
//!   (loaded at startup via `init_anthropic_key()`).
//!
//! The route requires `RAG_PUBLIC_ENABLED=1`. Rate-limited per IP: default 2/min,
//! overridable via `ASK_RATE_LIMIT` env var.

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, LazyLock, Mutex, OnceLock},
    time::Instant,
};

use aws_sdk_lambda::primitives::Blob;
use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use llm_anthropic::AnthropicProvider;
use llm_core::{ChatMessage, GenerationConfig, LlmProvider, LlmRequest, MessageRole};
use rag_core::{DefaultPromptAssembler, HybridRetriever, PromptAssembler, Retriever};
use rag_sqlite::RagStore;

use crate::db::Db;

pub use api_openapi::models::{
    AskCitation, AskProxyRequest, AskProxyResponse, AskRequest, AskResponse,
};

// ── Anthropic API key (local dev direct path) ────────────────────────────────

static ANTHROPIC_API_KEY: OnceLock<Option<String>> = OnceLock::new();

pub async fn init_anthropic_key() {
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
                    if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(&s) {
                        if let Some(v) = map.values().next().and_then(|v| v.as_str()) {
                            return v.to_string();
                        }
                    }
                }
                s
            }),
            Err(e) => {
                tracing::error!("Failed to fetch Anthropic key from Secrets Manager: {e}");
                None
            }
        }
    } else {
        std::env::var("ANTHROPIC_API_KEY").ok()
    };

    ANTHROPIC_API_KEY.set(value).ok();
}

pub fn get_anthropic_key() -> Option<&'static str> {
    ANTHROPIC_API_KEY.get().and_then(|v| v.as_deref())
}

// ── Rate limiter ──────────────────────────────────────────────────────────────

const RATE_WINDOW_SECS: u64 = 60;

static RATE_LIMIT: LazyLock<u32> = LazyLock::new(|| {
    std::env::var("ASK_RATE_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2)
});

struct RateEntry {
    count: u32,
    window_start: Instant,
}

static RATE_MAP: LazyLock<Mutex<HashMap<String, RateEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn check_rate_limit(ip: &str) -> bool {
    let limit = *RATE_LIMIT;
    let mut map = RATE_MAP.lock().unwrap();
    let now = Instant::now();
    let entry = map.entry(ip.to_string()).or_insert(RateEntry {
        count: 0,
        window_start: now,
    });
    if now.duration_since(entry.window_start).as_secs() >= RATE_WINDOW_SECS {
        entry.count = 0;
        entry.window_start = now;
    }
    if entry.count >= limit {
        return false;
    }
    entry.count += 1;
    true
}

fn extract_client_ip(
    headers: &HeaderMap,
    connect_info: Option<&ConnectInfo<SocketAddr>>,
) -> String {
    if let Some(ip) = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        return ip.to_string();
    }
    if let Some(ci) = connect_info {
        return ci.0.ip().to_string();
    }
    "unknown".to_string()
}

// ── Handler ───────────────────────────────────────────────────────────────────

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<serde_json::Value>)>;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": msg })))
}

#[utoipa::path(
    post,
    path = "/api/ask",
    tag = "ask",
    request_body = AskRequest,
    responses(
        (status = 200, description = "Grounded answer with citations", body = AskResponse),
        (status = 429, description = "Rate limit exceeded"),
        (status = 503, description = "RAG not enabled or no LLM backend available"),
    )
)]
pub async fn ask(
    headers: HeaderMap,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    State(rag): State<Arc<RagStore>>,
    State(db): State<Arc<Db>>,
    Json(req): Json<AskRequest>,
) -> ApiResult<AskResponse> {
    let ip = extract_client_ip(&headers, connect_info.as_ref());

    // Gate 1: public enablement flag
    if std::env::var("RAG_PUBLIC_ENABLED").as_deref() != Ok("1") {
        return Err(err(StatusCode::SERVICE_UNAVAILABLE, "RAG Q&A not enabled"));
    }

    // Gate 2: need either Lambda proxy name OR a direct Anthropic key
    let proxy_fn_name = std::env::var("LLM_PROXY_LAMBDA_NAME").ok();
    let direct_key = get_anthropic_key();
    if proxy_fn_name.is_none() && direct_key.is_none() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "No LLM backend configured (set LLM_PROXY_LAMBDA_NAME or ANTHROPIC_API_KEY_ARN)",
        ));
    }

    // Gate 3: rate limit
    if !check_rate_limit(&ip) {
        return Err(err(
            StatusCode::TOO_MANY_REQUESTS,
            &format!("Rate limit exceeded ({}/min)", *RATE_LIMIT),
        ));
    }

    // Clamp top_k
    let top_k = req.top_k.clamp(1, 20);

    // Retrieve relevant chunks via FTS5 BM25 + live portfolio data
    let hybrid = HybridRetriever {
        fts: Arc::clone(&rag),
        portfolio: Arc::clone(&db),
    };
    let chunks = hybrid.retrieve(&req.query, top_k).await.map_err(|e| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("retrieval failed: {e}"),
        )
    })?;

    if chunks.is_empty() {
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "No relevant source chunks found for this query",
        ));
    }

    // Assemble grounded prompt (ADR-016 citation format)
    let assembler = DefaultPromptAssembler;
    let bundle = assembler.assemble(&req.query, &chunks);

    let citations: Vec<AskCitation> = bundle
        .citations
        .iter()
        .map(|c| {
            let url = if c.kind == "portfolio" && c.sha == "live" {
                // Generate UI URLs for portfolio sources
                // Format: portfolio://{entity_type}/{slug} or portfolio://{entity_type}
                if let Some((entity_type, slug)) = c.path.strip_prefix("portfolio://").and_then(|s| {
                    let parts: Vec<&str> = s.splitn(2, '/').collect();
                    if parts.len() == 2 {
                        Some((parts[0], parts[1]))
                    } else if parts.len() == 1 {
                        Some((parts[0], ""))
                    } else {
                        None
                    }
                }) {
                    match entity_type {
                        "job" => format!("/?view=timeline#{}", slug),
                        "competency" => format!("/?view=capabilities#{}", slug),
                        "about" => "/?view=timeline".to_string(),
                        _ => "/?view=timeline".to_string(),
                    }
                } else {
                    "/?view=timeline".to_string()
                }
            } else {
                // GitHub URLs for code sources
                format!(
                    "https://github.com/shantopagla/deploy-baba/blob/{}/{}",
                    c.sha, c.path
                )
            };
            AskCitation {
                kind: c.kind.clone(),
                path: c.path.clone(),
                sha: c.sha.clone(),
                ord: c.ord,
                url,
            }
        })
        .collect();

    // Dual-mode generation (ADR-004):
    //   Production: invoke llm-proxy Lambda
    //   Local dev:  call Anthropic directly
    let proxy_resp = if let Some(fn_name) = proxy_fn_name {
        invoke_proxy_lambda(&fn_name, &bundle.system_prompt, &bundle.user_message).await?
    } else {
        generate_direct(
            direct_key.unwrap(),
            &bundle.system_prompt,
            &bundle.user_message,
        )
        .await?
    };

    Ok(Json(AskResponse {
        answer: proxy_resp.content,
        citations,
        model: proxy_resp.model,
        input_tokens: proxy_resp.input_tokens,
        output_tokens: proxy_resp.output_tokens,
    }))
}

async fn invoke_proxy_lambda(
    fn_name: &str,
    system_prompt: &str,
    user_message: &str,
) -> Result<AskProxyResponse, (StatusCode, Json<serde_json::Value>)> {
    let proxy_req = AskProxyRequest {
        system_prompt: system_prompt.to_owned(),
        user_message: user_message.to_owned(),
        max_tokens: 1024,
        temperature: 0.2,
        tools: vec![],
        api_base_url: std::env::var("PORTFOLIO_API_BASE_URL").ok(),
    };

    let payload_bytes = serde_json::to_vec(&proxy_req).map_err(|e| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("serialization failed: {e}"),
        )
    })?;

    let config = aws_config::load_from_env().await;
    let lambda_client = aws_sdk_lambda::Client::new(&config);

    let invoke_resp = lambda_client
        .invoke()
        .function_name(fn_name)
        .payload(Blob::new(payload_bytes))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("llm-proxy Lambda invocation failed: {e}");
            err(StatusCode::BAD_GATEWAY, "LLM proxy invocation failed")
        })?;

    invoke_resp
        .payload()
        .and_then(|blob| serde_json::from_slice::<AskProxyResponse>(blob.as_ref()).ok())
        .ok_or_else(|| err(StatusCode::BAD_GATEWAY, "Invalid response from LLM proxy"))
}

async fn generate_direct(
    api_key: &str,
    system_prompt: &str,
    user_message: &str,
) -> Result<AskProxyResponse, (StatusCode, Json<serde_json::Value>)> {
    let provider = AnthropicProvider::new(api_key.to_owned());
    let llm_req = LlmRequest {
        model: provider.default_model().to_owned(),
        messages: vec![ChatMessage::text(MessageRole::User, user_message)],
        system: Some(system_prompt.to_owned()),
        tools: vec![],
        grounding: None,
        config: GenerationConfig {
            max_tokens: 1024,
            temperature: 0.2,
            prompt_version: "ask-v1",
        },
    };

    let resp = provider.generate(llm_req).await.map_err(|e| {
        tracing::error!("Direct LLM generation failed: {e}");
        err(StatusCode::BAD_GATEWAY, &format!("LLM error: {e}"))
    })?;

    Ok(AskProxyResponse {
        content: resp.content,
        model: resp.model,
        input_tokens: resp.input_tokens,
        output_tokens: resp.output_tokens,
        tools_used: vec![],
        turns: 1,
    })
}

pub fn router() -> axum::Router<crate::state::AppState> {
    use axum::routing::post;
    axum::Router::new().route("/ask", post(ask))
}
