use crate::audit::{AuditEvent, AuditLogger, AuditResult};
use crate::error::{McpError, McpResult};
use crate::policy::Policy;
use crate::tool::Tool;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::{debug, info, warn};

pub struct ToolRegistry {
    tools: HashMap<&'static str, Box<dyn ErasedTool>>,
    policy: Policy,
    audit_logger: Mutex<AuditLogger>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            policy: Policy::default(),
            audit_logger: Mutex::new(AuditLogger::disabled()),
        }
    }

    pub fn new_with_policy(policy: Policy) -> Self {
        Self {
            tools: HashMap::new(),
            policy,
            audit_logger: Mutex::new(AuditLogger::disabled()),
        }
    }

    pub fn new_with_audit(policy: Policy, audit_logger: AuditLogger) -> Self {
        Self {
            tools: HashMap::new(),
            policy,
            audit_logger: Mutex::new(audit_logger),
        }
    }

    pub fn set_policy(&mut self, policy: Policy) {
        info!("Updating tool registry policy");
        self.policy = policy;
    }

    pub fn get_policy(&self) -> &Policy {
        &self.policy
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(T::NAME, Box::new(tool));
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    pub fn list(&self) -> Value {
        let tools: Vec<Value> = self
            .tools
            .iter()
            .filter(|(name, _)| self.policy.can_list_tool(name))
            .map(|(_, t)| t.describe())
            .collect();
        json!({ "tools": tools })
    }

    pub fn call(&self, name: &str, args: Value) -> McpResult<Value> {
        self.call_with_audit(name, args, "unknown".to_string(), serde_json::Value::Null)
    }

    pub fn call_with_audit(
        &self,
        name: &str,
        args: Value,
        request_id: String,
        jsonrpc_id: Value,
    ) -> McpResult<Value> {
        let start_time = std::time::Instant::now();
        let mut policy_checks = Vec::new();

        debug!(tool = name, request_id = request_id, "Tool call requested");

        // Track that we're performing policy check
        policy_checks.push("policy_authorization".to_string());

        // Check policy first
        let policy_result = self.policy.check(name, &args);
        match &policy_result {
            Ok(()) => {
                debug!(tool = name, request_id = request_id, "Policy check passed");
            }
            Err(policy_error) => {
                let duration = start_time.elapsed().as_millis() as u64;
                warn!(
                    tool = name,
                    request_id = request_id,
                    error = %policy_error,
                    "Policy check failed"
                );

                if let Err(audit_err) = self.audit_log(AuditEvent {
                    request_id: request_id.clone(),
                    jsonrpc_id: jsonrpc_id.clone(),
                    tool_name: name.to_string(),
                    arguments: args.clone(),
                    result: AuditResult::PolicyDenied {
                        reason: policy_error.to_string(),
                    },
                    duration_ms: duration,
                    policy_checks: policy_checks.clone(),
                }) {
                    warn!(
                        request_id = request_id,
                        error = %audit_err,
                        "Failed to log audit entry"
                    );
                }

                return Err(policy_error.clone());
            }
        }

        // Find the tool
        let tool = match self.tools.get(name) {
            Some(tool) => tool,
            None => {
                let duration = start_time.elapsed().as_millis() as u64;
                let error = McpError::tool_not_found(name);

                if let Err(audit_err) = self.audit_log(AuditEvent {
                    request_id: request_id.clone(),
                    jsonrpc_id: jsonrpc_id.clone(),
                    tool_name: name.to_string(),
                    arguments: args.clone(),
                    result: AuditResult::Error {
                        message: error.to_string(),
                        code: Some(error.error_code()),
                    },
                    duration_ms: duration,
                    policy_checks: policy_checks.clone(),
                }) {
                    warn!(
                        request_id = request_id,
                        error = %audit_err,
                        "Failed to log audit entry"
                    );
                }

                return Err(error);
            }
        };

        // Execute the tool
        match tool.call(name, args.clone()) {
            Ok(result) => {
                let duration = start_time.elapsed().as_millis() as u64;
                debug!(
                    tool = name,
                    request_id = request_id,
                    duration_ms = duration,
                    "Tool execution completed successfully"
                );

                if let Err(audit_err) = self.audit_log(AuditEvent {
                    request_id: request_id.clone(),
                    jsonrpc_id: jsonrpc_id.clone(),
                    tool_name: name.to_string(),
                    arguments: args.clone(),
                    result: AuditResult::Success {
                        output: result.clone(),
                    },
                    duration_ms: duration,
                    policy_checks: policy_checks.clone(),
                }) {
                    warn!(
                        request_id = request_id,
                        error = %audit_err,
                        "Failed to log audit entry"
                    );
                }

                Ok(result)
            }
            Err(tool_error) => {
                let duration = start_time.elapsed().as_millis() as u64;
                warn!(
                    tool = name,
                    request_id = request_id,
                    error = %tool_error,
                    "Tool execution failed"
                );

                if let Err(audit_err) = self.audit_log(AuditEvent {
                    request_id: request_id.clone(),
                    jsonrpc_id: jsonrpc_id.clone(),
                    tool_name: name.to_string(),
                    arguments: args.clone(),
                    result: AuditResult::Error {
                        message: tool_error.to_string(),
                        code: Some(tool_error.error_code()),
                    },
                    duration_ms: duration,
                    policy_checks: policy_checks.clone(),
                }) {
                    warn!(
                        request_id = request_id,
                        error = %audit_err,
                        "Failed to log audit entry"
                    );
                }

                Err(tool_error)
            }
        }
    }

    fn audit_log(&self, event: AuditEvent) -> McpResult<()> {
        let mut logger = self.audit_logger.lock().map_err(|_| McpError::AuditError {
            details: "Audit logger mutex poisoned".to_string(),
        })?;
        logger.log_tool_call(event)
    }
}

/* ---------------- Type Erasure ---------------- */

trait ErasedTool {
    fn describe(&self) -> Value;
    fn call(&self, tool_name: &str, args: Value) -> McpResult<Value>;
}

impl<T: Tool> ErasedTool for T {
    fn describe(&self) -> Value {
        json!({
            "name": T::NAME,
            "description": T::DESCRIPTION,
            "inputSchema": T::schema()
        })
    }

    fn call(&self, tool_name: &str, args: Value) -> McpResult<Value> {
        let input: T::Input = serde_json::from_value(args)
            .map_err(|e| McpError::invalid_arguments(tool_name, e.to_string()))?;
        let output = self.run(input)?;
        serde_json::to_value(output)
            .map_err(|e| McpError::tool_execution_failed(tool_name, e.to_string()))
    }
}
