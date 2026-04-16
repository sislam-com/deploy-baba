use anyhow::Result;
use rag_sqlite::RagStore;
use std::sync::{Arc, OnceLock};

mod auth;
mod db;
mod middleware;
mod openapi;
mod router;
mod routes;
mod state;
mod tailor;

// ─── Anthropic API key ────────────────────────────────────────────────────────
//
// Loaded once per cold start. Sources, in order:
//   1. ANTHROPIC_API_KEY_ARN env var → fetch from Secrets Manager (Lambda)
//   2. ANTHROPIC_API_KEY env var     → direct value (local dev / CI with key)
//   3. Absent                        → None (LLM routes return 503)

static ANTHROPIC_API_KEY: OnceLock<Option<String>> = OnceLock::new();

async fn init_anthropic_key() {
    if ANTHROPIC_API_KEY.get().is_some() {
        return;
    }

    let value = if let Ok(arn) = std::env::var("ANTHROPIC_API_KEY_ARN") {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_secretsmanager::Client::new(&config);
        match client.get_secret_value().secret_id(&arn).send().await {
            Ok(resp) => resp.secret_string().map(|s| s.to_string()),
            Err(e) => {
                tracing::error!("Failed to fetch ANTHROPIC_API_KEY from Secrets Manager: {e}");
                None
            }
        }
    } else if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        tracing::info!("→ ANTHROPIC_API_KEY loaded from env (dev mode)");
        Some(key)
    } else {
        tracing::warn!("ANTHROPIC_API_KEY_ARN and ANTHROPIC_API_KEY not set — LLM routes disabled");
        None
    };

    ANTHROPIC_API_KEY.set(value).ok();
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().without_time().init();

    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "deploy-baba.db".to_string());
    let db = Arc::new(db::Db::open(&db_path)?);
    tracing::info!("→ Database ready at {}", db_path);

    // Open a separate connection for the RAG store (WAL mode allows concurrent readers).
    let rag_conn = rusqlite::Connection::open(&db_path)?;
    let rag = Arc::new(RagStore::new(rag_conn).map_err(|e| anyhow::anyhow!("{e}"))?);
    tracing::info!("→ RAG store ready");

    let auth_config = Arc::new(auth::AuthConfig::from_env());
    tracing::info!("→ Auth ready (dev_mode={})", auth_config.dev_mode);

    routes::contact::init_pow_secret().await;
    tracing::info!("→ PoW secret ready");

    init_anthropic_key().await;
    let anthropic_api_key = ANTHROPIC_API_KEY
        .get()
        .and_then(|v| v.as_deref())
        .map(|s| Arc::new(s.to_owned()));
    tracing::info!(
        "→ Anthropic key ready (present={})",
        anthropic_api_key.is_some()
    );

    let app_state = state::AppState {
        db,
        auth: auth_config,
        anthropic_api_key,
        rag,
    };

    let app = router::build(app_state);

    if std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
        tracing::info!("→ Starting as AWS Lambda function");
        lambda_http::run(app)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    } else {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
        tracing::info!("→ http://localhost:3000");
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await?;
    }

    Ok(())
}
