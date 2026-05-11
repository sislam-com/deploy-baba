# W-LLM: llm-core + llm-anthropic
**Crates:** `crates/llm-core/`, `crates/llm-anthropic/` | **Status:** WIP
**Coverage floor:** N/A (library crates) | **Depends on:** — | **Depended on by:** W-RST, W-UI (via W-RST)

---

## W-LLM.1 Purpose

Pluggable LLM provider abstraction mirroring the `api-core` + adapter
pattern used elsewhere in the workspace. `llm-core` defines the vendor-agnostic
trait surface; `llm-anthropic` is the first concrete implementation. Future
adapters (`llm-openai`, `llm-bedrock`, `llm-ollama`, `llm-fastembed`) slot in
as independent crates without touching `services/ui`.

The grounding contract — which enforces that the generator may only rephrase
existing resume bullets, never invent content — is also enforced here in the
prompt-assembly layer, making it universal across all adapters.

The agent loop infrastructure — `ToolExecutor` trait and `run_agent_loop()`
orchestrator — also lives in `llm-core`, making agentic behavior
provider-agnostic (ADR-023). Any adapter (Anthropic, OpenAI, Bedrock) that
parses tool calls can participate in agentic workflows without implementing
its own loop.

Template crate for structure and conventions: `crates/api-core/`.

---

## W-LLM.2 Public API Surface (shape; not binding until implementation)

```rust
// crates/llm-core/src/lib.rs

pub trait LlmProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn default_model(&self) -> &str;
    async fn generate(&self, req: LlmRequest) -> Result<LlmResponse, LlmError>;
}

pub trait EmbeddingProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn embedding_dim(&self) -> usize;
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, LlmError>;
}

pub enum MessageContent {
    Text(String),
    ToolResult { tool_use_id: String, content: String, is_error: bool },
}

pub struct ChatMessage {
    pub role: MessageRole,
    pub content: MessageContent,       // was: String (P5 breaking change, ADR-023)
}

pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<ToolDef>,           // for tool-use / structured output
    pub grounding: Option<GroundingContract>,
    pub config: GenerationConfig,
}

pub struct LlmResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub model: String,
    pub stop_reason: StopReason,
}

pub struct ToolCall {
    pub id: String,                    // tool_use block id from API (P5, ADR-023)
    pub name: String,
    pub arguments: serde_json::Value,
}

pub struct ToolResult {
    pub name: String,
    pub content: String,
    pub is_error: bool,
}

pub trait ToolExecutor: Send + Sync {
    fn available_tools(&self) -> Vec<ToolDef>;
    async fn execute(&self, call: &ToolCall) -> Result<ToolResult, LlmError>;
}

pub async fn run_agent_loop(
    provider: &dyn LlmProvider,
    executor: &dyn ToolExecutor,
    initial_request: LlmRequest,
    max_turns: usize,       // safety: 5
    token_budget: u32,       // cumulative ceiling
) -> Result<AgentResult, LlmError>;

pub struct GroundingContract {
    /// Exact bullet text strings the generator is allowed to rephrase.
    /// Anything not in this list is forbidden in the output.
    pub allowed_source_text: Vec<String>,
    pub refusal_policy: RefusalPolicy,
}

pub enum RefusalPolicy {
    /// Return LlmError::GroundingViolation if the model produces
    /// content that doesn't derive from allowed_source_text.
    /// (Enforcement is prompt-layer; post-generation verification is optional.)
    WarnAndLog,
    // Future: HardBlock (post-generation string-similarity check)
}

pub struct GenerationConfig {
    pub max_tokens: u32,
    pub temperature: f32,
    pub prompt_version: &'static str,
}

// Error type
#[derive(thiserror::Error, Debug)]
pub enum LlmError {
    #[error("provider rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    #[error("upstream provider error: {message}")]
    Upstream { message: String },
    #[error("context window exceeded: {token_count} tokens > {limit}")]
    ContextTooLarge { token_count: u32, limit: u32 },
    #[error("grounding contract violation detected")]
    GroundingViolation,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
```

Exact signatures finalized during implementation; the shape is the
commitment. All errors use `thiserror` per workspace convention
(no `anyhow` in library crates, per global CLAUDE.md).

---

## W-LLM.3 Implementation Notes

