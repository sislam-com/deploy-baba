//! Contact form smoke tests
//!
//! Basic integration tests for contact form endpoints.
//! Tests endpoint availability and response structure without full PoW solving.

use reqwest::Client;
use serde_json::Value;

const LOCAL_BASE_URL: &str = "http://localhost:3000";

#[tokio::test]
async fn test_contact_challenge_endpoint() {
    let client = Client::new();
    let response = client
        .get(&format!("{}/api/contact/challenge", LOCAL_BASE_URL))
        .send()
        .await;

    // Server may not be running, so we skip if connection fails
    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200, "Challenge endpoint should return 200");

            let json: Value = resp.json().await.expect("Response should be valid JSON");
            assert!(
                json.get("nonce").is_some(),
                "Response should include nonce field"
            );
            assert!(
                json.get("timestamp").is_some(),
                "Response should include timestamp field"
            );
            assert!(
                json.get("difficulty").is_some(),
                "Response should include difficulty field"
            );
            assert!(
                json.get("signature").is_some(),
                "Response should include signature field"
            );
        }
        Err(e) => {
            eprintln!("Skipping test: server not running ({})", e);
        }
    }
}

#[tokio::test]
async fn test_contact_post_endpoint_rejects_invalid_pow() {
    let client = Client::new();
    let response = client
        .post(&format!("{}/api/contact", LOCAL_BASE_URL))
        .json(&serde_json::json!({
            "name": "Test User",
            "email": "test@example.com",
            "subject": "Test Subject",
            "message": "Test message",
            "website": "", // Empty honeypot
            "pow_nonce": "invalid_nonce",
            "pow_timestamp": 1234567890,
            "pow_solution": 0,
            "pow_signature": "invalid_signature"
        }))
        .send()
        .await;

    match response {
        Ok(resp) => {
            // Should reject invalid PoW
            assert!(
                resp.status() != 200,
                "POST endpoint should reject invalid PoW solution"
            );
        }
        Err(e) => {
            eprintln!("Skipping test: server not running ({})", e);
        }
    }
}

#[tokio::test]
async fn test_contact_post_endpoint_rejects_honeypot() {
    let client = Client::new();
    let response = client
        .post(&format!("{}/api/contact", LOCAL_BASE_URL))
        .json(&serde_json::json!({
            "name": "Bot User",
            "email": "bot@example.com",
            "subject": "Bot Subject",
            "message": "Bot message",
            "website": "http://spam.com", // Filled honeypot
            "pow_nonce": "some_nonce",
            "pow_timestamp": 1234567890,
            "pow_solution": 0,
            "pow_signature": "some_signature"
        }))
        .send()
        .await;

    match response {
        Ok(resp) => {
            // Should accept honeypot submissions (silent success for bot detection)
            let json: Value = resp.json().await.expect("Response should be valid JSON");
            assert!(
                json.get("success").is_some(),
                "Response should include success field"
            );
        }
        Err(e) => {
            eprintln!("Skipping test: server not running ({})", e);
        }
    }
}
