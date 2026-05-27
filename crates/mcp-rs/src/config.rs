use crate::audit::AuditConfig;
use crate::error::{McpError, McpResult};
use crate::policy::Policy;
use crate::resource::ResourceConfig;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Complete configuration for MCP-RS server
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,

    /// Policy configuration
    pub policy: PolicyConfig,

    /// Audit logging configuration
    pub audit: AuditConfigToml,

    /// Tracing/logging configuration
    pub logging: LoggingConfig,

    /// MCP Resources exposed to clients
    #[serde(default)]
    pub resources: Vec<ResourceConfig>,
}

/// Server-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server name reported in initialize response
    pub name: String,

    /// Server version reported in initialize response
    pub version: String,

    /// Whether to enable health checks
    pub enable_health_checks: bool,

    /// Request timeout in seconds
    pub request_timeout_secs: Option<u64>,

    /// Workspace root used to resolve relative paths and command working directories
    #[serde(default)]
    pub workspace_root: Option<String>,

    /// Transport mode: "stdio" or "http"
    #[serde(default)]
    pub transport: Option<String>,

    /// Address used by HTTP transport
    #[serde(default)]
    pub http_addr: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: "mcp-rs".to_string(),
            version: "0.1.0".to_string(),
            enable_health_checks: true,
            request_timeout_secs: None,
            workspace_root: None,
            transport: None,
            http_addr: None,
        }
    }
}

/// Policy configuration (TOML-serializable version of Policy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Allowed paths for file access
    pub allowed_paths: Vec<String>,

    /// Denied paths (overrides allowed_paths)
    pub denied_paths: Vec<String>,

    /// Maximum file size in MB for read operations
    pub max_file_size_mb: u64,

    /// Commands allowed for check_command tool
    pub allowed_commands: Vec<String>,

    /// Whether to allow environment variable access
    pub allow_env_access: bool,

    /// Whether to allow cargo operations
    pub allow_cargo_operations: bool,

    /// Whether to force a read-only cloud-safe tool surface
    #[serde(default)]
    pub read_only: bool,

    /// Explicit allowlist of tool names. Empty means all registered tools are eligible.
    #[serde(default)]
    pub enabled_tools: Vec<String>,

    /// Explicit denylist of tool names.
    #[serde(default)]
    pub disabled_tools: Vec<String>,

    /// Policy mode: "default", "restrictive", or "permissive"
    pub mode: String,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            allowed_paths: vec![".".to_string(), "/tmp".to_string(), "/var/tmp".to_string()],
            denied_paths: vec![
                "/etc".to_string(),
                "/System".to_string(),
                "/usr/bin".to_string(),
                "/usr/sbin".to_string(),
                "/sbin".to_string(),
                "/bin".to_string(),
                "/boot".to_string(),
                "/root".to_string(),
            ],
            max_file_size_mb: 10,
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
            mode: "default".to_string(),
        }
    }
}

/// Audit configuration (TOML-serializable version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfigToml {
    /// Whether audit logging is enabled
    pub enabled: bool,

    /// Path to audit log file
    pub log_path: String,

    /// Log successful operations
    pub log_successes: bool,

    /// Log policy denials
    pub log_policy_denials: bool,

    /// Log errors
    pub log_errors: bool,

    /// Maximum log file size in MB before rotation
    pub max_file_size_mb: u64,

    /// Pretty print JSON logs
    pub pretty_print: bool,
}

