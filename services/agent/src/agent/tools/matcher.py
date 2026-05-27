"""JD keyword matching tool — invokes the portfolio tailor matcher."""

from __future__ import annotations

import json

from langchain_core.tools import tool

from agent.tools.resume import _invoke_ui_lambda


@tool
def match_jd_keywords(job_description: str) -> str:
    """Match a job description against the candidate's resume data.

    Extracts keywords from the JD and scores them against existing resume bullets.
    Returns ranked matched bullets with relevance scores.

    Args:
        job_description: The full job description text to match against.
    """
    data = _invoke_ui_lambda(
        "POST",
        "/api/v1/tailor/match",
        json.dumps({"job_description": job_description}),
    )
    return json.dumps(data, indent=2)
