use crate::error::{McpError, McpResult};
use crate::workspace;
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Policy configuration for tool access control
#[derive(Debug, Clone)]
pub struct Policy {
    /// Paths that are allowed to be accessed
    pub allowed_paths: Vec<PathBuf>,
    /// Paths that are explicitly denied (takes precedence over allowed)
    pub denied_paths: Vec<PathBuf>,
    /// Maximum file size in bytes for read operations
    pub max_file_size: u64,
    /// Commands that are allowed for check_command tool
    pub allowed_commands: Vec<String>,
    /// Whether to allow access to environment variables
    pub allow_env_access: bool,
    /// Whether to allow cargo operations
    pub allow_cargo_operations: bool,
    /// Whether to force a read-only cloud-safe tool surface
    pub read_only: bool,
    /// Explicit allowlist of tool names. Empty means all registered tools can be listed/called.
    pub enabled_tools: Vec<String>,
    /// Explicit denylist of tool names.
    pub disabled_tools: Vec<String>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            // Default to allow current directory and common safe paths
            allowed_paths: vec![
                PathBuf::from("."),
                PathBuf::from("/tmp"),
                PathBuf::from("/var/tmp"),
            ],
            // Deny sensitive system paths by default
            denied_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/System"),
                PathBuf::from("/usr/bin"),
                PathBuf::from("/usr/sbin"),
                PathBuf::from("/sbin"),
                PathBuf::from("/bin"),
                PathBuf::from("/boot"),
                PathBuf::from("/root"),
                #[cfg(target_os = "macos")]
                PathBuf::from("/System"),
                #[cfg(target_os = "macos")]
                PathBuf::from("/Library"),
                #[cfg(target_os = "windows")]
                PathBuf::from("C:\\Windows"),
                #[cfg(target_os = "windows")]
                PathBuf::from("C:\\Program Files"),
            ],
            max_file_size: 10 * 1024 * 1024, // 10MB default limit
            allowed_commands: vec![
                "cargo".to_string(),
                "rustc".to_string(),
                "git".to_string(),
                "node".to_string(),
                "npm".to_string(),
                "yarn".to_string(),
                "python".to_string(),
                "python3".to_string(),
            ],
            allow_env_access: true,
            allow_cargo_operations: true,
            read_only: false,
            enabled_tools: vec![],
            disabled_tools: vec![],
        }
    }
}

impl Policy {
    /// Create a new policy with default settings
    #[allow(dead_code)]
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a restrictive policy for production use
    pub fn restrictive() -> Self {
        Self {
            allowed_paths: vec![PathBuf::from(".")],
            denied_paths: vec![
                PathBuf::from("/"),
                PathBuf::from("/etc"),
                PathBuf::from("/var"),
                PathBuf::from("/usr"),
                PathBuf::from("/home"),
                PathBuf::from("/root"),
                PathBuf::from("/tmp"),
            ],
            max_file_size: 1024 * 1024, // 1MB limit
            allowed_commands: vec!["cargo".to_string()],
            allow_env_access: false,
            allow_cargo_operations: true,
            read_only: false,
            enabled_tools: vec![],
            disabled_tools: vec![],
        }
    }

    /// Check if a tool call should be allowed
    pub fn check(&self, tool_name: &str, args: &Value) -> McpResult<()> {
        debug!(tool = tool_name, "Checking policy for tool call");

        if !self.is_tool_enabled(tool_name) {
            return Err(McpError::policy_denied(format!(
                "Tool '{}' is disabled by policy",
                tool_name
            )));
        }

        if self.read_only && !Self::is_read_only_tool(tool_name) {
            return Err(McpError::policy_denied(format!(
                "Tool '{}' is not available in read-only mode",
                tool_name
            )));
        }

        match tool_name {
            // File access tools
            "read_file" => self.check_file_access(args, "path")?,
            "list_directory" => self.check_file_access(args, "path")?,
            "grep_file" => self.check_file_access(args, "path")?,
            "file_stats" => self.check_file_access(args, "path")?,

            // Project search tools (path is optional, defaults to cwd)
            "grep_project" => self.check_file_access_optional(args, "path")?,
            "find_files" => self.check_file_access_optional(args, "path")?,

            // Environment access
            "read_env" => {
                if !self.allow_env_access {
                    return Err(McpError::policy_denied(
                        "Environment variable access is disabled",
                    ));
                }
            }

            // Command checking
            "check_command" => self.check_command_access(args)?,

            // Cargo operations
            "cargo_check" | "cargo_build" | "cargo_test" => {
                if !self.allow_cargo_operations {
                    return Err(McpError::policy_denied("Cargo operations are disabled"));
                }
                self.check_file_access_optional(args, "path")?;
            }

            // TOML reading
            "read_toml" => self.check_file_access(args, "path")?,

            // JSON reading
            "json_query" => self.check_file_access(args, "path")?,

            // Markdown reading
            "markdown_query" => self.check_file_access(args, "path")?,

            // SQLite tools (read-only)
            "sqlite_query" | "sqlite_schema" => self.check_file_access(args, "db_path")?,

            // Migration listing
            "migration_list" => self.check_file_access(args, "migrations_dir")?,

            // Just recipe execution
            "just_run" => self.check_command_in_allowlist("just")?,

            // Workspace inspection
            "workspace_info" => {
                if !self.allow_cargo_operations {
                    return Err(McpError::policy_denied("Cargo operations are disabled"));
                }
                self.check_file_access_optional(args, "path")?;
            }

            // Safe tools that don't need restrictions
            "say_hello" | "system_info" | "health" => {
                // These tools are always allowed as they don't access sensitive resources
            }

            _ => {
                warn!(tool = tool_name, "Unknown tool - denying by default");
                return Err(McpError::policy_denied(format!(
                    "Unknown tool: {}",
                    tool_name
                )));
            }
        }

        debug!(tool = tool_name, "Tool call approved by policy");
        Ok(())
    }

