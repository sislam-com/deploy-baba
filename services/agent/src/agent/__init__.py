"""PydanticAI agentic service for sislam.com."""

from agent.agent import AgentDeps, CoverLetterOutput, get_agent
from agent.preground import fetch_resume, match_keywords

__all__ = [
    "AgentDeps",
    "CoverLetterOutput",
    "fetch_resume",
    "get_agent",
    "match_keywords",
]
