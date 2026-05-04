//! Vendor-agnostic LLM provider traits and grounding contract.
//!
//! `llm-core` defines the trait surface that all LLM consumers program against.
//! Concrete implementations live in adapter crates (`llm-anthropic`, etc.).
//! The grounding contract — which enforces that generators may only rephrase
//! existing source content, never invent — is enforced here at the prompt-assembly
//! layer so it applies uniformly across all adapters.
//!
//! # Design
//!
//! Mirrors the workspace `-core` + adapter pattern (`api-core` → `api-openapi`).
//! Zero vendor SDK dependencies in this crate.
//!
//! # Example
//!
//! ```rust,no_run
//! use llm_core::{LlmProvider, LlmRequest, GenerationConfig, ChatMessage, MessageRole};
//!
//! async fn summarise(provider: &dyn LlmProvider, text: &str) -> Result<String, llm_core::LlmError> {
//!     let req = LlmRequest {
//!         model: provider.default_model().to_owned(),
//!         messages: vec![ChatMessage::text(MessageRole::User, text)],
//!         system: Some("Summarise the following in one sentence.".to_owned()),
//!         tools: vec![],
//!         grounding: None,
//!         config: GenerationConfig { max_tokens: 200, temperature: 0.3, prompt_version: "demo-v1" },
//!     };
//!     let resp = provider.generate(req).await?;
//!     Ok(resp.content)
//! }
//! ```

pub mod agent_loop;
pub mod error;
pub mod grounding;
pub mod testing;
pub mod tool_executor;
pub mod types;

pub use agent_loop::{run_agent_loop, AgentResult};
pub use error::LlmError;
pub use grounding::{assemble_grounded_prompt, GroundingContract, RefusalPolicy};
pub use tool_executor::{ToolExecutor, ToolResult};
pub use types::{
    ChatMessage, GenerationConfig, LlmRequest, LlmResponse, MessageContent, MessageRole,
    StopReason, ToolCall, ToolDef,
};

use async_trait::async_trait;

/// Vendor-agnostic text generation provider.
///
/// Implementors wrap a specific LLM vendor (Anthropic, OpenAI, Bedrock, …).
/// Consumers always program against this trait — never against concrete types.
///
/// # Object safety
///
/// This trait is object-safe via `#[async_trait]`. Use `Arc<dyn LlmProvider>`
/// to share a provider across request handlers.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Stable identifier for this provider, e.g. `"anthropic"`.
    fn provider_id(&self) -> &'static str;

    /// Default model name used when the caller does not specify one.
    fn default_model(&self) -> &str;

    /// Send a generation request and return the response.
    ///
    /// # Errors
    ///
    /// Returns [`LlmError`] on rate limits, upstream errors, or oversized context.
    async fn generate(&self, req: LlmRequest) -> Result<LlmResponse, LlmError>;
}

/// Vendor-agnostic text embedding provider.
///
/// Defined for forward-compatibility. No concrete implementation ships at MVP;
/// the resume-tailor matcher uses pure-Rust keyword scoring instead. A concrete
/// impl (`llm-fastembed` or via `llm-anthropic`) will be added when W-RAG P2
/// or W-RST.4.11 is implemented.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Stable identifier for this embedding provider.
    fn provider_id(&self) -> &'static str;

    /// Dimension of the embedding vectors produced.
    fn embedding_dim(&self) -> usize;

    /// Embed a batch of texts into dense float vectors.
    ///
    /// # Errors
    ///
    /// Returns [`LlmError`] on upstream or rate-limit failures.
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, LlmError>;
}
