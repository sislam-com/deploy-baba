//! `POST /api/ask` — RAG Q&A over the deploy-baba codebase (W-RAG.6.1).
//!
//! # Availability
//!
//! The route requires:
//! 1. `RAG_PUBLIC_ENABLED=1` env var (gate for P3 rollout)
//! 2. A live Anthropic API key in `AppState.anthropic_api_key`
//!
//! Missing either returns a 503. Rate-limited per IP: default 2/min, overridable
//! via `ASK_RATE_LIMIT` env var (useful for local dev).
//!
//! # Flow
//!
//! ```text
//! POST /api/ask
//!   → check RAG_PUBLIC_ENABLED + api key
//!   → rate-limit check (per IP, ASK_RATE_LIMIT/min, default 2)
//!   → RagStore::retrieve (FTS5 BM25, top_k)
//!   → DefaultPromptAssembler::assemble
//!   → AnthropicProvider::generate
//!   → AskResponse { answer, citations, model, tokens }
//! ```

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, LazyLock, Mutex},
    time::Instant,
};

use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use llm_anthropic::AnthropicProvider;
use llm_core::{ChatMessage, GenerationConfig, LlmProvider, LlmRequest, MessageRole};
use rag_core::{DefaultPromptAssembler, PromptAssembler, Retriever};
use rag_sqlite::RagStore;

use crate::state::AppState;
pub use api_openapi::models::{AskCitation, AskRequest, AskResponse};

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
        (status = 503, description = "RAG not enabled or API key not configured"),
    )
)]
pub async fn ask(
    headers: HeaderMap,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    State(state): State<AppState>,
    State(rag): State<Arc<RagStore>>,
    Json(req): Json<AskRequest>,
) -> ApiResult<AskResponse> {
    let ip = extract_client_ip(&headers, connect_info.as_ref());
    // Gate 1: public enablement flag
    if std::env::var("RAG_PUBLIC_ENABLED").as_deref() != Ok("1") {
        return Err(err(StatusCode::SERVICE_UNAVAILABLE, "RAG Q&A not enabled"));
    }

    // Gate 2: API key presence
    let api_key = state
        .anthropic_api_key
        .as_ref()
        .ok_or_else(|| err(StatusCode::SERVICE_UNAVAILABLE, "LLM not configured"))?
        .as_str()
        .to_owned();

    // Gate 3: rate limit
    if !check_rate_limit(&ip) {
        return Err(err(
            StatusCode::TOO_MANY_REQUESTS,
            &format!("Rate limit exceeded ({}/min)", *RATE_LIMIT),
        ));
    }

    // Clamp top_k
    let top_k = req.top_k.clamp(1, 20);

    // Retrieve relevant chunks via FTS5 BM25
    let chunks = rag.retrieve(&req.query, top_k).await.map_err(|e| {
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

    // Generate via Anthropic
    let provider = AnthropicProvider::new(api_key);
    let llm_req = LlmRequest {
        model: provider.default_model().to_owned(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: bundle.user_message,
        }],
        system: Some(bundle.system_prompt),
        tools: vec![],
        grounding: None,
        config: GenerationConfig {
            max_tokens: 1024,
            temperature: 0.2,
            prompt_version: "ask-v1",
        },
    };

    let llm_resp = provider.generate(llm_req).await.map_err(|e| {
        tracing::error!("Anthropic generate failed: {e}");
        err(StatusCode::BAD_GATEWAY, &format!("LLM error: {e}"))
    })?;

    let citations: Vec<AskCitation> = bundle
        .citations
        .into_iter()
        .map(|c| AskCitation {
            kind: c.kind,
            path: c.path,
            sha: c.sha,
            ord: c.ord,
        })
        .collect();

    Ok(Json(AskResponse {
        answer: llm_resp.content,
        citations,
        model: llm_resp.model,
        input_tokens: llm_resp.input_tokens,
        output_tokens: llm_resp.output_tokens,
    }))
}

pub fn router() -> axum::Router<AppState> {
    use axum::routing::post;
    axum::Router::new().route("/ask", post(ask))
}
