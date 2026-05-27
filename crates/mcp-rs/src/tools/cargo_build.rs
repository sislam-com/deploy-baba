use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;

pub struct CargoBuild;

#[derive(Deserialize)]
pub struct Input {
    #[serde(default)]
    pub release: bool,
    #[serde(default)]
    pub package: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
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
    pub artifacts: Vec<String>,
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
    #[serde(default)]
    executable: Option<String>,
    #[serde(default)]
    filenames: Option<Vec<String>>,
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

impl Tool for CargoBuild {
    const NAME: &'static str = "cargo_build";
    const DESCRIPTION: &'static str = "Run cargo build to compile the project";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let mut cmd = Command::new("cargo");
        cmd.arg("build").arg("--message-format=json");

        let working_dir = input
            .path
            .as_ref()
            .map(workspace::resolve_path)
            .unwrap_or_else(workspace::root);
        cmd.current_dir(working_dir);

        // Add release flag if requested
        if input.release {
            cmd.arg("--release");
        }

        // Add package filter if provided
        if let Some(ref package) = input.package {
            cmd.arg("--package").arg(package);
        }

        // Execute command
        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    errors: vec![],
                    warnings: vec![],
                    artifacts: vec![],
                    error_count: 0,
                    warning_count: 0,
                    error: Some(format!("Failed to execute cargo build: {}", e)),
                });
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut artifacts = Vec::new();

        // Parse JSON output line by line
        for line in stdout.lines() {
            if let Ok(msg) = serde_json::from_str::<CargoMessage>(line) {
                match msg.reason.as_str() {
                    "compiler-message" => {
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
                    "compiler-artifact" => {
                        // Capture built artifacts
                        if let Some(executable) = msg.executable {
                            artifacts.push(executable);
                        } else if let Some(filenames) = msg.filenames {
                            for filename in filenames {
                                // Only include interesting artifacts (not .d files, etc.)
                                if !filename.ends_with(".d") && !filename.ends_with(".rmeta") {
                                    artifacts.push(filename);
                                }
                            }
                        }
                    }
                    _ => {}
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
            artifacts,
            error_count,
            warning_count,
            error: exec_error,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "release": {
                    "type": "boolean",
                    "description": "Build in release mode with optimizations",
                    "default": false
                },
                "package": {
                    "type": "string",
                    "description": "Specific package to build in a workspace"
                },
                "path": {
                    "type": "string",
                    "description": "Working directory path for cargo build (defaults to configured workspace root)"
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
    }

    #[test]
    fn test_cargo_build_current_project_debug() {
        // Test cargo build on the current project (debug mode)
        let tool = CargoBuild;
        let output = tool
            .run(Input {
                release: false,
                package: None,
                path: None,
            })
            .unwrap();

        // The current project should build without errors
        assert!(output.success);
        assert_eq!(output.error_count, 0);
        assert!(output.error.is_none());
    }

    #[test]
    fn test_cargo_build_nonexistent_path() {
        let tool = CargoBuild;
        let output = tool
            .run(Input {
                release: false,
                package: None,
                path: Some("/nonexistent/path/to/project".to_string()),
            })
            .unwrap();

        // Should fail because path doesn't exist
        assert!(!output.success);
    }

    #[test]
    fn test_diagnostic_struct() {
        let diagnostic = Diagnostic {
            message: "test warning".to_string(),
            file: Some("src/lib.rs".to_string()),
            line: Some(20),
            column: Some(10),
            level: DiagnosticLevel::Warning,
        };

        assert_eq!(diagnostic.message, "test warning");
        assert_eq!(diagnostic.file, Some("src/lib.rs".to_string()));
        assert_eq!(diagnostic.line, Some(20));
        assert_eq!(diagnostic.column, Some(10));
        assert_eq!(diagnostic.level, DiagnosticLevel::Warning);
    }

    #[test]
    fn test_output_has_artifacts_field() {
        let output = Output {
            success: true,
            errors: vec![],
            warnings: vec![],
            artifacts: vec!["target/debug/myapp".to_string()],
            error_count: 0,
            warning_count: 0,
            error: None,
        };

        assert_eq!(output.artifacts.len(), 1);
        assert!(output.artifacts[0].contains("myapp"));
    }

    #[test]
    fn test_schema_has_empty_required() {
        let schema = CargoBuild::schema();
        let required = schema["required"].as_array().unwrap();
        assert!(required.is_empty());
    }

    #[test]
    fn test_schema_has_release_property() {
        let schema = CargoBuild::schema();
        assert!(schema["properties"]["release"].is_object());
        assert_eq!(schema["properties"]["release"]["type"], "boolean");
    }
}
