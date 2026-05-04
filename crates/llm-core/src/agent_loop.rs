//! Provider-agnostic agent loop for tool-dispatch workflows (ADR-023).

use crate::error::LlmError;
use crate::tool_executor::{ToolExecutor, ToolResult};
use crate::types::{ChatMessage, LlmRequest, MessageRole, StopReason, ToolCall};
use crate::LlmProvider;

#[derive(Debug, Clone)]
pub struct AgentResult {
    pub final_content: String,
    pub tool_calls_made: Vec<(ToolCall, ToolResult)>,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
    pub turns: usize,
    pub model: String,
}

pub async fn run_agent_loop(
    provider: &dyn LlmProvider,
    executor: &dyn ToolExecutor,
    mut request: LlmRequest,
    max_turns: usize,
    token_budget: u32,
) -> Result<AgentResult, LlmError> {
    let mut tool_calls_made = Vec::new();
    let mut total_input = 0u32;
    let mut total_output = 0u32;
    let mut model = String::new();

    request.tools = executor.available_tools();

    for turn in 0..max_turns {
        if total_input + total_output >= token_budget {
            tracing::info!(
                turn,
                total_input,
                total_output,
                "agent loop: token budget exhausted"
            );
            break;
        }

        let resp = provider.generate(request.clone()).await?;
        total_input += resp.input_tokens;
        total_output += resp.output_tokens;
        model.clone_from(&resp.model);

        tracing::info!(
            turn,
            stop_reason = ?resp.stop_reason,
            tool_calls = resp.tool_calls.len(),
            input_tokens = resp.input_tokens,
            output_tokens = resp.output_tokens,
            cumulative_tokens = total_input + total_output,
            "agent loop turn"
        );

        match resp.stop_reason {
            StopReason::EndTurn | StopReason::MaxTokens | StopReason::StopSequence => {
                return Ok(AgentResult {
                    final_content: resp.content,
                    tool_calls_made,
                    total_input_tokens: total_input,
                    total_output_tokens: total_output,
                    turns: turn + 1,
                    model,
                });
            }
            StopReason::ToolUse => {
                // Append the assistant's response (text + tool calls are implicit)
                if !resp.content.is_empty() {
                    request
                        .messages
                        .push(ChatMessage::text(MessageRole::Assistant, &resp.content));
                }

                for call in &resp.tool_calls {
                    let result = executor.execute(call).await?;
                    request.messages.push(ChatMessage::tool_result(
                        &call.id,
                        &result.content,
                        result.is_error,
                    ));
                    tool_calls_made.push((call.clone(), result));
                }
            }
            StopReason::Other(_) => break,
        }
    }

    Err(LlmError::Other(format!(
        "agent loop exhausted {max_turns} turns without EndTurn"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::StubLlmProvider;
    use crate::tool_executor::ToolExecutor;
    use crate::types::{GenerationConfig, ToolDef};
    use async_trait::async_trait;

    struct EchoExecutor;

    #[async_trait]
    impl ToolExecutor for EchoExecutor {
        fn available_tools(&self) -> Vec<ToolDef> {
            vec![ToolDef {
                name: "echo".to_string(),
                description: "Echoes input".to_string(),
                input_schema: serde_json::json!({"type": "object", "properties": {"text": {"type": "string"}}}),
            }]
        }

        async fn execute(&self, call: &ToolCall) -> Result<ToolResult, LlmError> {
            let text = call.arguments["text"]
                .as_str()
                .unwrap_or("no text")
                .to_string();
            Ok(ToolResult {
                name: call.name.clone(),
                content: format!("echoed: {text}"),
                is_error: false,
            })
        }
    }

    fn base_request() -> LlmRequest {
        LlmRequest {
            model: String::new(),
            messages: vec![ChatMessage::text(MessageRole::User, "test query")],
            system: None,
            tools: vec![],
            grounding: None,
            config: GenerationConfig {
                max_tokens: 100,
                temperature: 0.0,
                prompt_version: "test-v1",
            },
        }
    }

    #[tokio::test]
    async fn single_turn_end_turn() {
        let stub = StubLlmProvider::new().with_default("direct answer");
        let result = run_agent_loop(&stub, &EchoExecutor, base_request(), 5, 4000)
            .await
            .unwrap();
        assert_eq!(result.turns, 1);
        assert!(result.tool_calls_made.is_empty());
        assert_eq!(result.final_content, "direct answer");
    }

    #[tokio::test]
    async fn tool_use_then_end_turn() {
        let tool_call = ToolCall {
            id: "tc_1".to_string(),
            name: "echo".to_string(),
            arguments: serde_json::json!({"text": "hello"}),
        };
        let stub = StubLlmProvider::new()
            .with_tool_response("test query", vec![tool_call])
            .with_default("final answer after tool");

        let result = run_agent_loop(&stub, &EchoExecutor, base_request(), 5, 4000)
            .await
            .unwrap();
        assert_eq!(result.turns, 2);
        assert_eq!(result.tool_calls_made.len(), 1);
        assert_eq!(result.tool_calls_made[0].1.content, "echoed: hello");
        assert_eq!(result.final_content, "final answer after tool");
    }

    #[tokio::test]
    async fn max_turns_exhausted() {
        let tool_call = ToolCall {
            id: "tc_1".to_string(),
            name: "echo".to_string(),
            arguments: serde_json::json!({"text": "loop"}),
        };
        let stub = StubLlmProvider::new().with_tool_response_always(vec![tool_call]);

        let err = run_agent_loop(&stub, &EchoExecutor, base_request(), 3, 100_000)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("exhausted"));
    }
}
