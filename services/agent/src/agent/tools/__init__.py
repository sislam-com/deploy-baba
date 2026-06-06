"""Agent tools — now integrated directly into the PydanticAI agent.

Tool functions that were previously LangChain @tool decorated are now either:
- PydanticAI @agent.tool methods in agent/agent.py (convert_to_pdf, upload_and_link)
- Pre-grounding functions in agent/preground.py (fetch_resume, match_keywords)

This module is kept for backward compatibility with imports but the individual
tool files are deprecated.
"""
