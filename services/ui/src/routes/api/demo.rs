use axum::{extract::Json, routing::post, Router};
use serde_json::Value;

use crate::state::AppState;

pub use api_openapi::models::{
    GenerateSpecRequest, GenerateSpecResponse, ParseConfigRequest, ParseConfigResponse,
};

#[utoipa::path(
    post,
    path = "/api/demo/config/parse",
    tag = "demo",
    request_body = ParseConfigRequest,
    responses(
        (status = 200, description = "Config parsed successfully", body = ParseConfigResponse),
        (status = 400, description = "Invalid request", body = ParseConfigResponse)
    )
)]
pub async fn parse_config(Json(req): Json<ParseConfigRequest>) -> Json<ParseConfigResponse> {
    let result = match req.format.to_lowercase().as_str() {
        "toml" => match toml::from_str::<Value>(&req.content) {
            Ok(val) => ParseConfigResponse {
                success: true,
                parsed: Some(val),
                error: None,
            },
            Err(e) => ParseConfigResponse {
                success: false,
                parsed: None,
                error: Some(format!("TOML parse error: {}", e)),
            },
        },
        "json" => match serde_json::from_str::<Value>(&req.content) {
            Ok(val) => ParseConfigResponse {
                success: true,
                parsed: Some(val),
                error: None,
            },
            Err(e) => ParseConfigResponse {
                success: false,
                parsed: None,
                error: Some(format!("JSON parse error: {}", e)),
            },
        },
        "yaml" => match serde_yaml::from_str::<Value>(&req.content) {
            Ok(val) => ParseConfigResponse {
                success: true,
                parsed: Some(val),
                error: None,
            },
            Err(e) => ParseConfigResponse {
                success: false,
                parsed: None,
                error: Some(format!("YAML parse error: {}", e)),
            },
        },
        _ => ParseConfigResponse {
            success: false,
            parsed: None,
            error: Some(format!(
                "Unsupported format: {}. Use 'toml', 'json', or 'yaml'",
                req.format
            )),
        },
    };

    Json(result)
}

#[utoipa::path(
    post,
    path = "/api/demo/spec/generate",
    tag = "demo",
    request_body = GenerateSpecRequest,
    responses(
        (status = 200, description = "OpenAPI spec generated", body = GenerateSpecResponse),
        (status = 400, description = "Invalid request", body = GenerateSpecResponse)
    )
)]
pub async fn generate_spec(Json(req): Json<GenerateSpecRequest>) -> Json<GenerateSpecResponse> {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for field in req.fields {
        let field_schema = match field.field_type.to_lowercase().as_str() {
            "string" => serde_json::json!({"type": "string"}),
            "integer" => serde_json::json!({"type": "integer", "format": "int64"}),
            "number" => serde_json::json!({"type": "number", "format": "float"}),
            "boolean" => serde_json::json!({"type": "boolean"}),
            "array" => serde_json::json!({"type": "array", "items": {"type": "string"}}),
            _ => serde_json::json!({"type": "string"}),
        };

        properties.insert(field.name.clone(), field_schema);

        if field.required {
            required.push(Value::String(field.name));
        }
    }

    let spec = serde_json::json!({
        "openapi": "3.0.0",
        "info": {
            "title": req.title,
            "version": "1.0.0"
        },
        "paths": {
            "/items": {
                "post": {
                    "summary": format!("Create a new {}", req.title),
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": properties,
                                    "required": required
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Created successfully"
                        }
                    }
                }
            }
        }
    });

    Json(GenerateSpecResponse { spec })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/config/parse", post(parse_config))
        .route("/spec/generate", post(generate_spec))
}
