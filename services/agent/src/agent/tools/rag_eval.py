"""RAG evaluation and sync tools for the LangGraph agent."""

from __future__ import annotations

import json
import os

import httpx
from langchain_core.tools import tool

_UI_BASE = os.environ.get("UI_SERVICE_URL", "http://localhost:3000")


def _call_ui(method: str, path: str) -> dict:
    """Call the UI service RAG endpoints."""
    url = f"{_UI_BASE}{path}"
    with httpx.Client(timeout=30) as client:
        resp = client.request(method, url)
        resp.raise_for_status()
        return resp.json()


@tool
def check_rag_health() -> str:
    """Get RAG system health: eval scores, corpus stats, and identified gaps."""
    try:
        data = _call_ui("GET", "/api/v1/rag/health")
        return json.dumps(data, indent=2)
    except httpx.HTTPError as e:
        return json.dumps({"error": f"Failed to reach UI service: {e}"})


@tool
def get_eval_report() -> str:
    """Get the latest RAG eval run results: pass rate, per-category breakdown, scores."""
    try:
        data = _call_ui("GET", "/api/v1/rag/eval/report")
        return json.dumps(data, indent=2)
    except httpx.HTTPError as e:
        return json.dumps({"error": f"Failed to reach UI service: {e}"})


@tool
def get_eval_failures() -> str:
    """Get details on failing RAG eval cases for analysis."""
    try:
        data = _call_ui("GET", "/api/v1/rag/eval/failures")
        return json.dumps(data, indent=2)
    except httpx.HTTPError as e:
        return json.dumps({"error": f"Failed to reach UI service: {e}"})


@tool
def get_corpus_gaps() -> str:
    """Scan workspace for unindexed files and identify corpus coverage gaps."""
    try:
        data = _call_ui("GET", "/api/v1/rag/corpus/gaps")
        return json.dumps(data, indent=2)
    except httpx.HTTPError as e:
        return json.dumps({"error": f"Failed to reach UI service: {e}"})


@tool
def get_reindex_status() -> str:
    """Show last ingest time per corpus, document and chunk counts."""
    try:
        data = _call_ui("GET", "/api/v1/rag/reindex/status")
        return json.dumps(data, indent=2)
    except httpx.HTTPError as e:
        return json.dumps({"error": f"Failed to reach UI service: {e}"})
