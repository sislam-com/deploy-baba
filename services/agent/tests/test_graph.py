"""Tests for the PydanticAI cover letter agent."""

from __future__ import annotations

import pytest

from agent.agent import AgentDeps, CoverLetterOutput, get_agent


@pytest.fixture(autouse=True)
def _set_api_key(monkeypatch: pytest.MonkeyPatch) -> None:
    """Set a dummy API key so the agent can be instantiated in tests."""
    monkeypatch.setenv("ANTHROPIC_API_KEY", "test-key-not-real")


@pytest.fixture()
def agent():
    """Get a fresh agent instance for each test."""
    import agent.agent as mod

    mod._agent = None  # force re-init
    return get_agent()


def test_agent_compiles(agent) -> None:
    """The PydanticAI agent should be instantiated without errors."""
    assert agent is not None


def test_agent_has_tools(agent) -> None:
    """The agent should have the expected tools registered."""
    tool_names = set(agent._function_toolset.tools.keys())
    assert "generate_html" in tool_names
    assert "convert_to_pdf" in tool_names
    assert "upload_and_link" in tool_names


def test_agent_has_three_tools(agent) -> None:
    """The agent should have exactly 3 tools."""
    assert len(agent._function_toolset.tools.keys()) == 3


def test_output_type_is_cover_letter(agent) -> None:
    """The agent output type should be CoverLetterOutput."""
    assert agent._output_type is CoverLetterOutput


def test_deps_type_is_agent_deps(agent) -> None:
    """The agent deps type should be AgentDeps."""
    assert agent._deps_type is AgentDeps


def test_cover_letter_output_schema() -> None:
    """CoverLetterOutput should have the expected fields."""
    fields = CoverLetterOutput.model_fields
    assert "html" in fields
    assert "download_url" in fields
    assert "summary" in fields
    assert "grounding_citations" in fields


def test_agent_deps_dataclass() -> None:
    """AgentDeps should be constructable with all required fields."""
    deps = AgentDeps(
        resume_summary='{"jobs": []}',
        matched_bullets=[{"bullet": "test", "score": 0.9}],
        job_description="Test JD",
        pdf_lambda_name="test-pdf-lambda",
        artifacts_bucket="test-bucket",
    )
    assert deps.resume_summary == '{"jobs": []}'
    assert len(deps.matched_bullets) == 1
    assert deps.job_description == "Test JD"


def test_lazy_init_respects_env_model(monkeypatch: pytest.MonkeyPatch) -> None:
    """The agent should pick up AGENT_MODEL from env."""
    import agent.agent as mod

    mod._agent = None
    monkeypatch.setenv("AGENT_MODEL", "anthropic:claude-sonnet-4-5-20250929")
    a = get_agent()
    assert a is not None
    mod._agent = None  # cleanup
