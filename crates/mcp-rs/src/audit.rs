use crate::error::{McpError, McpResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, warn};

/// Represents the result of a tool execution for audit purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuditResult {
    /// Tool executed successfully
    Success {
        /// The output value returned by the tool
        output: Value,
    },
    /// Tool execution was denied by policy
    PolicyDenied {
        /// Reason for policy denial
        reason: String,
    },
    /// Tool execution failed with an error
    Error {
        /// Error message
        message: String,
        /// Error code if available
        code: Option<i32>,
    },
}

/// A complete audit entry for a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Timestamp when the tool call was made (ISO 8601 format)
    pub timestamp: String,

    /// Unique request ID for correlation
    pub request_id: String,

    /// JSON-RPC request ID
    pub jsonrpc_id: Value,

    /// Name of the tool that was called
    pub tool_name: String,

    /// Arguments passed to the tool
    pub arguments: Value,

    /// Result of the tool execution
    pub result: AuditResult,

    /// Duration of the tool execution in milliseconds
    pub duration_ms: u64,

    /// Additional metadata
    pub metadata: AuditMetadata,
}

/// Additional metadata for audit entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetadata {
    /// Version of MCP-RS that processed this request
    pub server_version: String,

    /// Policy version/hash if available
    pub policy_version: Option<String>,

    /// Any policy checks that were performed
    pub policy_checks: Vec<String>,

    /// Source of the request (if known)
    pub source: Option<String>,
}

pub struct AuditEvent {
    pub request_id: String,
    pub jsonrpc_id: Value,
    pub tool_name: String,
    pub arguments: Value,
    pub result: AuditResult,
    pub duration_ms: u64,
    pub policy_checks: Vec<String>,
}

/// Configuration for audit logging
#[derive(Debug, Clone)]
pub struct AuditConfig {
    /// Whether audit logging is enabled
    pub enabled: bool,

    /// Path to the audit log file
    pub log_path: PathBuf,

    /// Whether to include successful operations in the audit log
    pub log_successes: bool,

    /// Whether to include policy denials in the audit log
    pub log_policy_denials: bool,

    /// Whether to include errors in the audit log
    pub log_errors: bool,

    /// Maximum size of a single audit log file in bytes before rotation
    #[allow(dead_code)]
    pub max_file_size: u64,

    /// Whether to pretty-print JSON in audit logs
    pub pretty_print: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_path: PathBuf::from("mcp-audit.jsonl"),
            log_successes: true,
            log_policy_denials: true,
            log_errors: true,
            max_file_size: 100 * 1024 * 1024, // 100MB
            pretty_print: false,
        }
    }
}

/// Audit logger that writes structured audit entries to a file
pub struct AuditLogger {
    config: AuditConfig,
    writer: Option<BufWriter<std::fs::File>>,
}

