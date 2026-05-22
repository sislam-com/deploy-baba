use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;

pub struct MarkdownQuery;

#[derive(Deserialize)]
pub struct Input {
    pub path: String,
    pub action: String,
    #[serde(default)]
    pub filter: Option<String>,
}

#[derive(Serialize)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub line: usize,
}

#[derive(Serialize)]
pub struct Output {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headings: Option<Vec<Heading>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tables: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<SearchMatch>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct SearchMatch {
    pub line: usize,
    pub text: String,
}

impl Tool for MarkdownQuery {
    const NAME: &'static str = "markdown_query";
    const DESCRIPTION: &'static str =
        "Navigate markdown files: list headings, get sections, find tables, search content";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let resolved_path = workspace::resolve_path(&input.path);
        let path = resolved_path.as_path();

        if !path.exists() {
            return Ok(Output {
                success: false,
                headings: None,
                section: None,
                tables: None,
                matches: None,
                error: Some(format!("File not found: {}", input.path)),
            });
        }

        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    headings: None,
                    section: None,
                    tables: None,
                    matches: None,
                    error: Some(format!("Failed to read file: {}", e)),
                });
            }
        };

        match input.action.as_str() {
            "list_headings" => {
                let headings = list_headings(&contents);
                Ok(Output {
                    success: true,
                    headings: Some(headings),
                    section: None,
                    tables: None,
                    matches: None,
                    error: None,
                })
            }
            "get_section" => {
                let filter = input.filter.unwrap_or_default();
                match get_section(&contents, &filter) {
                    Some(section) => Ok(Output {
                        success: true,
                        headings: None,
                        section: Some(section),
                        tables: None,
                        matches: None,
                        error: None,
                    }),
                    None => Ok(Output {
                        success: false,
                        headings: None,
                        section: None,
                        tables: None,
                        matches: None,
                        error: Some(format!("Section not found: {}", filter)),
                    }),
                }
            }
            "list_tables" => {
                let tables = list_tables(&contents);
                Ok(Output {
                    success: true,
                    headings: None,
                    section: None,
                    tables: Some(tables),
                    matches: None,
                    error: None,
                })
            }
            "search_content" => {
                let filter = input.filter.unwrap_or_default();
                let matches = search_content(&contents, &filter);
                Ok(Output {
                    success: true,
                    headings: None,
                    section: None,
                    tables: None,
                    matches: Some(matches),
                    error: None,
                })
            }
            other => Ok(Output {
                success: false,
                headings: None,
                section: None,
                tables: None,
                matches: None,
                error: Some(format!(
                    "Unknown action: {}. Use: list_headings, get_section, list_tables, search_content",
                    other
                )),
            }),
        }
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the markdown file"
                },
                "action": {
                    "type": "string",
                    "description": "Action to perform",
                    "enum": ["list_headings", "get_section", "list_tables", "search_content"]
                },
                "filter": {
                    "type": "string",
                    "description": "For get_section: heading text to find. For search_content: search term"
                }
            },
            "required": ["path", "action"]
        })
    }
}

fn list_headings(content: &str) -> Vec<Heading> {
    content
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let trimmed = line.trim_start();
            if !trimmed.starts_with('#') {
                return None;
            }
            let level = trimmed.chars().take_while(|c| *c == '#').count();
            if level > 6 || level == 0 {
                return None;
            }
            let text = trimmed[level..].trim().to_string();
            if text.is_empty() {
                return None;
            }
            Some(Heading {
                level: level as u8,
                text,
                line: i + 1,
            })
        })
        .collect()
}

fn get_section(content: &str, heading_text: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let lower_search = heading_text.to_lowercase();

    let mut start_idx = None;
    let mut start_level = 0u8;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') {
            continue;
        }
        let level = trimmed.chars().take_while(|c| *c == '#').count() as u8;
        let text = trimmed[level as usize..].trim();

        if let Some(start) = start_idx {
            if level <= start_level {
                return Some(lines[start..i].join("\n"));
            }
        } else if text.to_lowercase().contains(&lower_search) {
            start_idx = Some(i);
            start_level = level;
        }
    }

    start_idx.map(|idx| lines[idx..].join("\n"))
}

