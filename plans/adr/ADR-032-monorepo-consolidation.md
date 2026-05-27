# ADR-032: Monorepo Consolidation (agentic-workflow → portfolio)

**Date:** 2026-05-24
**Status:** Accepted
**Affected modules:** W-AGT, W-WEB, W-DX, W-OTF, W-CI

## Context

Two repos exist for related functionality:
- `portfolio` (deploy-baba): Rust-based portfolio platform on sislam.com, ~95% complete. Includes 10 library crates, 7 Lambda services, React SPA, full OpenTofu infra, RAG pipeline, LLM provider abstraction, and resume-tailor pipeline.
- `agentic-workflow`: Python/LangGraph chatbot platform, ~5% complete. Has a scaffolded FastAPI service, a demo LangGraph ReAct agent, and a React chat UI. Everything else (llm-core, rag-core, infra, packages) is planned but unbuilt.

The agentic-workflow project would duplicate 80% of what portfolio already has (LLM abstraction, RAG, infra, auth, deployment). Both target the same AWS account (us-east-1), the same Cognito user pool, and the same domain (sislam.com).

## Decision

> Absorb agentic-workflow into the portfolio monorepo. The LangGraph agent becomes `services/agent/` — a Python Lambda service following the existing microservices pattern (ADR-031). Archive the agentic-workflow repo.

### What moves into portfolio

| Source (agentic-workflow) | Destination (portfolio) |
|--------------------------|------------------------|
| `langgraph-app/` | `services/agent/` |
| `ui/src/components/{ChatPanel,ThreadList,MessageBubble}` | `web/src/components/chat/` |
| Thread/agent Pydantic models | `services/agent/src/models/` |

### What gets dropped (superseded)

| agentic-workflow component | Portfolio equivalent |
|---------------------------|---------------------|
| `packages/core/` | `crates/config-core` (DONE) |
| `packages/llm-core/` | `crates/llm-core` + `crates/llm-anthropic` (DONE) |
| `packages/rag-core/` | `crates/rag-core` + `crates/rag-sqlite` (DONE) |
| `infra/` | Portfolio's 23-file OpenTofu stack (deployed) |
| `services/api/` | LangGraph built-in API + service-protocol invoke |

### Polyglot workspace management

- Cargo workspace ignores non-Rust directories — `services/agent/` is invisible to `cargo`
- Python managed via `uv` with its own `pyproject.toml` in `services/agent/`
- Separate `just agent-*` recipes for the Python service
- CI pipeline adds a Python job alongside the Rust jobs

## Consequences

- Single repo for all sislam.com infrastructure, services, and UI
- `just quality` remains Rust-only; `just agent-test` handles Python quality
- The agentic-workflow repo's plan system, ADRs, and agent cache become obsolete
- Future agents (beyond cover letter) are added as tools in `services/agent/` rather than new repos