impl AuditLogger {
    /// Create a new audit logger with the given configuration
    pub fn new(config: AuditConfig) -> McpResult<Self> {
        if !config.enabled {
            return Ok(Self {
                config,
                writer: None,
            });
        }

        // Create parent directory if it doesn't exist
        if let Some(parent) = config.log_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| McpError::AuditError {
                details: format!("Failed to create audit log directory: {}", e),
            })?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.log_path)
            .map_err(|e| McpError::AuditError {
                details: format!("Failed to open audit log file: {}", e),
            })?;

        let writer = BufWriter::new(file);

        Ok(Self {
            config,
            writer: Some(writer),
        })
    }

    /// Create a disabled audit logger
    pub fn disabled() -> Self {
        Self {
            config: AuditConfig {
                enabled: false,
                ..Default::default()
            },
            writer: None,
        }
    }

    pub fn log_tool_call(&mut self, event: AuditEvent) -> McpResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let should_log = match &event.result {
            AuditResult::Success { .. } => self.config.log_successes,
            AuditResult::PolicyDenied { .. } => self.config.log_policy_denials,
            AuditResult::Error { .. } => self.config.log_errors,
        };

        if !should_log {
            debug!(
                "Skipping audit log for {} due to configuration",
                event.tool_name
            );
            return Ok(());
        }

        let tool_name = event.tool_name.clone();
        let entry = AuditEntry {
            timestamp: Self::current_timestamp(),
            request_id: event.request_id,
            jsonrpc_id: event.jsonrpc_id,
            tool_name: event.tool_name,
            arguments: event.arguments,
            result: event.result,
            duration_ms: event.duration_ms,
            metadata: AuditMetadata {
                server_version: "0.1.0".to_string(),
                policy_version: None,
                policy_checks: event.policy_checks,
                source: None,
            },
        };

        self.write_entry(&entry)?;
        debug!("Audit entry logged for tool: {}", tool_name);

        Ok(())
    }

    /// Write an audit entry to the log file
    fn write_entry(&mut self, entry: &AuditEntry) -> McpResult<()> {
        let writer = self.writer.as_mut().ok_or_else(|| McpError::AuditError {
            details: "Audit logger not initialized".to_string(),
        })?;

        let json_str = if self.config.pretty_print {
            serde_json::to_string_pretty(entry)
        } else {
            serde_json::to_string(entry)
        }
        .map_err(|e| McpError::AuditError {
            details: format!("Failed to serialize audit entry: {}", e),
        })?;

        writeln!(writer, "{}", json_str).map_err(|e| McpError::AuditError {
            details: format!("Failed to write audit entry: {}", e),
        })?;

        writer.flush().map_err(|e| McpError::AuditError {
            details: format!("Failed to flush audit log: {}", e),
        })?;

        Ok(())
    }

    /// Get current timestamp in ISO 8601 format
    fn current_timestamp() -> String {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => {
                let secs = duration.as_secs();
                let nanos = duration.subsec_nanos();
                // Create ISO 8601 timestamp
                let datetime = chrono::DateTime::from_timestamp(secs as i64, nanos)
                    .unwrap_or_else(chrono::Utc::now);
                datetime.to_rfc3339()
            }
            Err(_) => {
                warn!("Failed to get system time, using current UTC time");
                chrono::Utc::now().to_rfc3339()
            }
        }
    }

    /// Flush any pending writes
    #[allow(dead_code)]
    pub fn flush(&mut self) -> McpResult<()> {
        if let Some(writer) = &mut self.writer {
            writer.flush().map_err(|e| McpError::AuditError {
                details: format!("Failed to flush audit log: {}", e),
            })?;
        }
        Ok(())
    }

    /// Check if audit logging is enabled
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the current log file path
    #[allow(dead_code)]
    pub fn log_path(&self) -> Option<&Path> {
        if self.config.enabled {
            Some(&self.config.log_path)
        } else {
            None
        }
    }
}

/// Session replay functionality for debugging and analysis
#[allow(dead_code)]
pub struct SessionReplay {
    entries: Vec<AuditEntry>,
}

#[allow(dead_code)]
impl SessionReplay {
    /// Load audit entries from a log file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> McpResult<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| McpError::AuditError {
            details: format!("Failed to read audit log file: {}", e),
        })?;

        let mut entries = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<AuditEntry>(line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    warn!(
                        "Failed to parse audit entry on line {}: {}",
                        line_num + 1,
                        e
                    );
                    // Continue processing other entries instead of failing
                }
            }
        }

        Ok(Self { entries })
    }

    /// Get all audit entries
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Filter entries by tool name
    pub fn filter_by_tool(&self, tool_name: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.tool_name == tool_name)
            .collect()
    }

    /// Filter entries by result type
    pub fn filter_by_result_type(&self, result_type: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|entry| match &entry.result {
                AuditResult::Success { .. } => result_type == "success",
                AuditResult::PolicyDenied { .. } => result_type == "policy_denied",
                AuditResult::Error { .. } => result_type == "error",
            })
            .collect()
    }

    /// Get entries within a time range
    pub fn filter_by_time_range(&self, start: &str, end: &str) -> McpResult<Vec<&AuditEntry>> {
        let start_time =
            chrono::DateTime::parse_from_rfc3339(start).map_err(|e| McpError::AuditError {
                details: format!("Invalid start time format: {}", e),
            })?;

        let end_time =
            chrono::DateTime::parse_from_rfc3339(end).map_err(|e| McpError::AuditError {
                details: format!("Invalid end time format: {}", e),
            })?;

        let mut filtered = Vec::new();

        for entry in &self.entries {
            if let Ok(entry_time) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) {
                if entry_time >= start_time && entry_time <= end_time {
                    filtered.push(entry);
                }
            }
        }

        Ok(filtered)
    }

    /// Generate a summary report
    pub fn generate_summary(&self) -> AuditSummary {
        let total_entries = self.entries.len();
        let mut successful_calls = 0;
        let mut policy_denials = 0;
        let mut errors = 0;
        let mut tools_used = std::collections::HashSet::new();
        let mut total_duration = 0u64;

        for entry in &self.entries {
            match &entry.result {
                AuditResult::Success { .. } => successful_calls += 1,
                AuditResult::PolicyDenied { .. } => policy_denials += 1,
                AuditResult::Error { .. } => errors += 1,
            }
            tools_used.insert(entry.tool_name.clone());
            total_duration += entry.duration_ms;
        }

        let avg_duration = if total_entries > 0 {
            total_duration as f64 / total_entries as f64
        } else {
            0.0
        };

        AuditSummary {
            total_entries,
            successful_calls,
            policy_denials,
            errors,
            unique_tools_used: tools_used.len(),
            avg_duration_ms: avg_duration,
            total_duration_ms: total_duration,
        }
    }
}

