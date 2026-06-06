"""Dual-mode entry point for the agent service (ADR-004).

- Lambda: Mangum wraps the FastAPI app.
- Local dev: uvicorn on :3003.
"""

from __future__ import annotations

import asyncio
import json
import logging
import os
import time
from collections.abc import AsyncGenerator
from typing import Any

from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import JSONResponse, StreamingResponse
from mangum import Mangum

from agent.agent import AgentDeps, get_agent
from agent.preground import fetch_resume, match_keywords
from linkedin_oauth import _restore_token, load_linkedin_credentials
from linkedin_oauth import router as linkedin_router
from models import AgentEvent, CoverLetterRequest, CoverLetterResponse

logger = logging.getLogger(__name__)

# ── Rate limiter (2/day/IP, mirrors ask.rs pattern) ──────────────────────────

AGENT_RATE_LIMIT = int(os.environ.get("AGENT_RATE_LIMIT", "10"))
_RATE_WINDOW = 86400  # 24 hours
_rate_map: dict[str, dict[str, Any]] = {}


def _check_rate_limit(ip: str) -> bool:
    now = time.monotonic()
    entry = _rate_map.get(ip)
    if entry is None or now - entry["start"] >= _RATE_WINDOW:
        _rate_map[ip] = {"count": 1, "start": now}
        return True
    if entry["count"] >= AGENT_RATE_LIMIT:
        return False
    entry["count"] += 1
    return True


def _extract_client_ip(request: Request) -> str:
    if ip := request.headers.get("x-apigw-source-ip", "").strip():
        return ip
    if (forwarded := request.headers.get("x-forwarded-for", "")) and (
        ip := forwarded.split(",")[0].strip()
    ):
        return ip
    if request.client:
        return request.client.host
    return "unknown"


app = FastAPI(
    title="deploy-baba-agent",
    description="PydanticAI agentic service for sislam.com",
    version="0.2.0",
)
app.include_router(linkedin_router)


def _load_anthropic_key() -> None:
    """Load Anthropic API key from Secrets Manager if not already set."""
    if os.environ.get("ANTHROPIC_API_KEY"):
        return
    arn = os.environ.get("ANTHROPIC_API_KEY_ARN")
    if not arn:
        return
    try:
        import boto3
        import botocore.config

        cfg = botocore.config.Config(connect_timeout=5, read_timeout=5, retries={"max_attempts": 1})
        client = boto3.client(
            "secretsmanager",
            region_name=os.environ.get("AWS_REGION", "us-east-1"),
            config=cfg,
        )
        secret = client.get_secret_value(SecretId=arn)
        raw = secret["SecretString"].strip()
        if raw.startswith("{"):
            data = json.loads(raw)
            key = data.get("anthropic_api_key", "") or data.get("ANTHROPIC_ACCESS_KEY", "")
        else:
            key = raw
        os.environ["ANTHROPIC_API_KEY"] = key
    except Exception as exc:
        logger.warning("Failed to load Anthropic key from Secrets Manager: %s", exc)


def _trim_resume(resume: dict[str, Any]) -> dict[str, Any]:
    """Keep only fields relevant for cover letter generation."""
    return {
        "name": resume.get("name", ""),
        "title": resume.get("title", ""),
        "bio": resume.get("bio", ""),
        "jobs": resume.get("jobs", []),
        "competencies": resume.get("competencies", []),
    }


async def _build_deps(job_description: str) -> AgentDeps:
    """Pre-ground: fetch resume + match keywords before the agent starts."""
    resume = await fetch_resume()
    bullets = await match_keywords(job_description)

    return AgentDeps(
        resume_summary=json.dumps(_trim_resume(resume), indent=2),
        matched_bullets=bullets,
        job_description=job_description,
        pdf_lambda_name=os.environ.get("PDF_LAMBDA_NAME", ""),
        artifacts_bucket=os.environ.get("ARTIFACTS_BUCKET", ""),
    )


@app.on_event("startup")
async def startup() -> None:
    _load_anthropic_key()
    if os.environ.get("ANTHROPIC_API_KEY"):
        logger.info(
            "ANTHROPIC_API_KEY is set (starts with %s...)",
            os.environ["ANTHROPIC_API_KEY"][:8],
        )
    else:
        logger.warning("ANTHROPIC_API_KEY is NOT set — agent calls will fail")
    load_linkedin_credentials()
    await _restore_token()


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok", "service": "agent"}


