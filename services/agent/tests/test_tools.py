"""Tests for individual agent tools."""

from __future__ import annotations

from agent.tools.artifact import save_artifact
from agent.tools.generator import COVER_LETTER_SYSTEM_PROMPT


def test_system_prompt_has_grounding_rules() -> None:
    """The system prompt must enforce grounding."""
    assert "GROUNDING RULES" in COVER_LETTER_SYSTEM_PROMPT
    assert "Never invent" in COVER_LETTER_SYSTEM_PROMPT


def test_save_artifact_is_tool() -> None:
    """save_artifact should be a langchain tool."""
    assert save_artifact.name == "save_artifact"
    assert "S3" in save_artifact.description
