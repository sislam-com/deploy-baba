use crate::error::McpResult;
use crate::tool::Tool;
use crate::workspace;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;

pub struct WorkspaceInfo;

#[derive(Deserialize)]
pub struct Input {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub crate_name: Option<String>,
}

#[derive(Serialize)]
pub struct Output {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Tool for WorkspaceInfo {
    const NAME: &'static str = "workspace_info";
    const DESCRIPTION: &'static str = "Inspect Cargo workspace: members, dependencies, features";

    type Input = Input;
    type Output = Output;

    fn run(&self, input: Input) -> McpResult<Output> {
        let mut cmd = Command::new("cargo");
        cmd.args(["metadata", "--format-version=1", "--no-deps"]);

        let working_dir = input
            .path
            .as_ref()
            .map(workspace::resolve_path)
            .unwrap_or_else(workspace::root);
        cmd.current_dir(working_dir);

        let result = match cmd.output() {
            Ok(r) => r,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to run cargo metadata: {}", e)),
                });
            }
        };

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Ok(Output {
                success: false,
                data: None,
                error: Some(format!("cargo metadata failed: {}", stderr)),
            });
        }

        let metadata: serde_json::Value = match serde_json::from_slice(&result.stdout) {
            Ok(v) => v,
            Err(e) => {
                return Ok(Output {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to parse cargo metadata: {}", e)),
                });
            }
        };

        if let Some(ref crate_name) = input.crate_name {
            let packages = metadata["packages"].as_array();
            let found = packages.and_then(|pkgs| {
                pkgs.iter()
                    .find(|p| p["name"].as_str() == Some(crate_name))
                    .cloned()
            });

            return Ok(match found {
                Some(pkg) => Output {
                    success: true,
                    data: Some(pkg),
                    error: None,
                },
                None => Output {
                    success: false,
                    data: None,
                    error: Some(format!("Crate not found: {}", crate_name)),
                },
            });
        }

        let summary = summarize_metadata(&metadata);
        Ok(Output {
            success: true,
            data: Some(summary),
            error: None,
        })
    }

    fn schema() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the workspace root (defaults to current directory)"
                },
                "crate_name": {
                    "type": "string",
                    "description": "Filter to a specific crate by name"
                }
            }
        })
    }
}

fn summarize_metadata(metadata: &serde_json::Value) -> serde_json::Value {
    let workspace_root = metadata["workspace_root"].as_str().unwrap_or("");

    let members: Vec<serde_json::Value> = metadata["packages"]
        .as_array()
        .map(|pkgs| {
            pkgs.iter()
                .map(|p| {
                    let deps: Vec<&str> = p["dependencies"]
                        .as_array()
                        .map(|d| d.iter().filter_map(|dep| dep["name"].as_str()).collect())
                        .unwrap_or_default();

                    let features: Vec<&str> = p["features"]
                        .as_object()
                        .map(|f| f.keys().map(|k| k.as_str()).collect())
                        .unwrap_or_default();

                    json!({
                        "name": p["name"],
                        "version": p["version"],
                        "manifest_path": p["manifest_path"],
                        "dependency_count": deps.len(),
                        "dependencies": deps,
                        "features": features,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let workspace_members: Vec<&str> = metadata["workspace_members"]
        .as_array()
        .map(|m| m.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    json!({
        "workspace_root": workspace_root,
        "member_count": members.len(),
        "members": members,
        "workspace_member_ids": workspace_members,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_info_current() {
        let tool = WorkspaceInfo;
        let output = tool
            .run(Input {
                path: None,
                crate_name: None,
            })
            .unwrap();
        assert!(output.success);
        let data = output.data.unwrap();
        assert!(data["member_count"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn test_workspace_info_specific_crate() {
        let tool = WorkspaceInfo;
        let output = tool
            .run(Input {
                path: None,
                crate_name: Some("mcp-rs".to_string()),
            })
            .unwrap();
        assert!(output.success);
        let data = output.data.unwrap();
        assert_eq!(data["name"], "mcp-rs");
    }

    #[test]
    fn test_workspace_info_crate_not_found() {
        let tool = WorkspaceInfo;
        let output = tool
            .run(Input {
                path: None,
                crate_name: Some("nonexistent-crate-xyz".to_string()),
            })
            .unwrap();
        assert!(!output.success);
        assert!(output.error.unwrap().contains("not found"));
    }

    #[test]
    fn test_workspace_info_bad_path() {
        let tool = WorkspaceInfo;
        let output = tool
            .run(Input {
                path: Some("/nonexistent/path".to_string()),
                crate_name: None,
            })
            .unwrap();
        assert!(!output.success);
    }

    #[test]
    fn test_schema() {
        let schema = WorkspaceInfo::schema();
        assert!(schema["properties"]["path"].is_object());
        assert!(schema["properties"]["crate_name"].is_object());
    }
}