### llm-core (zero deps)
- Zero vendor SDK dependencies. Pure trait + associated types + error
  enum. Matches `api-core` philosophy.
- Prompt-assembly helpers in `llm-core/src/grounding.rs` enforce the
  grounding contract before constructing `LlmRequest.messages` for any
  adapter. Adapters call these helpers — they do not re-implement them.
- `llm-core/src/testing.rs` provides `StubLlmProvider` with canned
  deterministic responses. This is the test double for all
  `services/ui` integration tests — no real network calls in CI, ever.

### llm-anthropic (first adapter)
- Depends on `llm-core` + the Anthropic Rust SDK crate (or direct
  `reqwest`-based HTTP client if no suitable crate exists — check
  crates.io at implementation time; the `anthropic` and
  `anthropic-sdk` crates have existed since 2024).
- Has no dependency on `services/ui`. `services/ui` depends on it.
- Receives the Anthropic API key via constructor injection:
  `AnthropicProvider::new(api_key: String) -> Self`. The key is
  loaded in `services/ui/src/main.rs` from Secrets Manager using the
  same `init_pow_secret()` pattern (W-CTF, W-SEC).
- Implements `provider_id()` → `"anthropic"`, `default_model()` →
  `"claude-haiku-4-5-20251001"`.
- Uses the Anthropic Messages API. For structured output / tool use,
  maps `LlmRequest.tools` to Anthropic's `tool_choice` format.

### `ChatMessage.content` breaking change (ADR-023)

`ChatMessage.content` changes from `String` to `MessageContent` enum to support
tool-result content blocks alongside plain text. Convenience constructors
`ChatMessage::text(role, content)` and `ChatMessage::tool_result(id, content, is_error)`
minimize call-site migration churn. Affected files (6 total):

- `crates/llm-core/src/grounding.rs` — message construction
- `crates/llm-core/src/testing.rs` — stub response construction
- `crates/llm-anthropic/src/lib.rs` — message serialization
- `services/llm-proxy/src/main.rs` — request construction
- `xtask/src/rag.rs` — ask command
- `xtask/src/resume/generate.rs` — polish_bio

### Agent loop (`run_agent_loop`)

The loop lives in `crates/llm-core/src/agent_loop.rs`:

1. Sends `LlmRequest` to the provider
2. On `StopReason::ToolUse`, calls `ToolExecutor::execute()` for each tool call
3. Appends tool results as `MessageContent::ToolResult` messages
4. Repeats until `EndTurn`, `max_turns` (default 5), or `token_budget` (default 4000) exhaustion

Returns `AgentResult { final_content, tool_calls_made, total_tokens, turns, model }`.

### Cargo feature flag selection
`services/ui/Cargo.toml` feature flag:
```toml
[features]
default = ["llm-anthropic"]
llm-anthropic = ["dep:llm-anthropic"]
# llm-openai = ["dep:llm-openai"]   # future
# llm-bedrock = ["dep:llm-bedrock"] # future
```
`services/ui/src/tailor/` always programs against the `LlmProvider`
trait — never against `AnthropicProvider` directly. The concrete type
is injected at startup in `main.rs`.

### Template crate to follow
`crates/api-core/` — examine its `src/lib.rs`, `Cargo.toml`, and
`src/lib.rs` test conventions before scaffolding `llm-core`.

---

