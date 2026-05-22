use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use rusqlite::{params_from_iter, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct SqliteQuery;

const DEFAULT_MAX_ROWS: usize = 500;

#[derive(Deserialize)]
pub struct Input {
    pub db_path: String,
    pub query: String,
    #[serde(default)]
    pub params: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub max_rows: Option<usize>,
}

#[derive(Serialize)]
pub struct Output {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<Vec<serde_json::Value>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_count: Option<usize>,
    pub truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Tool for SqliteQuery {
    const NAME: &'static str = "sqlite_query";
    const DESCRIPTION: &'static str = "Execute read-only SQL queries against a SQLite database";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let resolved_path = workspace::resolve_path(&input.db_path);
        let path = resolved_path.as_path();
        if !path.exists() {
            return Ok(Output {
                success: false,
                columns: None,
                rows: None,
                row_count: None,
                truncated: false,
                error: Some(format!("Database not found: {}", input.db_path)),
            });
        }

        let trimmed = input.query.trim().to_uppercase();
        if !trimmed.starts_with("SELECT")
            && !trimmed.starts_with("PRAGMA")
            && !trimmed.starts_with("EXPLAIN")
            && !trimmed.starts_with("WITH")
        {
            return Ok(Output {
                success: false,
                columns: None,
                rows: None,
                row_count: None,
                truncated: false,
                error: Some(
                    "Only SELECT, PRAGMA, EXPLAIN, and WITH queries are allowed".to_string(),
                ),
            });
        }

        let conn = match rusqlite::Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        ) {
            Ok(c) => c,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    columns: None,
                    rows: None,
                    row_count: None,
                    truncated: false,
                    error: Some(format!("Failed to open database: {}", e)),
                });
            }
        };

        let max_rows = input.max_rows.unwrap_or(DEFAULT_MAX_ROWS);
        let params: Vec<String> = input
            .params
            .unwrap_or_default()
            .into_iter()
            .map(|v| match v {
                serde_json::Value::String(s) => s,
                other => other.to_string(),
            })
            .collect();

        let mut stmt = match conn.prepare(&input.query) {
            Ok(s) => s,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    columns: None,
                    rows: None,
                    row_count: None,
                    truncated: false,
                    error: Some(format!("Query preparation failed: {}", e)),
                });
            }
        };

        let columns: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let col_count = columns.len();
        let row_result = stmt.query_map(params_from_iter(params.iter()), |row| {
            let mut values = Vec::with_capacity(col_count);
            for i in 0..col_count {
                let val: rusqlite::types::Value = row.get(i)?;
                values.push(sqlite_to_json(val));
            }
            Ok(values)
        });

        let rows_iter = match row_result {
            Ok(r) => r,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    columns: None,
                    rows: None,
                    row_count: None,
                    truncated: false,
                    error: Some(format!("Query execution failed: {}", e)),
                });
            }
        };

        let mut rows = Vec::new();
        let mut truncated = false;

        for row in rows_iter {
            match row {
                Ok(values) => {
                    if rows.len() >= max_rows {
                        truncated = true;
                        break;
                    }
                    rows.push(values);
                }
                Err(e) => {
                    return Ok(Output {
                        success: false,
                        columns: Some(columns),
                        rows: None,
                        row_count: None,
                        truncated: false,
                        error: Some(format!("Row fetch error: {}", e)),
                    });
                }
            }
        }

        let row_count = rows.len();
        Ok(Output {
            success: true,
            columns: Some(columns),
            rows: Some(rows),
            row_count: Some(row_count),
            truncated,
            error: None,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "db_path": {
                    "type": "string",
                    "description": "Path to the SQLite database file"
                },
                "query": {
                    "type": "string",
                    "description": "SQL query to execute (SELECT, PRAGMA, EXPLAIN, or WITH only)"
                },
                "params": {
                    "type": "array",
                    "items": {},
                    "description": "Query parameters for prepared statements"
                },
                "max_rows": {
                    "type": "integer",
                    "description": "Maximum rows to return (default: 500)",
                    "default": 500
                }
            },
            "required": ["db_path", "query"]
        })
    }
}

