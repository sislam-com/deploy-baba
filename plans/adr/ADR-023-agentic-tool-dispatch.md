# ADR-023: Agentic Tool-Dispatch Architecture

**Date:** 2026-05-04
**Status:** Proposed
**Affected modules:** W-LLM (primary), W-RAG (primary consumer), W-UI (ask endpoint), W-APIO (proxy contract)

## Context

The existing LLM infrastructure (ADR-015) defines tool-use types (`ToolDef`, `ToolCall`,
`StopReason::ToolUse`) but has no dispatch loop — tools are parsed from API responses but never
executed. The RAG system (ADR-016) provides static chunk retrieval over indexed artifacts. To
promote the portfolio from a static Q&A to an agentic AI assistant that can query live data, we
need:

1. A provider-agnostic tool-dispatch loop in `llm-core`
2. A concrete tool executor in `llm-proxy` that uses the portfolio API as its toolset
3. An architecture that respects the VPC/non-VPC Lambda split (ADR-003)

## Decision

### Tool-dispatch loop in `llm-core`

A `ToolExecutor` trait and `run_agent_loop()` function live in `llm-core`, making agentic behavior
provider-agnostic. The loop:

1. Sends `LlmRequest` to the provider
2. On `StopReason::ToolUse`, calls `ToolExecutor::execute()` for each tool call
3. Appends tool results as `MessageContent::ToolResult` messages
4. Repeats until `EndTurn`, `max_turns`, or `token_budget` exhaustion

Safety limits: `max_turns=5`, `token_budget=4000` (enforced cumulatively across turns).

### HTTP call-back architecture for tool execution

The llm-proxy Lambda (non-VPC, for Anthropic API access) executes tools by calling back to the UI
Lambda's public API endpoints (`GET /api/jobs`, `GET /api/competencies`, etc.) over HTTP. This
bridges the VPC/non-VPC split without architectural changes:

```
llm-proxy Lambda → https://api.anthropic.com       (LLM generation, non-VPC)
llm-proxy Lambda → https://<function-url>/api/jobs  (tool execution, public endpoint)
UI Lambda        → EFS SQLite                       (data access, VPC)
```

The UI Lambda's public API serves as the tool backend. No new infrastructure required.

### Portfolio tool definitions

Six tools map to existing public API endpoints:

| Tool | Endpoint | Purpose |
|------|----------|---------|
| `list_jobs` | `GET /api/jobs` | All positions with company, title, dates, tech_stack |
| `get_job_details` | `GET /api/jobs/{slug}` | Specific job with accomplishment bullets |
| `list_competencies` | `GET /api/competencies` | All skill categories |
| `get_competency_details` | `GET /api/competencies/{slug}` | Competency with evidence |
| `get_resume` | `GET /api/resume` | Full resume aggregate |
| `search_codebase` | FTS via internal query | Keyword search across indexed codebase |

### `ChatMessage.content` breaking change

`ChatMessage.content` changes from `String` to `MessageContent` enum to support tool-result content
blocks. Convenience constructors (`ChatMessage::text()`, `ChatMessage::tool_result()`) minimize
call-site migration churn across 6 affected files.

### Extended RAG corpora

Two new `SourceKind` variants (`OpenApi`, `Portfolio`) and corresponding chunkers index the
portfolio's own API spec and domain data alongside code/plans. `PortfolioDataProvider` trait enables
live-data retrieval at ask-time, ensuring answers reflect dashboard edits without re-indexing.

### Grounding contract for agentic mode

The existing `GroundingContract` (ADR-015) is designed for static source text whitelists. For
agentic mode, grounding is dynamic (tool results obtained at runtime). The system prompt enforces
grounding via instruction rather than the compile-time whitelist. `LlmRequest.grounding` is `None`
for agentic requests.

## Alternatives Considered

### 1. Tool execution in UI Lambda (rejected)

Pass tool calls back from llm-proxy to UI Lambda for execution. Rejected because it requires a
multi-round-trip protocol between two Lambdas, adding latency and complexity. The HTTP call-back
approach is simpler — the proxy Lambda is the orchestrator, the UI Lambda is just an API.

### 2. Move llm-proxy into VPC (rejected)

If llm-proxy had EFS access, it could query SQLite directly. Rejected because the proxy exists
specifically to reach `api.anthropic.com` from a non-VPC context. Adding VPC would require a NAT
Gateway (~$32/month), violating ADR-005 (zero-cost philosophy).

### 3. Separate agent-core crate (rejected)

Put the agent loop in a new `crates/agent-core` crate. Rejected because `ToolExecutor` and
`run_agent_loop` are natural extensions of the existing `llm-core` trait surface. A separate crate
would split the provider abstraction without reducing complexity.

## Consequences

### Positive

- Agentic behavior is provider-agnostic (any LLM adapter can participate)
- Zero new managed infrastructure (reuses existing Lambda + API endpoints)
- Safety limits prevent runaway loops and unbounded cost
- Tool execution is auditable (`tools_used` + `turns` in response)
- Extended corpora give the RAG system awareness of its own API surface

### Negative

- `ChatMessage.content` breaking change affects 6 files across 4 crates
- Each agent turn is a separate LLM call (~3x cost of single-turn RAG)
- HTTP call-back adds latency per tool call (~100ms per Lambda invocation)

### Neutral

- Agentic conversations with Haiku 4.5 cost ~$0.001–0.003 each
- 100K daily token budget supports ~25–50 agentic conversations
- System prompt must enforce grounding via instruction (not compile-time `GroundingContract`) for
  dynamic tool-result context

## Cross-References

- → ADR-003 (Lambda Function URL — determines VPC/non-VPC split)
- → ADR-005 (Zero-Cost Philosophy — no NAT Gateway)
- → ADR-015 (LLM Provider Abstraction — trait surface being extended)
- → ADR-016 (RAG Architecture — retrieval system being extended)
- → `plans/modules/llm-core.md` (W-LLM.4.8–4.14)
- → `plans/modules/rag.md` (W-RAG.7.x–10.x)
- → `plans/cross-cutting/llm-policy.md` (agentic cost model)
