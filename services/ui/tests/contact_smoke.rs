//! Contact form smoke tests
//!
//! Basic integration tests for contact form endpoints.
//! Tests endpoint availability and response structure without full PoW solving.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use deploy_baba_ui::build;
use deploy_baba_ui::state::AppState;
use tower::ServiceExt;

fn test_state() -> AppState {
    let temp_db = tempfile::NamedTempFile::new().unwrap();
    let db = std::sync::Arc::new(
        deploy_baba_ui::db::Db::open(temp_db.path().to_str().unwrap()).unwrap(),
    );
    std::env::set_var("COGNITO_POOL_ID", "");
    let auth = std::sync::Arc::new(deploy_baba_ui::auth::AuthConfig::from_env());
    let rag = std::sync::Arc::new(
        rag_sqlite::RagStore::new(rusqlite::Connection::open_in_memory().unwrap()).unwrap(),
    );
    AppState::with_defaults(db, auth, rag)
}

#[tokio::test]
async fn test_contact_challenge_endpoint() {
    deploy_baba_ui::routes::contact::init_pow_secret().await;

    let app = build(test_state());
    let request = Request::builder()
        .uri("/api/contact/challenge")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Challenge endpoint should return 200"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json.get("nonce").is_some(), "Response should include nonce");
    assert!(
        json.get("timestamp").is_some(),
        "Response should include timestamp"
    );
    assert!(
        json.get("difficulty").is_some(),
        "Response should include difficulty"
    );
    assert!(
        json.get("signature").is_some(),
        "Response should include signature"
    );
}

#[tokio::test]
async fn test_contact_post_endpoint_rejects_invalid_pow() {
    deploy_baba_ui::routes::contact::init_pow_secret().await;

    let app = build(test_state());
    let body = serde_json::json!({
        "name": "Test User",
        "email": "test@example.com",
        "subject": "Test Subject",
        "message": "Test message",
        "website": "",
        "pow_nonce": "invalid_nonce",
        "pow_timestamp": 1234567890_u64,
        "pow_solution": 0,
        "pow_signature": "invalid_signature"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/contact")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_ne!(
        response.status(),
        StatusCode::OK,
        "POST endpoint should reject invalid PoW solution"
    );
}

#[tokio::test]
async fn test_contact_post_endpoint_rejects_honeypot() {
    deploy_baba_ui::routes::contact::init_pow_secret().await;

    let app = build(test_state());
    let body = serde_json::json!({
        "name": "Bot User",
        "email": "bot@example.com",
        "subject": "Bot Subject",
        "message": "Bot message",
        "website": "http://spam.com",
        "pow_nonce": "some_nonce",
        "pow_timestamp": 1234567890_u64,
        "pow_solution": 0,
        "pow_signature": "some_signature"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/contact")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let resp_body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&resp_body).unwrap();
    assert!(
        json.get("success").is_some(),
        "Honeypot response should include success field"
    );
}
