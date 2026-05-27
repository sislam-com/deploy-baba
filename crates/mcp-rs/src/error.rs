use serde_json::Value;
use thiserror::Error;

/// Comprehensive error type for MCP-RS operations
#[derive(Debug, Error, Clone)]
pub enum McpError {
    /// Tool not found in registry
    #[error("Tool not found: {tool_name}")]
    ToolNotFound { tool_name: String },

    /// Invalid arguments provided to tool
    #[error("Invalid arguments for tool '{tool_name}': {details}")]
    InvalidArguments { tool_name: String, details: String },

    /// Tool execution failed
    #[error("Tool execution failed for '{tool_name}': {details}")]
    ToolExecutionFailed { tool_name: String, details: String },

    /// Policy denied access
    #[error("Policy denied access: {reason}")]
    PolicyDenied { reason: String },

    /// File system operation failed
    #[error("File system error: {details}")]
    #[allow(dead_code)]
    FileSystemError { details: String },

    /// Path traversal or security violation
    #[error("Security violation: {details}")]
    SecurityViolation { details: String },

    /// JSON-RPC protocol error
    #[error("JSON-RPC error: {details}")]
    JsonRpcError { details: String },

    /// JSON parsing/serialization error
    #[error("JSON error: {details}")]
    JsonError { details: String },

    /// IO operation failed
    #[error("IO error: {details}")]
    IoError { details: String },

    /// Configuration error
    #[error("Configuration error: {details}")]
    ConfigError { details: String },

    /// Audit logging error
    #[error("Audit error: {details}")]
    AuditError { details: String },

    /// Resource not found
    #[error("Resource not found: {uri}")]
    ResourceNotFound { uri: String },

    /// Resource read error
    #[error("Resource read error for '{uri}': {details}")]
    ResourceReadError { uri: String, details: String },

    /// Generic internal error
    #[error("Internal error: {details}")]
    InternalError { details: String },
}

impl McpError {
    /// Get the JSON-RPC error code for this error type
    pub fn error_code(&self) -> i32 {
        match self {
            McpError::ToolNotFound { .. } => -32601, // Method not found
            McpError::InvalidArguments { .. } => -32602, // Invalid params
            McpError::JsonRpcError { .. } => -32600, // Invalid Request
            McpError::JsonError { .. } => -32700,    // Parse error
            McpError::PolicyDenied { .. } => -32000, // Server error (custom)
            McpError::SecurityViolation { .. } => -32001, // Server error (custom)
            McpError::FileSystemError { .. } => -32002, // Server error (custom)
            _ => -32603,                             // Internal error
        }
    }

    /// Create a ToolNotFound error
    pub fn tool_not_found(tool_name: impl Into<String>) -> Self {
        McpError::ToolNotFound {
            tool_name: tool_name.into(),
        }
    }

    /// Create an InvalidArguments error
    pub fn invalid_arguments(tool_name: impl Into<String>, details: impl Into<String>) -> Self {
        McpError::InvalidArguments {
            tool_name: tool_name.into(),
            details: details.into(),
        }
    }

    /// Create a ToolExecutionFailed error
    pub fn tool_execution_failed(tool_name: impl Into<String>, details: impl Into<String>) -> Self {
        McpError::ToolExecutionFailed {
            tool_name: tool_name.into(),
            details: details.into(),
        }
    }

    /// Create a PolicyDenied error
    pub fn policy_denied(reason: impl Into<String>) -> Self {
        McpError::PolicyDenied {
            reason: reason.into(),
        }
    }

    /// Create a SecurityViolation error
    pub fn security_violation(details: impl Into<String>) -> Self {
        McpError::SecurityViolation {
            details: details.into(),
        }
    }

    /// Create a FileSystemError
    #[allow(dead_code)]
    pub fn file_system_error(details: impl Into<String>) -> Self {
        McpError::FileSystemError {
            details: details.into(),
        }
    }
}

