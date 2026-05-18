# llm-openai

OpenAI adapter implementing llm-core traits for agentic AI workflows.

## Usage

```rust,no_run
use llm_openai::OpenAIProvider;
use llm_core::{LlmProvider, LlmRequest, GenerationConfig, ChatMessage, MessageRole};

#[tokio::main]
async fn main() {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap();
    let provider = OpenAIProvider::new(api_key);

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

## Models

- Default: `gpt-4o-mini` (cost-optimized for high-volume tasks)
- Upgrade: `gpt-4o` (higher quality for complex reasoning)

## Secret

The OpenAI API key is stored in AWS Secrets Manager as:
`deploy-baba/prod/openai-api-key`

## License

MIT
