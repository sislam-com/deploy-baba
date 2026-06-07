# Agentic RAG Extension — Stepped Execution Plan

**Last updated:** 2026-05-04
**Owner:** W-RAG, W-LLM
**Status:** Active
**Governs:** Phases 9–11 of execution-roadmap.md (ADR-023)

This document breaks Phases 9–11 into atomic, ordered implementation steps. Each step
produces a compilable, testable increment. Steps within a phase are **strictly sequential**
unless marked `[parallel]`. Steps across phases follow the dependency graph in
`execution-roadmap.md`.

For the *what and why*, see ADR-023 and the module plans. This doc only covers *how, in
what order, and how to verify each step*.

---

## Pre-flight Checklist

Before starting any step, confirm:

- [ ] On a clean feature branch off `main` (e.g., `feat/agentic-rag`)
- [ ] `just quality` passes (green baseline)
- [ ] `.agent-cache/index.json` git.sha matches HEAD (`just cache-status`)
- [ ] W-RAG P1 DONE — `just rag-index && just rag-query "ADR-016"` returns results
- [ ] W-LLM 4.1–4.5 DONE — `cargo test -p llm-core -p llm-anthropic` passes

---

## Phase 9 — Extended RAG Corpora

### Step 9.1: Add `OpenApi` + `Portfolio` to `SourceKind`

**Work item:** W-RAG.7.1
**Files:**
- `crates/rag-core/src/types.rs` — add 2 enum variants + `as_str()` + `Display`

**Changes:**
```rust
// In SourceKind enum, after Cache:
OpenApi,
Portfolio,

// In as_str():
SourceKind::OpenApi => "openapi",
SourceKind::Portfolio => "portfolio",
```

**Verify:**
```
cargo test -p rag-core
cargo clippy -p rag-core -- -D warnings
```

**Gate:** Compilation clean. No downstream breakage (new variants require new match arms — add
`_ => unreachable!()` temporarily in `chunk_file()` and `xtask/src/rag.rs` if needed to keep
compiling, but prefer adding the chunker modules immediately in Step 9.2).

---

### Step 9.2: OpenAPI chunker

**Work item:** W-RAG.7.2
**Files:**
- `crates/rag-core/src/chunk/openapi.rs` — **NEW**
- `crates/rag-core/src/chunk/mod.rs` — add `pub mod openapi;`

**Signature:** `pub fn chunk(path: &str, content: &str) -> Vec<Chunk>`

**Strategy:**
1. Parse `content` as `serde_json::Value`
2. Iterate `paths` object: for each path + operation (`get`, `post`, etc.), emit one `Chunk` with:
   - `content`: readable text block: `"Endpoint: {METHOD} {path}\nDescription: ...\nParameters: ...\nResponse: ..."`
   - `meta`: `{"endpoint": "{METHOD} {path}", "tag": "..."}`
   - `token_count`: word count of content
3. Iterate `components.schemas`: for each schema, emit one `Chunk` with:
   - `content`: `"Schema: {name}\nType: object\nFields: {field}: {type} — {description}, ..."`
   - `meta`: `{"schema": "{name}"}`
4. Apply oversize sliding-window split (800 tokens max, 50-word overlap) — reuse logic from
   `markdown.rs` or extract a shared helper

**Tests (minimum 3):**
- Fixture: minimal OpenAPI JSON with 2 paths + 1 schema → assert 3 chunks, correct meta
- Empty paths → 0 chunks (graceful)
- Oversize endpoint description → split into multiple chunks

**Verify:**
```
cargo test -p rag-core -- openapi
```

---

### Step 9.3: Portfolio data chunker

**Work item:** W-RAG.7.3
**Files:**
- `crates/rag-core/src/chunk/portfolio.rs` — **NEW**
- `crates/rag-core/src/chunk/mod.rs` — add `pub mod portfolio;`

**Signature:** `pub fn chunk(path: &str, content: &str) -> Vec<Chunk>`

