//! Smoke tests for health endpoint
//!
//! Basic verification that the health endpoint responds correctly.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use deploy_baba_ui::build;

use deploy_baba_ui::state::AppState;
use tower::ServiceExt;

// Test 1: Health endpoint returns 200
#[tokio::test]
async fn test_health_endpoint() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db = std::sync::Arc::new(
        deploy_baba_ui::db::Db::open(temp_db.path().to_str().unwrap()).unwrap(),
    );

    // Use from_env() which defaults to dev_mode when COGNITO_POOL_ID is not set
    std::env::set_var("COGNITO_POOL_ID", "");
    let auth = std::sync::Arc::new(deploy_baba_ui::auth::AuthConfig::from_env());

    let rag = std::sync::Arc::new(
        rag_sqlite::RagStore::new(rusqlite::Connection::open_in_memory().unwrap()).unwrap(),
    );

    deploy_baba_ui::routes::contact::init_pow_secret().await;

    let state = AppState::with_defaults(db, auth, rag);
    let app = build(state);

    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// Test 2: Health endpoint returns JSON response
#[tokio::test]
async fn test_health_returns_json() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db = std::sync::Arc::new(
        deploy_baba_ui::db::Db::open(temp_db.path().to_str().unwrap()).unwrap(),
    );

    std::env::set_var("COGNITO_POOL_ID", "");
    let auth = std::sync::Arc::new(deploy_baba_ui::auth::AuthConfig::from_env());

    let rag = std::sync::Arc::new(
        rag_sqlite::RagStore::new(rusqlite::Connection::open_in_memory().unwrap()).unwrap(),
    );

    deploy_baba_ui::routes::contact::init_pow_secret().await;

    let state = AppState::with_defaults(db, auth, rag);
    let app = build(state);

    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json.is_object());
    assert!(json.get("status").is_some());
}

// Test 3: Health endpoint includes expected fields
#[tokio::test]
async fn test_health_includes_expected_fields() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db = std::sync::Arc::new(
        deploy_baba_ui::db::Db::open(temp_db.path().to_str().unwrap()).unwrap(),
    );

    std::env::set_var("COGNITO_POOL_ID", "");
    let auth = std::sync::Arc::new(deploy_baba_ui::auth::AuthConfig::from_env());

    let rag = std::sync::Arc::new(
        rag_sqlite::RagStore::new(rusqlite::Connection::open_in_memory().unwrap()).unwrap(),
    );

    deploy_baba_ui::routes::contact::init_pow_secret().await;

    let state = AppState::with_defaults(db, auth, rag);
    let app = build(state);

    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "ok");
}
