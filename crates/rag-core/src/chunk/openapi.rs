//! OpenAPI specification chunker.
//!
//! Parses a JSON OpenAPI spec and emits one [`Chunk`] per path-operation
//! (e.g. `GET /api/jobs`) and one per component schema. Oversize chunks
//! are split with the standard sliding-window strategy.

use crate::types::Chunk;

const MAX_TOKENS: usize = 800;
const OVERLAP_WORDS: usize = 50;

pub fn chunk(path: &str, content: &str) -> Vec<Chunk> {
    let spec: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut chunks = Vec::new();
    let mut ord = 0usize;

    if let Some(paths) = spec.get("paths").and_then(|p| p.as_object()) {
        for (endpoint, ops) in paths {
            let ops = match ops.as_object() {
                Some(o) => o,
                None => continue,
            };
            for (method, detail) in ops {
                let method_upper = method.to_uppercase();
                let summary = detail.get("summary").and_then(|s| s.as_str()).unwrap_or("");
                let description = detail
                    .get("description")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");

                let mut text = format!("Endpoint: {method_upper} {endpoint}");
                if !summary.is_empty() {
                    text.push_str(&format!("\nSummary: {summary}"));
                }
                if !description.is_empty() {
                    text.push_str(&format!("\nDescription: {description}"));
                }

                if let Some(params) = detail.get("parameters").and_then(|p| p.as_array()) {
                    if !params.is_empty() {
                        text.push_str("\nParameters:");
                        for p in params {
                            let name = p.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                            let loc = p.get("in").and_then(|l| l.as_str()).unwrap_or("?");
                            let required =
                                p.get("required").and_then(|r| r.as_bool()).unwrap_or(false);
                            text.push_str(&format!(
                                "\n  - {name} ({loc}{})",
                                if required { ", required" } else { "" }
                            ));
                        }
                    }
                }

                if let Some(responses) = detail.get("responses").and_then(|r| r.as_object()) {
                    text.push_str("\nResponses:");
                    for (code, resp) in responses {
                        let desc = resp
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("");
                        text.push_str(&format!("\n  {code}: {desc}"));
                    }
                }

                let tag = detail
                    .get("tags")
                    .and_then(|t| t.as_array())
                    .and_then(|a| a.first())
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                emit_chunks(
                    path,
                    &text,
                    serde_json::json!({ "endpoint": format!("{method_upper} {endpoint}"), "tag": tag }),
                    &mut ord,
                    &mut chunks,
                );
            }
        }
    }

    if let Some(schemas) = spec
        .pointer("/components/schemas")
        .and_then(|s| s.as_object())
    {
        for (name, schema) in schemas {
            let schema_type = schema
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("object");
            let mut text = format!("Schema: {name}\nType: {schema_type}");

            if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
                text.push_str("\nFields:");
                for (field, field_schema) in props {
                    let ft = schema_type_str(field_schema);
                    let desc = field_schema
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("");
                    if desc.is_empty() {
                        text.push_str(&format!("\n  {field}: {ft}"));
                    } else {
                        text.push_str(&format!("\n  {field}: {ft} — {desc}"));
                    }
                }
            }

            emit_chunks(
                path,
                &text,
                serde_json::json!({ "schema": name }),
                &mut ord,
                &mut chunks,
            );
        }
    }

    chunks
}

fn schema_type_str(schema: &serde_json::Value) -> String {
    if let Some(t) = schema.get("type").and_then(|t| t.as_str()) {
        if t == "array" {
            if let Some(items) = schema.get("items") {
                return format!("array<{}>", schema_type_str(items));
            }
            return "array".to_string();
        }
        return t.to_string();
    }
    if let Some(r) = schema.get("$ref").and_then(|r| r.as_str()) {
        return r.rsplit('/').next().unwrap_or(r).to_string();
    }
    "unknown".to_string()
}

