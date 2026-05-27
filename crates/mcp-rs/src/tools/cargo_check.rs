use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;

pub struct CargoCheck;

#[derive(Deserialize)]
pub struct Input {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub package: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize)]
pub struct Diagnostic {
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub level: DiagnosticLevel,
}

#[derive(Serialize)]
pub struct Output {
    pub success: bool,
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
    pub error_count: u32,
    pub warning_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Represents cargo's JSON message format for compiler messages
#[derive(Deserialize)]
struct CargoMessage {
    reason: String,
    #[serde(default)]
    message: Option<CompilerMessage>,
}

#[derive(Deserialize)]
struct CompilerMessage {
    message: String,
    level: DiagnosticLevel,
    #[serde(default)]
    spans: Vec<Span>,
}

#[derive(Deserialize)]
struct Span {
    file_name: String,
    line_start: u32,
    column_start: u32,
    #[serde(default)]
    is_primary: bool,
}

impl Tool for CargoCheck {
    const NAME: &'static str = "cargo_check";
    const DESCRIPTION: &'static str = "Run cargo check to validate code without building";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let mut cmd = Command::new("cargo");
        cmd.arg("check").arg("--message-format=json");

        let working_dir = input
            .path
            .as_ref()
            .map(workspace::resolve_path)
            .unwrap_or_else(workspace::root);
        cmd.current_dir(working_dir);

        // Add package filter if provided
        if let Some(ref package) = input.package {
            cmd.arg("--package").arg(package);
        }

        // Execute with timeout (simulated via blocking)
        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    errors: vec![],
                    warnings: vec![],
                    error_count: 0,
                    warning_count: 0,
                    error: Some(format!("Failed to execute cargo check: {}", e)),
                });
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Parse JSON output line by line
        for line in stdout.lines() {
            if let Ok(msg) = serde_json::from_str::<CargoMessage>(line) {
                if msg.reason == "compiler-message" {
                    if let Some(compiler_msg) = msg.message {
                        let primary_span = compiler_msg
                            .spans
                            .iter()
                            .find(|s| s.is_primary)
                            .or_else(|| compiler_msg.spans.first());

                        let diagnostic = Diagnostic {
                            message: compiler_msg.message.clone(),
                            file: primary_span.map(|s| s.file_name.clone()),
                            line: primary_span.map(|s| s.line_start),
                            column: primary_span.map(|s| s.column_start),
                            level: compiler_msg.level.clone(),
                        };

                        match compiler_msg.level {
                            DiagnosticLevel::Error => errors.push(diagnostic),
                            DiagnosticLevel::Warning => warnings.push(diagnostic),
                            _ => {} // Ignore notes, help, etc.
                        }
                    }
                }
            }
        }

        let error_count = errors.len() as u32;
        let warning_count = warnings.len() as u32;

        // If no JSON output was parsed but there's stderr, capture it
        let exec_error = if errors.is_empty()
            && warnings.is_empty()
            && !output.status.success()
            && !stderr.is_empty()
        {
            Some(stderr.to_string())
        } else {
            None
        };

        Ok(Output {
            success: output.status.success(),
            errors,
            warnings,
            error_count,
            warning_count,
            error: exec_error,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Working directory path for cargo check (defaults to configured workspace root)"
                },
                "package": {
                    "type": "string",
                    "description": "Specific package to check in a workspace"
                }
            },
            "required": []
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_level_deserialization() {
        let error: DiagnosticLevel = serde_json::from_str("\"error\"").unwrap();
        assert_eq!(error, DiagnosticLevel::Error);

        let warning: DiagnosticLevel = serde_json::from_str("\"warning\"").unwrap();
        assert_eq!(warning, DiagnosticLevel::Warning);

        let note: DiagnosticLevel = serde_json::from_str("\"note\"").unwrap();
        assert_eq!(note, DiagnosticLevel::Note);

        let help: DiagnosticLevel = serde_json::from_str("\"help\"").unwrap();
        assert_eq!(help, DiagnosticLevel::Help);

        // Unknown level should deserialize to Unknown
        let unknown: DiagnosticLevel = serde_json::from_str("\"something_else\"").unwrap();
        assert_eq!(unknown, DiagnosticLevel::Unknown);
    }

    #[test]
    fn test_cargo_check_current_project() {
        // Test cargo check on the current project (should succeed)
        let tool = CargoCheck;
        let output = tool
            .run(Input {
                path: None,
                package: None,
            })
            .unwrap();

        // The current project should compile without errors
        assert!(output.success);
        assert_eq!(output.error_count, 0);
        assert!(output.error.is_none());
    }

    #[test]
    fn test_cargo_check_nonexistent_path() {
        let tool = CargoCheck;
        let output = tool
            .run(Input {
                path: Some("/nonexistent/path/to/project".to_string()),
                package: None,
            })
            .unwrap();

        // Should fail because path doesn't exist
        assert!(!output.success);
    }

    #[test]
    fn test_schema_has_empty_required() {
        let schema = CargoCheck::schema();
        let required = schema["required"].as_array().unwrap();
        assert!(required.is_empty());
    }

    #[test]
    fn test_diagnostic_struct() {
        let diagnostic = Diagnostic {
            message: "test error".to_string(),
            file: Some("src/main.rs".to_string()),
            line: Some(10),
            column: Some(5),
            level: DiagnosticLevel::Error,
        };

        assert_eq!(diagnostic.message, "test error");
        assert_eq!(diagnostic.file, Some("src/main.rs".to_string()));
        assert_eq!(diagnostic.line, Some(10));
        assert_eq!(diagnostic.column, Some(5));
    }
}