@app.post("/api/v1/agent/cover-letter", response_model=CoverLetterResponse)
async def cover_letter(request: Request, body: CoverLetterRequest) -> CoverLetterResponse:
    """Generate a tailored cover letter from a job description."""
    if not os.environ.get("ANTHROPIC_API_KEY"):
        raise HTTPException(status_code=503, detail="ANTHROPIC_API_KEY not configured")
    ip = _extract_client_ip(request)
    if not _check_rate_limit(ip):
        raise HTTPException(
            status_code=429,
            detail=f"Rate limit exceeded ({AGENT_RATE_LIMIT}/day)",
        )
    deps = await _build_deps(body.job_description)

    agent_timeout = int(os.environ.get("AGENT_TIMEOUT", "120"))
    try:
        result = await asyncio.wait_for(
            get_agent().run(
                f"Generate a tailored cover letter, convert to PDF, and provide a download link."
                f"\n\nJob Description:\n{body.job_description}",
                deps=deps,
            ),
            timeout=agent_timeout,
        )
    except TimeoutError as err:
        raise HTTPException(
            status_code=504,
            detail=f"Agent timed out after {agent_timeout}s — check ANTHROPIC_API_KEY is set",
        ) from err

    usage = result.usage()
    total_tokens = (usage.request_tokens or 0) + (usage.response_tokens or 0)
    if total_tokens > 30_000:
        logger.warning("Token budget exceeded: %d tokens (threshold: 30,000)", total_tokens)
    logger.info(
        "Agent run: %d input + %d output tokens",
        usage.request_tokens or 0,
        usage.response_tokens or 0,
    )

    output = result.output
    if not output.download_url:
        raise HTTPException(
            status_code=500, detail="Cover letter generation failed — no download URL"
        )

    return CoverLetterResponse(
        preview_html=output.html,
        download_url=output.download_url,
        summary=output.summary,
    )


@app.post("/api/v1/agent/cover-letter/stream", response_model=None)
async def cover_letter_stream(
    request: Request, body: CoverLetterRequest
) -> StreamingResponse | JSONResponse:
    """Stream cover letter generation with real-time agent status updates."""
    if not os.environ.get("ANTHROPIC_API_KEY"):
        return JSONResponse(
            status_code=503,
            content={"detail": "ANTHROPIC_API_KEY not configured"},
        )
    ip = _extract_client_ip(request)
    if not _check_rate_limit(ip):
        return JSONResponse(
            status_code=429,
            content={"detail": f"Rate limit exceeded ({AGENT_RATE_LIMIT}/day)"},
        )

    async def event_generator() -> AsyncGenerator[str]:
        try:
            # Pre-grounding phase
            event = AgentEvent(
                agent="preground", status="running", detail="Fetching resume data..."
            )
            yield f"event: agent\ndata: {event.model_dump_json()}\n\n"

            deps = await _build_deps(body.job_description)

            event = AgentEvent(agent="preground", status="completed", detail="Context loaded")
            yield f"event: agent\ndata: {event.model_dump_json()}\n\n"

            # Agent execution phase
            event = AgentEvent(
                agent="cover_letter_writer", status="running", detail="Generating cover letter..."
            )
            yield f"event: agent\ndata: {event.model_dump_json()}\n\n"

            agent_timeout = int(os.environ.get("AGENT_TIMEOUT", "120"))
            result = await asyncio.wait_for(
                get_agent().run(
                    "Generate a tailored cover letter, convert to PDF, and provide a download link."
                    f"\n\nJob Description:\n{body.job_description}",
                    deps=deps,
                ),
                timeout=agent_timeout,
            )

            output = result.output

            # Report tool calls from the agent run
            event = AgentEvent(
                agent="cover_letter_writer", status="completed", detail="Cover letter generated"
            )
            yield f"event: agent\ndata: {event.model_dump_json()}\n\n"

            event = AgentEvent(
                agent="pdf_uploader", status="completed", detail="PDF uploaded to S3"
            )
            yield f"event: agent\ndata: {event.model_dump_json()}\n\n"

            event = AgentEvent(
                agent="link_generator",
                status="completed",
                detail="Download link ready (valid for 30 days)",
            )
            yield f"event: agent\ndata: {event.model_dump_json()}\n\n"

            # Final result
            final_result: dict[str, Any] = {
                "download_url": output.download_url,
                "preview_html": output.html,
                "summary": output.summary,
            }
            yield f"event: result\ndata: {json.dumps(final_result)}\n\n"
            yield "event: done\ndata: {}\n\n"

        except TimeoutError:
            logger.error(
                "Agent timed out after %s seconds",
                os.environ.get("AGENT_TIMEOUT", "120"),
            )
            event = AgentEvent(
                agent="cover_letter_writer", status="failed", detail="Agent timed out"
            )
            yield f"event: agent\ndata: {event.model_dump_json()}\n\n"
            msg = json.dumps({"message": "Agent timed out — check ANTHROPIC_API_KEY is set"})
            yield f"event: error\ndata: {msg}\n\n"
        except Exception as exc:
            logger.exception("Agent stream error")
            event = AgentEvent(agent="cover_letter_writer", status="failed", detail=str(exc)[:200])
            yield f"event: agent\ndata: {event.model_dump_json()}\n\n"
            error_event = {"message": str(exc)}
            yield f"event: error\ndata: {json.dumps(error_event)}\n\n"

    return StreamingResponse(
        event_generator(),
        media_type="text/event-stream",
        headers={"Cache-Control": "no-cache", "Connection": "keep-alive"},
    )


handler = Mangum(app, lifespan="off")

if __name__ == "__main__":
    import uvicorn

    uvicorn.run("handler:app", host="0.0.0.0", port=3003, reload=True)