fn sqlite_to_json(val: rusqlite::types::Value) -> serde_json::Value {
    match val {
        rusqlite::types::Value::Null => serde_json::Value::Null,
        rusqlite::types::Value::Integer(i) => json!(i),
        rusqlite::types::Value::Real(f) => serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        rusqlite::types::Value::Text(s) => serde_json::Value::String(s),
        rusqlite::types::Value::Blob(b) => {
            json!(format!("<blob {} bytes>", b.len()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_db() -> NamedTempFile {
        let file = NamedTempFile::new().unwrap();
        let conn = rusqlite::Connection::open(file.path()).unwrap();
        conn.execute_batch(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER);
             INSERT INTO users (name, age) VALUES ('Alice', 30);
             INSERT INTO users (name, age) VALUES ('Bob', 25);
             INSERT INTO users (name, age) VALUES ('Charlie', 35);",
        )
        .unwrap();
        file
    }

    #[test]
    fn test_basic_select() {
        let db = create_test_db();
        let tool = SqliteQuery;
        let output = tool
            .run(Input {
                db_path: db.path().to_string_lossy().to_string(),
                query: "SELECT * FROM users ORDER BY id".to_string(),
                params: None,
                max_rows: None,
            })
            .unwrap();
        assert!(output.success);
        assert_eq!(output.row_count, Some(3));
        let columns = output.columns.unwrap();
        assert_eq!(columns, vec!["id", "name", "age"]);
        let rows = output.rows.unwrap();
        assert_eq!(rows[0][1], json!("Alice"));
    }

    #[test]
    fn test_parameterized_query() {
        let db = create_test_db();
        let tool = SqliteQuery;
        let output = tool
            .run(Input {
                db_path: db.path().to_string_lossy().to_string(),
                query: "SELECT name FROM users WHERE age > ?".to_string(),
                params: Some(vec![json!("28")]),
                max_rows: None,
            })
            .unwrap();
        assert!(output.success);
        assert_eq!(output.row_count, Some(2));
    }

    #[test]
    fn test_max_rows_truncation() {
        let db = create_test_db();
        let tool = SqliteQuery;
        let output = tool
            .run(Input {
                db_path: db.path().to_string_lossy().to_string(),
                query: "SELECT * FROM users".to_string(),
                params: None,
                max_rows: Some(2),
            })
            .unwrap();
        assert!(output.success);
        assert_eq!(output.row_count, Some(2));
        assert!(output.truncated);
    }

    #[test]
    fn test_write_query_rejected() {
        let db = create_test_db();
        let tool = SqliteQuery;
        let output = tool
            .run(Input {
                db_path: db.path().to_string_lossy().to_string(),
                query: "INSERT INTO users (name, age) VALUES ('Eve', 28)".to_string(),
                params: None,
                max_rows: None,
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.error.unwrap().contains("Only SELECT"));
    }

    #[test]
    fn test_pragma_allowed() {
        let db = create_test_db();
        let tool = SqliteQuery;
        let output = tool
            .run(Input {
                db_path: db.path().to_string_lossy().to_string(),
                query: "PRAGMA table_info(users)".to_string(),
                params: None,
                max_rows: None,
            })
            .unwrap();
        assert!(output.success);
        assert!(output.row_count.unwrap() > 0);
    }

    #[test]
    fn test_db_not_found() {
        let tool = SqliteQuery;
        let output = tool
            .run(Input {
                db_path: "/nonexistent/db.sqlite".to_string(),
                query: "SELECT 1".to_string(),
                params: None,
                max_rows: None,
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.error.unwrap().contains("not found"));
    }

    #[test]
    fn test_schema_has_required() {
        let schema = SqliteQuery::schema();
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("db_path")));
        assert!(required.contains(&json!("query")));
    }
}
