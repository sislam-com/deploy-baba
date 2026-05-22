use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use walkdir::WalkDir;

pub struct GrepProject;

#[derive(Deserialize)]
pub struct Input {
    pub pattern: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub include_glob: Option<String>,
    #[serde(default)]
    pub exclude_glob: Option<String>,
    #[serde(default)]
    pub max_results: Option<u32>,
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
pub struct FileMatch {
    pub file: String,
    pub matches: Vec<Match>,
}

#[derive(Serialize)]
pub struct Output {
    pub success: bool,
    pub matches: Vec<FileMatch>,
    pub files_searched: u32,
    pub total_matches: u32,
    pub truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Tool for GrepProject {
    const NAME: &'static str = "grep_project";
    const DESCRIPTION: &'static str = "Search for a pattern across multiple files in a directory";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let base_path = input.path.as_deref();
        let resolved_path = base_path
            .map(workspace::resolve_path)
            .unwrap_or_else(workspace::root);
        let path = resolved_path.as_path();
        let display_path = base_path.unwrap_or("<workspace_root>");

        // Check if path exists
        if !path.exists() {
            return Ok(Output {
                success: false,
                matches: vec![],
                files_searched: 0,
                total_matches: 0,
                truncated: false,
                error: Some(format!("Path not found: {}", display_path)),
            });
        }

        let max_results = input.max_results.unwrap_or(1000) as usize;

        // Compile include/exclude patterns
        let include_pattern = input
            .include_glob
            .as_ref()
            .and_then(|p| glob::Pattern::new(p).ok());
        let exclude_pattern = input
            .exclude_glob
            .as_ref()
            .and_then(|p| glob::Pattern::new(p).ok());

        // Try to compile regex pattern, fall back to literal search
        let regex_pattern = if input.case_sensitive {
            Regex::new(&input.pattern).ok()
        } else {
            Regex::new(&format!("(?i){}", regex::escape(&input.pattern))).ok()
        };

        let search_pattern = if input.case_sensitive {
            input.pattern.clone()
        } else {
            input.pattern.to_lowercase()
        };

        let mut file_matches = Vec::new();
        let mut files_searched = 0u32;
        let mut total_matches = 0u32;
        let mut truncated = false;

        // Walk through all files
        let walker = if path.is_file() {
            WalkDir::new(path).max_depth(0)
        } else {
            WalkDir::new(path)
        };

        'outer: for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }

            let file_path = entry.path();
            let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Apply include filter
            if let Some(ref pattern) = include_pattern {
                if !pattern.matches(file_name) && !pattern.matches(&file_path.to_string_lossy()) {
                    continue;
                }
            }

            // Apply exclude filter
            if let Some(ref pattern) = exclude_pattern {
                if pattern.matches(file_name) || pattern.matches(&file_path.to_string_lossy()) {
                    continue;
                }
            }

            // Skip binary files (simple heuristic)
            let contents = match fs::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue, // Skip files that can't be read as text
            };

            files_searched += 1;
            let mut matches_in_file = Vec::new();

            for (line_idx, line) in contents.lines().enumerate() {
                let found_matches: Vec<(usize, &str)> = if let Some(ref re) = regex_pattern {
                    re.find_iter(line)
                        .map(|m| (m.start(), m.as_str()))
                        .collect()
                } else {
                    // Fallback to literal search
                    let search_line = if input.case_sensitive {
                        line.to_string()
                    } else {
                        line.to_lowercase()
                    };

                    let mut found = Vec::new();
                    let mut start_pos = 0;
                    while let Some(pos) = search_line[start_pos..].find(&search_pattern) {
                        let column = start_pos + pos;
                        found.push((column, &line[column..column + search_pattern.len()]));
                        start_pos = column + 1;
                        if search_pattern.is_empty() {
                            break;
                        }
                    }
                    found
                };

                for (column, _) in found_matches {
                    if total_matches as usize >= max_results {
                        truncated = true;
                        break 'outer;
                    }

                    matches_in_file.push(Match {
                        line_number: (line_idx + 1) as u32,
                        content: line.to_string(),
                        column: (column + 1) as u32,
                    });
                    total_matches += 1;
                }
            }

            if !matches_in_file.is_empty() {
                file_matches.push(FileMatch {
                    file: file_path.to_string_lossy().to_string(),
                    matches: matches_in_file,
                });
            }
        }

