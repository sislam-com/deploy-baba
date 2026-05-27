use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ApiModel;

/// A legal document (Terms of Service, Privacy Policy, etc.).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LegalDocumentResponse {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub content: String,
    pub updated_at: String,
}

impl ApiModel for LegalDocumentResponse {
    fn schema_name() -> &'static str {
        "LegalDocumentResponse"
    }
    fn example() -> Self {
        Self {
            id: 1,
            slug: "terms".to_string(),
            title: "Terms of Service".to_string(),
            content: "These terms govern use of this site.".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }
}
