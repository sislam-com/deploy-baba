use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;

pub struct GrepFile;

#[derive(Deserialize)]
pub struct Input {
    pub path: String,
    pub pattern: String,
    #[serde(default = "default_case_sensitive")]
    pub case_sensitive: bool,
}

fn default_case_sensitive() -> bool {
    true
}

#[derive(Serialize)]
pub struct Match {
    pub line_number: u32,
    pub content: String,
    pub column: u32,
}

#[derive(Serialize)]
pub struct Output {
    pub success: bool,
    pub matches: Vec<Match>,
    pub total_matches: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Tool for GrepFile {
    const NAME: &'static str = "grep_file";
    const DESCRIPTION: &'static str = "Search within a single file for a pattern. Prefer native Grep/Bash for general searches; use this for MCP-scoped file searching.";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let resolved_path = workspace::resolve_path(&input.path);
        let path = resolved_path.as_path();

        // Check if file exists
        if !path.exists() {
            return Ok(Output {
                success: false,
                matches: vec![],
                total_matches: 0,
                error: Some(format!("File not found: {}", input.path)),
            });
        }

        // Check if it's a file
        if !path.is_file() {
            return Ok(Output {
                success: false,
                matches: vec![],
                total_matches: 0,
                error: Some(format!("Path is not a file: {}", input.path)),
            });
        }

        // Read file contents
        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    matches: vec![],
                    total_matches: 0,
                    error: Some(format!("Failed to read file: {}", e)),
                });
            }
        };

        // Prepare pattern for matching
        let search_pattern = if input.case_sensitive {
            input.pattern.clone()
        } else {
            input.pattern.to_lowercase()
        };

        let mut matches = Vec::new();

        for (line_idx, line) in contents.lines().enumerate() {
            let search_line = if input.case_sensitive {
                line.to_string()
            } else {
                line.to_lowercase()
            };

            // Find all occurrences in this line
            let mut start_pos = 0;
            while let Some(pos) = search_line[start_pos..].find(&search_pattern) {
                let column = start_pos + pos;
                matches.push(Match {
                    line_number: (line_idx + 1) as u32,
                    content: line.to_string(),
                    column: (column + 1) as u32, // 1-indexed
                });
                start_pos = column + 1;

                // Safety: prevent infinite loop on empty pattern
                if search_pattern.is_empty() {
                    break;
                }
            }
        }

        let total_matches = matches.len() as u32;

        Ok(Output {
            success: true,
            matches,
            total_matches,
            error: None,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to search"
                },
                "pattern": {
                    "type": "string",
                    "description": "The text pattern to search for"
                },
                "case_sensitive": {
                    "type": "boolean",
                    "description": "Whether the search should be case-sensitive",
                    "default": true
                }
            },
            "required": ["path", "pattern"]
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
    fn test_grep_file_basic() {
        let file = create_temp_file("hello world\nfoo bar\nhello again");
        let tool = GrepFile;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                pattern: "hello".to_string(),
                case_sensitive: true,
            })
            .unwrap();
        assert!(output.success);
        assert_eq!(output.total_matches, 2);
        assert_eq!(output.matches.len(), 2);
        assert_eq!(output.matches[0].line_number, 1);
        assert_eq!(output.matches[1].line_number, 3);
    }

    #[test]
    fn test_grep_file_case_insensitive() {
        let file = create_temp_file("Hello World\nHELLO again\nhello there");
        let tool = GrepFile;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                pattern: "hello".to_string(),
                case_sensitive: false,
            })
            .unwrap();
        assert!(output.success);
        assert_eq!(output.total_matches, 3);
    }

    #[test]
    fn test_grep_file_case_sensitive() {
        let file = create_temp_file("Hello World\nHELLO again\nhello there");
        let tool = GrepFile;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                pattern: "hello".to_string(),
                case_sensitive: true,
            })
            .unwrap();
        assert!(output.success);
        assert_eq!(output.total_matches, 1); // Only "hello there"
    }

    #[test]
    fn test_grep_file_no_matches() {
        let file = create_temp_file("hello world\nfoo bar");
        let tool = GrepFile;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                pattern: "nonexistent".to_string(),
                case_sensitive: true,
            })
            .unwrap();
        assert!(output.success);
        assert_eq!(output.total_matches, 0);
        assert!(output.matches.is_empty());
    }

    #[test]
    fn test_grep_file_multiple_matches_same_line() {
        let file = create_temp_file("foo foo foo");
        let tool = GrepFile;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                pattern: "foo".to_string(),
                case_sensitive: true,
            })
            .unwrap();
        assert!(output.success);
        assert_eq!(output.total_matches, 3);
        // All matches should be on line 1
        for m in &output.matches {
            assert_eq!(m.line_number, 1);
        }
        // Check columns: 1, 5, 9 (1-indexed)
        assert_eq!(output.matches[0].column, 1);
        assert_eq!(output.matches[1].column, 5);
        assert_eq!(output.matches[2].column, 9);
    }

    #[test]
    fn test_grep_file_not_found() {
        let tool = GrepFile;
        let output = tool
            .run(Input {
                path: "/nonexistent/file.txt".to_string(),
                pattern: "test".to_string(),
                case_sensitive: true,
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("File not found"));
    }

    #[test]
    fn test_grep_file_directory_path() {
        let tool = GrepFile;
        let output = tool
            .run(Input {
                path: "src".to_string(),
                pattern: "test".to_string(),
                case_sensitive: true,
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("not a file"));
    }

    #[test]
    fn test_grep_empty_pattern() {
        let file = create_temp_file("hello world");
        let tool = GrepFile;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                pattern: "".to_string(),
                case_sensitive: true,
            })
            .unwrap();
        // Empty pattern should not cause infinite loop
        assert!(output.success);
    }

    #[test]
    fn test_schema_has_required_fields() {
        let schema = GrepFile::schema();
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("path")));
        assert!(required.contains(&json!("pattern")));
    }
}
