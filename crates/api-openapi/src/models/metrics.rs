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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_query_example_matches_schema_defaults() {
        let query = MetricsQuery::example();

        assert_eq!(MetricsQuery::schema_name(), "MetricsQuery");
        assert_eq!(query.endpoint, None);
        assert_eq!(query.hours, 24);
    }

    #[test]
    fn metrics_query_deserializes_missing_hours_to_default() {
        let query: MetricsQuery = serde_json::from_str(r#"{"endpoint":"/api/v1/jobs"}"#).unwrap();

        assert_eq!(query.endpoint.as_deref(), Some("/api/v1/jobs"));
        assert_eq!(query.hours, 24);
    }

    #[test]
    fn metrics_query_deserializes_explicit_hours() {
        let query: MetricsQuery = serde_json::from_str(r#"{"hours":48}"#).unwrap();

        assert_eq!(query.endpoint, None);
        assert_eq!(query.hours, 48);
    }
}
