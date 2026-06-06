# ADR-033: Cover Letter Agent Architecture

**Date:** 2026-05-24
**Status:** Superseded by [ADR-035](ADR-035-pydantic-ai-agent-migration.md)
**Affected modules:** W-AGT, W-RST, W-RAG, W-LLM, W-WEB, W-UI

## Context

sislam.com needs a public-facing chatbot that generates tailored cover letters from pasted job descriptions. Portfolio already has the building blocks: a RAG pipeline (`/api/ask`), a resume-tailor pipeline (parser, matcher, generator in `services/ui/src/tailor/`), LLM provider abstraction (`crates/llm-core`), and a React chat UI (`web/src/routes/Ask.tsx`).

The cover letter flow is inherently agentic — it requires multiple LLM calls, tool-use loops, and conditional branching (e.g., if the JD mentions skills not in the resume). Building a full agent loop in Rust would take months; LangGraph provides this out of the box.

## Decision

> LangGraph orchestrates the cover letter flow as a ReAct agent with four tools. The agent calls back into portfolio's existing Rust services for data retrieval and keyword matching. The agent runs as its own Lambda (ADR-034). The feature is public-facing with rate limiting.

### Agent flow

```
sislam.com UI (Ask.tsx, public)
  └─ User pastes JD → "Generate cover letter"
      └─ POST /api/v1/agent/cover-letter
          └─ UI Lambda (service-protocol) → Agent Lambda
              └─ LangGraph ReAct agent
                  ├─ Tool: retrieve_resume_data → Lambda invoke → /api/v1/resume
                  ├─ Tool: match_jd_keywords → Lambda invoke → tailor matcher
                  ├─ Tool: generate_cover_letter → Anthropic API (grounded)
                  └─ Tool: save_artifact → S3 upload, presigned URL
              └─ Response: { preview_html, download_url, summary }
```

### Output format

HTML preview rendered inline in the chat UI + PDF download link (S3 presigned URL, 30-day expiry).

### Cost protection

- Rate limit: 2 generations per day per IP (same pattern as `/api/ask`)
- Per-request token budget enforced in LangGraph agent config
- Circuit breaker on LLM calls (reimplemented in Python, mirrors Rust `CircuitBreaker`)

### Why LangGraph, not pure Rust

1. Agent loop with tool dispatch, retries, and conversation memory is LangGraph's core strength
2. Heavy lifting (data retrieval, keyword matching) stays in Rust where it's already implemented
3. Separate Lambda = isolated failure domain, independent scaling
4. LangGraph supports streaming responses natively

### Why not the existing tailor pipeline directly

The resume-tailor (W-RST) is an admin-only, single-shot pipeline for resume customization. The cover letter agent is a public-facing, conversational, multi-step flow that produces a different artifact (cover letter vs. tailored resume). They share the keyword matcher but diverge in purpose.

## Consequences

- New `services/agent/` directory with Python/LangGraph code
- New agent Lambda in OpenTofu (ADR-034)
- `Ask.tsx` gains cover letter generation capability via intent detection
- LLM costs increase with public usage — rate limiting is mandatory
- Future agent actions (e.g., "explain my experience with X") can be added as new LangGraph tools
