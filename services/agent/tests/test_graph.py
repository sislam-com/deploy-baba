"""Tests for the LangGraph cover letter agent."""

from __future__ import annotations

from agent.graph import graph, tools


def test_graph_compiles() -> None:
    """The graph should compile without errors."""
    assert graph is not None
    assert graph.name == "Cover Letter Agent"


def test_tools_registered() -> None:
    """All four tools should be registered."""
    tool_names = {t.name for t in tools}
    assert tool_names == {
        "retrieve_resume_data",
        "match_jd_keywords",
        "generate_cover_letter",
        "save_artifact",
    }
