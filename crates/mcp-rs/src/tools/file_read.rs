use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;

pub struct ReadFile;

#[derive(Deserialize)]
pub struct Input {
    pub path: String,
    #[serde(default)]
    pub start_line: Option<u32>,
    #[serde(default)]
    pub end_line: Option<u32>,
}

#[derive(Serialize)]
pub struct Output {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Tool for ReadFile {
    const NAME: &'static str = "read_file";
    const DESCRIPTION: &'static str = "Read file contents with optional line range. Prefer native Read tool for general file access; use this when reading within MCP workspace scope.";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let resolved_path = workspace::resolve_path(&input.path);
        let path = resolved_path.as_path();

        // Check if file exists
        if !path.exists() {
            return Ok(Output {
                success: false,
                content: None,
                line_count: None,
                error: Some(format!("File not found: {}", input.path)),
            });
        }

        // Check if it's a file (not a directory)
        if !path.is_file() {
            return Ok(Output {
                success: false,
                content: None,
                line_count: None,
                error: Some(format!("Path is not a file: {}", input.path)),
            });
        }

        // Read file contents
        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    content: None,
                    line_count: None,
                    error: Some(format!("Failed to read file: {}", e)),
                });
            }
        };

        let lines: Vec<&str> = contents.lines().collect();
        let total_lines = lines.len() as u32;

        // Handle line range
        let start = input.start_line.unwrap_or(1).saturating_sub(1) as usize;
        let end = input.end_line.map(|e| e as usize).unwrap_or(lines.len());

        // Validate range
        if start >= lines.len() {
            return Ok(Output {
                success: false,
                content: None,
                line_count: Some(total_lines),
                error: Some(format!(
                    "Start line {} exceeds file length of {} lines",
                    start + 1,
                    total_lines
                )),
            });
        }

        let end = end.min(lines.len());
        let selected_lines: Vec<&str> = lines[start..end].to_vec();
        let content = selected_lines.join("\n");

        Ok(Output {
            success: true,
            content: Some(content),
            line_count: Some(total_lines),
            error: None,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                },
                "start_line": {
                    "type": "integer",
                    "description": "Starting line number (1-indexed, inclusive)",
                    "minimum": 1
                },
                "end_line": {
                    "type": "integer",
                    "description": "Ending line number (1-indexed, inclusive)",
                    "minimum": 1
                }
            },
            "required": ["path"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_read_file_success() {
        let file = create_temp_file("line1\nline2\nline3");
        let tool = ReadFile;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                start_line: None,
                end_line: None,
            })
            .unwrap();
        assert!(output.success);
        assert!(output.content.is_some());
        assert_eq!(output.line_count, Some(3));
        assert!(output.error.is_none());
    }

    #[test]
    fn test_read_file_with_line_range() {
        let file = create_temp_file("line1\nline2\nline3\nline4\nline5");
        let tool = ReadFile;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                start_line: Some(2),
                end_line: Some(4),
            })
            .unwrap();
        assert!(output.success);
        let content = output.content.unwrap();
        assert!(content.contains("line2"));
        assert!(content.contains("line3"));
        assert!(content.contains("line4"));
        assert!(!content.contains("line1"));
        assert!(!content.contains("line5"));
    }

    #[test]
    fn test_read_file_not_found() {
        let tool = ReadFile;
        let output = tool
            .run(Input {
                path: "/nonexistent/file/path.txt".to_string(),
                start_line: None,
                end_line: None,
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.content.is_none());
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("File not found"));
    }

    #[test]
    fn test_read_file_directory_path() {
        let tool = ReadFile;
        let output = tool
            .run(Input {
                path: "src".to_string(),
                start_line: None,
                end_line: None,
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("not a file"));
    }

    #[test]
    fn test_read_file_start_line_exceeds_length() {
        let file = create_temp_file("line1\nline2");
        let tool = ReadFile;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                start_line: Some(100),
                end_line: None,
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("exceeds file length"));
    }

    #[test]
    fn test_schema_has_required_path() {
        let schema = ReadFile::schema();
        assert_eq!(schema["required"][0], "path");
    }
}
