use axum::{extract::Path, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Serialize, Deserialize, Clone)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub traits: Vec<String>,
}

fn get_all_crates() -> Vec<CrateInfo> {
    vec![
        CrateInfo {
            name: "config-core".to_string(),
            version: "0.1.0".to_string(),
            description: "Universal zero-cost trait abstractions for configuration parsing"
                .to_string(),
            traits: vec!["ConfigParser".to_string(), "Validatable".to_string()],
        },
        CrateInfo {
            name: "config-toml".to_string(),
            version: "0.1.0".to_string(),
            description: "TOML implementation of config-core universal traits".to_string(),
            traits: vec!["ConfigParser".to_string(), "TomlFormat".to_string()],
        },
        CrateInfo {
            name: "config-yaml".to_string(),
            version: "0.1.0".to_string(),
            description: "YAML implementation of config-core universal traits".to_string(),
            traits: vec!["ConfigParser".to_string(), "YamlFormat".to_string()],
        },
        CrateInfo {
            name: "config-json".to_string(),
            version: "0.1.0".to_string(),
            description: "JSON implementation of config-core universal traits".to_string(),
            traits: vec!["ConfigParser".to_string(), "JsonFormat".to_string()],
        },
        CrateInfo {
            name: "api-core".to_string(),
            version: "0.1.0".to_string(),
            description: "Universal zero-cost trait abstractions for API specification generation"
                .to_string(),
            traits: vec!["SpecGenerator".to_string(), "Validatable".to_string()],
        },
        CrateInfo {
            name: "api-openapi".to_string(),
            version: "0.1.0".to_string(),
            description: "OpenAPI 3.0 specification generator implementing api-core traits"
                .to_string(),
            traits: vec!["SpecGenerator".to_string(), "OpenApiCompat".to_string()],
        },
        CrateInfo {
            name: "api-graphql".to_string(),
            version: "0.1.0".to_string(),
            description: "GraphQL schema generator implementing api-core traits".to_string(),
            traits: vec!["SpecGenerator".to_string(), "GraphqlCompat".to_string()],
        },
        CrateInfo {
            name: "api-grpc".to_string(),
            version: "0.1.0".to_string(),
            description: "gRPC/Protocol Buffer schema generator implementing api-core traits"
                .to_string(),
            traits: vec!["SpecGenerator".to_string(), "GrpcCompat".to_string()],
        },
        CrateInfo {
            name: "api-merger".to_string(),
            version: "0.1.0".to_string(),
            description: "Multi-format API specification merging with conflict resolution"
                .to_string(),
            traits: vec!["SpecMerger".to_string(), "ConflictResolver".to_string()],
        },
        CrateInfo {
            name: "infra-types".to_string(),
            version: "0.1.0".to_string(),
            description:
                "Cloud-agnostic infrastructure type definitions with SQLite + S3 backup support"
                    .to_string(),
            traits: vec!["InfraType".to_string(), "Persistent".to_string()],
        },
    ]
}

#[utoipa::path(
    get,
    path = "/api/crates",
    tag = "crates",
    responses(
        (status = 200, description = "List of all crates", body = Vec<CrateInfo>)
    )
)]
pub async fn list_crates() -> Json<Vec<CrateInfo>> {
    Json(get_all_crates())
}

#[utoipa::path(
    get,
    path = "/api/crates/{name}",
    tag = "crates",
    params(
        ("name" = String, Path, description = "Crate name")
    ),
    responses(
        (status = 200, description = "Crate details", body = CrateInfo),
        (status = 404, description = "Crate not found")
    )
)]
pub async fn get_crate(
    Path(name): Path<String>,
) -> Result<Json<CrateInfo>, (axum::http::StatusCode, String)> {
    get_all_crates()
        .into_iter()
        .find(|c| c.name == name)
        .map(Json)
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                format!("Crate '{}' not found", name),
            )
        })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_crates))
        .route("/:name", get(get_crate))
}