impl Default for AuditConfigToml {
    fn default() -> Self {
        Self {
            enabled: true,
            log_path: "mcp-audit.jsonl".to_string(),
            log_successes: true,
            log_policy_denials: true,
            log_errors: true,
            max_file_size_mb: 100,
            pretty_print: false,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level: "trace", "debug", "info", "warn", "error"
    pub level: String,

    /// Log format: "compact", "pretty", "json"
    pub format: String,

    /// Whether to include timestamps
    pub include_timestamps: bool,

    /// Whether to include source locations
    pub include_source: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "compact".to_string(),
            include_timestamps: true,
            include_source: false,
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> McpResult<Self> {
        let content =
            std::fs::read_to_string(path.as_ref()).map_err(|e| McpError::ConfigError {
                details: format!(
                    "Failed to read config file '{}': {}",
                    path.as_ref().display(),
                    e
                ),
            })?;

        let config: Config = toml::from_str(&content).map_err(|e| McpError::ConfigError {
            details: format!(
                "Failed to parse config file '{}': {}",
                path.as_ref().display(),
                e
            ),
        })?;

        config.validate()?;

        info!("Loaded configuration from: {}", path.as_ref().display());
        Ok(config)
    }

    /// Load configuration from file with fallback to default
    #[allow(dead_code)]
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(&path) {
            Ok(config) => config,
            Err(e) => {
                warn!(
                    "Failed to load config from '{}': {}. Using default configuration.",
                    path.as_ref().display(),
                    e
                );
                Default::default()
            }
        }
    }

    /// Save configuration to a TOML file
    #[allow(dead_code)]
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> McpResult<()> {
        let content = toml::to_string_pretty(self).map_err(|e| McpError::ConfigError {
            details: format!("Failed to serialize config: {}", e),
        })?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(|e| McpError::ConfigError {
                details: format!("Failed to create config directory: {}", e),
            })?;
        }

        std::fs::write(path.as_ref(), content).map_err(|e| McpError::ConfigError {
            details: format!(
                "Failed to write config file '{}': {}",
                path.as_ref().display(),
                e
            ),
        })?;

        info!("Saved configuration to: {}", path.as_ref().display());
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> McpResult<()> {
        if let Some(transport) = &self.server.transport {
            match transport.as_str() {
                "stdio" | "http" => {}
                _ => {
                    return Err(McpError::ConfigError {
                        details: format!(
                            "Invalid transport '{}'. Must be 'stdio' or 'http'",
                            transport
                        ),
                    })
                }
            }
        }

        // Validate policy mode
        match self.policy.mode.as_str() {
            "default" | "restrictive" | "permissive" => {}
            _ => {
                return Err(McpError::ConfigError {
                    details: format!(
                    "Invalid policy mode '{}'. Must be 'default', 'restrictive', or 'permissive'",
                    self.policy.mode
                ),
                })
            }
        }

        // Validate log level
        match self.logging.level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => {
                return Err(McpError::ConfigError {
                    details: format!(
                        "Invalid log level '{}'. Must be one of: trace, debug, info, warn, error",
                        self.logging.level
                    ),
                })
            }
        }

        // Validate log format
        match self.logging.format.to_lowercase().as_str() {
            "compact" | "pretty" | "json" => {}
            _ => {
                return Err(McpError::ConfigError {
                    details: format!(
                        "Invalid log format '{}'. Must be one of: compact, pretty, json",
                        self.logging.format
                    ),
                })
            }
        }

        // Validate file size limits
        if self.policy.max_file_size_mb == 0 {
            return Err(McpError::ConfigError {
                details: "Policy max_file_size_mb must be greater than 0".to_string(),
            });
        }

        if self.audit.max_file_size_mb == 0 {
            return Err(McpError::ConfigError {
                details: "Audit max_file_size_mb must be greater than 0".to_string(),
            });
        }

        Ok(())
    }

    /// Convert to Policy instance
    pub fn to_policy(&self) -> Policy {
        let mode_policy = match self.policy.mode.as_str() {
            "restrictive" => Policy::restrictive(),
            "permissive" => Self::create_permissive_policy(),
            _ => Policy::default(), // "default" mode
        };

        // Apply configuration overrides
        let mut policy = mode_policy;
        policy.allowed_paths = self
            .policy
            .allowed_paths
            .iter()
            .map(PathBuf::from)
            .collect();
        policy.denied_paths = self.policy.denied_paths.iter().map(PathBuf::from).collect();
        policy.max_file_size = self.policy.max_file_size_mb * 1024 * 1024; // Convert MB to bytes
        policy.allowed_commands = self.policy.allowed_commands.clone();
        policy.allow_env_access = self.policy.allow_env_access;
        policy.allow_cargo_operations = self.policy.allow_cargo_operations;
        policy.read_only = self.policy.read_only;
        policy.enabled_tools = self.policy.enabled_tools.clone();
        policy.disabled_tools = self.policy.disabled_tools.clone();

        policy
    }

