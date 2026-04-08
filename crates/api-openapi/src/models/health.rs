use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ApiModel;

/// Service health check response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Status string, always `"ok"` when the service is healthy.
    pub status: String,
    /// Semver version of the deployed binary.
    pub version: String,
}

impl ApiModel for HealthResponse {
    fn schema_name() -> &'static str {
        "HealthResponse"
    }
    fn example() -> Self {
        Self {
            status: "ok".to_string(),
            version: "0.1.0".to_string(),
        }
    }
}