    pub fn is_tool_enabled(&self, tool_name: &str) -> bool {
        let disabled: HashSet<&str> = self.disabled_tools.iter().map(String::as_str).collect();
        if disabled.contains(tool_name) {
            return false;
        }

        if self.enabled_tools.is_empty() {
            return true;
        }

        self.enabled_tools.iter().any(|name| name == tool_name)
    }

    pub fn can_list_tool(&self, tool_name: &str) -> bool {
        self.is_tool_enabled(tool_name) && (!self.read_only || Self::is_read_only_tool(tool_name))
    }

    fn is_read_only_tool(tool_name: &str) -> bool {
        matches!(
            tool_name,
            "say_hello"
                | "health"
                | "read_file"
                | "list_directory"
                | "grep_file"
                | "file_stats"
                | "grep_project"
                | "find_files"
                | "read_toml"
                | "json_query"
                | "markdown_query"
                | "migration_list"
        )
    }

    fn check_file_access_optional(&self, args: &Value, path_field: &str) -> McpResult<()> {
        match args.get(path_field).and_then(|v| v.as_str()) {
            Some(path_str) => {
                let path = Path::new(path_str);
                self.validate_path(path)?;
            }
            None => {
                self.validate_path(&workspace::root())?;
            }
        }

        Ok(())
    }

    /// Check file access permissions
    fn check_file_access(&self, args: &Value, path_field: &str) -> McpResult<()> {
        let path_str = args
            .get(path_field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                McpError::invalid_arguments(
                    "file_access",
                    format!("Missing or invalid '{}' field", path_field),
                )
            })?;

        let path = Path::new(path_str);
        self.validate_path(path)?;

        // Check file size if it's a file read operation
        if path_field == "path" && path.is_file() {
            if let Ok(metadata) = std::fs::metadata(path) {
                if metadata.len() > self.max_file_size {
                    return Err(McpError::policy_denied(format!(
                        "File size ({} bytes) exceeds limit ({} bytes)",
                        metadata.len(),
                        self.max_file_size
                    )));
                }
            }
        }

