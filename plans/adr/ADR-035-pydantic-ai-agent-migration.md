# ADR-035: PydanticAI Agent Migration

**Date:** 2026-06-05
**Status:** Accepted
**Supersedes:** ADR-033 (Cover Letter Agent Architecture)
**Affected modules:** W-AGT, W-WEB

## Context

The LangGraph/LangChain implementation from ADR-033 produced correct results but at excessive cost (~$20 per invocation in Anthropic API charges). Root causes:

1. **ReAct tool-loop pattern**: 15-20 LLM calls per run with ever-growing message history (each iteration re-sends all prior messages)
2. **Nested LLM calls**: `generate_cover_letter` tool instantiated its own `ChatAnthropic`, doubling cost on that step
3. **Full resume dump in every LLM context**: ~8-15k tokens re-sent with every ReAct iteration
4. **No pre-filtering**: data retrieval and keyword matching used LLM orchestration despite being deterministic operations
5. **Sonnet pricing**: $3/$15 per million tokens when Haiku ($0.25/$1.25) suffices for tool routing

## Decision

> Replace LangGraph/LangChain with PydanticAI. Pre-ground all deterministic work (resume retrieval, keyword matching) at zero LLM cost before the agent starts. Keep the architecture agentic — the LLM decides tool order and can retry — but with cost guardrails.

### Architecture

```
Request (JD text)
  │
  ├─ [LOCAL] Pre-grounding (zero tokens):
  │    ├─ fetch_resume() → Lambda invoke → /api/v1/resume
  │    └─ match_keywords() → Lambda invoke → /api/v1/tailor/match
  │    → Injected as AgentDeps (not fetched by LLM)
  │
  └─ [AGENT] PydanticAI Agent (Haiku default, 3 tools, structured output):
       │  deps: AgentDeps(resume_summary, matched_bullets, job_description, ...)
       │  tools: [generate_html, convert_to_pdf, upload_and_link]
       │  result_type: CoverLetterOutput(html, download_url, summary, grounding_citations)
       │  guardrails: retries=2, token budget warning at 30k
       │
       ├─ Agent decides: generate_html → HTML cover letter from grounded context
       ├─ Agent decides: convert_to_pdf → Lambda invoke → PDF service
       ├─ Agent decides: upload_and_link → S3 put + presigned URL
       └─ Returns: CoverLetterOutput (structured, validated)
```

### Why still agentic

- The LLM chooses tool order and can adapt (e.g., retry generation if HTML is malformed)
- New tools can be added without changing orchestration code
- The agent reasons about tailoring based on pre-grounded context
- PydanticAI supports streaming, structured output validation, and automatic retries

### Cost reduction (3 independent levers)

| Lever | Before | After | Reduction |
|-------|--------|-------|-----------|
| LLM calls per run | 15-20 (unbounded ReAct) | 3-5 (bounded agent) | ~4x |
| Input tokens per run | ~200k (cumulative) | ~15-25k (pre-grounded) | ~10x |
| Model | Sonnet 4.5 ($3/$15/M) | Haiku ($0.25/$1.25/M) | ~12x |
| **Combined** | **~$20/run** | **~$0.01-0.05/run** | **400-2000x** |

### Dependencies removed

- `langgraph` (pulls in LangSmith telemetry)
- `langchain-anthropic` (replaced by PydanticAI native Anthropic support)
- `langchain-core` (replaced by plain Pydantic models)

### Dependencies added

- `pydantic-ai>=0.2` (lightweight, builds on existing pydantic v2 dependency)
- `anthropic>=0.45` (direct SDK, no wrapper tax)

## Consequences

- `services/agent/src/agent/agent.py` replaces `graph.py` + `state.py` + `agents/` directory
- `services/agent/src/agent/preground.py` replaces `tools/resume.py` + `tools/matcher.py`
- `handler.py` rewritten for PydanticAI batch + streaming endpoints
- `rag_sync.py` rewritten from LangGraph ReAct to single PydanticAI agent call with pre-fetched metrics
- `useAgentStream.ts` gains `preground` phase (4 agents instead of 3)
- Model configurable via `AGENT_MODEL` env var (default: Haiku, override to Sonnet for quality testing)
- Token budget warning logged when `usage.total_tokens > 30_000`
- Old LangGraph tools in `tools/` directory are deprecated (kept for reference during transition)
