/// Shared primitives and error envelopes reused across the API.
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ApiModel;

/// Generic JSON error envelope returned on 4xx/5xx responses.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiError {
    /// Human-readable error message.
    pub message: String,
}

impl ApiModel for ApiError {
    fn schema_name() -> &'static str {
        "ApiError"
    }
    fn example() -> Self {
        Self {
            message: "Something went wrong".to_string(),
        }
    }
}