        Ok(())
    }

    /// Check command access permissions
    fn check_command_access(&self, args: &Value) -> McpResult<()> {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                McpError::invalid_arguments("check_command", "Missing or invalid 'command' field")
            })?;

        if !self
            .allowed_commands
            .iter()
            .any(|allowed| command.starts_with(allowed))
        {
            return Err(McpError::policy_denied(format!(
                "Command '{}' is not in allowlist",
                command
            )));
        }

        Ok(())
    }

    fn check_command_in_allowlist(&self, command: &str) -> McpResult<()> {
        if !self.allowed_commands.iter().any(|c| c == command) {
            return Err(McpError::policy_denied(format!(
                "Command '{}' is not in allowlist",
                command
            )));
        }
        Ok(())
    }

    /// Validate a path against policy rules
    pub fn validate_path(&self, path: &Path) -> McpResult<PathBuf> {
        // Attempt to canonicalize the path to resolve symlinks and relative components
        let canonical_path = match std::fs::canonicalize(path) {
            Ok(canonical) => canonical,
            Err(_) => {
                // If canonicalization fails, work with the absolute version
                if path.is_relative() {
                    workspace::resolve_path(path)
                } else {
                    path.to_path_buf()
                }
            }
        };

        debug!(path = ?canonical_path, "Validating path");

        // Check denied paths first (they take precedence)
        for denied in &self.denied_paths {
            let denied_canonical = match std::fs::canonicalize(denied) {
                Ok(canonical) => canonical,
                Err(_) => denied.clone(),
            };

            if canonical_path.starts_with(&denied_canonical) {
                return Err(McpError::security_violation(format!(
                    "Access denied to path: {} (matches denied pattern: {})",
                    canonical_path.display(),
                    denied.display()
                )));
            }
        }

        // Check allowed paths
        for allowed in &self.allowed_paths {
            let allowed_canonical = match std::fs::canonicalize(allowed) {
                Ok(canonical) => canonical,
                Err(_) => allowed.clone(),
            };

            if canonical_path.starts_with(&allowed_canonical) {
                debug!(path = ?canonical_path, "Path access granted");
                return Ok(canonical_path);
            }
        }

        // If no allowed path matches, deny access
        Err(McpError::security_violation(format!(
            "Access denied to path: {} (not in allowlist)",
            canonical_path.display()
        )))
    }

    /// Check if a path is within allowed boundaries (for internal use)
    #[allow(dead_code)]
    pub fn is_path_allowed(&self, path: &Path) -> bool {
        self.validate_path(path).is_ok()
    }

    /// Add an allowed path
    #[allow(dead_code)]
    pub fn add_allowed_path(&mut self, path: PathBuf) {
        self.allowed_paths.push(path);
    }

    /// Add a denied path
    #[allow(dead_code)]
    pub fn add_denied_path(&mut self, path: PathBuf) {
        self.denied_paths.push(path);
    }

    /// Set maximum file size
    #[allow(dead_code)]
    pub fn set_max_file_size(&mut self, size: u64) {
        self.max_file_size = size;
    }

    /// Add an allowed command
    #[allow(dead_code)]
    pub fn add_allowed_command(&mut self, command: String) {
        self.allowed_commands.push(command);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_policy() {
        let policy = Policy::default();
        assert!(policy.allow_env_access);
        assert!(policy.allow_cargo_operations);
        assert!(!policy.allowed_paths.is_empty());
        assert!(!policy.denied_paths.is_empty());
    }

    #[test]
    fn test_restrictive_policy() {
        let policy = Policy::restrictive();
        assert!(!policy.allow_env_access);
        assert_eq!(policy.allowed_paths.len(), 1);
        assert_eq!(policy.max_file_size, 1024 * 1024);
    }

    #[test]
    fn test_tool_permissions() {
        let policy = Policy::default();

        // Test safe tools
        assert!(policy.check("say_hello", &json!({})).is_ok());
        assert!(policy.check("system_info", &json!({})).is_ok());

        // Test env access
        assert!(policy.check("read_env", &json!({})).is_ok());

        let restrictive = Policy::restrictive();
        assert!(restrictive.check("read_env", &json!({})).is_err());
    }

    #[test]
    fn test_file_access_validation() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let mut policy = Policy::new();
        policy.allowed_paths = vec![temp_path.to_path_buf()];
        policy.denied_paths = vec![];

        // Create a test file
        let test_file = temp_path.join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        // Test allowed file access
        let args = json!({ "path": test_file.to_str().unwrap() });
        assert!(policy.check_file_access(&args, "path").is_ok());

        // Test denied path
        let denied_file = "/etc/passwd";
        let args = json!({ "path": denied_file });
        assert!(policy.check_file_access(&args, "path").is_err());
    }

    #[test]
    fn test_command_allowlist() {
        let mut policy = Policy::new();
        policy.allowed_commands = vec!["cargo".to_string(), "git".to_string()];

        // Test allowed command
        let args = json!({ "command": "cargo" });
        assert!(policy.check_command_access(&args).is_ok());

        // Test denied command
        let args = json!({ "command": "rm" });
        assert!(policy.check_command_access(&args).is_err());
    }

    #[test]
    fn test_path_traversal_protection() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let mut policy = Policy::new();
        policy.allowed_paths = vec![temp_path.to_path_buf()];

        // Test path traversal attempt
        let traversal_path = temp_path.join("../../../etc/passwd");
        assert!(policy.validate_path(&traversal_path).is_err());
    }

    #[test]
    fn test_file_size_limit() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        let test_file = temp_path.join("large_file.txt");

        // Create a file larger than 100 bytes
        fs::write(&test_file, "x".repeat(200)).unwrap();

        let mut policy = Policy::new();
        policy.allowed_paths = vec![temp_path.to_path_buf()];
        policy.max_file_size = 100; // 100 byte limit

        let args = json!({ "path": test_file.to_str().unwrap() });
        assert!(policy.check_file_access(&args, "path").is_err());
    }

    #[test]
    fn test_disabled_tool_is_denied_and_hidden() {
        let mut policy = Policy::new();
        policy.disabled_tools = vec!["read_env".to_string()];

        assert!(!policy.can_list_tool("read_env"));
        assert!(policy.check("read_env", &json!({})).is_err());
        assert!(policy.check("health", &json!({})).is_ok());
    }

    #[test]
    fn test_enabled_tools_allowlist() {
        let mut policy = Policy::new();
        policy.enabled_tools = vec!["health".to_string()];

        assert!(policy.can_list_tool("health"));
        assert!(!policy.can_list_tool("read_file"));
        assert!(policy
            .check("read_file", &json!({"path": "Cargo.toml"}))
            .is_err());
    }

    #[test]
    fn test_read_only_blocks_execution_tools() {
        let mut policy = Policy::new();
        policy.read_only = true;

        assert!(policy.can_list_tool("read_file"));
        assert!(!policy.can_list_tool("just_run"));
        assert!(policy
            .check("just_run", &json!({"recipe": "test"}))
            .is_err());
        assert!(policy.check("read_env", &json!({})).is_err());
        assert!(policy.check("cargo_test", &json!({})).is_err());
    }
}
