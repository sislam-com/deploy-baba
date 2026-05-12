# llm-core

Vendor-agnostic LLM provider traits and grounding contract for agentic AI workflows.

## Usage

```rust,no_run
use llm_core::{LlmProvider, LlmRequest, GenerationConfig, ChatMessage, MessageRole};

async fn summarise(provider: &dyn LlmProvider, text: &str) -> Result<String, llm_core::LlmError> {
    let req = LlmRequest {
        model: provider.default_model().to_owned(),
        messages: vec![ChatMessage::text(MessageRole::User, text)],
        system: Some("Summarise the following in one sentence.".to_owned()),
        tools: vec![],
        grounding: None,
        config: GenerationConfig { max_tokens: 200, temperature: 0.3, prompt_version: "demo-v1" },
    };
    let resp = provider.generate(req).await?;
    Ok(resp.content)
}
```

## Features

- `LlmProvider` - Universal trait for LLM providers (Anthropic, OpenAI, Bedrock, etc.)
- `EmbeddingProvider` - Universal trait for embedding providers
- `GroundingContract` - Enforces that generators only rephrase existing content
- `run_agent_loop()` - Agentic tool-dispatch orchestrator
- `ToolExecutor` - Trait for tool execution in agent workflows
- `ChatMessage` with `MessageContent` enum - Supports text and tool-result content
- `StubLlmProvider` - Test double for deterministic testing without API calls

## License

MIT
