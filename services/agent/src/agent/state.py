"""Agent conversation state."""

from __future__ import annotations

from typing import Annotated, Any

from langgraph.graph.message import add_messages
from typing_extensions import TypedDict


class AgentState(TypedDict):
    """Conversation state for the cover letter agent."""

    messages: Annotated[list[Any], add_messages]
    job_description: str
    resume_data: dict[str, Any] | None
    matched_bullets: list[dict[str, Any]] | None
    cover_letter_html: str | None
    download_url: str | None