**Strategy:**
1. Parse `content` as `serde_json::Value` (expects a JSON array of entities)
2. For each entity, detect type by presence of keys:
   - Has `company` + `title` → Job. Emit chunk: `"Job: {title} at {company} ({start_date}–{end_date})\nTech: {tech_stack}\nSummary: {summary}\nAccomplishments:\n- {detail_text}\n- ..."`
   - Has `name` + `icon` + `description` → Competency. Emit chunk: `"Competency: {name}\n{description}\nEvidence:\n- {highlight_text} ({company})\n- ..."`
   - Has `heading` + `body` → AboutSection. Emit chunk: `"About — {heading}\n{body}"`
   - Has `platform` + `url` → SocialLink. Emit chunk: `"Social: {platform} — {url}"`
3. Each entity → one chunk. Content is readable prose (not raw JSON).
4. Apply oversize split for long entities.

**Tests (minimum 3):**
- Fixture: 1 job with 3 details → 1 chunk containing all 3 bullet lines
- Fixture: 1 competency with 2 evidence items → 1 chunk
- Empty array → 0 chunks

**Verify:**
```
cargo test -p rag-core -- portfolio
```

---

### Step 9.4: Wire chunkers into dispatcher

**Work item:** W-RAG.7.4
**Files:**
- `crates/rag-core/src/chunk/mod.rs` — 2 new match arms in `chunk_file()`

**Changes:**
```rust
SourceKind::OpenApi => openapi::chunk(&path_str, content),
SourceKind::Portfolio => portfolio::chunk(&path_str, content),
```

**Verify:**
```
cargo test -p rag-core
cargo clippy -p rag-core -- -D warnings
```

**Gate:** All 4 existing chunker tests still pass (regression check).

---

### Step 9.5: Extend `xtask rag ingest` with 2 new corpora

**Work item:** W-RAG.7.5
**Files:**
- `xtask/Cargo.toml` — add `api-openapi = { workspace = true }` dependency (if not present)
- `xtask/src/rag.rs` — add OpenAPI + portfolio corpus emission after the 3 existing corpora

**Changes in `ingest()`:**

After the `for (label, kind, dirs) in &corpora` loop and before the cache block:

```rust
// ── OpenAPI spec corpus ──────────────────────────────────────────
println!("  Indexing OpenAPI spec...");
let spec = api_openapi::apidoc::full_spec();
let spec_json = serde_json::to_string_pretty(&spec)?;
let spec_chunks = chunk_file(&SourceKind::OpenApi, Path::new("api/openapi.json"), &spec_json);
store.upsert_document("openapi", "api/openapi.json", &git_sha, &spec_chunks)?;
println!("    1 file, {} chunks", spec_chunks.len());
total_docs += 1;
total_chunks += spec_chunks.len() as u64;

// ── Portfolio data corpus ────────────────────────────────────────
println!("  Indexing portfolio data...");
let portfolio_conn = Connection::open(db_path)?;
// Query jobs, competencies, about sections, social links from SQLite
// Serialize each table as JSON array, chunk, upsert
```

The portfolio data ingestion requires querying the portfolio SQLite DB. Two approaches:
- **Option A:** Query DB directly in xtask (add `rusqlite` queries inline)
- **Option B:** Serialize via existing `api-openapi` models

Prefer Option A — simpler, no trait wiring needed in xtask.

Also update the `ext` match in `index_corpus()` to handle the new variants:
```rust
SourceKind::OpenApi => "json",
SourceKind::Portfolio => "json",
```

**Verify:**
```
cargo build -p xtask
just rag-index
just rag-query "GET /api/jobs"       # expect OpenAPI chunk
just rag-query "AWS experience"      # expect portfolio chunk (if DB has jobs with AWS tech_stack)
just rag-query "ADR-016"             # regression: still returns plan chunk
```

**Gate (Phase 9 complete):** All 3 queries return expected results. `just quality` green.

---

### Step 9.6: Enhance prompt assembly for portfolio awareness

**Work item:** W-RAG.8.1
**Files:**
- `crates/rag-core/src/lib.rs` — modify `DefaultPromptAssembler::assemble()`

**Changes:**

After building `sources_text`, inspect chunk `source_kind` values. If any are `"openapi"` or
`"portfolio"`, prepend an additional paragraph to the system prompt:

```rust
let has_portfolio = chunks.iter().any(|c| c.source_kind == "portfolio" || c.source_kind == "openapi");
let portfolio_preamble = if has_portfolio {
    "When sources include portfolio data (jobs, competencies, about sections), answer as the \
     portfolio owner's assistant. When sources include API documentation, explain endpoints \
     precisely with method, path, parameters, and response shapes.\n\n"
} else {
    ""
};
// Insert before the existing system prompt text
```

