use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::ApiModel;

/// Query parameters for `GET /api/v1/metrics`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct MetricsQuery {
    /// Filter to a specific endpoint path (e.g. `/api/v1/jobs`).
    #[serde(default)]
    pub endpoint: Option<String>,
    /// Time window in hours (default 24).
    #[serde(default = "default_hours")]
    pub hours: u32,
}

fn default_hours() -> u32 {
    24
}

impl ApiModel for MetricsQuery {
    fn schema_name() -> &'static str {
        "MetricsQuery"
    }
    fn example() -> Self {
        Self {
            endpoint: None,
            hours: 24,
        }
    }
}
