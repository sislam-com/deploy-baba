use anyhow::Result;
use base64::Engine as _;
use lambda_runtime::{service_fn, LambdaEvent};
use rag_sqlite::RagStore;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt as _;

mod auth;
mod db;
mod middleware;
mod openapi;
mod router;
mod routes;
mod state;
mod tailor;

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

    tracing::info!(
        "→ LLM proxy configured: {}",
        std::env::var("LLM_PROXY_LAMBDA_NAME").is_ok()
    );

    let app_state = state::AppState {
        db,
        auth: auth_config,
        rag,
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
    _state: state::AppState,
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

    Err(format!("unknown event: {payload:?}").into())
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
