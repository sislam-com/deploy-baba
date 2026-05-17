use anyhow::Result;
use aws_sdk_s3::Client as S3Client;
use base64::Engine as _;
use flate2::read::GzDecoder;
use lambda_runtime::{service_fn, LambdaEvent};
use rag_sqlite::RagStore;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::{io::Read as _, sync::Arc};
use tower::ServiceExt as _;

mod auth;
mod db;
mod middleware;

mod openapi;
mod router;
mod routes;
mod state;
mod telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init_telemetry();

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

    routes::api::ask::init_anthropic_key().await;
    routes::api::ask::init_openai_key().await;
    let has_proxy = std::env::var("LLM_PROXY_LAMBDA_NAME").is_ok();
    let has_anthropic = routes::api::ask::get_anthropic_key().is_some();
    let has_openai = routes::api::ask::get_openai_key().is_some();
    tracing::info!(
        "→ LLM backend: proxy={has_proxy}, anthropic={has_anthropic}, openai={has_openai}",
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
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
        tracing::info!("→ http://localhost:3001");
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

    // Operational actions (EventBridge / manual invoke)
    match payload.get("action").and_then(|a| a.as_str()) {
        Some("backup") => handle_backup(&_state).await,
        Some("ingest-rag") => handle_ingest_rag(&_state).await,
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

    // Inject the API Gateway v2 sourceIp as a trusted header so handlers can
    // extract the real client IP behind CloudFront + API Gateway without
    // relying on x-forwarded-for (which can be spoofed or stripped).
    if let Some(source_ip) = payload
        .get("requestContext")
        .and_then(|rc| rc.get("http"))
        .and_then(|h| h.get("sourceIp"))
        .and_then(|v| v.as_str())
    {
        builder = builder.header("x-apigw-source-ip", source_ip);
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

// ── Operational handlers ──────────────────────────────────────────────────────

/// Back up the EFS SQLite database to S3.
///
/// Invoked by EventBridge on a schedule (`{"action":"backup","source":"eventbridge"}`).
async fn handle_backup(state: &state::AppState) -> Result<Value, lambda_runtime::Error> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write as _;

    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "/mnt/db/baba.db".to_string());
    let bucket = std::env::var("S3_BACKUP_BUCKET")
        .map_err(|_| "S3_BACKUP_BUCKET env var not set".to_string())?;

    let data = std::fs::read(&db_path).map_err(|e| format!("read {db_path}: {e}"))?;

    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    enc.write_all(&data).map_err(|e| format!("compress: {e}"))?;
    let compressed = enc.finish().map_err(|e| format!("compress finish: {e}"))?;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("time: {e}"))?
        .as_secs();
    let key = format!("db-backups/app-{ts}.db.gz");

    let aws_cfg = aws_config::load_from_env().await;
    let s3 = S3Client::new(&aws_cfg);
    s3.put_object()
        .bucket(&bucket)
        .key(&key)
        .body(aws_sdk_s3::primitives::ByteStream::from(compressed))
        .send()
        .await
        .map_err(|e| format!("s3 put: {e}"))?;

    tracing::info!("db backup → s3://{bucket}/{key}");
    let _ = state; // suppress unused-var warning
    Ok(json!({ "status": "ok", "key": key }))
}

/// Download a pre-indexed `rag-index.db.gz` from S3 and populate the EFS RAG store.
///
/// Uses ATTACH + bulk INSERT...SELECT + single FTS5 rebuild (O(n) vs the
/// per-document O(n²) rebuild that `RagStore::upsert_document` does in a loop).
///
/// Invoked manually after running `rag-index` locally and uploading to S3:
/// `{"action":"ingest-rag"}`
async fn handle_ingest_rag(_state: &state::AppState) -> Result<Value, lambda_runtime::Error> {
    let bucket = std::env::var("S3_BACKUP_BUCKET")
        .map_err(|_| "S3_BACKUP_BUCKET env var not set".to_string())?;
    let db_path = std::env::var("DB_PATH").unwrap_or_else(|_| "/mnt/db/baba.db".to_string());
    let key = "rag-index.db.gz";
    let tmp = "/tmp/rag-index.db";

    // 1. Download + decompress from S3
    tracing::info!("downloading rag index from s3://{bucket}/{key}");
    let aws_cfg = aws_config::load_from_env().await;
    let s3 = S3Client::new(&aws_cfg);
    let obj = s3
        .get_object()
        .bucket(&bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| format!("s3 get {key}: {e}"))?;

    let compressed = obj
        .body
        .collect()
        .await
        .map_err(|e| format!("read body: {e}"))?
        .into_bytes()
        .to_vec();

    let mut dec = GzDecoder::new(compressed.as_slice());
    let mut raw = Vec::new();
    dec.read_to_end(&mut raw)
        .map_err(|e| format!("decompress: {e}"))?;

    std::fs::write(tmp, &raw).map_err(|e| format!("write {tmp}: {e}"))?;
    tracing::info!("rag index written to {tmp} ({} bytes)", raw.len());

    // 2. Bulk-import via ATTACH + INSERT...SELECT + single FTS5 rebuild.
    //    Opens a second connection to the EFS DB (WAL mode allows concurrent writers).
    let conn = Connection::open(&db_path).map_err(|e| format!("open {db_path}: {e}"))?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .map_err(|e| format!("WAL pragma: {e}"))?;

    conn.execute_batch(&format!("ATTACH DATABASE '{tmp}' AS rag_src;"))
        .map_err(|e| format!("ATTACH: {e}"))?;

    // Clean-slate replace: DELETE first so plain INSERT...SELECT works on any SQLite version.
    // FTS5 rebuild at the end re-syncs rag_chunks_fts from rag_chunks.
    conn.execute_batch("DELETE FROM rag_chunks; DELETE FROM rag_documents;")
        .map_err(|e| format!("clear rag tables: {e}"))?;

    conn.execute_batch(
        "INSERT INTO rag_documents (source_kind, source_path, git_sha, updated_at)
         SELECT source_kind, source_path, git_sha, updated_at
         FROM rag_src.rag_documents;",
    )
    .map_err(|e| format!("insert documents: {e}"))?;

    conn.execute_batch(
        "INSERT INTO rag_chunks (document_id, ord, content, token_count, meta_json)
         SELECT d.id, sc.ord, sc.content, sc.token_count, sc.meta_json
         FROM rag_src.rag_chunks sc
         JOIN rag_src.rag_documents sd ON sd.id = sc.document_id
         JOIN rag_documents d ON d.source_kind = sd.source_kind
                              AND d.source_path = sd.source_path;",
    )
    .map_err(|e| format!("insert chunks: {e}"))?;

    conn.execute_batch("INSERT INTO rag_chunks_fts(rag_chunks_fts) VALUES('rebuild');")
        .map_err(|e| format!("fts rebuild: {e}"))?;

    conn.execute_batch("DETACH DATABASE rag_src;")
        .map_err(|e| format!("detach: {e}"))?;

    let docs: i64 = conn
        .query_row("SELECT COUNT(*) FROM rag_documents", [], |r| r.get(0))
        .map_err(|e| format!("count docs: {e}"))?;
    let chunks: i64 = conn
        .query_row("SELECT COUNT(*) FROM rag_chunks", [], |r| r.get(0))
        .map_err(|e| format!("count chunks: {e}"))?;

    tracing::info!("rag ingest complete: {docs} docs, {chunks} chunks");
    Ok(json!({ "status": "ok", "docs": docs, "chunks": chunks }))
}