**Tests:**
- Existing `assembler_includes_all_citations` still passes (regression)
- New test: mixed `openapi` + `plan` chunks → system prompt contains "portfolio owner's assistant"
- New test: all `rust` chunks → system prompt does NOT contain "portfolio owner's assistant"

**Verify:**
```
cargo test -p rag-core
```

---

### Step 9.7: Add filtered retrieval

**Work item:** W-RAG.8.2
**Files:**
- `crates/rag-sqlite/src/lib.rs` — add `retrieve_filtered()` method

**Changes:**

Add a public method to `RagStore`:

```rust
pub async fn retrieve_filtered(
    &self,
    query: &str,
    top_k: usize,
    kinds: Option<&[&str]>,
) -> Result<Vec<RankedChunk>, RagError>
```

The existing `Retriever::retrieve()` impl delegates to `self.retrieve_filtered(query, top_k, None)`.

When `kinds` is `Some`, append `AND rd.source_kind IN (?, ?, ...)` to the FTS query.

**Tests:**
- Insert chunks with `source_kind` = "rust" and "portfolio"
- `retrieve_filtered(query, 10, Some(&["portfolio"]))` → only portfolio chunks
- `retrieve_filtered(query, 10, None)` → both kinds (backward compat)

**Verify:**
```
cargo test -p rag-sqlite
```

**Gate (Steps 9.6–9.7 complete):** Phase 9 fully done. Commit, run `just quality`, refresh cache.

---

## Phase 10 — Live-Data Retrieval

### Step 10.1: Define `PortfolioDataProvider` trait

**Work item:** W-RAG.9.1
**Files:**
- `crates/rag-core/src/portfolio.rs` — **NEW**
- `crates/rag-core/src/lib.rs` — add `pub mod portfolio;` and re-export trait

**Trait definition:**
```rust
use crate::RagError;
use async_trait::async_trait;

#[async_trait]
pub trait PortfolioDataProvider: Send + Sync {
    async fn get_jobs_summary(&self) -> Result<Vec<serde_json::Value>, RagError>;
    async fn get_job_details(&self, slug: &str) -> Result<Option<serde_json::Value>, RagError>;
    async fn get_competencies_summary(&self) -> Result<Vec<serde_json::Value>, RagError>;
    async fn get_about_sections(&self) -> Result<Vec<serde_json::Value>, RagError>;
}
```

**Verify:**
```
cargo check -p rag-core
```

---

### Step 10.2: Implement `PortfolioDataProvider` for `Db`

**Work item:** W-RAG.9.2
**Files:**
- `services/ui/src/db.rs` — add `impl PortfolioDataProvider for Db` (or a newtype)

**Strategy:** Reuse the same SQL queries from the existing route handlers
(`routes/api/jobs.rs`, `routes/api/competencies.rs`, etc.). Wrap results as
`serde_json::Value` via `serde_json::to_value()`.

The `Db` struct wraps `Mutex<Connection>`. Each trait method locks, queries, maps to JSON,
returns.

**Tests:**
- Seed a test DB with `run_migrations()` + insert 2 jobs + 1 competency
- Call `get_jobs_summary()` → 2 results
- Call `get_job_details("test-slug")` → Some with detail bullets

**Verify:**
```
cargo test -p deploy-baba-ui -- portfolio_data
```

---

### Step 10.3: Create `HybridRetriever`

**Work item:** W-RAG.9.3
**Files:**
- `crates/rag-core/src/hybrid.rs` — **NEW** (or inline in `crates/rag-core/src/lib.rs`)
- `crates/rag-core/src/lib.rs` — add `pub mod hybrid;` and re-export

**Structure:**
```rust
pub struct HybridRetriever<R: Retriever, P: PortfolioDataProvider> {
    pub fts: R,
    pub portfolio: P,
}
```

**`Retriever` impl:**
1. Run `fts.retrieve(query, top_k)` as normal
2. Check if any returned chunks have `source_kind == "portfolio"` or `"openapi"`,
   OR if query contains portfolio keywords (`["experience", "skills", "job", "work",
   "competency", "resume", "about", "contact", "social", "endpoint", "API"]`)
