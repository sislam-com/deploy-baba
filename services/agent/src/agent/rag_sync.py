"""RAG sync graph — periodic check + analysis of RAG quality.

Run standalone: `cd services/agent && uv run python -m agent.rag_sync`
Or as part of `just rag-sync`.
"""

from __future__ import annotations

import os
from typing import Any

from langchain_anthropic import ChatAnthropic
from langchain_core.messages import HumanMessage, SystemMessage
from langgraph.graph import StateGraph
from langgraph.prebuilt import ToolNode
from typing_extensions import TypedDict

from agent.tools.rag_eval import (
    check_rag_health,
    get_corpus_gaps,
    get_eval_failures,
    get_eval_report,
    get_reindex_status,
)

rag_tools = [
    check_rag_health,
    get_eval_report,
    get_eval_failures,
    get_corpus_gaps,
    get_reindex_status,
]
rag_tool_node = ToolNode(rag_tools)


class RAGSyncState(TypedDict):
    messages: list[Any]
    health: dict[str, Any] | None
    failures: list[dict[str, Any]] | None
    suggestions: list[str] | None
    report: str | None


SYNC_SYSTEM_PROMPT = """\
You are a RAG quality analyst for the deploy-baba portfolio project. Your job is to:

1. Check the current RAG system health using check_rag_health.
2. Get the latest eval report using get_eval_report.
3. If the pass rate is below 85%, get failure details using get_eval_failures.
4. Check for corpus gaps using get_corpus_gaps.
5. Check reindex status using get_reindex_status.

Then produce a structured improvement report with:
- Overall health summary (1-2 sentences)
- Category scorecard (pass rate per category)
- Top 3 actionable improvements ranked by expected impact
- Any corpus gaps that need indexing
- Whether a reindex is needed (based on staleness)

Be specific and actionable. Reference actual file paths, corpus names, and eval case IDs."""


def _get_llm() -> ChatAnthropic:
    model = os.environ.get("ANTHROPIC_MODEL", "claude-sonnet-4-5-20250929")
    return ChatAnthropic(model=model, max_tokens=2048)


async def sync_agent_node(state: RAGSyncState) -> dict[str, list[Any]]:
    llm = _get_llm()
    llm_with_tools = llm.bind_tools(rag_tools)

    messages = state.get("messages", [])
    if not any(getattr(m, "type", None) == "system" for m in messages):
        messages = [SystemMessage(content=SYNC_SYSTEM_PROMPT), *messages]

    response = await llm_with_tools.ainvoke(messages)
    return {"messages": [response]}


async def should_continue(state: RAGSyncState) -> str:
    messages = state.get("messages", [])
    if not messages:
        return "__end__"
    last = messages[-1]
    if hasattr(last, "tool_calls") and last.tool_calls:
        return "tools"
    return "__end__"


workflow = StateGraph(RAGSyncState)
workflow.add_node("agent", sync_agent_node)
workflow.add_node("tools", rag_tool_node)
workflow.set_entry_point("agent")
workflow.add_conditional_edges("agent", should_continue, {"tools": "tools", "__end__": "__end__"})
workflow.add_edge("tools", "agent")
rag_sync_graph = workflow.compile(name="RAG Sync Agent")


async def run_sync() -> str:
    """Run the RAG sync graph and return the final report."""
    result = await rag_sync_graph.ainvoke(
        {
            "messages": [
                HumanMessage(
                    content="Analyze the current RAG system quality "
                    "and produce an improvement report.",
                )
            ],
            "health": None,
            "failures": None,
            "suggestions": None,
            "report": None,
        }
    )
    messages = result.get("messages", [])
    if messages:
        last = messages[-1]
        return str(last.content) if hasattr(last, "content") else str(last)
    return "No report generated."


if __name__ == "__main__":
    import asyncio

    report = asyncio.run(run_sync())
    print(report)
