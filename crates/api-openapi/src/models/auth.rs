use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ApiModel;

/// Auth status returned by `GET /api/auth/me`.
///
/// The SPA calls this on mount to decide whether to show the dashboard gate.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthMe {
    pub authenticated: bool,
    pub email: Option<String>,
}

impl ApiModel for AuthMe {
    fn schema_name() -> &'static str {
        "AuthMe"
    }
    fn example() -> Self {
        Self {
            authenticated: true,
            email: Some("admin@example.com".to_string()),
        }
    }
}