## W-LLM.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-LLM.4.1 | Scaffold `crates/llm-core`: `Cargo.toml` (workspace member, zero vendor deps), `src/lib.rs` (trait surface, associated types, error enum), `src/grounding.rs` (prompt-assembly helpers), `src/testing.rs` (stub provider) | DONE | 8 tests pass (6 unit + 2 doc); `cargo clippy -D warnings` clean |
| W-LLM.4.2 | Scaffold `crates/llm-anthropic`: `Cargo.toml` (dep on llm-core + reqwest), `src/lib.rs` (`AnthropicProvider` impl, constructor injection, model constants) | DONE | Direct HTTP against Anthropic Messages API; clippy clean |
| W-LLM.4.3 | Add `crates/llm-core` + `crates/llm-anthropic` to `[workspace.members]` in root `Cargo.toml`; add `async-trait` to workspace deps; add path entries to `[workspace.dependencies]` | DONE | Workspace plumbing; `services/ui` feature wiring deferred to W-RST PR |
| W-LLM.4.4 | Per-crate README files for `llm-core` and `llm-anthropic` (W-DX.3 alignment) | DONE | MIT license for both crates; describes traits, feature flags, secret name |
| W-LLM.4.5 | `llm-anthropic` integration tests: `provider_id_is_anthropic`, `default_model_is_haiku` (CI-safe); `live_generate_*` (3 `#[ignore]` tests, run via `just test-llm PROFILE`) | DONE (2026-04-15) | |
| W-LLM.4.6 | **Future**: `crates/llm-openai`, `crates/llm-bedrock`, `crates/llm-ollama`, `crates/llm-gemini` — additional `LlmProvider` adapters. Not scheduled. | DEFERRED | |
| W-LLM.4.7 | **Future**: `crates/llm-fastembed` — local ONNX `EmbeddingProvider` impl. Ships alongside W-RST.4.11. ADR-016 created at that point. | DEFERRED | |
| W-LLM.4.8 | Add `id: String` field to `ToolCall` struct | DONE | `crates/llm-core/src/types.rs`; Anthropic tool_result id support |
| W-LLM.4.9 | Extend `ChatMessage.content` from `String` to `MessageContent` enum (Text + ToolResult) | DONE | Breaking change; `ChatMessage::text()`, `ChatMessage::tool_result()`, `text_content()` |
| W-LLM.4.10 | Define `ToolExecutor` trait + `ToolResult` struct | DONE | `crates/llm-core/src/tool_executor.rs`; `available_tools()`, `execute()` |
| W-LLM.4.11 | Implement `run_agent_loop()` orchestrator | DONE | `crates/llm-core/src/agent_loop.rs`; loops on `StopReason::ToolUse`; 3 tests |
| W-LLM.4.12 | Update `StubLlmProvider` for tool-use testing (`with_tool_response`) | DONE | `crates/llm-core/src/testing.rs`; `with_tool_response()`, `with_tool_response_always()` |
| W-LLM.4.13 | Update Anthropic adapter: parse `id` from `ContentBlock::ToolUse`, serialize `MessageContent::ToolResult` as content-array | DONE | `crates/llm-anthropic/src/lib.rs`; tool_result wire format |
| W-LLM.4.14 | Migrate all call-sites for `ChatMessage.content` breaking change | DONE | 7 files migrated: grounding.rs, testing.rs, lib.rs doc, llm-anthropic, llm-proxy, xtask/rag.rs, xtask/resume/generate.rs |

---

## W-LLM.5 Test Strategy

- **`llm-core` unit tests** — grounding contract prompt assembly (given
  `allowed_source_text`, assert the constructed prompt contains exactly
  those strings and the refusal instruction); `LlmError` variant round-trips.
- **`StubLlmProvider` in `testing.rs`** — returns canned `LlmResponse`
  values for a given `LlmRequest.messages` fingerprint. Used by all
  `services/ui` integration tests; no real API calls in CI.
- **`llm-anthropic` integration tests** — run only when
  `ANTHROPIC_API_KEY` env var is present; skip otherwise. Tests a
  real `generate()` call against the Anthropic API with minimal tokens
  (e.g., `max_tokens: 10`). Standard practice for vendor-SDK adapters.
- Quality gate: `just quality` must pass with `StubLlmProvider` and
  zero real API calls. Any test that touches `AnthropicProvider` directly
  must be `#[ignore]` or feature-gated.

---

## W-LLM.6 Cross-References

- → ADR-015 (structural decision — pluggable framework + grounding contract)
- → ADR-023 (Agentic Tool-Dispatch Architecture — agent loop + ToolExecutor)
- → ADR-012 (OpenAPI SSOT — LlmRequest/LlmResponse may surface via OpenAPI models)
- → W-RST (primary consumer of `LlmProvider` trait)
- → W-SEC (secret `deploy-baba/prod/anthropic-api-key`)
- → W-DX.3 (per-crate README coverage)
- → `plans/cross-cutting/llm-policy.md` (operational rules, provider registry, agentic cost model)
- → `crates/api-core/` (template crate for structure and conventions)
- → workspace `claude-api` skill (Anthropic SDK usage guidance)
- ← W-RAG (PortfolioToolExecutor consumes ToolExecutor trait + agent loop)
