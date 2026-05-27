use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;

pub struct ReadToml;

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

impl Tool for ReadToml {
    const NAME: &'static str = "read_toml";
    const DESCRIPTION: &'static str = "Parse and query TOML files";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let resolved_path = workspace::resolve_path(&input.path);
        let path = resolved_path.as_path();

        // Check if file exists
        if !path.exists() {
            return Ok(Output {
                success: false,
                data: None,
                error: Some(format!("File not found: {}", input.path)),
            });
        }

        // Check if it's a file
        if !path.is_file() {
            return Ok(Output {
                success: false,
                data: None,
                error: Some(format!("Path is not a file: {}", input.path)),
            });
        }

        // Read file contents
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

        // Parse TOML
        let toml_value: toml::Value = match contents.parse() {
            Ok(v) => v,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to parse TOML: {}", e)),
                });
            }
        };

        // Convert TOML to JSON for serialization
        let json_value = toml_to_json(&toml_value);

        // If a query is provided, navigate to that path
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
                    "description": "Path to the TOML file to read"
                },
                "query": {
                    "type": "string",
                    "description": "Dot-separated path to query (e.g., 'package.name', 'dependencies')"
                }
            },
            "required": ["path"]
        })
    }
}

/// Convert a TOML Value to a JSON Value
fn toml_to_json(toml: &toml::Value) -> serde_json::Value {
    match toml {
        toml::Value::String(s) => serde_json::Value::String(s.clone()),
        toml::Value::Integer(i) => serde_json::Value::Number((*i).into()),
        toml::Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
        toml::Value::Array(arr) => serde_json::Value::Array(arr.iter().map(toml_to_json).collect()),
        toml::Value::Table(table) => {
            let map: serde_json::Map<String, serde_json::Value> = table
                .iter()
                .map(|(k, v)| (k.clone(), toml_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
    }
}

/// Query a JSON value using a dot-separated path
fn query_value(value: &serde_json::Value, query: &str) -> Option<serde_json::Value> {
    let parts: Vec<&str> = query.split('.').collect();
    let mut current = value.clone();

    for part in parts {
        match current {
            serde_json::Value::Object(map) => {
                current = map.get(part)?.clone();
            }
            serde_json::Value::Array(arr) => {
                // Try to parse as array index
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

    fn create_temp_toml(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_read_toml_basic() {
        let file = create_temp_toml(
            r#"
[package]
name = "test-package"
version = "1.0.0"
"#,
        );
        let tool = ReadToml;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                query: None,
            })
            .unwrap();

        assert!(output.success);
        assert!(output.data.is_some());
        assert!(output.error.is_none());
    }

    #[test]
    fn test_read_toml_query_string() {
        let file = create_temp_toml(
            r#"
[package]
name = "test-package"
version = "1.0.0"
"#,
        );
        let tool = ReadToml;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                query: Some("package.name".to_string()),
            })
            .unwrap();

        assert!(output.success);
        assert_eq!(output.data, Some(json!("test-package")));
    }

    #[test]
    fn test_read_toml_query_nested() {
        let file = create_temp_toml(
            r#"
[package]
name = "test"

[dependencies]
serde = "1.0"
"#,
        );
        let tool = ReadToml;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                query: Some("dependencies.serde".to_string()),
            })
            .unwrap();

        assert!(output.success);
        assert_eq!(output.data, Some(json!("1.0")));
    }

    #[test]
    fn test_read_toml_query_not_found() {
        let file = create_temp_toml(
            r#"
[package]
name = "test"
"#,
        );
        let tool = ReadToml;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                query: Some("nonexistent.path".to_string()),
            })
            .unwrap();

        assert!(!output.success);
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("not found"));
    }

    #[test]
    fn test_read_toml_file_not_found() {
        let tool = ReadToml;
        let output = tool
            .run(Input {
                path: "/nonexistent/file.toml".to_string(),
                query: None,
            })
            .unwrap();

        assert!(!output.success);
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("File not found"));
    }

    #[test]
    fn test_read_toml_invalid_toml() {
        let file = create_temp_toml("this is not valid toml { [ }");
        let tool = ReadToml;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                query: None,
            })
            .unwrap();

        assert!(!output.success);
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("Failed to parse"));
    }

    #[test]
    fn test_read_toml_cargo_toml() {
        // Test with the actual Cargo.toml
        let tool = ReadToml;
        let output = tool
            .run(Input {
                path: "Cargo.toml".to_string(),
                query: Some("package.name".to_string()),
            })
            .unwrap();

        assert!(output.success);
        assert_eq!(output.data, Some(json!("mcp-rs")));
    }

    #[test]
    fn test_toml_to_json_types() {
        let toml_str = r#"
string = "hello"
integer = 42
float = 3.14
boolean = true
array = [1, 2, 3]

[table]
key = "value"
"#;
        let toml_value: toml::Value = toml_str.parse().unwrap();
        let json_value = toml_to_json(&toml_value);

        assert_eq!(json_value["string"], json!("hello"));
        assert_eq!(json_value["integer"], json!(42));
        assert_eq!(json_value["boolean"], json!(true));
        assert_eq!(json_value["array"], json!([1, 2, 3]));
        assert_eq!(json_value["table"]["key"], json!("value"));
    }

    #[test]
    fn test_query_value_array_index() {
        let json = json!({
            "items": ["first", "second", "third"]
        });

        let result = query_value(&json, "items.1");
        assert_eq!(result, Some(json!("second")));
    }

    #[test]
    fn test_schema_has_required_path() {
        let schema = ReadToml::schema();
        assert_eq!(schema["required"][0], "path");
    }
}