/// Summary statistics for audit entries
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AuditSummary {
    pub total_entries: usize,
    pub successful_calls: usize,
    pub policy_denials: usize,
    pub errors: usize,
    pub unique_tools_used: usize,
    pub avg_duration_ms: f64,
    pub total_duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::{NamedTempFile, TempDir};

    fn create_test_config() -> AuditConfig {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_audit.jsonl");

        AuditConfig {
            enabled: true,
            log_path,
            log_successes: true,
            log_policy_denials: true,
            log_errors: true,
            max_file_size: 1024,
            pretty_print: false,
        }
    }

    #[test]
    fn test_audit_logger_creation() {
        let config = create_test_config();
        let logger = AuditLogger::new(config);
        assert!(logger.is_ok());
    }

    #[test]
    fn test_disabled_logger() {
        let logger = AuditLogger::disabled();
        assert!(!logger.is_enabled());
    }

    #[test]
    fn test_audit_entry_logging() {
        let config = create_test_config();
        let mut logger = AuditLogger::new(config).unwrap();

        let result = logger.log_tool_call(AuditEvent {
            request_id: "req_1".to_string(),
            jsonrpc_id: json!(1),
            tool_name: "test_tool".to_string(),
            arguments: json!({"key": "value"}),
            result: AuditResult::Success {
                output: json!({"result": "success"}),
            },
            duration_ms: 100,
            policy_checks: vec!["path_check".to_string()],
        });

        assert!(result.is_ok());
        assert!(logger.flush().is_ok());
    }

    #[test]
    fn test_session_replay_loading() {
        let config = create_test_config();
        let mut logger = AuditLogger::new(config.clone()).unwrap();

        // Log a few entries
        logger
            .log_tool_call(AuditEvent {
                request_id: "req_1".to_string(),
                jsonrpc_id: json!(1),
                tool_name: "tool1".to_string(),
                arguments: json!({}),
                result: AuditResult::Success { output: json!({}) },
                duration_ms: 50,
                policy_checks: vec![],
            })
            .unwrap();

        logger
            .log_tool_call(AuditEvent {
                request_id: "req_2".to_string(),
                jsonrpc_id: json!(2),
                tool_name: "tool2".to_string(),
                arguments: json!({}),
                result: AuditResult::PolicyDenied {
                    reason: "test".to_string(),
                },
                duration_ms: 25,
                policy_checks: vec![],
            })
            .unwrap();

        logger.flush().unwrap();

        // Load and verify
        let replay = SessionReplay::load_from_file(&config.log_path).unwrap();
        assert_eq!(replay.entries().len(), 2);

        let tool1_entries = replay.filter_by_tool("tool1");
        assert_eq!(tool1_entries.len(), 1);

        let successful_entries = replay.filter_by_result_type("success");
        assert_eq!(successful_entries.len(), 1);

        let summary = replay.generate_summary();
        assert_eq!(summary.total_entries, 2);
        assert_eq!(summary.successful_calls, 1);
        assert_eq!(summary.policy_denials, 1);
        assert_eq!(summary.unique_tools_used, 2);
    }

    #[test]
    fn test_audit_result_serialization() {
        let success = AuditResult::Success {
            output: json!({"key": "value"}),
        };
        let serialized = serde_json::to_string(&success).unwrap();
        assert!(serialized.contains("Success"));

        let error = AuditResult::Error {
            message: "test error".to_string(),
            code: Some(-32603),
        };
        let serialized = serde_json::to_string(&error).unwrap();
        assert!(serialized.contains("Error"));
    }
}