        Ok(Output {
            success: true,
            matches: file_matches,
            files_searched,
            total_matches,
            truncated,
            error: None,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The text or regex pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Base path to search in (defaults to configured workspace root)"
                },
                "include_glob": {
                    "type": "string",
                    "description": "Glob pattern to include files (e.g., '*.rs', '*.txt')"
                },
                "exclude_glob": {
                    "type": "string",
                    "description": "Glob pattern to exclude files (e.g., '*.log', 'target/*')"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of matches to return (default: 1000)",
                    "default": 1000
                },
                "case_sensitive": {
                    "type": "boolean",
                    "description": "Whether the search should be case-sensitive",
                    "default": true
                }
            },
            "required": ["pattern"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let dir = TempDir::new().unwrap();

        // Create some Rust files
        let mut file1 = File::create(dir.path().join("main.rs")).unwrap();
        writeln!(file1, "fn main() {{").unwrap();
        writeln!(file1, "    println!(\"Hello\");").unwrap();
        writeln!(file1, "}}").unwrap();

        let mut file2 = File::create(dir.path().join("lib.rs")).unwrap();
        writeln!(file2, "pub fn hello() {{").unwrap();
        writeln!(file2, "    println!(\"Hello from lib\");").unwrap();
        writeln!(file2, "}}").unwrap();

        // Create a text file
        let mut file3 = File::create(dir.path().join("readme.txt")).unwrap();
        writeln!(file3, "This is a readme file").unwrap();
        writeln!(file3, "Hello world").unwrap();

        dir
    }

    #[test]
    fn test_grep_project_basic() {
        let dir = create_test_project();
        let tool = GrepProject;
        let output = tool
            .run(Input {
                pattern: "Hello".to_string(),
                path: Some(dir.path().to_string_lossy().to_string()),
                include_glob: None,
                exclude_glob: None,
                max_results: None,
                case_sensitive: true,
            })
            .unwrap();
        assert!(output.success);
        assert!(output.total_matches >= 3); // "Hello" appears in multiple files
        assert!(output.files_searched >= 2);
    }

    #[test]
    fn test_grep_project_with_include_glob() {
        let dir = create_test_project();
        let tool = GrepProject;
        let output = tool
            .run(Input {
                pattern: "Hello".to_string(),
                path: Some(dir.path().to_string_lossy().to_string()),
                include_glob: Some("*.rs".to_string()),
                exclude_glob: None,
                max_results: None,
                case_sensitive: true,
            })
            .unwrap();
        assert!(output.success);
        // Should only search .rs files
        for file_match in &output.matches {
            assert!(file_match.file.ends_with(".rs"));
        }
    }

    #[test]
    fn test_grep_project_with_exclude_glob() {
        let dir = create_test_project();
        let tool = GrepProject;
        let output = tool
            .run(Input {
                pattern: "Hello".to_string(),
                path: Some(dir.path().to_string_lossy().to_string()),
                include_glob: None,
                exclude_glob: Some("*.txt".to_string()),
                max_results: None,
                case_sensitive: true,
            })
            .unwrap();
        assert!(output.success);
        // Should not include .txt files
        for file_match in &output.matches {
            assert!(!file_match.file.ends_with(".txt"));
        }
    }

    #[test]
    fn test_grep_project_case_insensitive() {
        let dir = create_test_project();
        let tool = GrepProject;
        let output = tool
            .run(Input {
                pattern: "hello".to_string(),
                path: Some(dir.path().to_string_lossy().to_string()),
                include_glob: None,
                exclude_glob: None,
                max_results: None,
                case_sensitive: false,
            })
            .unwrap();
        assert!(output.success);
        // Should match "Hello" case-insensitively
        assert!(output.total_matches >= 3);
    }

    #[test]
    fn test_grep_project_max_results() {
        let dir = create_test_project();
        let tool = GrepProject;
        let output = tool
            .run(Input {
                pattern: "Hello".to_string(),
                path: Some(dir.path().to_string_lossy().to_string()),
                include_glob: None,
                exclude_glob: None,
                max_results: Some(1),
                case_sensitive: true,
            })
            .unwrap();
        assert!(output.success);
        assert!(output.total_matches <= 1);
        assert!(output.truncated);
    }

    #[test]
    fn test_grep_project_path_not_found() {
        let tool = GrepProject;
        let output = tool
            .run(Input {
                pattern: "test".to_string(),
                path: Some("/nonexistent/path".to_string()),
                include_glob: None,
                exclude_glob: None,
                max_results: None,
                case_sensitive: true,
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("not found"));
    }

    #[test]
    fn test_grep_project_no_matches() {
        let dir = create_test_project();
        let tool = GrepProject;
        let output = tool
            .run(Input {
                pattern: "xyznonexistent123".to_string(),
                path: Some(dir.path().to_string_lossy().to_string()),
                include_glob: None,
                exclude_glob: None,
                max_results: None,
                case_sensitive: true,
            })
            .unwrap();
        assert!(output.success);
        assert_eq!(output.total_matches, 0);
        assert!(output.matches.is_empty());
    }

    #[test]
    fn test_schema_has_required_pattern() {
        let schema = GrepProject::schema();
        assert_eq!(schema["required"][0], "pattern");
    }
}
