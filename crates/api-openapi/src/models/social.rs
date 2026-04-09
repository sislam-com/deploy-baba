use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ApiModel;

/// A social link as used in nav rendering (url + label only).
///
/// This is the same shape as `services/ui/src/db.rs::SocialLink`.
/// `db.rs` re-exports this type via `pub use api_openapi::models::social::SocialLink`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SocialLink {
    pub url: String,
    pub label: String,
}

impl ApiModel for SocialLink {
    fn schema_name() -> &'static str {
        "SocialLink"
    }
    fn example() -> Self {
        Self {
            url: "https://github.com/shantopagla".to_string(),
            label: "GitHub".to_string(),
        }
    }
}

/// Input for creating or updating a social link (admin).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SocialLinkInput {
    pub platform: String,
    pub url: String,
    pub label: String,
    pub icon: Option<String>,
    pub visible: bool,
    pub sort_order: i64,
}

impl ApiModel for SocialLinkInput {
    fn schema_name() -> &'static str {
        "SocialLinkInput"
    }
    fn example() -> Self {
        Self {
            platform: "github".to_string(),
            url: "https://github.com/shantopagla".to_string(),
            label: "GitHub".to_string(),
            icon: Some("github".to_string()),
            visible: true,
            sort_order: 1,
        }
    }
}

/// A full social link record returned by admin endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SocialLinkResponse {
    pub id: i64,
    pub platform: String,
    pub url: String,
    pub label: String,
    pub icon: Option<String>,
    pub visible: bool,
    pub sort_order: i64,
}

impl ApiModel for SocialLinkResponse {
    fn schema_name() -> &'static str {
        "SocialLinkResponse"
    }
    fn example() -> Self {
        Self {
            id: 1,
            platform: "github".to_string(),
            url: "https://github.com/shantopagla".to_string(),
            label: "GitHub".to_string(),
            icon: Some("github".to_string()),
            visible: true,
            sort_order: 1,
        }
    }
}