3. If yes, call `portfolio.get_jobs_summary()` + `portfolio.get_competencies_summary()` +
   `portfolio.get_about_sections()`
4. Convert each result to a virtual `RankedChunk` with `source_kind="portfolio"`,
   `git_sha="live"`, `score=0.0` (sorted after FTS results but included)
5. Merge into results, cap at `top_k`

**Tests:**
- With seeded DB + FTS index: query "what jobs?" → both FTS chunks + live virtual chunks
- Query about pure codebase ("fn main") → no live chunks injected

**Verify:**
```
cargo test -p rag-core -- hybrid
```

---

### Step 10.4: Wire `HybridRetriever` into ask handler

**Work item:** W-RAG.9.4
**Files:**
- `services/ui/src/routes/api/ask.rs` — replace `RagStore` with `HybridRetriever`
- `services/ui/src/state.rs` — update `AppState` if needed (or use `Arc<Db>` already there)

**Changes:**

The ask handler currently receives `State(rag): State<Arc<RagStore>>`. Change to construct
a `HybridRetriever` from the `RagStore` + `Db` in `AppState`:

```rust
let hybrid = HybridRetriever { fts: rag.as_ref(), portfolio: db.as_ref() };
let chunks = hybrid.retrieve(&req.query, top_k).await.map_err(/* ... */)?;
```

**Verify:**
```
cargo build -p deploy-baba-ui
just ui  # start local server
# In another terminal:
curl -X POST http://localhost:3000/api/ask -H 'Content-Type: application/json' \
  -d '{"query": "what jobs does the owner have?"}'
# Expect answer grounded in live DB data
```

**Gate (Phase 10 complete):** Live data reflected without re-index. `just quality` green.
Commit, refresh cache.

---

## Phase 11 — Agentic Core

### Step 11.1: Add `id` field to `ToolCall`

**Work item:** W-LLM.4.8
**Files:**
- `crates/llm-core/src/types.rs` — add `pub id: String` to `ToolCall`

**Breaking change scope:** Every construction site for `ToolCall` must now include `id`.
Currently only `llm-anthropic` constructs `ToolCall` (in response parsing). `StubLlmProvider`
in `testing.rs` may also construct them. Grep for `ToolCall {` across the workspace.

**Verify:**
```
cargo check --workspace
```

Fix all compilation errors before proceeding.

---

### Step 11.2: Extend `ChatMessage` with `MessageContent` enum

**Work item:** W-LLM.4.9
**Files:**
- `crates/llm-core/src/types.rs` — replace `content: String` with `content: MessageContent`

**Changes:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    Text { text: String },
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(default)]
        is_error: bool,
    },
}

impl ChatMessage {
    pub fn text(role: MessageRole, content: impl Into<String>) -> Self {
        Self { role, content: MessageContent::Text { text: content.into() } }
    }

