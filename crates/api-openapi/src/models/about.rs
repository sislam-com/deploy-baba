use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ApiModel;

/// Input for creating or updating an about section (admin).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AboutSectionInput {
    /// Which page this section belongs to: `"me"` or `"repo"`.
    pub page: String,
    pub slug: String,
    pub heading: String,
    pub body: String,
    pub icon: Option<String>,
    pub sort_order: i64,
}

impl ApiModel for AboutSectionInput {
    fn schema_name() -> &'static str {
        "AboutSectionInput"
    }
    fn example() -> Self {
        Self {
            page: "me".to_string(),
            slug: "background".to_string(),
            heading: "Background".to_string(),
            body: "I build zero-cost Rust systems on AWS.".to_string(),
            icon: Some("person".to_string()),
            sort_order: 1,
        }
    }
}

/// A persisted about section returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AboutSectionResponse {
    pub id: i64,
    pub page: String,
    pub slug: String,
    pub heading: String,
    pub body: String,
    pub icon: Option<String>,
    pub sort_order: i64,
}

impl ApiModel for AboutSectionResponse {
    fn schema_name() -> &'static str {
        "AboutSectionResponse"
    }
    fn example() -> Self {
        Self {
            id: 1,
            page: "me".to_string(),
            slug: "background".to_string(),
            heading: "Background".to_string(),
            body: "I build zero-cost Rust systems on AWS.".to_string(),
            icon: Some("person".to_string()),
            sort_order: 1,
        }
    }
}
