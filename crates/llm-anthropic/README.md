# llm-anthropic

Anthropic Claude adapter implementing LLM provider traits for agentic AI workflows.

## Usage

```rust,no_run
use llm_anthropic::AnthropicProvider;
use llm_core::{LlmProvider, LlmRequest, GenerationConfig, ChatMessage, MessageRole};

#[tokio::main]
async fn main() {
    let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap();
    let provider = AnthropicProvider::new(api_key);

    let req = LlmRequest {
        model: provider.default_model().to_owned(),
        messages: vec![ChatMessage::text(MessageRole::User, "Hello!")],
        system: None,
        tools: vec![],
        grounding: None,
        config: GenerationConfig { max_tokens: 50, temperature: 0.5, prompt_version: "demo-v1" },
    };
    let resp = provider.generate(req).await.unwrap();
    println!("{}", resp.content);
}
```

## Features

- `AnthropicProvider` - Anthropic Claude adapter implementing `LlmProvider` trait
- Direct HTTP client via `reqwest` (no Anthropic SDK dependency)
- Support for Claude Haiku (default) and Sonnet models
- Tool-use support with `tool_choice` parameter
- Streaming responses support
- Grounding contract enforcement via prompt assembly
- Constructor injection for API key (loaded from Secrets Manager in production)

## License

MIT
