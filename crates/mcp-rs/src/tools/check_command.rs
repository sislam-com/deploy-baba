use crate::error::McpResult;
use crate::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;

pub struct CheckCommand;

#[derive(Deserialize)]
pub struct Input {
    pub command: String,
}

#[derive(Serialize)]
pub struct Output {
    pub exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Tool for CheckCommand {
    const NAME: &'static str = "check_command";
    const DESCRIPTION: &'static str = "Check if a command exists and get its path and version";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        // Use the `which` crate to find the command
        let path = match which::which(&input.command) {
            Ok(p) => Some(p.to_string_lossy().to_string()),
            Err(_) => None,
        };

        if path.is_none() {
            return Ok(Output {
                exists: false,
                path: None,
                version: None,
                error: Some(format!("Command '{}' not found in PATH", input.command)),
            });
        }

        // Try to get version using common flags
        let version = get_version(&input.command);

        Ok(Output {
            exists: true,
            path,
            version,
            error: None,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Name of the command to check (e.g., 'rustc', 'cargo', 'git')"
                }
            },
            "required": ["command"]
        })
    }
}

/// Try to get version using common version flags
fn get_version(command: &str) -> Option<String> {
    // Try --version first (most common)
    if let Some(version) = try_version_flag(command, "--version") {
        return Some(version);
    }

    // Try -v
    if let Some(version) = try_version_flag(command, "-v") {
        return Some(version);
    }

    // Try -V (used by some commands like cargo)
    if let Some(version) = try_version_flag(command, "-V") {
        return Some(version);
    }

    // Try version (no dash, used by some commands)
    if let Some(version) = try_version_flag(command, "version") {
        return Some(version);
    }

    None
}

fn try_version_flag(command: &str, flag: &str) -> Option<String> {
    let output = Command::new(command).arg(flag).output().ok()?;

    if output.status.success() || !output.stdout.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Some commands output version to stderr
        let version_str = if !stdout.trim().is_empty() {
            stdout.to_string()
        } else if !stderr.trim().is_empty() {
            stderr.to_string()
        } else {
            return None;
        };

        // Extract just the first line which usually contains the version
        let first_line = version_str.lines().next()?.trim().to_string();
        if !first_line.is_empty() {
            return Some(first_line);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_command_rustc() {
        let tool = CheckCommand;
        let output = tool
            .run(Input {
                command: "rustc".to_string(),
            })
            .unwrap();

        assert!(output.exists);
        assert!(output.path.is_some());
        assert!(output.version.is_some());
        assert!(output.error.is_none());

        // Version should contain "rustc"
        assert!(output.version.unwrap().contains("rustc"));
    }

    #[test]
    fn test_check_command_cargo() {
        let tool = CheckCommand;
        let output = tool
            .run(Input {
                command: "cargo".to_string(),
            })
            .unwrap();

        assert!(output.exists);
        assert!(output.path.is_some());
        assert!(output.version.is_some());
        assert!(output.error.is_none());
    }

    #[test]
    fn test_check_command_nonexistent() {
        let tool = CheckCommand;
        let output = tool
            .run(Input {
                command: "nonexistent_command_xyz_12345".to_string(),
            })
            .unwrap();

        assert!(!output.exists);
        assert!(output.path.is_none());
        assert!(output.version.is_none());
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("not found"));
    }

    #[test]
    fn test_check_command_ls() {
        // ls should exist on Unix-like systems
        let tool = CheckCommand;
        let output = tool
            .run(Input {
                command: "ls".to_string(),
            })
            .unwrap();

        assert!(output.exists);
        assert!(output.path.is_some());
    }

    #[test]
    fn test_schema_has_required_command() {
        let schema = CheckCommand::schema();
        assert_eq!(schema["required"][0], "command");
    }
}
