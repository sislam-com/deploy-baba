use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ApiModel;

/// Proof-of-work challenge issued by `GET /api/contact/challenge`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChallengeResponse {
    /// Random hex nonce to embed in the PoW computation.
    pub nonce: String,
    /// Unix timestamp (seconds) when the challenge was issued.
    pub timestamp: i64,
    /// Number of leading zero bits required in `SHA256(nonce:solution)`.
    pub difficulty: u32,
    /// HMAC-SHA256 signature over `nonce:timestamp:difficulty`.
    pub signature: String,
}

impl ApiModel for ChallengeResponse {
    fn schema_name() -> &'static str {
        "ChallengeResponse"
    }
    fn example() -> Self {
        Self {
            nonce: "a3f8c1d2e4b5f607".to_string(),
            timestamp: 1712534400,
            difficulty: 18,
            signature: "deadbeef1234567890abcdef".to_string(),
        }
    }
}

/// Request body for `POST /api/contact`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ContactSubmitRequest {
    pub name: String,
    pub email: String,
    pub subject: String,
    pub message: String,
    /// Honeypot — must be empty for legitimate submissions.
    #[serde(default)]
    pub website: String,
    /// Nonce from the challenge response.
    pub pow_nonce: String,
    /// Timestamp from the challenge response.
    pub pow_timestamp: i64,
    /// Counter that satisfies the proof-of-work difficulty.
    pub pow_solution: u64,
    /// HMAC signature from the challenge response.
    pub pow_signature: String,
}

impl ApiModel for ContactSubmitRequest {
    fn schema_name() -> &'static str {
        "ContactSubmitRequest"
    }
    fn example() -> Self {
        Self {
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
            subject: "Hello".to_string(),
            message: "I'd love to chat about your work.".to_string(),
            website: String::new(),
            pow_nonce: "a3f8c1d2e4b5f607".to_string(),
            pow_timestamp: 1712534400,
            pow_solution: 123456,
            pow_signature: "deadbeef1234567890abcdef".to_string(),
        }
    }
}

/// Response from `POST /api/contact`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ContactResponse {
    pub success: bool,
    pub message: String,
}

impl ApiModel for ContactResponse {
    fn schema_name() -> &'static str {
        "ContactResponse"
    }
    fn example() -> Self {
        Self {
            success: true,
            message: "Message sent!".to_string(),
        }
    }
}
