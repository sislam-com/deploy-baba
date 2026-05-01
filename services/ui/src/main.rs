use anyhow::Result;
use base64::Engine as _;
use lambda_runtime::{service_fn, LambdaEvent};
use rag_sqlite::RagStore;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tower::ServiceExt as _;

mod auth;
mod db;
mod middleware;
mod openapi;
mod router;
mod routes;
mod state;
mod sync;
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

    let spa_root =
        PathBuf::from(std::env::var("SPA_ROOT").unwrap_or_else(|_| "/mnt/spa/active".to_owned()));
    let spa_bucket = std::env::var("SPA_BUCKET").unwrap_or_default();
    tracing::info!("→ SPA root: {:?}", spa_root);

    let sdk_config = aws_config::load_from_env().await;
    let s3 = aws_sdk_s3::Client::new(&sdk_config);

    let app_state = state::AppState {
        db,
        auth: auth_config,
        anthropic_api_key,
        rag,
        spa_root,
        spa_bucket,
        s3,
    };

    let app = router::build(app_state.clone());

    if std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
        tracing::info!("→ Starting as AWS Lambda function");
        lambda_runtime::run(service_fn(move |event: LambdaEvent<Value>| {
            let state = app_state.clone();
            let app = app.clone();
            async move { dispatch(event, app, state).await }
        }))
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

async fn dispatch(
    event: LambdaEvent<Value>,
    app: axum::Router,
    state: state::AppState,
) -> Result<Value, lambda_runtime::Error> {
    let payload = event.payload;

    // Lambda Function URL / API GW v2 HTTP events always have requestContext.http
    if payload
        .get("requestContext")
        .and_then(|rc| rc.get("http"))
        .is_some()
    {
        return handle_http(payload, app).await;
    }

    match payload.get("action").and_then(|v| v.as_str()) {
        Some("sync-spa") => {
            let sp: sync::SyncPayload = serde_json::from_value(payload)
                .map_err(|e| format!("invalid sync payload: {e}"))?;
            let resp = sync::handle(sp, &state.s3, &state.spa_bucket)
                .await
                .map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(resp)?)
        }
        Some("prune") => {
            let keep = payload.get("keep").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
            let removed = sync::prune(keep).await.map_err(|e| e.to_string())?;
            Ok(json!({"status": "ok", "removed": removed}))
        }
        _ => Err(format!("unknown event: {payload:?}").into()),
    }
}

async fn handle_http(payload: Value, app: axum::Router) -> Result<Value, lambda_runtime::Error> {
    let method = payload["requestContext"]["http"]["method"]
        .as_str()
        .unwrap_or("GET");
    let path = payload["rawPath"].as_str().unwrap_or("/");
    let qs = payload["rawQueryString"].as_str().unwrap_or("");
    let uri = if qs.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{qs}")
    };

    let mut builder = axum::http::Request::builder().method(method).uri(&uri);

    if let Some(hdrs) = payload["headers"].as_object() {
        for (k, v) in hdrs {
            if let Some(v_str) = v.as_str() {
                builder = builder.header(k.as_str(), v_str);
            }
        }
    }

    let body_bytes: Vec<u8> = match payload.get("body") {
        Some(Value::String(s)) if !s.is_empty() => {
            if payload["isBase64Encoded"].as_bool().unwrap_or(false) {
                base64::engine::general_purpose::STANDARD
                    .decode(s)
                    .unwrap_or_default()
            } else {
                s.as_bytes().to_vec()
            }
        }
        _ => vec![],
    };

    let req = builder
        .body(axum::body::Body::from(body_bytes))
        .map_err(|e| format!("build request: {e}"))?;

    let resp = app.oneshot(req).await.map_err(|e| format!("axum: {e}"))?;

    let status = resp.status().as_u16();
    let mut headers_out = serde_json::Map::new();
    for (k, v) in resp.headers() {
        if let Ok(v_str) = v.to_str() {
            headers_out.insert(k.as_str().to_string(), Value::String(v_str.to_string()));
        }
    }

    let out_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .map_err(|e| format!("collect body: {e}"))?;

    let (body_str, is_b64) = match std::str::from_utf8(&out_bytes) {
        Ok(s) => (s.to_string(), false),
        Err(_) => (
            base64::engine::general_purpose::STANDARD.encode(&out_bytes),
            true,
        ),
    };

    Ok(json!({
        "statusCode": status,
        "headers": headers_out,
        "body": body_str,
        "isBase64Encoded": is_b64,
    }))
}