    pub fn tool_result(tool_use_id: impl Into<String>, content: impl Into<String>, is_error: bool) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::ToolResult {
                tool_use_id: tool_use_id.into(),
                content: content.into(),
                is_error,
            },
        }
    }
}
```

**Do NOT fix call-sites yet** — that's Step 11.5. This step only defines the type.
Expect compilation failures across the workspace.

---

### Step 11.3: Define `ToolExecutor` trait + `ToolResult`

**Work item:** W-LLM.4.10
**Files:**
- `crates/llm-core/src/tool_executor.rs` — **NEW**
- `crates/llm-core/src/lib.rs` — add `pub mod tool_executor;` and re-export

**Contents:**
```rust
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
```

**Verify:**
```
cargo check -p llm-core  # (may still fail due to Step 11.2 breakage — that's expected)
```

---

### Step 11.4: Implement `run_agent_loop()`

**Work item:** W-LLM.4.11
**Files:**
- `crates/llm-core/src/agent_loop.rs` — **NEW**
- `crates/llm-core/src/lib.rs` — add `pub mod agent_loop;` and re-export

**Core logic:**
```rust
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

    // Include tool definitions in the request
    request.tools = executor.available_tools();

    for turn in 0..max_turns {
        // Budget check
        if total_input + total_output >= token_budget {
            break;
        }

        let resp = provider.generate(request.clone()).await?;
        total_input += resp.input_tokens;
        total_output += resp.output_tokens;
        model = resp.model.clone();

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
                // Append assistant message with tool calls
                // Execute each tool, collect results
                // Append tool result messages
                // Continue loop
                for call in &resp.tool_calls {
                    let result = executor.execute(call).await?;
                    request.messages.push(ChatMessage::tool_result(
                        &call.id, &result.content, result.is_error,
                    ));
                    tool_calls_made.push((call.clone(), result));
                }
            }
            StopReason::Other(_) => break,
        }
    }

    // Exhausted max_turns — return whatever we have
    Err(LlmError::Other("Agent loop exhausted max_turns without EndTurn".into()))
}
```

**Note:** The exact Anthropic message format for assistant tool-use turns + user tool-result
turns must match the [Anthropic Messages API spec](https://docs.anthropic.com/en/docs/tool-use).
The assistant turn must include the full content blocks (text + tool_use), and the user turn
must include `tool_result` blocks referencing the `tool_use_id`. This may require extending
`ChatMessage` further — decide during implementation.

**Tests (minimum 3 — in `agent_loop.rs` or separate test file):**
- Stub: first call → ToolUse, second call → EndTurn. Assert 2 turns, 1 tool call.
- Stub: always returns ToolUse. Assert exits after `max_turns` with error.
- Stub: EndTurn on first call. Assert 1 turn, 0 tool calls.

---

### Step 11.5: Migrate all `ChatMessage.content` call-sites

**Work item:** W-LLM.4.14
**Files (6 total):**

| File | Change |
|------|--------|
| `crates/llm-core/src/grounding.rs` | `ChatMessage { role, content: "..." }` → `ChatMessage::text(role, "...")` |
| `crates/llm-core/src/testing.rs` | Same pattern; also update `StubLlmProvider` response construction |
| `crates/llm-anthropic/src/lib.rs` | Deserialize: extract text from `MessageContent::Text`; Serialize: handle both variants |
| `services/llm-proxy/src/main.rs:68` | `ChatMessage { role: MessageRole::User, content: req.user_message }` → `ChatMessage::text(MessageRole::User, req.user_message)` |
| `xtask/src/rag.rs` | Same pattern in ask command |
| `xtask/src/resume/generate.rs` | Same pattern in polish_bio |

**Strategy:** Run `cargo check --workspace` after each file. Fix one file at a time.

**Verify:**
```
cargo check --workspace  # must compile clean
cargo test --workspace   # all existing tests pass
cargo clippy --workspace -- -D warnings
```

**Gate:** Full workspace compiles, all tests green. This is the most critical step — the
breaking change is fully landed.

---

### Step 11.6: Update `StubLlmProvider` for tool-use testing

**Work item:** W-LLM.4.12
**Files:**
- `crates/llm-core/src/testing.rs` — add `with_tool_response()`

**Changes:**

Add a method that registers a response with `stop_reason: StopReason::ToolUse` and
populated `tool_calls`. On subsequent calls (after tool results are appended), the stub
falls through to the default text response with `StopReason::EndTurn`.

```rust
pub fn with_tool_response(mut self, key: &str, tool_calls: Vec<ToolCall>) -> Self {
    // Store (key, tool_calls) for matching
    self
}
```

**Tests:** Agent loop tests from Step 11.4 should use this.

**Verify:**
```
cargo test -p llm-core
```

---

### Step 11.7: Update Anthropic adapter for tool_result messages

**Work item:** W-LLM.4.13
**Files:**
- `crates/llm-anthropic/src/lib.rs`

**Changes:**

1. **Parsing response:** Extract `id` from `ContentBlock::ToolUse`:
   ```rust
   ToolUse { id, name, input } => ToolCall { id, name, arguments: input }
   ```

2. **Serializing request:** Handle `MessageContent::ToolResult` in the request builder.
   Anthropic expects:
   ```json
   { "role": "user", "content": [
     { "type": "tool_result", "tool_use_id": "...", "content": "...", "is_error": false }
   ]}
   ```
   The current `ApiMessage` uses `content: &'a str`. Extend to support structured content
   blocks (use `serde_json::Value` or a dedicated enum for the wire type).

