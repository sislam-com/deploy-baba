"""PydanticAI cover letter agent with pre-grounded context and cost guardrails."""

from __future__ import annotations

import asyncio
import base64
import hashlib
import json
import os
from dataclasses import dataclass
from datetime import date
from typing import Any

import boto3
import botocore.config
from pydantic import BaseModel, Field
from pydantic_ai import Agent, RunContext

COVER_LETTER_SYSTEM_PROMPT = """\
You are a professional cover letter writer. You have tools to generate HTML, \
convert it to PDF, and upload it to S3 with a download link.

Your workflow:
1. Review the pre-grounded resume data and matched bullets in your context.
2. Call generate_html to produce a tailored cover letter.
3. Call convert_to_pdf with the HTML to get a PDF.
4. Call upload_and_link with the PDF to get a download URL.
5. Return the final structured output.

GROUNDING RULES (non-negotiable):
1. Only reference skills, roles, and achievements present in the resume data provided.
2. Never invent experience, certifications, or skills not in the source material.
3. You may rephrase and reorder, but every claim must trace back to a specific resume bullet.
4. Match the tone to the target company — formal for enterprise, conversational for startups.
5. Keep the cover letter under 400 words."""


class CoverLetterOutput(BaseModel):
    """Structured output from the cover letter agent."""

    html: str = Field(description="Cover letter as semantic HTML (<p>, <strong>, <em>)")
    download_url: str = Field(description="S3 presigned URL for PDF download")
    summary: str = Field(description="1-2 sentence summary of how the letter was tailored")
    grounding_citations: list[str] = Field(
        default_factory=list,
        description="Resume bullets that were referenced in the cover letter",
    )


@dataclass
class AgentDeps:
    """Pre-grounded context injected before the agent starts."""

    resume_summary: str
    matched_bullets: list[dict[str, Any]]
    job_description: str
    pdf_lambda_name: str
    artifacts_bucket: str


_agent: Agent[AgentDeps, CoverLetterOutput] | None = None


def _get_model_name() -> str:
    return os.environ.get("AGENT_MODEL", "anthropic:claude-haiku-4-5-20251001")


def get_agent() -> Agent[AgentDeps, CoverLetterOutput]:
    """Lazy-init the agent so it doesn't require ANTHROPIC_API_KEY at import time."""
    global _agent
    if _agent is None:
        _agent = Agent(
            _get_model_name(),
            output_type=CoverLetterOutput,
            system_prompt=COVER_LETTER_SYSTEM_PROMPT,
            deps_type=AgentDeps,
            retries=2,
        )
        _register_tools(_agent)
    return _agent


def _register_tools(agent: Agent[AgentDeps, CoverLetterOutput]) -> None:
    """Register tools on the agent instance."""

    @agent.tool
    async def generate_html(ctx: RunContext[AgentDeps]) -> str:
        """Generate the cover letter as HTML using the pre-grounded resume data and matched bullets.

        Returns clean HTML with semantic tags. Do NOT include <html>/<head>/<body> wrappers.
        """
        bullets_section = ""
        if ctx.deps.matched_bullets:
            bullets_section = (
                f"\nTop Matched Bullets:\n{json.dumps(ctx.deps.matched_bullets, indent=2)}\n"
            )

        prompt = (
            f"Job Description:\n{ctx.deps.job_description}\n\n"
            f"Resume Data:\n{ctx.deps.resume_summary}\n"
            f"{bullets_section}\n"
            "Write the cover letter now. Output ONLY the HTML content, "
            "no markdown, no explanations."
        )
        return prompt

    @agent.tool
    async def convert_to_pdf(ctx: RunContext[AgentDeps], html: str) -> str:
        """Convert cover letter HTML to a PDF via the PDF Lambda service.

        Args:
            html: The generated cover letter HTML content.
        """
        pdf_service_url = os.environ.get("PDF_SERVICE_URL")
        if pdf_service_url:
            import httpx

            async with httpx.AsyncClient(timeout=30) as client:
                resp = await client.post(
                    f"{pdf_service_url}/convert", json={"html": html}
                )
                resp.raise_for_status()
                return resp.json()["pdf_base64"]

        if not ctx.deps.pdf_lambda_name:
            return base64.b64encode(html.encode()).decode()

        cfg = botocore.config.Config(
            connect_timeout=10, read_timeout=30, retries={"max_attempts": 1}
        )
        client = boto3.client(
            "lambda",
            region_name=os.environ.get("AWS_REGION", "us-east-1"),
            config=cfg,
        )

        event = {
            "resource": "/convert",
            "path": "/convert",
            "httpMethod": "POST",
            "headers": {"content-type": "application/json"},
            "multiValueHeaders": {},
            "queryStringParameters": {},
            "multiValueQueryStringParameters": {},
            "pathParameters": {},
            "stageVariables": None,
            "body": json.dumps({"html": html}),
            "isBase64Encoded": False,
            "requestContext": {
                "resourcePath": "/convert",
                "httpMethod": "POST",
                "path": "/convert",
                "stage": "prod",
                "requestId": "agent-pdf",
                "identity": {"sourceIp": "127.0.0.1"},
            },
        }

        payload = json.dumps(event).encode()
        response = await asyncio.to_thread(
            client.invoke,
            FunctionName=ctx.deps.pdf_lambda_name,
            InvocationType="RequestResponse",
            Payload=payload,
        )
        result = json.loads(response["Payload"].read())

        if response.get("FunctionError"):
            raise RuntimeError(f"PDF Lambda error: {result}")
        if result.get("statusCode", 200) >= 400:
            raise RuntimeError(f"PDF Lambda HTTP {result.get('statusCode')}: {result.get('body')}")

        body = json.loads(result.get("body", "{}"))
        pdf_base64: str = body.get("pdf_base64", "")
        if not pdf_base64:
            raise RuntimeError("PDF Lambda returned empty result")

        return pdf_base64

    @agent.tool
    async def upload_and_link(ctx: RunContext[AgentDeps], pdf_base64: str) -> str:
        """Upload the PDF to S3 and generate a presigned download URL valid for 30 days.

        Args:
            pdf_base64: Base64-encoded PDF bytes from convert_to_pdf.
        """
        if not ctx.deps.artifacts_bucket:
            return "#dev-mode-no-download"

        pdf_bytes = base64.b64decode(pdf_base64)
        bucket = ctx.deps.artifacts_bucket
        today = date.today().isoformat()
        content_hash = hashlib.sha256(pdf_bytes).hexdigest()[:12]
        key = f"cover-letters/{today}/{content_hash}.pdf"

        cfg = botocore.config.Config(
            connect_timeout=10, read_timeout=30, retries={"max_attempts": 1}
        )
        endpoint_url = os.environ.get("S3_ENDPOINT_URL")
        s3 = boto3.client(
            "s3",
            region_name=os.environ.get("AWS_REGION", "us-east-1"),
            config=cfg,
            **({"endpoint_url": endpoint_url} if endpoint_url else {}),
        )
        await asyncio.to_thread(
            s3.put_object, Bucket=bucket, Key=key, Body=pdf_bytes, ContentType="application/pdf"
        )

        url: str = s3.generate_presigned_url(
            "get_object", Params={"Bucket": bucket, "Key": key}, ExpiresIn=30 * 24 * 3600
        )
        return url
