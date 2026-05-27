use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;

pub struct ListDirectory;

#[derive(Deserialize)]
pub struct Input {
    pub path: String,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub pattern: Option<String>,
}

#[derive(Serialize)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

#[derive(Serialize)]
pub struct Output {
    pub entries: Vec<Entry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Tool for ListDirectory {
    const NAME: &'static str = "list_directory";
    const DESCRIPTION: &'static str =
        "List directory contents with optional recursion and pattern filtering. Prefer native Bash ls for general listings; use this for MCP workspace-scoped directory browsing.";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let resolved_path = workspace::resolve_path(&input.path);
        let path = resolved_path.as_path();

        // Check if path exists
        if !path.exists() {
            return Ok(Output {
                entries: vec![],
                error: Some(format!("Path not found: {}", input.path)),
            });
        }

        // Check if it's a directory
        if !path.is_dir() {
            return Ok(Output {
                entries: vec![],
                error: Some(format!("Path is not a directory: {}", input.path)),
            });
        }

        let mut entries = Vec::new();

        if input.recursive {
            if let Err(e) = Self::collect_entries_recursive(path, &input.pattern, &mut entries) {
                return Ok(Output {
                    entries,
                    error: Some(format!("Error during directory traversal: {}", e)),
                });
            }
        } else if let Err(e) = Self::collect_entries(path, &input.pattern, &mut entries) {
            return Ok(Output {
                entries,
                error: Some(format!("Error reading directory: {}", e)),
            });
        }

        // Sort entries: directories first, then alphabetically
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        Ok(Output {
            entries,
            error: None,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the directory to list"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Whether to list contents recursively",
                    "default": false
                },
                "pattern": {
                    "type": "string",
                    "description": "Filter entries by name pattern (simple substring match)"
                }
            },
            "required": ["path"]
        })
    }
}

impl ListDirectory {
    fn collect_entries(
        dir: &Path,
        pattern: &Option<String>,
        entries: &mut Vec<Entry>,
    ) -> std::io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();

            // Apply pattern filter
            if let Some(ref pat) = pattern {
                if !name.to_lowercase().contains(&pat.to_lowercase()) {
                    continue;
                }
            }

            let size = if metadata.is_file() {
                Some(metadata.len())
            } else {
                None
            };

            entries.push(Entry {
                name,
                path: entry.path().to_string_lossy().to_string(),
                is_dir: metadata.is_dir(),
                size,
            });
        }
        Ok(())
    }

    fn collect_entries_recursive(
        dir: &Path,
        pattern: &Option<String>,
        entries: &mut Vec<Entry>,
    ) -> std::io::Result<()> {
        Self::collect_entries(dir, pattern, entries)?;

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.metadata()?.is_dir() {
                Self::collect_entries_recursive(&entry.path(), pattern, entries)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    fn create_test_directory() -> TempDir {
        let dir = TempDir::new().unwrap();
        // Create some files
        File::create(dir.path().join("file1.txt")).unwrap();
        File::create(dir.path().join("file2.rs")).unwrap();
        // Create a subdirectory with a file
        fs::create_dir(dir.path().join("subdir")).unwrap();
        File::create(dir.path().join("subdir").join("nested.txt")).unwrap();
        dir
    }

    #[test]
    fn test_list_directory_basic() {
        let dir = create_test_directory();
        let tool = ListDirectory;
        let output = tool
            .run(Input {
                path: dir.path().to_string_lossy().to_string(),
                recursive: false,
                pattern: None,
            })
            .unwrap();
        assert!(output.error.is_none());
        assert!(!output.entries.is_empty());
        // Should have file1.txt, file2.rs, subdir
        assert_eq!(output.entries.len(), 3);
    }

    #[test]
    fn test_list_directory_recursive() {
        let dir = create_test_directory();
        let tool = ListDirectory;
        let output = tool
            .run(Input {
                path: dir.path().to_string_lossy().to_string(),
                recursive: true,
                pattern: None,
            })
            .unwrap();
        assert!(output.error.is_none());
        // Should have file1.txt, file2.rs, subdir, and nested.txt
        assert_eq!(output.entries.len(), 4);
    }

    #[test]
    fn test_list_directory_with_pattern() {
        let dir = create_test_directory();
        let tool = ListDirectory;
        let output = tool
            .run(Input {
                path: dir.path().to_string_lossy().to_string(),
                recursive: true,
                pattern: Some(".txt".to_string()),
            })
            .unwrap();
        assert!(output.error.is_none());
        // Should only have file1.txt and nested.txt
        assert_eq!(output.entries.len(), 2);
        for entry in &output.entries {
            assert!(entry.name.contains(".txt"));
        }
    }

    #[test]
    fn test_list_directory_not_found() {
        let tool = ListDirectory;
        let output = tool
            .run(Input {
                path: "/nonexistent/directory".to_string(),
                recursive: false,
                pattern: None,
            })
            .unwrap();
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("not found"));
    }

    #[test]
    fn test_list_directory_file_path() {
        let dir = create_test_directory();
        let file_path = dir.path().join("file1.txt");
        let tool = ListDirectory;
        let output = tool
            .run(Input {
                path: file_path.to_string_lossy().to_string(),
                recursive: false,
                pattern: None,
            })
            .unwrap();
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("not a directory"));
    }

    #[test]
    fn test_entries_sorted_dirs_first() {
        let dir = create_test_directory();
        let tool = ListDirectory;
        let output = tool
            .run(Input {
                path: dir.path().to_string_lossy().to_string(),
                recursive: false,
                pattern: None,
            })
            .unwrap();
        assert!(output.error.is_none());
        // First entry should be the directory
        assert!(output.entries[0].is_dir);
    }

    #[test]
    fn test_schema_has_required_path() {
        let schema = ListDirectory::schema();
        assert_eq!(schema["required"][0], "path");
    }
}
