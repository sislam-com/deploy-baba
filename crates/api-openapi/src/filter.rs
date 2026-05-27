/// Public-view filter for OpenAPI specifications.
///
/// Strips admin-tagged paths, admin-only operations, and unreferenced schemas
/// from a full spec, producing a safe public-facing document.
use std::collections::BTreeSet;
use utoipa::openapi::{OpenApi, RefOr, Schema};

/// Return a copy of `spec` with all admin information removed:
///
/// 1. Drop any operation whose tags contain `"admin"`.
/// 2. Drop any `PathItem` that has no remaining operations.
/// 3. Garbage-collect `components.schemas` — remove entries not reachable
///    from any remaining operation's request bodies or response bodies.
/// 4. Strip `cookieAuth` and `bearerAuth` security schemes so API clients
///    cannot discover that protected endpoints exist.
pub fn public_view(spec: &OpenApi) -> OpenApi {
    let mut out = spec.clone();

    // ── Step 1 & 2: filter paths ──────────────────────────────────────────────
    out.paths.paths.retain(|_path, item| {
        item.operations.retain(|_method, op| !is_admin_op(op));
        !item.operations.is_empty()
    });

    // ── Step 3: GC unreferenced schemas ──────────────────────────────────────
    if let Some(components) = &mut out.components {
        let reachable = collect_refs(&out.paths.paths);
        components
            .schemas
            .retain(|name, _| reachable.contains(name.as_str()));

        // ── Step 4: strip security schemes ───────────────────────────────────
        components
            .security_schemes
            .retain(|name, _| name != "cookieAuth" && name != "bearerAuth");
    }

    // Also clear top-level security requirements
    out.security = None;

    out
}

fn is_admin_op(op: &utoipa::openapi::path::Operation) -> bool {
    op.tags
        .as_ref()
        .map(|tags| tags.iter().any(|t| t == "admin"))
        .unwrap_or(false)
}

/// Walk surviving paths and collect every schema name referenced via `$ref`.
fn collect_refs(
    paths: &std::collections::BTreeMap<String, utoipa::openapi::PathItem>,
) -> BTreeSet<String> {
    let mut refs: BTreeSet<String> = BTreeSet::new();

    for item in paths.values() {
        for op in item.operations.values() {
            // Collect refs from request body
            if let Some(body) = &op.request_body {
                for media in body.content.values() {
                    collect_schema_refs(&media.schema, &mut refs);
                }
            }
            // Collect refs from responses
            for response in op.responses.responses.values() {
                if let RefOr::T(response) = response {
                    for media in response.content.values() {
                        collect_schema_refs(&media.schema, &mut refs);
                    }
                }
            }
        }
    }

    refs
}

/// Recursively extract schema names from a `RefOr<Schema>`.
fn collect_schema_refs(schema: &RefOr<Schema>, refs: &mut BTreeSet<String>) {
    match schema {
        RefOr::Ref(r) => {
            // e.g. "#/components/schemas/Job" → "Job"
            if let Some(name) = r.ref_location.strip_prefix("#/components/schemas/") {
                refs.insert(name.to_string());
            }
        }
        RefOr::T(s) => {
            collect_schema_type_refs(s, refs);
        }
    }
}