    /// Convert to AuditConfig instance
    pub fn to_audit_config(&self) -> AuditConfig {
        AuditConfig {
            enabled: self.audit.enabled,
            log_path: PathBuf::from(&self.audit.log_path),
            log_successes: self.audit.log_successes,
            log_policy_denials: self.audit.log_policy_denials,
            log_errors: self.audit.log_errors,
            max_file_size: self.audit.max_file_size_mb * 1024 * 1024, // Convert MB to bytes
            pretty_print: self.audit.pretty_print,
        }
    }

    fn create_permissive_policy() -> Policy {
        Policy {
            allowed_paths: vec![PathBuf::from("/")],
            denied_paths: vec![],
            max_file_size: 100 * 1024 * 1024,
            allow_env_access: true,
            allow_cargo_operations: true,
            read_only: false,
            enabled_tools: vec![],
            disabled_tools: vec![],
            allowed_commands: vec![
                "cargo".to_string(),
                "rustc".to_string(),
                "git".to_string(),
                "node".to_string(),
                "npm".to_string(),
                "yarn".to_string(),
                "python".to_string(),
                "python3".to_string(),
                "make".to_string(),
                "cmake".to_string(),
                "gcc".to_string(),
                "clang".to_string(),
            ],
        }
    }

    /// Get the tracing filter string based on logging configuration
    pub fn get_tracing_filter(&self) -> String {
        format!("mcp_rs={}", self.logging.level.to_lowercase())
    }

    /// Create a sample configuration file content
    #[allow(dead_code)]
    pub fn create_sample() -> String {
        let config = Config::default();
        toml::to_string_pretty(&config).expect("Failed to serialize default config")
    }
}

/// Configuration loading utilities
#[allow(dead_code)]
pub fn load_config() -> Config {
    load_config_from(None)
}

pub fn load_config_from(explicit_path: Option<PathBuf>) -> Config {
    if let Some(path) = explicit_path {
        match Config::load_from_file(&path) {
            Ok(config) => {
                info!(
                    "Loaded configuration from explicit path: {}",
                    path.display()
                );
                return config;
            }
            Err(e) => {
                warn!("Failed to load config from '{}': {}", path.display(), e);
            }
        }
    }

    if let Ok(path) = std::env::var("MCP_RS_CONFIG") {
        let path = expand_home(PathBuf::from(path));
        match Config::load_from_file(&path) {
            Ok(config) => {
                info!(
                    "Loaded configuration from MCP_RS_CONFIG: {}",
                    path.display()
                );
                return config;
            }
            Err(e) => {
                warn!(
                    "Failed to load config from MCP_RS_CONFIG '{}': {}",
                    path.display(),
                    e
                );
            }
        }
    }

    // Try loading from various locations in order
    let config_paths = [
        PathBuf::from("mcp-rs.toml"),
        PathBuf::from(".mcp-rs.toml"),
        PathBuf::from("config/mcp-rs.toml"),
        expand_home(PathBuf::from("~/.config/mcp-rs/config.toml")),
        PathBuf::from("/etc/mcp-rs/config.toml"),
    ];

    for path in &config_paths {
        if path.exists() {
            match Config::load_from_file(path) {
                Ok(config) => {
                    info!("Loaded configuration from: {}", path.display());
                    return config;
                }
                Err(e) => {
                    warn!("Failed to load config from '{}': {}", path.display(), e);
                }
            }
        }
    }

    info!("No configuration file found, using defaults");
    Config::default()
}

fn expand_home(path: PathBuf) -> PathBuf {
    let raw = path.to_string_lossy();
    if raw == "~" || raw.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            let suffix = raw.strip_prefix("~/").unwrap_or("");
            return PathBuf::from(home).join(suffix);
        }
    }
    path
}

