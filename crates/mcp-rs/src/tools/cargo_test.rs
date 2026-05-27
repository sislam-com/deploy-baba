use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;

pub struct CargoTest;

#[derive(Deserialize)]
pub struct Input {
    #[serde(default)]
    pub test_name: Option<String>,
    #[serde(default)]
    pub package: Option<String>,
    #[serde(default)]
    pub no_capture: bool,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Serialize)]
pub struct TestFailure {
    pub name: String,
    pub stdout: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize)]
pub struct Output {
    pub success: bool,
    pub passed: u32,
    pub failed: u32,
    pub ignored: u32,
    pub total: u32,
    pub failures: Vec<TestFailure>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Tool for CargoTest {
    const NAME: &'static str = "cargo_test";
    const DESCRIPTION: &'static str = "Run cargo test to execute tests";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let mut cmd = Command::new("cargo");
        cmd.arg("test");

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

        // Add test name filter if provided
        if let Some(ref test_name) = input.test_name {
            cmd.arg(test_name);
        }

        // Add -- separator and test binary args if needed
        if input.no_capture {
            cmd.arg("--");
            cmd.arg("--nocapture");
        }

        // Execute command
        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    passed: 0,
                    failed: 0,
                    ignored: 0,
                    total: 0,
                    failures: vec![],
                    stdout: None,
                    error: Some(format!("Failed to execute cargo test: {}", e)),
                });
            }
        };

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let stderr_str = String::from_utf8_lossy(&output.stderr);

        let mut passed = 0u32;
        let mut failed = 0u32;
        let mut ignored = 0u32;
        let mut failures: Vec<TestFailure> = Vec::new();
        let mut found_summary = false;

        // Parse traditional test output
        let combined_output = format!("{}\n{}", stdout_str, stderr_str);

        // Track failed test names first
        for line in combined_output.lines() {
            // Match lines like "test some_test ... FAILED"
            if line.starts_with("test ") && line.ends_with(" FAILED") {
                let test_part = line.strip_prefix("test ").unwrap_or("");
                if let Some(name) = test_part.strip_suffix(" ... FAILED") {
                    if !failures.iter().any(|f| f.name == name) {
                        failures.push(TestFailure {
                            name: name.to_string(),
                            stdout: None,
                            message: None,
                        });
                    }
                }
            }
        }

        // Look for "test result: ok/FAILED. X passed; Y failed; Z ignored"
        for line in combined_output.lines() {
            if line.starts_with("test result:") {
                // Parse the summary line
                if let Some(passed_match) = extract_number(line, "passed") {
                    passed = passed_match;
                }
                if let Some(failed_match) = extract_number(line, "failed") {
                    failed = failed_match;
                }
                if let Some(ignored_match) = extract_number(line, "ignored") {
                    ignored = ignored_match;
                }
                found_summary = true;
            }
        }

        let total = passed + failed + ignored;

        // Include full output for debugging if tests failed
        let full_stdout = if failed > 0 || !output.status.success() {
            Some(combined_output.clone())
        } else {
            None
        };

        // Capture error if execution failed without proper test output
        let exec_error = if !found_summary && !output.status.success() {
            Some(format!("cargo test failed:\n{}", stderr_str))
        } else {
            None
        };

        Ok(Output {
            success: output.status.success() && failed == 0,
            passed,
            failed,
            ignored,
            total,
            failures,
            stdout: full_stdout,
            error: exec_error,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "test_name": {
                    "type": "string",
                    "description": "Specific test name or pattern to run"
                },
                "package": {
                    "type": "string",
                    "description": "Specific package to test in a workspace"
                },
                "no_capture": {
                    "type": "boolean",
                    "description": "Whether to show test output (--nocapture)",
                    "default": false
                },
                "path": {
                    "type": "string",
                    "description": "Working directory path for cargo test (defaults to configured workspace root)"
                }
            },
            "required": []
        })
    }
}

/// Helper function to extract a number before a word from a string
fn extract_number(s: &str, word: &str) -> Option<u32> {
    let parts: Vec<&str> = s.split(word).collect();
    if parts.len() >= 2 {
        // Get the part before the word and find the last number
        let before = parts[0];
        let nums: Vec<&str> = before
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .collect();
        if let Some(last) = nums.last() {
            return last.parse().ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_number_passed() {
        let line = "test result: ok. 10 passed; 0 failed; 2 ignored";
        assert_eq!(extract_number(line, "passed"), Some(10));
    }

    #[test]
    fn test_extract_number_failed() {
        let line = "test result: FAILED. 5 passed; 3 failed; 1 ignored";
        assert_eq!(extract_number(line, "failed"), Some(3));
    }

    #[test]
    fn test_extract_number_ignored() {
        let line = "test result: ok. 10 passed; 0 failed; 5 ignored";
        assert_eq!(extract_number(line, "ignored"), Some(5));
    }

    #[test]
    fn test_extract_number_not_found() {
        let line = "some random line";
        assert_eq!(extract_number(line, "passed"), None);
    }

    #[test]
    fn test_extract_number_zero() {
        let line = "test result: ok. 0 passed; 0 failed; 0 ignored";
        assert_eq!(extract_number(line, "passed"), Some(0));
        assert_eq!(extract_number(line, "failed"), Some(0));
        assert_eq!(extract_number(line, "ignored"), Some(0));
    }

    #[test]
    fn test_cargo_test_with_specific_filter() {
        // Test cargo test with a specific filter to avoid infinite recursion
        // (running cargo test without filter would recursively run this test)
        let tool = CargoTest;
        let output = tool
            .run(Input {
                // Filter to a non-cargo_test test to avoid recursion
                test_name: Some("test_extract_number_passed".to_string()),
                package: None,
                no_capture: false,
                path: None,
            })
            .unwrap();

        // Should find and run only matching test
        assert!(output.passed >= 1 || output.error.is_none());
    }

    #[test]
    fn test_cargo_test_nonexistent_path() {
        let tool = CargoTest;
        let output = tool
            .run(Input {
                test_name: None,
                package: None,
                no_capture: false,
                path: Some("/nonexistent/path".to_string()),
            })
            .unwrap();

        // Should fail because path doesn't exist
        assert!(!output.success);
    }

    #[test]
    fn test_test_failure_struct() {
        let failure = TestFailure {
            name: "test_something".to_string(),
            stdout: Some("test output".to_string()),
            message: Some("assertion failed".to_string()),
        };

        assert_eq!(failure.name, "test_something");
        assert_eq!(failure.stdout, Some("test output".to_string()));
        assert_eq!(failure.message, Some("assertion failed".to_string()));
    }

    #[test]
    fn test_schema_has_empty_required() {
        let schema = CargoTest::schema();
        let required = schema["required"].as_array().unwrap();
        assert!(required.is_empty());
    }
}
