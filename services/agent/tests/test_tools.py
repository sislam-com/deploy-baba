"""Tests for agent system prompt and tool configuration."""

from __future__ import annotations

from agent.agent import COVER_LETTER_SYSTEM_PROMPT


def test_system_prompt_has_grounding_rules() -> None:
    """The system prompt must enforce grounding."""
    assert "GROUNDING RULES" in COVER_LETTER_SYSTEM_PROMPT
    assert "Never invent" in COVER_LETTER_SYSTEM_PROMPT
