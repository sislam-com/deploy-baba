use rand::Rng;
use service_protocol::ServiceResponse;
use sha2::{Digest, Sha256};

pub async fn issue_challenge() -> ServiceResponse {
    let nonce: u64 = rand::thread_rng().gen();
    let challenge = format!("{:x}", Sha256::digest(nonce.to_le_bytes()));

    ServiceResponse::ok(serde_json::json!({
        "challenge": challenge,
        "difficulty": 4,
    }))
}
