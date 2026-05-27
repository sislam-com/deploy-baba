use crate::error::{McpError, McpResult};
use crate::workspace;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;
use tracing::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub uri: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub resource_type: ResourceType,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    File,
    Glob,
    Command,
}

pub struct ResourceRegistry {
    resources: Vec<ResourceConfig>,
}

impl ResourceRegistry {
    pub fn new(resources: Vec<ResourceConfig>) -> Self {
        Self { resources }
    }

    pub fn list(&self) -> Value {
        let items: Vec<Value> = self
            .resources
            .iter()
            .map(|r| {
                let mut item = json!({
                    "uri": r.uri,
                    "name": r.name,
                });
                if let Some(ref desc) = r.description {
                    item["description"] = json!(desc);
                }
                if let Some(ref mime) = r.mime_type {
                    item["mimeType"] = json!(mime);
                }
                item
            })
            .collect();
        json!({ "resources": items })
    }

    pub fn read(&self, uri: &str) -> McpResult<Value> {
        let resource = self
            .resources
            .iter()
            .find(|r| r.uri == uri)
            .ok_or_else(|| McpError::ResourceNotFound {
                uri: uri.to_string(),
            })?;

        debug!(uri = uri, "Reading resource");

        let (text, mime_type) = match resource.resource_type {
            ResourceType::File => self.read_file(resource)?,
            ResourceType::Glob => self.read_glob(resource)?,
            ResourceType::Command => self.read_command(resource)?,
        };

        Ok(json!({
            "contents": [{
                "uri": resource.uri,
                "mimeType": mime_type,
                "text": text,
            }]
        }))
    }

    fn read_file(&self, resource: &ResourceConfig) -> McpResult<(String, String)> {
        let path = resource
            .path
            .as_deref()
            .ok_or_else(|| McpError::ConfigError {
                details: format!(
                    "Resource '{}' of type 'file' requires a 'path' field",
                    resource.uri
                ),
            })?;

        let p = workspace::resolve_path(path);
        if !p.exists() {
            return Err(McpError::ResourceNotFound {
                uri: format!("{} (file: {})", resource.uri, path),
            });
        }

        let content = std::fs::read_to_string(&p).map_err(|e| McpError::ResourceReadError {
            uri: resource.uri.clone(),
            details: format!("Failed to read file '{}': {}", path, e),
        })?;

        let mime = resource
            .mime_type
            .clone()
            .unwrap_or_else(|| guess_mime(path));

        Ok((content, mime))
    }

