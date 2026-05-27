use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use rusqlite::OpenFlags;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct SqliteSchema;

#[derive(Deserialize)]
pub struct Input {
    pub db_path: String,
    #[serde(default)]
    pub table: Option<String>,
}

#[derive(Serialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub not_null: bool,
    pub primary_key: bool,
    pub default_value: Option<String>,
}

#[derive(Serialize)]
pub struct IndexInfo {
    pub name: String,
    pub unique: bool,
    pub columns: Vec<String>,
}

#[derive(Serialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub indexes: Vec<IndexInfo>,
    pub row_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sql: Option<String>,
}

#[derive(Serialize)]
pub struct Output {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tables: Option<Vec<TableInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Tool for SqliteSchema {
    const NAME: &'static str = "sqlite_schema";
    const DESCRIPTION: &'static str = "Inspect SQLite database schema: tables, columns, indexes";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let resolved_path = workspace::resolve_path(&input.db_path);
        let path = resolved_path.as_path();
        if !path.exists() {
            return Ok(Output {
                success: false,
                tables: None,
                error: Some(format!("Database not found: {}", input.db_path)),
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
                    tables: None,
                    error: Some(format!("Failed to open database: {}", e)),
                });
            }
        };

        let table_names = if let Some(ref table) = input.table {
            vec![table.clone()]
        } else {
            match get_table_names(&conn) {
                Ok(names) => names,
                Err(e) => {
                    return Ok(Output {
                        success: false,
                        tables: None,
                        error: Some(format!("Failed to list tables: {}", e)),
                    });
                }
            }
        };

        let mut tables = Vec::new();
        for name in &table_names {
            match get_table_info(&conn, name) {
                Ok(info) => tables.push(info),
                Err(e) => {
                    return Ok(Output {
                        success: false,
                        tables: None,
                        error: Some(format!("Failed to inspect table '{}': {}", name, e)),
                    });
                }
            }
        }

        Ok(Output {
            success: true,
            tables: Some(tables),
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
                "table": {
                    "type": "string",
                    "description": "Specific table to inspect (omit for all tables)"
                }
            },
            "required": ["db_path"]
        })
    }
}

fn get_table_names(conn: &rusqlite::Connection) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )?;
    let names = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(names)
}

fn get_table_info(conn: &rusqlite::Connection, table: &str) -> Result<TableInfo, rusqlite::Error> {
    let mut col_stmt = conn.prepare(&format!("PRAGMA table_info(\"{}\")", table))?;
    let columns = col_stmt
        .query_map([], |row| {
            Ok(ColumnInfo {
                name: row.get(1)?,
                data_type: row.get(2)?,
                not_null: row.get::<_, i32>(3)? != 0,
                primary_key: row.get::<_, i32>(5)? != 0,
                default_value: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut idx_stmt = conn.prepare(&format!("PRAGMA index_list(\"{}\")", table))?;
    let index_list: Vec<(String, bool)> = idx_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(1)?, row.get::<_, i32>(2)? != 0))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut indexes = Vec::new();
    for (idx_name, unique) in index_list {
        let mut info_stmt = conn.prepare(&format!("PRAGMA index_info(\"{}\")", idx_name))?;
        let cols = info_stmt
            .query_map([], |row| row.get::<_, String>(2))?
            .collect::<Result<Vec<_>, _>>()?;
        indexes.push(IndexInfo {
            name: idx_name,
            unique,
            columns: cols,
        });
    }

    let row_count: i64 =
        conn.query_row(&format!("SELECT COUNT(*) FROM \"{}\"", table), [], |row| {
            row.get(0)
        })?;

    let sql: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name=?",
            [table],
            |row| row.get(0),
        )
        .ok();

    Ok(TableInfo {
        name: table.to_string(),
        columns,
        indexes,
        row_count,
        sql,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_db() -> NamedTempFile {
        let file = NamedTempFile::new().unwrap();
        let conn = rusqlite::Connection::open(file.path()).unwrap();
        conn.execute_batch(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT UNIQUE
             );
             CREATE INDEX idx_users_name ON users(name);
             INSERT INTO users (name, email) VALUES ('Alice', 'alice@test.com');
             INSERT INTO users (name, email) VALUES ('Bob', 'bob@test.com');

             CREATE TABLE posts (
                id INTEGER PRIMARY KEY,
                user_id INTEGER REFERENCES users(id),
                title TEXT NOT NULL,
                body TEXT
             );",
        )
        .unwrap();
        file
    }

    #[test]
    fn test_list_all_tables() {
        let db = create_test_db();
        let tool = SqliteSchema;
        let output = tool
            .run(Input {
                db_path: db.path().to_string_lossy().to_string(),
                table: None,
            })
            .unwrap();
        assert!(output.success);
        let tables = output.tables.unwrap();
        assert_eq!(tables.len(), 2);
        let names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"users"));
        assert!(names.contains(&"posts"));
    }

    #[test]
    fn test_specific_table() {
        let db = create_test_db();
        let tool = SqliteSchema;
        let output = tool
            .run(Input {
                db_path: db.path().to_string_lossy().to_string(),
                table: Some("users".to_string()),
            })
            .unwrap();
        assert!(output.success);
        let tables = output.tables.unwrap();
        assert_eq!(tables.len(), 1);
        let users = &tables[0];
        assert_eq!(users.name, "users");
        assert_eq!(users.row_count, 2);
        assert_eq!(users.columns.len(), 3);

        let name_col = users.columns.iter().find(|c| c.name == "name").unwrap();
        assert!(name_col.not_null);

        let id_col = users.columns.iter().find(|c| c.name == "id").unwrap();
        assert!(id_col.primary_key);
    }

    #[test]
    fn test_indexes() {
        let db = create_test_db();
        let tool = SqliteSchema;
        let output = tool
            .run(Input {
                db_path: db.path().to_string_lossy().to_string(),
                table: Some("users".to_string()),
            })
            .unwrap();
        let tables = output.tables.unwrap();
        let users = &tables[0];
        assert!(!users.indexes.is_empty());
        let name_idx = users
            .indexes
            .iter()
            .find(|i| i.columns.contains(&"name".to_string()));
        assert!(name_idx.is_some());
    }

    #[test]
    fn test_has_create_sql() {
        let db = create_test_db();
        let tool = SqliteSchema;
        let output = tool
            .run(Input {
                db_path: db.path().to_string_lossy().to_string(),
                table: Some("users".to_string()),
            })
            .unwrap();
        let tables = output.tables.unwrap();
        assert!(tables[0].sql.as_ref().unwrap().contains("CREATE TABLE"));
    }

    #[test]
    fn test_db_not_found() {
        let tool = SqliteSchema;
        let output = tool
            .run(Input {
                db_path: "/nonexistent/db.sqlite".to_string(),
                table: None,
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.error.unwrap().contains("not found"));
    }

    #[test]
    fn test_schema_has_required() {
        let schema = SqliteSchema::schema();
        assert_eq!(schema["required"][0], "db_path");
    }
}
