"""Dual-mode entry point for the agent service (ADR-004).

- Lambda: Mangum wraps the FastAPI app.
- Local dev: uvicorn on :3003.
"""

from __future__ import annotations

import json
import os

from fastapi import FastAPI, HTTPException, Request
from langchain_core.messages import HumanMessage
from mangum import Mangum

from agent.graph import graph
from models import CoverLetterRequest, CoverLetterResponse

app = FastAPI(
    title="deploy-baba-agent",
    description="LangGraph agentic service for sislam.com",
    version="0.1.0",
)


def _load_anthropic_key() -> None:
    """Load Anthropic API key from Secrets Manager if not already set."""
    if os.environ.get("ANTHROPIC_API_KEY"):
        return
    arn = os.environ.get("ANTHROPIC_API_KEY_ARN")
    if not arn:
        return
    import boto3

    client = boto3.client("secretsmanager", region_name=os.environ.get("AWS_REGION", "us-east-1"))
    secret = client.get_secret_value(SecretId=arn)
    data = json.loads(secret["SecretString"])
    os.environ["ANTHROPIC_API_KEY"] = data.get("anthropic_api_key", "")


@app.on_event("startup")
async def startup() -> None:
    _load_anthropic_key()


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok", "service": "agent"}


@app.post("/api/v1/agent/cover-letter", response_model=CoverLetterResponse)
async def cover_letter(request: Request, body: CoverLetterRequest) -> CoverLetterResponse:
    """Generate a tailored cover letter from a job description."""
    prompt = (
        f"Please generate a cover letter for the following job description:\n\n"
        f"{body.job_description}"
    )
    result = await graph.ainvoke(
        {
            "messages": [HumanMessage(content=prompt)],
            "job_description": body.job_description,
            "resume_data": None,
            "matched_bullets": None,
            "cover_letter_html": None,
            "download_url": None,
        }
    )

    messages = result.get("messages", [])
    last_message = messages[-1] if messages else None
    content = str(last_message.content) if last_message else ""

    download_url = result.get("download_url", "")
    if not download_url:
        for msg in reversed(messages):
            if hasattr(msg, "content") and "download_url" in str(msg.content):
                try:
                    data = json.loads(str(msg.content))
                    download_url = data.get("download_url", "")
                    break
                except (json.JSONDecodeError, TypeError):
                    continue

    if not download_url:
        raise HTTPException(status_code=500, detail="Cover letter generation failed")

    return CoverLetterResponse(
        preview_html=content,
        download_url=download_url,
        summary="Cover letter generated and tailored to the provided job description.",
    )


handler = Mangum(app, lifespan="off")

if __name__ == "__main__":
    import uvicorn

    uvicorn.run("handler:app", host="0.0.0.0", port=3003, reload=True)
