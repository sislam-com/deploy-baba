use async_trait::async_trait;
use llm_core::{LlmError, ToolCall, ToolDef, ToolExecutor, ToolResult};

use crate::tools::portfolio_tools;

pub struct PortfolioToolExecutor {
    api_base_url: String,
    client: reqwest::Client,
}

impl PortfolioToolExecutor {
    pub fn new(api_base_url: String) -> Self {
        Self {
            api_base_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ToolExecutor for PortfolioToolExecutor {
    fn available_tools(&self) -> Vec<ToolDef> {
        portfolio_tools()
    }

    async fn execute(&self, call: &ToolCall) -> Result<ToolResult, LlmError> {
        let url = match call.name.as_str() {
            "list_jobs" => format!("{}/api/jobs", self.api_base_url),
            "get_job_details" => {
                let slug = call.arguments["slug"].as_str().unwrap_or("unknown");
                format!("{}/api/jobs/{}", self.api_base_url, slug)
            }
            "list_competencies" => format!("{}/api/competencies", self.api_base_url),
            "get_about" => format!("{}/api/about/sections", self.api_base_url),
            other => {
                return Ok(ToolResult {
                    name: call.name.clone(),
                    content: format!("Unknown tool: {other}"),
                    is_error: true,
                });
            }
        };

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LlmError::Other(format!("HTTP request to {url} failed: {e}")))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| LlmError::Other(format!("Failed to read response body: {e}")))?;

        if !status.is_success() {
            return Ok(ToolResult {
                name: call.name.clone(),
                content: format!("API returned {status}: {body}"),
                is_error: true,
            });
        }

        Ok(ToolResult {
            name: call.name.clone(),
            content: body,
            is_error: false,
        })
    }
}
