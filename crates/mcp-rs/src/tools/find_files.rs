use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use glob::Pattern;
use serde::{Deserialize, Serialize};
use serde_json::json;
use walkdir::WalkDir;

pub struct FindFiles;

#[derive(Deserialize)]
pub struct Input {
    pub pattern: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub max_depth: Option<u32>,
    #[serde(default)]
    pub max_results: Option<u32>,
    #[serde(default = "default_include_dirs")]
    pub include_dirs: bool,
}

fn default_include_dirs() -> bool {
    false
}

#[derive(Serialize)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
}

#[derive(Serialize)]
pub struct Output {
    pub success: bool,
    pub files: Vec<FileEntry>,
    pub count: u32,
    pub truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Tool for FindFiles {
    const NAME: &'static str = "find_files";
    const DESCRIPTION: &'static str = "Find files by name pattern using glob matching";

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
                files: vec![],
                count: 0,
                truncated: false,
                error: Some(format!("Path not found: {}", display_path)),
            });
        }

        // Compile glob pattern
        let pattern = match Pattern::new(&input.pattern) {
            Ok(p) => p,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    files: vec![],
                    count: 0,
                    truncated: false,
                    error: Some(format!("Invalid glob pattern: {}", e)),
                });
            }
        };

        let max_results = input.max_results.unwrap_or(1000) as usize;

        // Build walker with optional max_depth
        let mut walker = WalkDir::new(path);
        if let Some(depth) = input.max_depth {
            walker = walker.max_depth(depth as usize);
        }

        let mut files = Vec::new();
        let mut truncated = false;

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let file_type = entry.file_type();

            // Skip directories unless include_dirs is true
            if file_type.is_dir() && !input.include_dirs {
                continue;
            }

            let entry_path = entry.path();
            let file_name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Match against the pattern
            // Try matching both filename and full path
            if !pattern.matches(file_name) && !pattern.matches(&entry_path.to_string_lossy()) {
                continue;
            }

            if files.len() >= max_results {
                truncated = true;
                break;
            }

            files.push(FileEntry {
                path: entry_path.to_string_lossy().to_string(),
                name: file_name.to_string(),
                is_dir: file_type.is_dir(),
            });
        }

        let count = files.len() as u32;

        Ok(Output {
            success: true,
            files,
            count,
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
                    "description": "Glob pattern to match file names (e.g., '*.rs', 'test_*.py', 'README*')"
                },
                "path": {
                    "type": "string",
                    "description": "Base path to search in (defaults to current directory)"
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum directory depth to search (unlimited if not specified)",
                    "minimum": 1
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return (default: 1000)",
                    "default": 1000
                },
                "include_dirs": {
                    "type": "boolean",
                    "description": "Whether to include directories in results (default: false)",
                    "default": false
                }
            },
            "required": ["pattern"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn create_test_structure() -> TempDir {
        let dir = TempDir::new().unwrap();

        // Create files
        File::create(dir.path().join("file1.rs")).unwrap();
        File::create(dir.path().join("file2.rs")).unwrap();
        File::create(dir.path().join("readme.txt")).unwrap();

        // Create subdirectory with files
        fs::create_dir(dir.path().join("subdir")).unwrap();
        File::create(dir.path().join("subdir").join("nested.rs")).unwrap();
        File::create(dir.path().join("subdir").join("data.json")).unwrap();

        dir
    }

    #[test]
    fn test_find_files_basic() {
        let dir = create_test_structure();
        let tool = FindFiles;
        let output = tool
            .run(Input {
                pattern: "*.rs".to_string(),
                path: Some(dir.path().to_string_lossy().to_string()),
                max_depth: None,
                max_results: None,
                include_dirs: false,
            })
            .unwrap();

        assert!(output.success);
        assert_eq!(output.count, 3); // file1.rs, file2.rs, nested.rs
        for file in &output.files {
            assert!(file.name.ends_with(".rs"));
            assert!(!file.is_dir);
        }
    }

    #[test]
    fn test_find_files_with_max_depth() {
        let dir = create_test_structure();
        let tool = FindFiles;
        let output = tool
            .run(Input {
                pattern: "*.rs".to_string(),
                path: Some(dir.path().to_string_lossy().to_string()),
                max_depth: Some(1), // Only root level
                max_results: None,
                include_dirs: false,
            })
            .unwrap();

        assert!(output.success);
        assert_eq!(output.count, 2); // Only file1.rs, file2.rs (not nested.rs)
    }

    #[test]
    fn test_find_files_include_dirs() {
        let dir = create_test_structure();
        let tool = FindFiles;
        let output = tool
            .run(Input {
                pattern: "*".to_string(),
                path: Some(dir.path().to_string_lossy().to_string()),
                max_depth: Some(1),
                max_results: None,
                include_dirs: true,
            })
            .unwrap();

        assert!(output.success);
        // Should include the root dir, files, and subdir
        let dirs: Vec<_> = output.files.iter().filter(|f| f.is_dir).collect();
        assert!(!dirs.is_empty());
    }

    #[test]
    fn test_find_files_max_results() {
        let dir = create_test_structure();
        let tool = FindFiles;
        let output = tool
            .run(Input {
                pattern: "*".to_string(),
                path: Some(dir.path().to_string_lossy().to_string()),
                max_depth: None,
                max_results: Some(2),
                include_dirs: false,
            })
            .unwrap();

        assert!(output.success);
        assert!(output.count <= 2);
        assert!(output.truncated);
    }

    #[test]
    fn test_find_files_path_not_found() {
        let tool = FindFiles;
        let output = tool
            .run(Input {
                pattern: "*.rs".to_string(),
                path: Some("/nonexistent/path".to_string()),
                max_depth: None,
                max_results: None,
                include_dirs: false,
            })
            .unwrap();

        assert!(!output.success);
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("not found"));
    }

    #[test]
    fn test_find_files_invalid_pattern() {
        let dir = create_test_structure();
        let tool = FindFiles;
        let output = tool
            .run(Input {
                pattern: "[invalid".to_string(), // Invalid glob
                path: Some(dir.path().to_string_lossy().to_string()),
                max_depth: None,
                max_results: None,
                include_dirs: false,
            })
            .unwrap();

        assert!(!output.success);
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("Invalid glob"));
    }

    #[test]
    fn test_find_files_no_matches() {
        let dir = create_test_structure();
        let tool = FindFiles;
        let output = tool
            .run(Input {
                pattern: "*.xyz".to_string(),
                path: Some(dir.path().to_string_lossy().to_string()),
                max_depth: None,
                max_results: None,
                include_dirs: false,
            })
            .unwrap();

        assert!(output.success);
        assert_eq!(output.count, 0);
        assert!(output.files.is_empty());
    }

    #[test]
    fn test_schema_has_required_pattern() {
        let schema = FindFiles::schema();
        assert_eq!(schema["required"][0], "pattern");
    }
}