**Tests:**
- Existing `provider_id_is_anthropic` and `default_model_is_haiku` still pass
- New test: serialize a request with `MessageContent::ToolResult` → correct JSON structure

**Verify:**
```
cargo test -p llm-anthropic
```

---

### Step 11.8: Define portfolio tools

**Work item:** W-RAG.10.1
**Files:**
- `services/llm-proxy/src/tools.rs` — **NEW**

**Contents:**

Define 6 `ToolDef` structs:

| Tool name | Description | Input schema | Maps to |
|-----------|-------------|--------------|---------|
| `list_jobs` | "List all job positions with company, title, dates, and tech stack" | `{}` (no params) | `GET /api/jobs` |
| `get_job_details` | "Get details for a specific job position including accomplishments" | `{"slug": "string"}` | `GET /api/jobs/{slug}` |
| `list_competencies` | "List all competency categories with descriptions" | `{}` | `GET /api/competencies` |
| `get_competency_details` | "Get evidence and details for a specific competency" | `{"slug": "string"}` | `GET /api/competencies/{slug}` |
| `get_resume` | "Get full resume including summary, jobs, and competencies" | `{}` | `GET /api/resume` |
| `get_about` | "Get about sections describing the portfolio owner" | `{}` | `GET /api/about` |

Export as `pub fn portfolio_tools() -> Vec<ToolDef>`.

**Verify:**
```
cargo check -p llm-proxy
```

---

### Step 11.9: Implement `PortfolioToolExecutor`

**Work item:** W-RAG.10.2
**Files:**
- `services/llm-proxy/src/tool_executor.rs` — **NEW**
- `services/llm-proxy/Cargo.toml` — add `llm-core` dependency (check if not already present)

**Structure:**
```rust
pub struct PortfolioToolExecutor {
    api_base_url: String,
    client: reqwest::Client,
}

impl PortfolioToolExecutor {
    pub fn new(api_base_url: String) -> Self { ... }
}

#[async_trait]
impl ToolExecutor for PortfolioToolExecutor {
    fn available_tools(&self) -> Vec<ToolDef> {
        tools::portfolio_tools()
    }

    async fn execute(&self, call: &ToolCall) -> Result<ToolResult, LlmError> {
        let url = match call.name.as_str() {
            "list_jobs" => format!("{}/api/jobs", self.api_base_url),
            "get_job_details" => {
                let slug = call.arguments["slug"].as_str().unwrap_or("unknown");
                format!("{}/api/jobs/{}", self.api_base_url, slug)
            }
            // ... other tools
            _ => return Err(LlmError::Other(format!("Unknown tool: {}", call.name).into())),
        };

        let resp = self.client.get(&url).send().await
            .map_err(|e| LlmError::Network(e.to_string()))?;
        let body = resp.text().await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        Ok(ToolResult { name: call.name.clone(), content: body, is_error: false })
    }
}
```

**Verify:**
```
cargo check -p llm-proxy
```

---

### Step 11.10: Extend proxy request/response contract

**Work item:** W-RAG.10.4
**Files:**
- `crates/api-openapi/src/models/ask.rs` — add fields to `AskProxyRequest` and `AskProxyResponse`

**Changes:**
```rust
pub struct AskProxyRequest {
    pub system_prompt: String,
    pub user_message: String,
    pub max_tokens: u32,
    pub temperature: f32,
    #[serde(default)]
    pub tools: Vec<serde_json::Value>,
    #[serde(default)]
    pub api_base_url: Option<String>,
}

pub struct AskProxyResponse {
    pub content: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub tools_used: Vec<String>,
    #[serde(default)]
    pub turns: u32,
}
```

Use `#[serde(default)]` on new fields for backward compatibility — existing callers that
don't send `tools` will work unchanged (single-turn passthrough).

**Verify:**
```
cargo test -p api-openapi
cargo check -p llm-proxy -p deploy-baba-ui
```

---

### Step 11.11: Wire agent loop into llm-proxy handler

**Work item:** W-RAG.10.3
**Files:**
- `services/llm-proxy/src/main.rs` — add agentic path

**Changes:**

In `handler()`, after building the `AnthropicProvider`:

