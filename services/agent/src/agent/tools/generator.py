"""Grounded cover letter generation tool — direct Anthropic API call."""

from __future__ import annotations

from langchain_core.tools import tool

COVER_LETTER_SYSTEM_PROMPT = """\
You are a professional cover letter writer. Generate a tailored cover letter based on \
the candidate's resume data and the job description.

GROUNDING RULES (non-negotiable):
1. Only reference skills, roles, and achievements present in the resume data provided.
2. Never invent experience, certifications, or skills not in the source material.
3. You may rephrase and reorder, but every claim must trace back to a specific resume bullet.
4. Match the tone to the target company — formal for enterprise, conversational for startups.
5. Keep it under 400 words.

Output the cover letter as clean HTML with semantic tags (<p>, <strong>, <em>). \
Do not include <html>, <head>, or <body> tags — just the content."""


@tool
def generate_cover_letter(
    job_description: str,
    resume_summary: str,
    matched_bullets: str = "",
) -> str:
    """Generate a tailored cover letter using the candidate's resume data.

    Uses grounded generation to ensure the cover letter only references real experience.

    Args:
        job_description: The target job description.
        resume_summary: JSON summary of the candidate's resume data.
        matched_bullets: Optional JSON array of matched resume bullets with relevance scores.
    """
    from langchain_anthropic import ChatAnthropic

    llm = ChatAnthropic(model="claude-sonnet-4-5-20250929", max_tokens=2048)
    bullets_section = f"Top Matched Bullets:\n{matched_bullets}\n\n" if matched_bullets else ""
    user_prompt = (
        f"Job Description:\n{job_description}\n\n"
        f"Resume Data:\n{resume_summary}\n\n"
        f"{bullets_section}"
        "Write the cover letter now."
    )
    response = llm.invoke(
        [
            {"role": "system", "content": COVER_LETTER_SYSTEM_PROMPT},
            {"role": "user", "content": user_prompt},
        ]
    )
    return str(response.content)
