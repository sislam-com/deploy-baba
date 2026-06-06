# W-AGT: agent
**Path:** `services/agent/` | **Status:** DONE (PydanticAI agent, infra, cover letter flow, routing, UI, rate limit, CI all complete)
**Coverage floor:** N/A (Python) | **Depends on:** W-LLM, W-RST, W-RAG, W-OTF, W-UI | **Depended on by:** W-WEB (CoverLetter.tsx)

---

## W-AGT.1 Purpose

PydanticAI-based agentic service for sislam.com (ADR-035 supersedes ADR-033's LangGraph architecture). The first agent action is public cover letter generation: a visitor pastes a job description, and the agent orchestrates resume data retrieval, keyword matching, grounded cover letter generation, PDF conversion, and S3 artifact storage. Returns an HTML preview inline plus a PDF download link.

This is the first Python Lambda in the portfolio project (ADR-034). It follows the existing microservices pattern (ADR-031) — invoked by the UI Lambda via service-protocol, no direct DB access.

---

## W-AGT.2 Public API Surface

```
POST /api/v1/agent/cover-letter
  Body:    { "job_description": "<string>" }
  Returns: { "preview_html": "<string>",
             "download_url": "<s3 presigned url>",
             "summary": "<string>" }
```

Routed from UI Lambda via service-protocol invoke to the agent Lambda.

Rate limit: 2 generations per day per IP (enforced at UI Lambda routing layer, same pattern as `/api/ask`).

---

## W-AGT.3 Implementation Notes

### Architecture (ADR-035: PydanticAI, supersedes ADR-033 LangGraph)

PydanticAI agent with pre-grounded context and 3 tools:

```
services/agent/
├── pyproject.toml          # uv project: pydantic-ai, anthropic, boto3, mangum
├── src/
│   ├── agent/
│   │   ├── __init__.py
│   │   ├── agent.py        # PydanticAI Agent: CoverLetterOutput, AgentDeps, 3 tools
│   │   ├── preground.py    # Pre-grounding: fetch_resume, match_keywords (zero LLM tokens)
│   │   ├── rag_sync.py     # RAG quality analysis (PydanticAI agent)
│   │   └── tools/          # (deprecated — tools now in agent.py / preground.py)
│   ├── handler.py          # Mangum Lambda handler (dual-mode with uvicorn)
│   └── models.py           # Pydantic: CoverLetterRequest, CoverLetterResponse, AgentEvent
└── tests/
    └── test_graph.py       # 8 tests: agent compilation, tools, schema, deps
```

### Pre-grounding (zero LLM tokens)

**fetch_resume:** Invokes UI Lambda to `GET /api/v1/resume`. Returns JSON with jobs, competencies, tech stack. Injected as `AgentDeps.resume_summary` before agent starts.

**match_keywords:** Invokes UI Lambda to `POST /api/v1/tailor/match`. Returns ranked matched bullets with scores. Injected as `AgentDeps.matched_bullets` before agent starts.

### Agent tools (LLM-orchestrated)

**generate_html:** Assembles the pre-grounded resume data + matched bullets into a prompt. System prompt enforces grounding: only rephrase/reorder skills present in the resume data, never invent. Returns HTML cover letter content.

**convert_to_pdf:** Invokes the PDF Lambda to convert HTML to base64-encoded PDF.

**upload_and_link:** Uploads PDF to S3 under `cover-letters/{date}/{hash}.pdf`, generates presigned download URL (30-day expiry).

### Dual-mode entry point

Same pattern as portfolio's Rust services (ADR-004):
- **Lambda:** Mangum wraps the FastAPI app
- **Local dev:** `uvicorn` on `:3003`

### Secrets

Reads `ANTHROPIC_API_KEY` from Secrets Manager at cold start (same ARN as llm-proxy).

---

## W-AGT.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-AGT.4.1 | Scaffold `services/agent/` with pyproject.toml, src layout, handler.py (Mangum dual-mode). Verify `just agent-dev` starts locally. | DONE (2026-05-24) | ADR-034 structure |
| W-AGT.4.2 | Pre-grounding: `preground.py` — `fetch_resume()` + `match_keywords()` via Lambda SDK invoke. Injected as `AgentDeps` before agent starts. | DONE (2026-06-05) | ADR-035; replaces tools/resume.py + tools/matcher.py |
| W-AGT.4.3 | Add thin `POST /api/v1/tailor/match` endpoint in Rust UI Lambda that exposes `matcher.rs` scoring. | DONE (2026-06-05) | `services/ui/src/routes/api/tailor.rs` |
| W-AGT.4.4 | PydanticAI agent: `agent.py` — `CoverLetterOutput` structured output, 3 tools (`generate_html`, `convert_to_pdf`, `upload_and_link`), Haiku default model. | DONE (2026-06-05) | ADR-035; replaces graph.py + tools/generator.py + tools/artifact.py |
| W-AGT.4.5 | Handler rewrite: `handler.py` — batch + SSE streaming endpoints, pre-grounding → agent → result pipeline, token budget logging. | DONE (2026-06-05) | ADR-035; replaces LangGraph handler |
| W-AGT.4.6 | Dependency swap: remove `langgraph`, `langchain-anthropic`, `langchain-core`; add `pydantic-ai`, `anthropic`. | DONE (2026-06-05) | ADR-035 |
| W-AGT.4.7 | Web hook: `useAgentStream.ts` — 4-phase SSE (preground → writer → uploader → linker). | DONE (2026-06-05) | ADR-035 |
| W-AGT.4.8 | Tests: 8 unit tests for PydanticAI agent (compilation, tools, output type, deps, schema, env override). | DONE (2026-06-05) | ADR-035 |
| W-AGT.4.9 | `infra/agent-lambda.tf` — Lambda function, IAM role, CloudWatch log group. | DONE (2026-06-04) | ADR-034 |
| W-AGT.4.10 | IAM: allow invocation of agent Lambda. | N/A | APIGW routes directly to agent Lambda (`apigateway.tf:198-204`); UI Lambda not involved |
| W-AGT.4.11 | S3 lifecycle rule for `cover-letters/` prefix (30-day expiry). | DONE (2026-06-04) | `agent-lambda.tf:119-134` |
| W-AGT.4.12 | Route `POST /api/v1/agent/*` to agent Lambda. | DONE (2026-06-04) | CloudFront→APIGW→Agent (`cdn.tf:248`, `apigateway.tf:179-204`) |
| W-AGT.4.13 | `CoverLetter.tsx` — dedicated cover letter page with agent stream UI, HTML preview, PDF download. | DONE (2026-06-05) | Route, nav, hook, tests all wired |
| W-AGT.4.14 | Rate limiting for agent endpoint — 2/day/IP in FastAPI handler. | DONE (2026-06-05) | `handler.py` — in-memory rate limiter, mirrors ask.rs pattern |
| W-AGT.4.15 | `just agent-build` + `just agent-deploy` justfile recipes. | DONE (2026-05-24) | In justfile |
| W-AGT.4.16 | CI workflow: Python test + build job in `.github/workflows/`. | DONE (2026-06-04) | `ci.yml:63-91` — lint, format, mypy, pytest, zip build |
| W-AGT.4.17 | RAG sync: `rag_sync.py` rewritten from LangGraph ReAct to PydanticAI agent with pre-fetched metrics. | DONE (2026-06-05) | ADR-035; single LLM call vs 6-8 ReAct iterations |

---

## W-AGT.5 Test Strategy

- **Unit tests for each tool** — mock Lambda SDK invoke and S3 client. Assert correct `ServiceRequest` payloads and response parsing.
- **Graph integration test** — mock all tools, run the full graph with a sample JD. Assert tool call sequence and final response shape.
- **E2E test** — paste a JD into dev.sislam.com Ask UI → verify HTML preview renders and PDF download link works.
- **Rate limit test** — verify 3rd request in same day returns 429.

---

## W-AGT.6 Cross-References

- → ADR-031 (microservices pattern — agent follows same invoke protocol)
- → ADR-032 (monorepo consolidation — agent absorbed from agentic-workflow)
- → ADR-033 (cover letter agent architecture — flow and tool design; **superseded by ADR-035**)
- → ADR-034 (agent Lambda deployment — IAM, build, OpenTofu)
- → ADR-035 (PydanticAI agent migration — replaces LangGraph/LangChain with pre-grounded PydanticAI agent)
- → W-RST (resume-tailor — shares keyword matcher via W-AGT.4.3)
- → W-RAG (RAG pipeline — agent may use RAG retrieval for grounding context)
- → W-LLM (LLM provider abstraction — agent uses Anthropic directly via langchain)
- → W-UI (service-protocol routing — UI Lambda invokes agent Lambda)
- → W-WEB (Ask.tsx — gains agent mode for cover letter generation)
- → W-OTF (OpenTofu — new agent-lambda.tf)
