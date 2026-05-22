use crate::error::McpResult;
use crate::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::process::Command;

pub struct SystemInfo;

#[derive(Deserialize)]
pub struct Input {}

#[derive(Serialize)]
pub struct Output {
    pub os: String,
    pub arch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo_version: Option<String>,
    pub cwd: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Tool for SystemInfo {
    const NAME: &'static str = "system_info";
    const DESCRIPTION: &'static str =
        "Get system information including OS, architecture, and Rust versions";

    type Input = Input;
    type Output = Output;

    fn run(&self, _input: Input) -> McpResult<Output> {
        let os = env::consts::OS.to_string();
        let arch = env::consts::ARCH.to_string();

        // Get current working directory
        let cwd = env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        // Get home directory
        let home = env::var("HOME").or_else(|_| env::var("USERPROFILE")).ok();

        // Get current user
        let user = env::var("USER").or_else(|_| env::var("USERNAME")).ok();

        // Get Rust version
        let rust_version = get_command_version("rustc", "--version");

        // Get Cargo version
        let cargo_version = get_command_version("cargo", "--version");

        Ok(Output {
            os,
            arch,
            rust_version,
            cargo_version,
            cwd,
            home,
            user,
            error: None,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }
}

fn get_command_version(command: &str, flag: &str) -> Option<String> {
    let output = Command::new(command).arg(flag).output().ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let first_line = stdout.lines().next()?.trim().to_string();
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
    fn test_system_info_basic() {
        let tool = SystemInfo;
        let output = tool.run(Input {}).unwrap();

        // OS should be a valid value
        assert!(!output.os.is_empty());
        assert!(["macos", "linux", "windows"].contains(&output.os.as_str()));

        // Arch should be a valid value
        assert!(!output.arch.is_empty());

        // CWD should not be empty
        assert!(!output.cwd.is_empty());

        // No error
        assert!(output.error.is_none());
    }

    #[test]
    fn test_system_info_has_rust_version() {
        let tool = SystemInfo;
        let output = tool.run(Input {}).unwrap();

        // Rust should be installed (since we're running Rust tests)
        assert!(output.rust_version.is_some());
        assert!(output.rust_version.unwrap().contains("rustc"));
    }

    #[test]
    fn test_system_info_has_cargo_version() {
        let tool = SystemInfo;
        let output = tool.run(Input {}).unwrap();

        // Cargo should be installed
        assert!(output.cargo_version.is_some());
        assert!(output.cargo_version.unwrap().contains("cargo"));
    }

    #[test]
    fn test_system_info_has_home() {
        let tool = SystemInfo;
        let output = tool.run(Input {}).unwrap();

        // HOME should be set on most systems
        assert!(output.home.is_some());
    }

    #[test]
    fn test_system_info_has_user() {
        let tool = SystemInfo;
        let output = tool.run(Input {}).unwrap();

        // USER should be set on most systems
        assert!(output.user.is_some());
    }

    #[test]
    fn test_get_command_version() {
        // Test the helper function
        let version = get_command_version("rustc", "--version");
        assert!(version.is_some());
        assert!(version.unwrap().contains("rustc"));
    }

    #[test]
    fn test_schema_has_empty_required() {
        let schema = SystemInfo::schema();
        let required = schema["required"].as_array().unwrap();
        assert!(required.is_empty());
    }
}