fn list_tables(content: &str) -> Vec<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut tables = Vec::new();
    let mut current_table: Vec<&str> = Vec::new();
    let mut in_table = false;

    for line in &lines {
        let is_table_line = line.trim_start().starts_with('|');
        if is_table_line {
            in_table = true;
            current_table.push(line);
        } else if in_table {
            if !current_table.is_empty() {
                tables.push(current_table.join("\n"));
                current_table.clear();
            }
            in_table = false;
        }
    }

    if !current_table.is_empty() {
        tables.push(current_table.join("\n"));
    }

    tables
}

fn search_content(content: &str, term: &str) -> Vec<SearchMatch> {
    let lower_term = term.to_lowercase();
    content
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            if line.to_lowercase().contains(&lower_term) {
                Some(SearchMatch {
                    line: i + 1,
                    text: line.to_string(),
                })
            } else {
                None
            }
        })
        .take(100)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    const SAMPLE_MD: &str = r#"# Project Title

Some intro text.

## Installation

Run `cargo install`.

### Prerequisites

- Rust 1.70+
- Git

## Usage

| Command | Description |
|---------|-------------|
| build   | Build project |
| test    | Run tests |

## API Reference

Another section here.
"#;

    fn create_temp_md(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_list_headings() {
        let file = create_temp_md(SAMPLE_MD);
        let tool = MarkdownQuery;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                action: "list_headings".to_string(),
                filter: None,
            })
            .unwrap();
        assert!(output.success);
        let headings = output.headings.unwrap();
        assert_eq!(headings.len(), 5);
        assert_eq!(headings[0].level, 1);
        assert_eq!(headings[0].text, "Project Title");
        assert_eq!(headings[2].level, 3);
        assert_eq!(headings[2].text, "Prerequisites");
    }

    #[test]
    fn test_get_section() {
        let file = create_temp_md(SAMPLE_MD);
        let tool = MarkdownQuery;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                action: "get_section".to_string(),
                filter: Some("Installation".to_string()),
            })
            .unwrap();
        assert!(output.success);
        let section = output.section.unwrap();
        assert!(section.contains("cargo install"));
        assert!(section.contains("Prerequisites"));
        assert!(!section.contains("API Reference"));
    }

    #[test]
    fn test_get_section_not_found() {
        let file = create_temp_md(SAMPLE_MD);
        let tool = MarkdownQuery;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                action: "get_section".to_string(),
                filter: Some("Nonexistent".to_string()),
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.error.unwrap().contains("not found"));
    }

    #[test]
    fn test_list_tables() {
        let file = create_temp_md(SAMPLE_MD);
        let tool = MarkdownQuery;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                action: "list_tables".to_string(),
                filter: None,
            })
            .unwrap();
        assert!(output.success);
        let tables = output.tables.unwrap();
        assert_eq!(tables.len(), 1);
        assert!(tables[0].contains("Command"));
        assert!(tables[0].contains("build"));
    }

    #[test]
    fn test_search_content() {
        let file = create_temp_md(SAMPLE_MD);
        let tool = MarkdownQuery;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                action: "search_content".to_string(),
                filter: Some("cargo".to_string()),
            })
            .unwrap();
        assert!(output.success);
        let matches = output.matches.unwrap();
        assert!(!matches.is_empty());
        assert!(matches[0].text.contains("cargo"));
    }

    #[test]
    fn test_unknown_action() {
        let file = create_temp_md(SAMPLE_MD);
        let tool = MarkdownQuery;
        let output = tool
            .run(Input {
                path: file.path().to_string_lossy().to_string(),
                action: "invalid".to_string(),
                filter: None,
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.error.unwrap().contains("Unknown action"));
    }

    #[test]
    fn test_file_not_found() {
        let tool = MarkdownQuery;
        let output = tool
            .run(Input {
                path: "/nonexistent/file.md".to_string(),
                action: "list_headings".to_string(),
                filter: None,
            })
            .unwrap();
        assert!(!output.success);
    }

    #[test]
    fn test_schema_has_required() {
        let schema = MarkdownQuery::schema();
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("path")));
        assert!(required.contains(&json!("action")));
    }
}