    fn read_glob(&self, resource: &ResourceConfig) -> McpResult<(String, String)> {
        let pattern = resource
            .pattern
            .as_deref()
            .or(resource.path.as_deref())
            .ok_or_else(|| McpError::ConfigError {
                details: format!(
                    "Resource '{}' of type 'glob' requires a 'pattern' or 'path' field",
                    resource.uri
                ),
            })?;

        let resolved_pattern = if Path::new(pattern).is_absolute() {
            pattern.to_string()
        } else {
            workspace::resolve_path(pattern)
                .to_string_lossy()
                .to_string()
        };

        let mut files: Vec<String> = Vec::new();
        for entry in glob::glob(&resolved_pattern).map_err(|e| McpError::ResourceReadError {
            uri: resource.uri.clone(),
            details: format!("Invalid glob pattern '{}': {}", resolved_pattern, e),
        })? {
            match entry {
                Ok(path) => {
                    if path.is_file() {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
                Err(e) => {
                    warn!(uri = resource.uri, error = %e, "Glob entry error");
                }
            }
        }

        files.sort();
        let content = files.join("\n");
        Ok((content, "text/plain".to_string()))
    }

    fn read_command(&self, resource: &ResourceConfig) -> McpResult<(String, String)> {
        let command = resource
            .command
            .as_deref()
            .ok_or_else(|| McpError::ConfigError {
                details: format!(
                    "Resource '{}' of type 'command' requires a 'command' field",
                    resource.uri
                ),
            })?;

        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(McpError::ConfigError {
                details: format!("Resource '{}' has empty command", resource.uri),
            });
        }

        let output = Command::new(parts[0])
            .args(&parts[1..])
            .current_dir(workspace::root())
            .output()
            .map_err(|e| McpError::ResourceReadError {
                uri: resource.uri.clone(),
                details: format!("Failed to execute command '{}': {}", command, e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(McpError::ResourceReadError {
                uri: resource.uri.clone(),
                details: format!("Command '{}' failed: {}", command, stderr),
            });
        }

        let content = String::from_utf8_lossy(&output.stdout).to_string();
        let mime = resource
            .mime_type
            .clone()
            .unwrap_or_else(|| "text/plain".to_string());

        Ok((content, mime))
    }
}

fn guess_mime(path: &str) -> String {
    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("json") => "application/json",
        Some("toml") => "application/toml",
        Some("yaml" | "yml") => "application/yaml",
        Some("md") => "text/markdown",
        Some("sql") => "application/sql",
        Some("rs") => "text/x-rust",
        Some("ts" | "tsx") => "text/typescript",
        Some("js" | "jsx") => "text/javascript",
        Some("html") => "text/html",
        Some("css") => "text/css",
        _ => "text/plain",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_list_resources() {
        let registry = ResourceRegistry::new(vec![
            ResourceConfig {
                uri: "project://cache".to_string(),
                name: "Agent Cache".to_string(),
                description: Some("Project knowledge snapshot".to_string()),
                resource_type: ResourceType::File,
                path: Some("/tmp/test.json".to_string()),
                pattern: None,
                command: None,
                mime_type: Some("application/json".to_string()),
            },
            ResourceConfig {
                uri: "project://plans".to_string(),
                name: "Plan Index".to_string(),
                description: None,
                resource_type: ResourceType::File,
                path: Some("/tmp/INDEX.md".to_string()),
                pattern: None,
                command: None,
                mime_type: None,
            },
        ]);

        let list = registry.list();
        let resources = list["resources"].as_array().unwrap();
        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0]["uri"], "project://cache");
        assert_eq!(resources[0]["name"], "Agent Cache");
        assert_eq!(resources[0]["description"], "Project knowledge snapshot");
        assert_eq!(resources[1]["uri"], "project://plans");
    }

    #[test]
    fn test_read_file_resource() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, r#"{{"status": "ok"}}"#).unwrap();

        let registry = ResourceRegistry::new(vec![ResourceConfig {
            uri: "test://file".to_string(),
            name: "Test".to_string(),
            description: None,
            resource_type: ResourceType::File,
            path: Some(file.path().to_string_lossy().to_string()),
            pattern: None,
            command: None,
            mime_type: None,
        }]);

        let result = registry.read("test://file").unwrap();
        let contents = &result["contents"][0];
        assert_eq!(contents["uri"], "test://file");
        assert!(contents["text"].as_str().unwrap().contains("status"));
    }

    #[test]
    fn test_read_glob_resource() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.md"), "# A").unwrap();
        std::fs::write(dir.path().join("b.md"), "# B").unwrap();
        std::fs::write(dir.path().join("c.txt"), "C").unwrap();

        let pattern = format!("{}/*.md", dir.path().display());
        let registry = ResourceRegistry::new(vec![ResourceConfig {
            uri: "test://glob".to_string(),
            name: "Markdown files".to_string(),
            description: None,
            resource_type: ResourceType::Glob,
            path: None,
            pattern: Some(pattern),
            command: None,
            mime_type: None,
        }]);

        let result = registry.read("test://glob").unwrap();
        let text = result["contents"][0]["text"].as_str().unwrap();
        assert!(text.contains("a.md"));
        assert!(text.contains("b.md"));
        assert!(!text.contains("c.txt"));
    }

    #[test]
    fn test_read_command_resource() {
        let registry = ResourceRegistry::new(vec![ResourceConfig {
            uri: "test://cmd".to_string(),
            name: "Echo".to_string(),
            description: None,
            resource_type: ResourceType::Command,
            path: None,
            pattern: None,
            command: Some("echo hello-resource".to_string()),
            mime_type: None,
        }]);

        let result = registry.read("test://cmd").unwrap();
        let text = result["contents"][0]["text"].as_str().unwrap();
        assert!(text.contains("hello-resource"));
    }

    #[test]
    fn test_resource_not_found() {
        let registry = ResourceRegistry::new(vec![]);
        let result = registry.read("nonexistent://uri");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_resource_missing_file() {
        let registry = ResourceRegistry::new(vec![ResourceConfig {
            uri: "test://missing".to_string(),
            name: "Missing".to_string(),
            description: None,
            resource_type: ResourceType::File,
            path: Some("/nonexistent/file.json".to_string()),
            pattern: None,
            command: None,
            mime_type: None,
        }]);

        let result = registry.read("test://missing");
        assert!(result.is_err());
    }

    #[test]
    fn test_guess_mime() {
        assert_eq!(guess_mime("test.json"), "application/json");
        assert_eq!(guess_mime("test.md"), "text/markdown");
        assert_eq!(guess_mime("test.rs"), "text/x-rust");
        assert_eq!(guess_mime("test.toml"), "application/toml");
        assert_eq!(guess_mime("test.unknown"), "text/plain");
    }
}
