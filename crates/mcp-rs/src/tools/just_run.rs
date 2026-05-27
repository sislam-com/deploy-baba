use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;
use std::time::Instant;

pub struct JustRun;

const MAX_OUTPUT_BYTES: usize = 100 * 1024;

#[derive(Deserialize)]
pub struct Input {
    pub recipe: String,
    #[serde(default)]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Serialize)]
pub struct Output {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    pub truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Tool for JustRun {
    const NAME: &'static str = "just_run";
    const DESCRIPTION: &'static str = "Execute a just recipe in a project directory";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let mut cmd = Command::new("just");
        cmd.arg(&input.recipe);

        if let Some(ref args) = input.args {
            cmd.args(args);
        }

        let working_dir = input
            .path
            .as_ref()
            .map(workspace::resolve_path)
            .unwrap_or_else(workspace::root);
        cmd.current_dir(working_dir);

        let start = Instant::now();
        let result = match cmd.output() {
            Ok(r) => r,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    exit_code: None,
                    stdout: None,
                    stderr: None,
                    duration_ms: Some(start.elapsed().as_millis() as u64),
                    truncated: false,
                    error: Some(format!("Failed to execute just: {}", e)),
                });
            }
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        let mut stdout_str = String::from_utf8_lossy(&result.stdout).to_string();
        let mut stderr_str = String::from_utf8_lossy(&result.stderr).to_string();
        let mut truncated = false;

        if stdout_str.len() > MAX_OUTPUT_BYTES {
            stdout_str.truncate(MAX_OUTPUT_BYTES);
            stdout_str.push_str("\n... [truncated]");
            truncated = true;
        }
        if stderr_str.len() > MAX_OUTPUT_BYTES {
            stderr_str.truncate(MAX_OUTPUT_BYTES);
            stderr_str.push_str("\n... [truncated]");
            truncated = true;
        }

        Ok(Output {
            success: result.status.success(),
            exit_code: result.status.code(),
            stdout: Some(stdout_str),
            stderr: Some(stderr_str),
            duration_ms: Some(duration_ms),
            truncated,
            error: None,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "recipe": {
                    "type": "string",
                    "description": "The just recipe to execute"
                },
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Arguments to pass to the recipe"
                },
                "path": {
                    "type": "string",
                    "description": "Working directory (defaults to current directory)"
                }
            },
            "required": ["recipe"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_just_not_found_recipe() {
        let tool = JustRun;
        let output = tool
            .run(Input {
                recipe: "nonexistent_recipe_xyz".to_string(),
                args: None,
                path: Some("/tmp".to_string()),
            })
            .unwrap();
        // just should either fail to find the recipe or fail to find a justfile
        assert!(!output.success);
    }

    #[test]
    fn test_just_in_project() {
        let tool = JustRun;
        let output = tool
            .run(Input {
                recipe: "--list".to_string(),
                args: None,
                path: None,
            })
            .unwrap();
        // Should succeed if we're in the mcp-rs project root with a justfile
        if output.success {
            assert!(output.stdout.unwrap().contains("Available recipes"));
        }
    }

    #[test]
    fn test_schema_has_required() {
        let schema = JustRun::schema();
        assert_eq!(schema["required"][0], "recipe");
    }
}