fn collect_schema_type_refs(schema: &Schema, refs: &mut BTreeSet<String>) {
    match schema {
        Schema::Object(obj) => {
            for prop in obj.properties.values() {
                collect_schema_refs(prop, refs);
            }
            if let Some(additional) = &obj.additional_properties {
                use utoipa::openapi::schema::AdditionalProperties;
                if let AdditionalProperties::RefOr(s) = additional.as_ref() {
                    collect_schema_refs(s, refs);
                }
            }
        }
        Schema::Array(arr) => {
            collect_schema_refs(&arr.items, refs);
        }
        Schema::AllOf(ao) => {
            for item in &ao.items {
                collect_schema_refs(item, refs);
            }
        }
        Schema::OneOf(oo) => {
            for item in &oo.items {
                collect_schema_refs(item, refs);
            }
        }
        Schema::AnyOf(ao) => {
            for item in &ao.items {
                collect_schema_refs(item, refs);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use utoipa::openapi::{
        path::{Operation, OperationBuilder, PathItemBuilder, PathsBuilder},
        ContentBuilder, OpenApiBuilder, ResponseBuilder,
    };

    fn build_op(tag: &str) -> Operation {
        OperationBuilder::new()
            .tag(tag)
            .response(
                "200",
                ResponseBuilder::new()
                    .description("ok")
                    .content("application/json", ContentBuilder::new().build())
                    .build(),
            )
            .build()
    }

    #[test]
    fn admin_op_detection() {
        let admin_op = build_op("admin");
        assert!(is_admin_op(&admin_op));

        let public_op = build_op("health");
        assert!(!is_admin_op(&public_op));
    }

    #[test]
    fn filter_removes_admin_paths_and_keeps_public() {
        use utoipa::openapi::path::PathItemType;
        let spec = OpenApiBuilder::new()
            .paths(
                PathsBuilder::new()
                    .path(
                        "/api/admin/jobs",
                        PathItemBuilder::new()
                            .operation(PathItemType::Post, build_op("admin"))
                            .build(),
                    )
                    .path(
                        "/health",
                        PathItemBuilder::new()
                            .operation(PathItemType::Get, build_op("health"))
                            .build(),
                    )
                    .build(),
            )
            .build();

        let filtered = public_view(&spec);
        assert!(!filtered.paths.paths.contains_key("/api/admin/jobs"));
        assert!(filtered.paths.paths.contains_key("/health"));
    }

    #[test]
    fn filter_clears_top_level_security() {
        let spec = OpenApiBuilder::new().build();
        let filtered = public_view(&spec);
        assert!(filtered.security.is_none());
    }

    #[test]
    fn collect_schema_type_refs_traverses_nested_variants() {
        let object_schema: Schema = serde_json::from_value(json!({
            "type": "object",
            "properties": {
                "child": { "$ref": "#/components/schemas/Child" }
            },
            "additionalProperties": { "$ref": "#/components/schemas/Metadata" }
        }))
        .unwrap();
        let array_schema: Schema = serde_json::from_value(json!({
            "type": "array",
            "items": { "$ref": "#/components/schemas/ArrayItem" }
        }))
        .unwrap();
        let all_of_schema: Schema = serde_json::from_value(json!({
            "allOf": [{ "$ref": "#/components/schemas/AllOfItem" }]
        }))
        .unwrap();
        let one_of_schema: Schema = serde_json::from_value(json!({
            "oneOf": [{ "$ref": "#/components/schemas/OneOfItem" }]
        }))
        .unwrap();
        let any_of_schema: Schema = serde_json::from_value(json!({
            "anyOf": [{ "$ref": "#/components/schemas/AnyOfItem" }]
        }))
        .unwrap();

        let mut refs = BTreeSet::new();
        collect_schema_type_refs(&object_schema, &mut refs);
        collect_schema_type_refs(&array_schema, &mut refs);
        collect_schema_type_refs(&all_of_schema, &mut refs);
        collect_schema_type_refs(&one_of_schema, &mut refs);
        collect_schema_type_refs(&any_of_schema, &mut refs);

        assert_eq!(
            refs,
            BTreeSet::from([
                "AllOfItem".to_string(),
                "AnyOfItem".to_string(),
                "ArrayItem".to_string(),
                "Child".to_string(),
                "Metadata".to_string(),
                "OneOfItem".to_string(),
            ])
        );
    }

    #[test]
    fn public_view_removes_unreachable_schemas_and_auth_schemes() {
        let spec: OpenApi = serde_json::from_value(json!({
            "openapi": "3.0.3",
            "info": { "title": "Test", "version": "1.0.0" },
            "paths": {
                "/public": {
                    "post": {
                        "tags": ["public"],
                        "security": [{ "apiKeyAuth": [] }],
                        "requestBody": {
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "items": {
                                                "type": "array",
                                                "items": { "$ref": "#/components/schemas/PublicItem" }
                                            }
                                        },
                                        "additionalProperties": {
                                            "$ref": "#/components/schemas/PublicMetadata"
                                        }
                                    }
                                }
                            }
                        },
                        "responses": {
                            "200": {
                                "description": "ok",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "oneOf": [
                                                { "$ref": "#/components/schemas/PublicItem" },
                                                { "$ref": "#/components/schemas/PublicMetadata" }
                                            ]
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "/admin": {
                    "get": {
                        "tags": ["admin"],
                        "responses": {
                            "200": {
                                "description": "ok",
                                "content": {
                                    "application/json": {
                                        "schema": { "$ref": "#/components/schemas/AdminResponse" }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "components": {
                "schemas": {
                    "PublicItem": { "type": "object" },
                    "PublicMetadata": { "type": "object" },
                    "AdminResponse": { "type": "object" },
                    "UnusedSchema": { "type": "object" }
                },
                "securitySchemes": {
                    "cookieAuth": { "type": "apiKey", "in": "cookie", "name": "session" },
                    "bearerAuth": { "type": "http", "scheme": "bearer" },
                    "apiKeyAuth": { "type": "apiKey", "in": "header", "name": "x-api-key" }
                }
            },
            "security": [{ "cookieAuth": [] }]
        }))
        .unwrap();

        let filtered = public_view(&spec);
        let components = filtered.components.as_ref().unwrap();

        assert!(filtered.paths.paths.contains_key("/public"));
        assert!(!filtered.paths.paths.contains_key("/admin"));
        assert!(components.schemas.contains_key("PublicItem"));
        assert!(components.schemas.contains_key("PublicMetadata"));
        assert!(!components.schemas.contains_key("AdminResponse"));
        assert!(!components.schemas.contains_key("UnusedSchema"));
        assert!(!components.security_schemes.contains_key("cookieAuth"));
        assert!(!components.security_schemes.contains_key("bearerAuth"));
        assert!(components.security_schemes.contains_key("apiKeyAuth"));
        assert!(filtered.security.is_none());
    }
}