```rust
if !req.tools.is_empty() {
    // Agentic path
    let base_url = req.api_base_url
        .ok_or("api_base_url required when tools are provided")?;
    let executor = PortfolioToolExecutor::new(base_url);
    let result = run_agent_loop(&provider, &executor, llm_req, 5, 4000).await
        .map_err(|e| format!("Agent loop error: {e}"))?;
    return Ok(AskProxyResponse {
        content: result.final_content,
        model: result.model,
        input_tokens: result.total_input_tokens,
        output_tokens: result.total_output_tokens,
        tools_used: result.tool_calls_made.iter().map(|(c, _)| c.name.clone()).collect(),
        turns: result.turns as u32,
    });
}
// Existing single-turn path unchanged
```

**Verify:**
```
cargo build -p llm-proxy
```

---

### Step 11.12: Update ask handler for agentic mode

**Work item:** W-RAG.10.5
**Files:**
- `services/ui/src/routes/api/ask.rs`

**Changes:**

Include tool definitions and base URL in the proxy request:

```rust
let proxy_req = AskProxyRequest {
    system_prompt: bundle.system_prompt,
    user_message: bundle.user_message,
    max_tokens: 1024,
    temperature: 0.2,
    tools: serde_json::to_value(tools::portfolio_tools())
        .map(|v| vec![v]).unwrap_or_default(),
    api_base_url: std::env::var("PORTFOLIO_API_BASE_URL").ok(),
};
```

When `PORTFOLIO_API_BASE_URL` is not set, `tools` and `api_base_url` are empty/None →
proxy falls back to single-turn (backward compatible).

Update `AskResponse` construction to include `tools_used` and `turns` from the proxy response.

**Verify:**
```
cargo build -p deploy-baba-ui
```

---

### Step 11.13: Evolve the system prompt

**Work item:** W-RAG.10.6
**Files:**
- `crates/rag-core/src/lib.rs` — update `DefaultPromptAssembler::assemble()`

**Changes:**

When tools are available (detected by the presence of `openapi` or `portfolio` chunks), use
the evolved prompt:

```
You are the portfolio assistant for Shanto, a senior Rust engineer and cloud architect.
You have access to codebase sources AND tools that query live portfolio data.

For codebase/architecture questions: use the provided source documents, cite with [source N].
For professional experience questions: use the available tools to query live data.
Ground every claim in sources or tool results — do not invent experience or skills.
```

**Verify:**
```
cargo test -p rag-core
```

---

### Step 11.14: End-to-end verification

**Work items:** All Phase 11

**Full verification sequence:**
```
just quality                                           # workspace green
just rag-index                                         # re-index with 9 corpora
just rag-query "GET /api/jobs"                         # OpenAPI chunks
just rag-query "AWS"                                   # portfolio chunks
just ask "What AWS experience does the owner have?"    # agentic answer with tool calls
just ask "How does the Lambda deploy work?"             # code-grounded answer (no tools)
just ask "What endpoints does this API have?"           # OpenAPI-grounded answer
```

**Gate (Phase 11 complete):** All commands produce correct, grounded results. Agent loop
tests pass with `StubLlmProvider`. `just quality` green. Commit, run `just cache-refresh`.

---

## Post-Implementation

After all 3 phases land:

1. **Update `.agent-cache/index.json`:** `just cache-refresh`
2. **Update `plans/INDEX.md`:** Mark W-RAG.7–10 and W-LLM.4.8–4.14 items as DONE
3. **Update module Status headers:** `rag.md` → `WIP (P1–P5 DONE)`; `llm-core.md` status update
4. **Run `/plan-sync`** to verify no status mismatches
5. **Deploy:** `just lambda-deploy` for both UI Lambda and llm-proxy Lambda
6. **Infra:** `just infra-plan` to add `PORTFOLIO_API_BASE_URL` env var to llm-proxy Lambda config

---

## Cross-References

- → ADR-023 (Agentic Tool-Dispatch Architecture)
- → `plans/cross-cutting/execution-roadmap.md` (Phases 9–11 summary)
- → `plans/modules/rag.md` (W-RAG.7.x–10.x work items)
- → `plans/modules/llm-core.md` (W-LLM.4.8–4.14 work items)
- → `plans/cross-cutting/llm-policy.md` (agentic cost model)
- → `plans/CONVENTIONS.md` (WBS notation, domain codes, status codes)
