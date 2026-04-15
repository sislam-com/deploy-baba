use thiserror::Error;

/// All errors that can occur during an LLM provider call.
#[derive(Error, Debug)]
pub enum LlmError {
    /// The provider rate-limited this request. Callers should retry after the
    /// indicated number of seconds.
    #[error("provider rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    /// The upstream provider returned a server-side error.
    #[error("upstream provider error: {message}")]
    Upstream { message: String },

    /// The request would exceed the model's context window.
    #[error("context window exceeded: {token_count} tokens > {limit}")]
    ContextTooLarge { token_count: u32, limit: u32 },

    /// The prompt assembled for this request would violate the grounding contract.
    /// Callers should not retry without adjusting the request.
    #[error("grounding contract violation: {reason}")]
    GroundingViolation { reason: String },

    /// Network or I/O error communicating with the provider.
    #[error("network error: {0}")]
    Network(String),

    /// Response from the provider could not be deserialised.
    #[error("response parse error: {0}")]
    Parse(String),

    /// A catch-all for unexpected errors.
    #[error("llm error: {0}")]
    Other(String),
}
