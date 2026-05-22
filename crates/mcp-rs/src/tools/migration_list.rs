use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;

pub struct MigrationList;

#[derive(Deserialize)]
pub struct Input {
    pub migrations_dir: String,
}

#[derive(Serialize)]
pub struct Migration {
    pub number: u32,
    pub name: String,
    pub file_path: String,
    pub line_count: u32,
}

#[derive(Serialize)]
pub struct Output {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migrations: Option<Vec<Migration>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Tool for MigrationList {
    const NAME: &'static str = "migration_list";
    const DESCRIPTION: &'static str = "List SQL migration files (convention: NNN_name.sql)";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let resolved_dir = workspace::resolve_path(&input.migrations_dir);
        let dir = resolved_dir.as_path();

        if !dir.exists() {
            return Ok(Output {
                success: false,
                migrations: None,
                total: None,
                error: Some(format!("Directory not found: {}", input.migrations_dir)),
            });
        }

        if !dir.is_dir() {
            return Ok(Output {
                success: false,
                migrations: None,
                total: None,
                error: Some(format!("Path is not a directory: {}", input.migrations_dir)),
            });
        }

        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    migrations: None,
                    total: None,
                    error: Some(format!("Failed to read directory: {}", e)),
                });
            }
        };

        let mut migrations: Vec<Migration> = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            if !file_name.ends_with(".sql") {
                continue;
            }

            if let Some(migration) = parse_migration_file(&path, &file_name) {
                migrations.push(migration);
            }
        }

        migrations.sort_by_key(|m| m.number);
        let total = migrations.len();

        Ok(Output {
            success: true,
            migrations: Some(migrations),
            total: Some(total),
            error: None,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "migrations_dir": {
                    "type": "string",
                    "description": "Path to the migrations directory"
                }
            },
            "required": ["migrations_dir"]
        })
    }
}

fn parse_migration_file(path: &Path, file_name: &str) -> Option<Migration> {
    let stem = file_name.strip_suffix(".sql")?;
    let underscore_pos = stem.find('_')?;
    let number_str = &stem[..underscore_pos];
    let number: u32 = number_str.parse().ok()?;
    let name = stem[underscore_pos + 1..].to_string();

    let line_count = fs::read_to_string(path)
        .map(|c| c.lines().count() as u32)
        .unwrap_or(0);

    Some(Migration {
        number,
        name,
        file_path: path.to_string_lossy().to_string(),
        line_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_migrations_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        let files = [
            ("001_create_users.sql", "CREATE TABLE users (id INTEGER PRIMARY KEY);\n"),
            ("002_add_email.sql", "ALTER TABLE users ADD COLUMN email TEXT;\nALTER TABLE users ADD COLUMN name TEXT;\n"),
            ("003_create_posts.sql", "CREATE TABLE posts (\n  id INTEGER PRIMARY KEY,\n  user_id INTEGER,\n  title TEXT\n);\n"),
            ("README.md", "# Migrations\n"),
        ];
        for (name, content) in files {
            let path = dir.path().join(name);
            let mut f = fs::File::create(&path).unwrap();
            write!(f, "{}", content).unwrap();
        }
        dir
    }

    #[test]
    fn test_list_migrations() {
        let dir = create_migrations_dir();
        let tool = MigrationList;
        let output = tool
            .run(Input {
                migrations_dir: dir.path().to_string_lossy().to_string(),
            })
            .unwrap();
        assert!(output.success);
        let migrations = output.migrations.unwrap();
        assert_eq!(migrations.len(), 3);
        assert_eq!(migrations[0].number, 1);
        assert_eq!(migrations[0].name, "create_users");
        assert_eq!(migrations[1].number, 2);
        assert_eq!(migrations[2].number, 3);
        assert_eq!(output.total, Some(3));
    }

    #[test]
    fn test_migration_line_counts() {
        let dir = create_migrations_dir();
        let tool = MigrationList;
        let output = tool
            .run(Input {
                migrations_dir: dir.path().to_string_lossy().to_string(),
            })
            .unwrap();
        let migrations = output.migrations.unwrap();
        assert_eq!(migrations[0].line_count, 1);
        assert_eq!(migrations[1].line_count, 2);
        assert_eq!(migrations[2].line_count, 5);
    }

    #[test]
    fn test_dir_not_found() {
        let tool = MigrationList;
        let output = tool
            .run(Input {
                migrations_dir: "/nonexistent/dir".to_string(),
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.error.unwrap().contains("not found"));
    }

    #[test]
    fn test_empty_dir() {
        let dir = TempDir::new().unwrap();
        let tool = MigrationList;
        let output = tool
            .run(Input {
                migrations_dir: dir.path().to_string_lossy().to_string(),
            })
            .unwrap();
        assert!(output.success);
        assert_eq!(output.total, Some(0));
    }

    #[test]
    fn test_schema_has_required() {
        let schema = MigrationList::schema();
        assert_eq!(schema["required"][0], "migrations_dir");
    }
}