fn emit_chunks(
    path: &str,
    text: &str,
    meta_base: serde_json::Value,
    ord: &mut usize,
    chunks: &mut Vec<Chunk>,
) {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return;
    }
    if words.len() <= MAX_TOKENS {
        let mut meta = meta_base;
        meta["path"] = serde_json::json!(path);
        chunks.push(Chunk {
            ord: *ord,
            content: text.to_owned(),
            token_count: words.len(),
            meta,
        });
        *ord += 1;
    } else {
        let mut start = 0;
        while start < words.len() {
            let end = (start + MAX_TOKENS).min(words.len());
            let chunk_text = words[start..end].join(" ");
            let token_count = end - start;
            let mut meta = meta_base.clone();
            meta["path"] = serde_json::json!(path);
            meta["window_start"] = serde_json::json!(start);
            chunks.push(Chunk {
                ord: *ord,
                content: chunk_text,
                token_count,
                meta,
            });
            *ord += 1;
            if end == words.len() {
                break;
            }
            start += MAX_TOKENS - OVERLAP_WORDS;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_spec() -> String {
        serde_json::json!({
            "openapi": "3.0.3",
            "info": { "title": "Test API", "version": "1.0.0" },
            "paths": {
                "/api/jobs": {
                    "get": {
                        "summary": "List all jobs",
                        "tags": ["jobs"],
                        "parameters": [
                            { "name": "limit", "in": "query", "required": false }
                        ],
                        "responses": {
                            "200": { "description": "Job list" }
                        }
                    }
                },
                "/api/jobs/{slug}": {
                    "get": {
                        "summary": "Get job by slug",
                        "tags": ["jobs"],
                        "parameters": [
                            { "name": "slug", "in": "path", "required": true }
                        ],
                        "responses": {
                            "200": { "description": "Job details" },
                            "404": { "description": "Not found" }
                        }
                    }
                }
            },
            "components": {
                "schemas": {
                    "Job": {
                        "type": "object",
                        "properties": {
                            "slug": { "type": "string", "description": "URL-safe identifier" },
                            "company": { "type": "string" },
                            "title": { "type": "string" }
                        }
                    }
                }
            }
        })
        .to_string()
    }

    #[test]
    fn produces_chunks_for_paths_and_schemas() {
        let chunks = chunk("api/openapi.json", &fixture_spec());
        assert_eq!(chunks.len(), 3, "2 path ops + 1 schema");
    }

    #[test]
    fn endpoint_chunk_has_correct_meta() {
        let chunks = chunk("api/openapi.json", &fixture_spec());
        let first = &chunks[0];
        assert!(first.meta["endpoint"].as_str().unwrap().starts_with("GET"));
        assert_eq!(first.meta["tag"].as_str().unwrap(), "jobs");
    }

    #[test]
    fn schema_chunk_contains_fields() {
        let chunks = chunk("api/openapi.json", &fixture_spec());
        let schema_chunk = chunks
            .iter()
            .find(|c| c.meta.get("schema").is_some())
            .unwrap();
        assert!(schema_chunk.content.contains("Schema: Job"));
        assert!(schema_chunk.content.contains("slug"));
    }

    #[test]
    fn empty_paths_returns_empty() {
        let spec = serde_json::json!({
            "openapi": "3.0.3",
            "info": { "title": "Empty", "version": "1.0.0" },
            "paths": {}
        })
        .to_string();
        let chunks = chunk("api/openapi.json", &spec);
        assert!(chunks.is_empty());
    }

    #[test]
    fn invalid_json_returns_empty() {
        let chunks = chunk("bad.json", "not valid json");
        assert!(chunks.is_empty());
    }

    #[test]
    fn oversize_description_is_split() {
        let long_desc = "word ".repeat(1600);
        let spec = serde_json::json!({
            "openapi": "3.0.3",
            "info": { "title": "Test", "version": "1.0.0" },
            "paths": {
                "/api/big": {
                    "get": {
                        "summary": "Big endpoint",
                        "description": long_desc,
                        "responses": { "200": { "description": "ok" } }
                    }
                }
            }
        })
        .to_string();
        let chunks = chunk("api/openapi.json", &spec);
        assert!(chunks.len() >= 2, "oversize should split");
        for c in &chunks {
            assert!(c.token_count <= MAX_TOKENS);
        }
    }
}
