"""LangGraph ReAct agent for cover letter generation.

Orchestrates: resume retrieval → JD matching → grounded generation → S3 storage.
Deployed as a Lambda via Mangum (ADR-034).
"""

from __future__ import annotations

import os

from langchain_anthropic import ChatAnthropic
from langchain_core.messages import AIMessage
from langgraph.graph import StateGraph
from langgraph.prebuilt import ToolNode

from agent.state import AgentState
from agent.tools import (
    generate_cover_letter,
    match_jd_keywords,
    retrieve_resume_data,
    save_artifact,
)

tools = [retrieve_resume_data, match_jd_keywords, generate_cover_letter, save_artifact]
tool_node = ToolNode(tools)


def _get_llm() -> ChatAnthropic:
    """Return the configured Anthropic LLM with tools bound."""
    model = os.environ.get("ANTHROPIC_MODEL", "claude-sonnet-4-5-20250929")
    return ChatAnthropic(model=model, max_tokens=4096)


SYSTEM_PROMPT = """\
You are a cover letter generation assistant on sislam.com, a portfolio site for a software engineer.

When a user provides a job description, follow these steps:
1. Call retrieve_resume_data to get the candidate's full resume data.
2. Call match_jd_keywords with the job description to find the most relevant experience.
3. Call generate_cover_letter with the JD, resume summary, and matched bullets.
4. Call save_artifact with the generated HTML cover letter to get a download URL.
5. Present the cover letter HTML to the user along with the download link.

Always complete all steps. Never skip the matching step — it ensures the cover letter \
is grounded in real experience."""


async def agent_node(state: AgentState) -> dict:
    """Call the LLM with tools bound."""
    llm = _get_llm()
    llm_with_tools = llm.bind_tools(tools)

    messages = state["messages"]
    if not any(m.type == "system" for m in messages if hasattr(m, "type")):
        from langchain_core.messages import SystemMessage

        messages = [SystemMessage(content=SYSTEM_PROMPT), *messages]

    response = await llm_with_tools.ainvoke(messages)
    return {"messages": [response]}


async def should_continue(state: AgentState) -> str:
    """Route to tools if the LLM requested tool calls, otherwise end."""
    messages = state.get("messages", [])
    if not messages:
        return "__end__"
    last_message = messages[-1]
    if isinstance(last_message, AIMessage) and last_message.tool_calls:
        return "tools"
    return "__end__"


workflow = StateGraph(AgentState)
workflow.add_node("agent", agent_node)
workflow.add_node("tools", tool_node)
workflow.set_entry_point("agent")
workflow.add_conditional_edges("agent", should_continue, {"tools": "tools", "__end__": "__end__"})
workflow.add_edge("tools", "agent")
graph = workflow.compile(name="Cover Letter Agent")
