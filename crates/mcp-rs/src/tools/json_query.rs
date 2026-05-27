use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;

pub struct JsonQuery;

#[derive(Deserialize)]
pub struct Input {
    pub path: String,
    #[serde(default)]
    pub query: Option<String>,
}

#[derive(Serialize)]
pub struct Output {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Tool for JsonQuery {
    const NAME: &'static str = "json_query";
    const DESCRIPTION: &'static str = "Read and query JSON files with dot-path navigation";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let resolved_path = workspace::resolve_path(&input.path);
        let path = resolved_path.as_path();

        if !path.exists() {
            return Ok(Output {
                success: false,
                data: None,
                error: Some(format!("File not found: {}", input.path)),
            });
        }

        if !path.is_file() {
            return Ok(Output {
                success: false,
                data: None,
                error: Some(format!("Path is not a file: {}", input.path)),
            });
        }

        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to read file: {}", e)),
                });
            }
        };

        let json_value: serde_json::Value = match serde_json::from_str(&contents) {
            Ok(v) => v,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to parse JSON: {}", e)),
                });
            }
        };

        let result = if let Some(ref query) = input.query {
            query_value(&json_value, query)
        } else {
            Some(json_value)
        };

        Ok(match result {
            Some(data) => Output {
                success: true,
                data: Some(data),
                error: None,
            },
            None => Output {
                success: false,
                data: None,
                error: Some(format!(
                    "Query path not found: {}",
                    input.query.unwrap_or_default()
                )),
            },
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the JSON file to read"
                },
                "query": {
                    "type": "string",
                    "description": "Dot-separated path to query (e.g., 'package.name', 'dependencies.0.name')"
                }
            },
            "required": ["path"]
        })
    }
}

fn query_value(value: &serde_json::Value, query: &str) -> Option<serde_json::Value> {
    let parts: Vec<&str> = query.split('.').collect();
    let mut current = value.clone();

    for part in parts {
        match current {
            serde_json::Value::Object(map) => {
                current = map.get(part)?.clone();
            }
            serde_json::Value::Array(arr) => {
                let index: usize = part.parse().ok()?;
                current = arr.get(index)?.clone();
            }
            _ => return None,
        }
    }

    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_json(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_read_json_basic() {
        let file = create_temp_json(r#"{"name": "test", "version": "1.0"}"#);
        let tool = JsonQuery;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                query: None,
            })
            .unwrap();
        assert!(output.success);
        assert!(output.data.is_some());
    }

    #[test]
    fn test_query_nested() {
        let file = create_temp_json(r#"{"package": {"name": "test", "version": "1.0"}}"#);
        let tool = JsonQuery;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                query: Some("package.name".to_string()),
            })
            .unwrap();
        assert!(output.success);
        assert_eq!(output.data, Some(json!("test")));
    }

    #[test]
    fn test_query_array_index() {
        let file = create_temp_json(r#"{"items": ["a", "b", "c"]}"#);
        let tool = JsonQuery;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                query: Some("items.1".to_string()),
            })
            .unwrap();
        assert!(output.success);
        assert_eq!(output.data, Some(json!("b")));
    }

    #[test]
    fn test_query_not_found() {
        let file = create_temp_json(r#"{"name": "test"}"#);
        let tool = JsonQuery;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                query: Some("nonexistent.path".to_string()),
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.error.unwrap().contains("not found"));
    }

    #[test]
    fn test_file_not_found() {
        let tool = JsonQuery;
        let output = tool
            .run(Input {
                path: "/nonexistent/file.json".to_string(),
                query: None,
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.error.unwrap().contains("File not found"));
    }

    #[test]
    fn test_invalid_json() {
        let file = create_temp_json("not valid json {{{");
        let tool = JsonQuery;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                query: None,
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.error.unwrap().contains("Failed to parse"));
    }

    #[test]
    fn test_schema_has_required_path() {
        let schema = JsonQuery::schema();
        assert_eq!(schema["required"][0], "path");
    }
}