/// Result type for MCP operations
pub type McpResult<T> = Result<T, McpError>;

// Conversion implementations for common error types
impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::JsonError {
            details: err.to_string(),
        }
    }
}

impl From<std::io::Error> for McpError {
    fn from(err: std::io::Error) -> Self {
        McpError::IoError {
            details: err.to_string(),
        }
    }
}

impl From<String> for McpError {
    fn from(msg: String) -> Self {
        McpError::InternalError { details: msg }
    }
}

impl From<&str> for McpError {
    fn from(msg: &str) -> Self {
        McpError::InternalError {
            details: msg.to_string(),
        }
    }
}

/// Helper for creating JSON-RPC error responses
pub fn create_error_response(id: Value, error: &McpError) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": error.error_code(),
            "message": error.to_string()
        }
    })
}

/// Request validation helpers
pub mod validation {
    use super::*;
    use serde_json::Value;

    /// Validate basic JSON-RPC request structure
    pub fn validate_jsonrpc_request(value: &Value) -> McpResult<()> {
        // Check if it's an object
        let obj = value.as_object().ok_or_else(|| McpError::JsonRpcError {
            details: "Request must be a JSON object".to_string(),
        })?;

        // Check required fields
        if !obj.contains_key("jsonrpc") {
            return Err(McpError::JsonRpcError {
                details: "Missing 'jsonrpc' field".to_string(),
            });
        }

        if !obj.contains_key("method") {
            return Err(McpError::JsonRpcError {
                details: "Missing 'method' field".to_string(),
            });
        }

        // For requests (not notifications), id is required
        if !obj.contains_key("id") {
            return Err(McpError::JsonRpcError {
                details: "Missing 'id' field in request".to_string(),
            });
        }

        // Validate jsonrpc version
        match obj.get("jsonrpc") {
            Some(Value::String(version)) if version == "2.0" => {}
            _ => {
                return Err(McpError::JsonRpcError {
                    details: "Invalid 'jsonrpc' version, must be '2.0'".to_string(),
                });
            }
        }

        // Validate method is a string
        if !obj.get("method").unwrap().is_string() {
            return Err(McpError::JsonRpcError {
                details: "'method' field must be a string".to_string(),
            });
        }

        Ok(())
    }

    /// Validate tools/call parameters
    pub fn validate_tool_call_params(params: &Value) -> McpResult<(&str, &Value)> {
        let obj = params
            .as_object()
            .ok_or_else(|| McpError::InvalidArguments {
                tool_name: "unknown".to_string(),
                details: "Parameters must be a JSON object".to_string(),
            })?;

        let name =
            obj.get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| McpError::InvalidArguments {
                    tool_name: "unknown".to_string(),
                    details: "Missing or invalid 'name' parameter".to_string(),
                })?;

        let arguments = obj.get("arguments").unwrap_or(&Value::Null);

        Ok((name, arguments))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_error_codes() {
        assert_eq!(McpError::tool_not_found("test").error_code(), -32601);
        assert_eq!(
            McpError::invalid_arguments("test", "bad args").error_code(),
            -32602
        );
        assert_eq!(
            McpError::policy_denied("access denied").error_code(),
            -32000
        );
    }

    #[test]
    fn test_validation_valid_request() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "id": 1
        });

        assert!(validation::validate_jsonrpc_request(&request).is_ok());
    }

    #[test]
    fn test_validation_missing_fields() {
        let request = json!({
            "method": "tools/call"
        });

        assert!(validation::validate_jsonrpc_request(&request).is_err());
    }

    #[test]
    fn test_tool_call_validation() {
        let params = json!({
            "name": "test_tool",
            "arguments": {"key": "value"}
        });

        let result = validation::validate_tool_call_params(&params);
        assert!(result.is_ok());
        let (name, args) = result.unwrap();
        assert_eq!(name, "test_tool");
        assert_eq!(args, &json!({"key": "value"}));
    }
}
