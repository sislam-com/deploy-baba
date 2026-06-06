"""RAG sync — periodic check + analysis of RAG quality.

Run standalone: `cd services/agent && uv run python -m agent.rag_sync`
Or as part of `just rag-sync`.

Uses PydanticAI with Haiku for cost-efficient analysis of RAG health data.
Pre-fetches all RAG metrics before the LLM call (zero-token pre-grounding).
"""

from __future__ import annotations

import json
import os
from dataclasses import dataclass
from typing import Any

import httpx
from pydantic import BaseModel, Field
from pydantic_ai import Agent

_UI_BASE = os.environ.get("UI_SERVICE_URL", "http://localhost:3000")


class RAGReport(BaseModel):
    """Structured output from the RAG sync analysis."""

    health_summary: str = Field(description="1-2 sentence overall health assessment")
    pass_rate: float = Field(description="Overall eval pass rate as a percentage")
    category_scores: dict[str, float] = Field(
        default_factory=dict, description="Pass rate per evaluation category"
    )
    top_improvements: list[str] = Field(
        description="Top 3 actionable improvements ranked by impact"
    )
    corpus_gaps: list[str] = Field(
        default_factory=list, description="Unindexed files or missing corpus coverage"
    )
    needs_reindex: bool = Field(description="Whether a reindex is recommended")


@dataclass
class RAGSyncDeps:
    """Pre-fetched RAG metrics — all gathered before the LLM call."""

    health: dict[str, Any]
    eval_report: dict[str, Any]
    eval_failures: dict[str, Any]
    corpus_gaps: dict[str, Any]
    reindex_status: dict[str, Any]


SYNC_SYSTEM_PROMPT = """\
You are a RAG quality analyst for the deploy-baba portfolio project.

You have been given pre-fetched RAG metrics as context. Analyze them and produce
a structured improvement report. Be specific and actionable — reference actual
file paths, corpus names, and eval case IDs where possible."""

rag_sync_agent = Agent(
    os.environ.get("AGENT_MODEL", "anthropic:claude-haiku-4-5-20251001"),
    output_type=RAGReport,
    system_prompt=SYNC_SYSTEM_PROMPT,
    deps_type=RAGSyncDeps,
    retries=2,
)


def _call_ui(method: str, path: str) -> dict[str, Any]:
    """Call the UI service RAG endpoints."""
    url = f"{_UI_BASE}{path}"
    with httpx.Client(timeout=30) as client:
        resp = client.request(method, url)
        resp.raise_for_status()
        result: dict[str, Any] = resp.json()
        return result


def _safe_call(method: str, path: str) -> dict[str, Any]:
    """Call UI service, return error dict on failure."""
    try:
        return _call_ui(method, path)
    except httpx.HTTPError as e:
        return {"error": str(e)}


async def _prefetch_rag_data() -> RAGSyncDeps:
    """Fetch all 5 RAG endpoints before the LLM call (zero tokens)."""
    return RAGSyncDeps(
        health=_safe_call("GET", "/api/v1/rag/health"),
        eval_report=_safe_call("GET", "/api/v1/rag/eval/report"),
        eval_failures=_safe_call("GET", "/api/v1/rag/eval/failures"),
        corpus_gaps=_safe_call("GET", "/api/v1/rag/corpus/gaps"),
        reindex_status=_safe_call("GET", "/api/v1/rag/reindex/status"),
    )


async def run_sync() -> str:
    """Run the RAG sync analysis and return the structured report."""
    deps = await _prefetch_rag_data()

    context = (
        f"RAG Health:\n{json.dumps(deps.health, indent=2)}\n\n"
        f"Eval Report:\n{json.dumps(deps.eval_report, indent=2)}\n\n"
        f"Eval Failures:\n{json.dumps(deps.eval_failures, indent=2)}\n\n"
        f"Corpus Gaps:\n{json.dumps(deps.corpus_gaps, indent=2)}\n\n"
        f"Reindex Status:\n{json.dumps(deps.reindex_status, indent=2)}"
    )

    result = await rag_sync_agent.run(
        f"Analyze the following RAG metrics and produce an improvement report:\n\n{context}",
        deps=deps,
    )
    return result.output.model_dump_json(indent=2)


if __name__ == "__main__":
    import asyncio

    report = asyncio.run(run_sync())
    print(report)
