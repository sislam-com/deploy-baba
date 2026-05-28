# ADR-030: SaaS AI-DLC Pattern

**Date:** 2026-05-22
**Status:** Proposed
**Affected modules:** W-SAAS, W-RAG, W-MCP, W-AIL, W-LLM

## Context

The deploy-baba portfolio has organically built a complete AI-assisted development lifecycle (AI-DLC)
stack: agent cache for session memory, modular plan system, anti-rot agents, 9-corpus RAG with
grounded retrieval, agentic tool dispatch, and a Cognito-authenticated MCP gateway. Each component
was built to serve the portfolio itself, but together they form a replicable pattern for any
software project.

The next step is to package this pattern as a SaaS-ready product that can onboard external
repositories and provide AI-DLC services: project understanding from day one, ongoing plan/drift
maintenance, RAG-grounded development assistance, and health dashboards tracking accuracy over time.

## Decision

Formalize the AI-DLC as a six-pillar replicable pattern and build a `project-onboard` flow as the
first external-facing feature:

### The Six Pillars

1. **Project Onboarding** — Auto-analyze a git repo to generate a plan index, agent cache, initial
   ADRs, and a tailored MCP configuration. This is the entry point for any new project.

2. **Session Lifecycle** — The agent cache protocol (startup check → stale diff → targeted re-read)
   provides session-to-session continuity without expensive full-repo scans.

3. **Anti-rot Maintenance** — plan-doctor and drift-detector subagents audit plan accuracy; skills
   (`/plan-sync`, `/cache-refresh`, `/memory-curate`) apply corrections with user gating.

4. **RAG-Grounded Answers** — Multi-corpus retrieval (code, infra, plans, API specs, domain data)
   with FTS+embedding hybrid search, live-data injection, and deterministic groundedness scoring.

5. **Agentic Tool Execution** — Provider-agnostic agent loop with safety budgets (max_turns,
   token_budget) and tool-dispatch via HTTP callbacks. Any LLM provider can participate.

6. **Health Dashboard** — Eval-driven accuracy tracking with retrieval quality scores, plan coverage
   percentages, drift item counts, and cache freshness metrics.

### External Repo Onboarding

The `project-onboard` flow accepts any git repo URL and performs read-only analysis:

- Clone into a sandboxed tmpdir (no execution of repo code)
- Detect language (Rust, TypeScript, Python, Go, etc.) via file extensions and build files
- Discover build system (Cargo, npm, pip, go.mod, Makefile, etc.)
- Identify framework patterns (axum, express, django, gin, etc.)
- Generate plan artifacts: `plans/INDEX.md`, `.agent-cache/index.json`, initial ADRs
- Generate MCP configuration: `.mcp-rs.toml` scoped to the repo's structure
- Index the repo into a RAG store (per-language chunkers already exist for Rust/HCL/Markdown)

### Eval-Driven Accuracy Loop

Every RAG query and onboarding analysis produces measurable outputs:
- Retrieval precision/recall scores (via `rag-core/eval.rs`)
- Groundedness scores on generated answers
- Plan coverage vs actual codebase

These metrics are stored in a `rag_eval_results` SQLite table and exposed via
`/api/v1/eval/dashboard`. Iterative improvements to chunkers, prompts, or corpora are measured
against this baseline.

## Consequences

### Positive

- Transforms the portfolio from a showcase into a product prototype with a clear value proposition
- Reuses 100% of existing infrastructure — no new AWS services needed
- The onboarding flow validates every AI-DLC component end-to-end on a fresh codebase
- Eval metrics provide a quantitative improvement signal for each iteration

### Negative

- External repo analysis requires supporting multiple languages (currently only Rust/HCL/Markdown
  chunkers exist)
- Onboarding quality depends heavily on chunker quality for the target language
- The sandboxed analysis cannot capture runtime behavior or dynamic configuration
- Multi-tenant concerns (isolation, billing, auth) are deferred — v1 is single-user

## Cross-References

- → ADR-016 (RAG Architecture — retrieval backend reused by onboarding)
- → ADR-017 (AI-DLC — session lifecycle formalized here)
- → ADR-018 (Anti-rot Agents — pillar 3)
- → ADR-023 (Agentic Tool-Dispatch — pillar 5)
- → ADR-028 (Private Cloud MCP Gateway — deployment surface)
- → `plans/modules/saas-onboard.md` (W-SAAS implementation plan)
- → `plans/cross-cutting/ai-dlc.md` (session lifecycle specification)
