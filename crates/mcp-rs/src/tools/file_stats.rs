use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

pub struct FileStats;

#[derive(Deserialize)]
pub struct Input {
    pub path: String,
}

#[derive(Serialize)]
pub struct Output {
    pub exists: bool,
    pub is_file: bool,
    pub is_dir: bool,
    pub is_symlink: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Tool for FileStats {
    const NAME: &'static str = "file_stats";
    const DESCRIPTION: &'static str =
        "Get file or directory statistics including size, timestamps, and permissions";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let resolved_path = workspace::resolve_path(&input.path);
        let path = resolved_path.as_path();

        // Check if path exists
        if !path.exists() {
            return Ok(Output {
                exists: false,
                is_file: false,
                is_dir: false,
                is_symlink: false,
                size: None,
                modified: None,
                created: None,
                permissions: None,
                error: None,
            });
        }

        // Get metadata (follow symlinks)
        let metadata = match fs::metadata(path) {
            Ok(m) => m,
            Err(e) => {
                return Ok(Output {
                    exists: true,
                    is_file: false,
                    is_dir: false,
                    is_symlink: false,
                    size: None,
                    modified: None,
                    created: None,
                    permissions: None,
                    error: Some(format!("Failed to get metadata: {}", e)),
                });
            }
        };

        // Check if symlink (using symlink_metadata which doesn't follow)
        let is_symlink = fs::symlink_metadata(path)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false);

        let is_file = metadata.is_file();
        let is_dir = metadata.is_dir();

        // Get size (only meaningful for files)
        let size = if is_file {
            Some(metadata.len())
        } else if is_dir {
            // Calculate directory size
            Self::calculate_dir_size(path).ok()
        } else {
            None
        };

        // Get modified time
        let modified = metadata.modified().ok().and_then(|time| {
            time.duration_since(UNIX_EPOCH)
                .ok()
                .map(|d| format_timestamp(d.as_secs()))
        });

        // Get created time
        let created = metadata.created().ok().and_then(|time| {
            time.duration_since(UNIX_EPOCH)
                .ok()
                .map(|d| format_timestamp(d.as_secs()))
        });

        // Get permissions (Unix-style on Unix systems)
        #[cfg(unix)]
        let permissions = {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();
            Some(format!("{:o}", mode & 0o777))
        };

        #[cfg(not(unix))]
        let permissions = {
            let readonly = metadata.permissions().readonly();
            Some(if readonly {
                "readonly".to_string()
            } else {
                "read-write".to_string()
            })
        };

        Ok(Output {
            exists: true,
            is_file,
            is_dir,
            is_symlink,
            size,
            modified,
            created,
            permissions,
            error: None,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file or directory to get statistics for"
                }
            },
            "required": ["path"]
        })
    }
}

impl FileStats {
    fn calculate_dir_size(path: &Path) -> std::io::Result<u64> {
        let mut total_size = 0u64;

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                // Recursively calculate subdirectory size
                if let Ok(subdir_size) = Self::calculate_dir_size(&entry.path()) {
                    total_size += subdir_size;
                }
            }
        }

        Ok(total_size)
    }
}

fn format_timestamp(secs: u64) -> String {
    // Simple ISO 8601 formatting without external dependencies
    // secs is seconds since Unix epoch
    let days_since_epoch = secs / 86400;
    let remaining_secs = secs % 86400;
    let hours = remaining_secs / 3600;
    let minutes = (remaining_secs % 3600) / 60;
    let seconds = remaining_secs % 60;

    // Calculate year, month, day from days since epoch
    // This is a simplified calculation (doesn't handle all edge cases perfectly)
    let mut days = days_since_epoch as i64;
    let mut year = 1970i32;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let mut month = 1u32;
    let days_in_months = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    for &dim in &days_in_months {
        if days < dim as i64 {
            break;
        }
        days -= dim as i64;
        month += 1;
    }

    let day = days + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_file_stats_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();

        let tool = FileStats;
        let output = tool
            .run(Input {
                path: file_path.to_string_lossy().to_string(),
            })
            .unwrap();

        assert!(output.exists);
        assert!(output.is_file);
        assert!(!output.is_dir);
        assert!(!output.is_symlink);
        assert!(output.size.is_some());
        assert!(output.modified.is_some());
        assert!(output.permissions.is_some());
        assert!(output.error.is_none());
    }

    #[test]
    fn test_file_stats_directory() {
        let dir = TempDir::new().unwrap();

        let tool = FileStats;
        let output = tool
            .run(Input {
                path: dir.path().to_string_lossy().to_string(),
            })
            .unwrap();

        assert!(output.exists);
        assert!(!output.is_file);
        assert!(output.is_dir);
        assert!(output.error.is_none());
    }

    #[test]
    fn test_file_stats_nonexistent() {
        let tool = FileStats;
        let output = tool
            .run(Input {
                path: "/nonexistent/path/file.txt".to_string(),
            })
            .unwrap();

        assert!(!output.exists);
        assert!(!output.is_file);
        assert!(!output.is_dir);
        assert!(output.error.is_none()); // No error, just exists=false
    }

    #[test]
    fn test_file_stats_directory_size() {
        let dir = TempDir::new().unwrap();

        // Create files in directory
        let mut file1 = File::create(dir.path().join("file1.txt")).unwrap();
        write!(file1, "hello").unwrap(); // 5 bytes

        let mut file2 = File::create(dir.path().join("file2.txt")).unwrap();
        write!(file2, "world!").unwrap(); // 6 bytes

        let tool = FileStats;
        let output = tool
            .run(Input {
                path: dir.path().to_string_lossy().to_string(),
            })
            .unwrap();

        assert!(output.exists);
        assert!(output.is_dir);
        assert!(output.size.is_some());
        assert!(output.size.unwrap() >= 11); // At least 5 + 6 bytes
    }

    #[test]
    fn test_format_timestamp() {
        // Test Unix epoch (1970-01-01 00:00:00)
        assert_eq!(format_timestamp(0), "1970-01-01T00:00:00Z");

        // Test a known date: 2024-01-01 12:00:00 UTC
        // = 1704110400 seconds since epoch
        let result = format_timestamp(1704110400);
        assert!(result.starts_with("2024-01-01"));
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000)); // Divisible by 400
        assert!(is_leap_year(2024)); // Divisible by 4, not 100
        assert!(!is_leap_year(1900)); // Divisible by 100, not 400
        assert!(!is_leap_year(2023)); // Not divisible by 4
    }

    #[test]
    fn test_schema_has_required_path() {
        let schema = FileStats::schema();
        assert_eq!(schema["required"][0], "path");
    }
}