/// Create a default configuration file if it doesn't exist
#[allow(dead_code)]
pub fn create_default_config_if_missing() -> McpResult<()> {
    let config_path = Path::new("mcp-rs.toml");

    if !config_path.exists() {
        let content = Config::create_sample();
        std::fs::write(config_path, content).map_err(|e| McpError::ConfigError {
            details: format!("Failed to create default config file: {}", e),
        })?;

        info!("Created default configuration file: mcp-rs.toml");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.server.name, "mcp-rs");
        assert_eq!(config.policy.mode, "default");
        assert!(config.audit.enabled);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.server.name, deserialized.server.name);
        assert_eq!(config.policy.mode, deserialized.policy.mode);
    }

    #[test]
    fn test_config_file_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let sample_config = Config::create_sample();
        temp_file.write_all(sample_config.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let loaded_config = Config::load_from_file(temp_file.path()).unwrap();
        assert!(loaded_config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config_validation() {
        let mut config = Config::default();
        config.policy.mode = "invalid_mode".to_string();
        assert!(config.validate().is_err());

        config.policy.mode = "default".to_string();
        config.logging.level = "invalid_level".to_string();
        assert!(config.validate().is_err());

        config.logging.level = "info".to_string();
        config.policy.max_file_size_mb = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_policy_conversion() {
        let config = Config::default();
        let policy = config.to_policy();
        assert_eq!(
            policy.max_file_size,
            config.policy.max_file_size_mb * 1024 * 1024
        );
        assert_eq!(policy.allow_env_access, config.policy.allow_env_access);
    }

    #[test]
    fn test_audit_config_conversion() {
        let config = Config::default();
        let audit_config = config.to_audit_config();
        assert_eq!(audit_config.enabled, config.audit.enabled);
        assert_eq!(
            audit_config.max_file_size,
            config.audit.max_file_size_mb * 1024 * 1024
        );
    }

    #[test]
    fn test_tracing_filter() {
        let mut config = Config::default();
        config.logging.level = "debug".to_string();
        assert_eq!(config.get_tracing_filter(), "mcp_rs=debug");
    }

    #[test]
    fn test_explicit_config_path_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = r#"
[server]
name = "explicit-mcp"
version = "9.9.9"
enable_health_checks = true
transport = "http"
http_addr = "127.0.0.1:9999"

[policy]
mode = "default"
allowed_paths = ["."]
denied_paths = []
max_file_size_mb = 1
allowed_commands = []
allow_env_access = false
allow_cargo_operations = false
read_only = true
enabled_tools = ["health", "read_file"]
disabled_tools = ["read_env"]

[audit]
enabled = false
log_path = "audit.jsonl"
log_successes = true
log_policy_denials = true
log_errors = true
max_file_size_mb = 1
pretty_print = false

[logging]
level = "warn"
format = "compact"
include_timestamps = true
include_source = false
"#;
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = load_config_from(Some(temp_file.path().to_path_buf()));
        assert_eq!(config.server.name, "explicit-mcp");
        assert_eq!(config.server.transport.as_deref(), Some("http"));
        assert!(config.policy.read_only);
        assert_eq!(config.policy.enabled_tools, vec!["health", "read_file"]);
        assert_eq!(config.policy.disabled_tools, vec!["read_env"]);
    }

    #[test]
    fn test_permissive_policy() {
        let mut config = Config::default();
        config.policy.mode = "permissive".to_string();
        // Override config to get actual permissive behavior
        config.policy.allowed_paths = vec!["/".to_string()];
        config.policy.denied_paths = vec![];
        let policy = config.to_policy();
        assert_eq!(policy.allowed_paths, vec![PathBuf::from("/")]);
        assert!(policy.denied_paths.is_empty());
    }

    #[test]
    fn test_restrictive_policy() {
        let mut config = Config::default();
        config.policy.mode = "restrictive".to_string();
        // Override config to get actual restrictive behavior
        config.policy.allowed_paths = vec![".".to_string()];
        let policy = config.to_policy();
        assert_eq!(policy.allowed_paths, vec![PathBuf::from(".")]);
        // Should have denied paths from config defaults
        assert!(!policy.denied_paths.is_empty());
    }
}
