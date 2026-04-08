use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ApiModel;

/// Information about a single deploy-baba library crate.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CrateInfo {
    /// Crate package name (e.g. `"api-openapi"`).
    pub name: String,
    /// Semver version string.
    pub version: String,
    /// Short description of the crate's purpose.
    pub description: String,
    /// Trait names this crate implements or exposes.
    pub traits: Vec<String>,
}

impl ApiModel for CrateInfo {
    fn schema_name() -> &'static str {
        "CrateInfo"
    }
    fn example() -> Self {
        Self {
            name: "api-openapi".to_string(),
            version: "0.1.0".to_string(),
            description: "OpenAPI 3.0 specification generator implementing api-core traits"
                .to_string(),
            traits: vec!["SpecGenerator".to_string(), "OpenApiCompat".to_string()],
        }
    }
}
