//! Tool execution trait for agentic workflows (ADR-023).

use crate::error::LlmError;
use crate::types::{ToolCall, ToolDef};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub name: String,
    pub content: String,
    pub is_error: bool,
}

#[async_trait]
pub trait ToolExecutor: Send + Sync {
    fn available_tools(&self) -> Vec<ToolDef>;
    async fn execute(&self, call: &ToolCall) -> Result<ToolResult, LlmError>;
}
