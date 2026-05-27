# W-AGT: agent
**Path:** `services/agent/` | **Status:** TODO
**Coverage floor:** N/A (Python) | **Depends on:** W-LLM, W-RST, W-RAG, W-OTF, W-UI | **Depended on by:** W-WEB (Ask.tsx agent mode)

---

## W-AGT.1 Purpose

LangGraph-based agentic service for sislam.com. The first agent action is public cover letter generation: a visitor pastes a job description, and the agent orchestrates resume data retrieval, keyword matching, grounded cover letter generation, and S3 artifact storage. Returns an HTML preview inline plus a PDF download link.

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

### Architecture

LangGraph ReAct agent with four tools:

```
services/agent/
├── pyproject.toml          # uv project: langgraph, langchain-anthropic, boto3, mangum
├── src/
│   ├── agent/
│   │   ├── __init__.py
│   │   ├── graph.py        # StateGraph: entry → agent_node ↔ tools → respond
│   │   ├── state.py        # TypedDict state with messages + artifacts
│   │   └── tools/
│   │       ├── __init__.py
│   │       ├── resume.py   # retrieve_resume_data — Lambda invoke → UI Lambda
│   │       ├── matcher.py  # match_jd_keywords — Lambda invoke → tailor endpoint
│   │       ├── generator.py # generate_cover_letter — Anthropic API (grounded)
│   │       └── artifact.py # save_artifact — S3 upload, presigned URL
│   ├── handler.py          # Mangum Lambda handler (dual-mode with uvicorn)
│   └── models.py           # Pydantic: CoverLetterRequest, CoverLetterResponse
└── tests/
    ├── conftest.py
    ├── test_graph.py
    └── test_tools.py
```

### Tool details

**retrieve_resume_data:** Invokes UI Lambda with `ServiceRequest` to `GET /api/v1/resume`. Returns JSON with jobs, competencies, tech stack. Used as grounding context for the LLM.

**match_jd_keywords:** Invokes UI Lambda with `ServiceRequest` to `POST /api/v1/tailor/match` (new thin endpoint over existing `matcher.rs`). Returns ranked matched bullets with scores.

**generate_cover_letter:** Direct Anthropic API call via `langchain-anthropic`. System prompt enforces grounding: only rephrase/reorder skills present in the resume data, never invent. Outputs structured HTML cover letter.

**save_artifact:** Converts HTML to PDF (via weasyprint or similar), uploads both to S3 under `cover-letters/{date}/{hash}.{html,pdf}`. Returns presigned download URL (30-day expiry).

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
| W-AGT.4.1 | Scaffold `services/agent/` with pyproject.toml, src layout, handler.py (Mangum dual-mode). Verify `just agent-dev` starts locally. | TODO | ADR-034 structure |
| W-AGT.4.2 | Implement `tools/resume.py` — Lambda SDK invoke to UI Lambda for resume JSON. Mock in tests. | TODO | Reuses existing `/api/v1/resume` endpoint |
| W-AGT.4.3 | Add thin `POST /api/v1/tailor/match` endpoint in Rust UI Lambda that exposes `matcher.rs` scoring. | TODO | New route in `services/ui/src/routes/api/` |
| W-AGT.4.4 | Implement `tools/matcher.py` — Lambda SDK invoke to tailor match endpoint. | TODO | Depends on W-AGT.4.3 |
| W-AGT.4.5 | Implement `tools/generator.py` — Anthropic API cover letter generation with grounding prompt. | TODO | Uses langchain-anthropic; grounding enforced at prompt layer |
| W-AGT.4.6 | Implement `tools/artifact.py` — S3 upload (HTML + PDF) and presigned URL generation. | TODO | PDF generation via weasyprint or headless Chrome |
| W-AGT.4.7 | Implement `graph.py` — LangGraph StateGraph wiring all four tools into ReAct loop. | TODO | Depends on W-AGT.4.2–4.6 |
| W-AGT.4.8 | Implement `handler.py` + `models.py` — FastAPI endpoint wrapping the graph, Mangum handler. | TODO | |
| W-AGT.4.9 | `infra/agent-lambda.tf` — Lambda function, IAM role, CloudWatch log group. | TODO | ADR-034 |
| W-AGT.4.10 | Modify `infra/iam.tf` — allow UI Lambda to invoke agent Lambda. | TODO | |
| W-AGT.4.11 | Modify `infra/s3.tf` — add lifecycle rule for `cover-letters/` prefix (30-day expiry). | TODO | |
| W-AGT.4.12 | Wire service-protocol routing in UI Lambda: `POST /api/v1/agent/*` → agent Lambda. | TODO | Follows ADR-031 pattern |
| W-AGT.4.13 | Extend `Ask.tsx` — intent detection for JD paste, cover letter action button, HTML preview, PDF download. | TODO | |
| W-AGT.4.14 | Rate limiting for agent endpoint — 2/day/IP at UI Lambda routing layer. | TODO | Same pattern as ask.rs |
| W-AGT.4.15 | `just agent-build` + `just agent-deploy` justfile recipes. | TODO | |
| W-AGT.4.16 | CI workflow: Python test + build job in `.github/workflows/`. | TODO | |

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
- → ADR-033 (cover letter agent architecture — flow and tool design)
- → ADR-034 (agent Lambda deployment — IAM, build, OpenTofu)
- → W-RST (resume-tailor — shares keyword matcher via W-AGT.4.3)
- → W-RAG (RAG pipeline — agent may use RAG retrieval for grounding context)
- → W-LLM (LLM provider abstraction — agent uses Anthropic directly via langchain)
- → W-UI (service-protocol routing — UI Lambda invokes agent Lambda)
- → W-WEB (Ask.tsx — gains agent mode for cover letter generation)
- → W-OTF (OpenTofu — new agent-lambda.tf)
