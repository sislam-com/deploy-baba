use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use super::ApiModel;

/// Request body for `POST /api/demo/config/parse`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParseConfigRequest {
    /// Format of the config content: `"toml"`, `"json"`, or `"yaml"`.
    pub format: String,
    /// Raw config text to parse.
    pub content: String,
}

impl ApiModel for ParseConfigRequest {
    fn schema_name() -> &'static str {
        "ParseConfigRequest"
    }
    fn example() -> Self {
        Self {
            format: "toml".to_string(),
            content: "[section]\nkey = \"value\"".to_string(),
        }
    }
}

/// Response from `POST /api/demo/config/parse`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParseConfigResponse {
    /// Whether parsing succeeded.
    pub success: bool,
    /// Parsed representation as JSON, if successful.
    pub parsed: Option<Value>,
    /// Human-readable error message, if parsing failed.
    pub error: Option<String>,
}

impl ApiModel for ParseConfigResponse {
    fn schema_name() -> &'static str {
        "ParseConfigResponse"
    }
    fn example() -> Self {
        Self {
            success: true,
            parsed: Some(serde_json::json!({"section": {"key": "value"}})),
            error: None,
        }
    }
}

/// A single typed field definition used in `GenerateSpecRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Field {
    /// Field name.
    pub name: String,
    /// JSON Schema primitive type: `"string"`, `"integer"`, `"boolean"`, etc.
    #[serde(rename = "type")]
    pub field_type: String,
    /// Whether this field is required in the generated schema.
    pub required: bool,
}

impl ApiModel for Field {
    fn schema_name() -> &'static str {
        "Field"
    }
    fn example() -> Self {
        Self {
            name: "email".to_string(),
            field_type: "string".to_string(),
            required: true,
        }
    }
}

/// Request body for `POST /api/demo/spec/generate`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GenerateSpecRequest {
    /// Title of the generated OpenAPI spec.
    pub title: String,
    /// Fields that form the resource schema.
    pub fields: Vec<Field>,
}

impl ApiModel for GenerateSpecRequest {
    fn schema_name() -> &'static str {
        "GenerateSpecRequest"
    }
    fn example() -> Self {
        Self {
            title: "User".to_string(),
            fields: vec![Field::example()],
        }
    }
}

/// Response from `POST /api/demo/spec/generate`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GenerateSpecResponse {
    /// The generated OpenAPI 3.0 spec as a JSON object.
    pub spec: Value,
}

impl ApiModel for GenerateSpecResponse {
    fn schema_name() -> &'static str {
        "GenerateSpecResponse"
    }
    fn example() -> Self {
        Self {
            spec: serde_json::json!({
                "openapi": "3.0.0",
                "info": {"title": "User", "version": "1.0.0"},
                "paths": {}
            }),
        }
    }
}
