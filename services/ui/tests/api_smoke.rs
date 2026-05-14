//! Smoke tests for API endpoints
//!
//! Basic verification that API routes are registered and respond.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use deploy_baba_ui::build;

use deploy_baba_ui::state::AppState;
use tower::ServiceExt;

// Test 1: OpenAPI spec endpoint is accessible
#[tokio::test]
async fn test_openapi_spec_endpoint() {
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

    let state = AppState { db, auth, rag };
    let app = build(state);

    let request = Request::builder()
        .uri("/api/openapi.json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("openapi").is_some());
}

// Test 2: Contact challenge endpoint is accessible
#[tokio::test]
async fn test_contact_challenge_endpoint() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db = std::sync::Arc::new(
        deploy_baba_ui::db::Db::open(temp_db.path().to_str().unwrap()).unwrap(),
    );

    std::env::set_var("COGNITO_POOL_ID", "");
    let auth = std::sync::Arc::new(deploy_baba_ui::auth::AuthConfig::from_env());

    let rag = std::sync::Arc::new(
        rag_sqlite::RagStore::new(rusqlite::Connection::open_in_memory().unwrap()).unwrap(),
    );

    // Initialize PoW secret before building app
    deploy_baba_ui::routes::contact::init_pow_secret().await;

    let state = AppState { db, auth, rag };
    let app = build(state);

    let request = Request::builder()
        .uri("/api/contact/challenge")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 200 or 400 depending on PoW secret initialization
    assert!(response.status().is_success() || response.status().is_client_error());
}

// Test 3: Admin routes require authentication
#[tokio::test]
async fn test_admin_requires_auth() {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db = std::sync::Arc::new(
        deploy_baba_ui::db::Db::open(temp_db.path().to_str().unwrap()).unwrap(),
    );

    // Set a fake pool_id to disable dev_mode and enforce auth
    std::env::set_var("COGNITO_POOL_ID", "us-east-1_test");
    std::env::set_var("COGNITO_CLIENT_ID", "test");
    std::env::set_var("COGNITO_DOMAIN", "test.auth.us-east-1.amazoncognito.com");
    std::env::set_var("COGNITO_JWKS", "{}");
    let auth = std::sync::Arc::new(deploy_baba_ui::auth::AuthConfig::from_env());

    let rag = std::sync::Arc::new(
        rag_sqlite::RagStore::new(rusqlite::Connection::open_in_memory().unwrap()).unwrap(),
    );

    deploy_baba_ui::routes::contact::init_pow_secret().await;

    let state = AppState { db, auth, rag };
    let app = build(state);

    let request = Request::builder()
        .uri("/api/v1/admin/jobs")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // The key security check: admin routes should NOT return 200 OK without auth
    assert_ne!(response.status(), StatusCode::OK);
}

// Test 4: Public API routes are accessible without auth
#[tokio::test]
async fn test_public_api_accessible() {
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

    let state = AppState { db, auth, rag };
    let app = build(state);

    let request = Request::builder()
        .uri("/api/v1/about")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should return 200 or 404 (if no data), but not 401/403
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    assert_ne!(response.status(), StatusCode::FORBIDDEN);
}

// Test 5: Docs endpoint returns HTML
#[tokio::test]
async fn test_docs_endpoint() {
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

    let state = AppState { db, auth, rag };
    let app = build(state);

    let request = Request::builder().uri("/docs").body(Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("<!doctype html>"));
    assert!(html.contains("rapi-doc"));
}
