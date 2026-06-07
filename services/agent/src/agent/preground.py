"""Pre-grounding: fetch resume + keyword match before the agent starts (zero LLM tokens).

Dual-mode (ADR-004):
  - Local dev:  UI_BASE_URL is set (e.g. http://localhost:3001) → direct HTTP via httpx.
  - Lambda:     UI_LAMBDA_NAME is set → boto3 Lambda invoke with synthetic API Gateway event.
"""

from __future__ import annotations

import json
import logging
import os
from typing import Any, cast

import httpx

logger = logging.getLogger(__name__)


def _ui_base_url() -> str | None:
    return os.environ.get("UI_BASE_URL")


def _lambda_client() -> Any:
    import boto3

    return boto3.client("lambda", region_name=os.environ.get("AWS_REGION", "us-east-1"))


async def _http_get(path: str) -> dict[str, Any]:
    base = _ui_base_url()
    async with httpx.AsyncClient(timeout=10) as client:
        resp = await client.get(f"{base}{path}")
        resp.raise_for_status()
        result: dict[str, Any] = resp.json()
        return result


async def _http_post(path: str, body: dict[str, Any]) -> Any:
    base = _ui_base_url()
    async with httpx.AsyncClient(timeout=10) as client:
        resp = await client.post(f"{base}{path}", json=body)
        resp.raise_for_status()
        return resp.json()


def _lambda_invoke(fn: str, event: dict[str, Any]) -> dict[str, Any]:
    client = _lambda_client()
    response = client.invoke(
        FunctionName=fn, InvocationType="RequestResponse", Payload=json.dumps(event).encode()
    )
    result: dict[str, Any] = json.loads(response["Payload"].read())
    if result.get("statusCode", 200) >= 400:
        raise RuntimeError(
            f"UI Lambda returned HTTP {result.get('statusCode')}: {result.get('body')}"
        )
    return result


async def fetch_resume(lambda_name: str | None = None) -> dict[str, Any]:
    """Retrieve resume data from the UI service. Returns parsed JSON."""
    if _ui_base_url():
        return await _http_get("/api/v1/resume")

    fn = lambda_name or os.environ["UI_LAMBDA_NAME"]
    event = {
        "version": "2.0",
        "routeKey": "$default",
        "rawPath": "/api/v1/resume",
        "rawQueryString": "",
        "headers": {"content-type": "application/json"},
        "queryStringParameters": {},
        "requestContext": {
            "http": {"method": "GET", "path": "/api/v1/resume", "protocol": "HTTP/1.1"},
            "requestId": "preground-resume",
        },
        "body": None,
        "isBase64Encoded": False,
    }
    result = _lambda_invoke(fn, event)
    parsed: dict[str, Any] = json.loads(result.get("body", "{}"))
    return parsed


async def match_keywords(
    job_description: str, lambda_name: str | None = None
) -> list[dict[str, Any]]:
    """Match JD keywords against resume via the UI service's tailor endpoint.

    Returns empty list if the endpoint is not yet implemented (W-RST pending).
    """
    if _ui_base_url():
        try:
            body = await _http_post("/api/v1/tailor/match", {"job_description": job_description})
        except httpx.HTTPStatusError as exc:
            logger.warning(
                "tailor/match unavailable (%s) — skipping keyword matching",
                exc.response.status_code,
            )
            return []
        if isinstance(body, dict):
            return cast(list[dict[str, Any]], body.get("matches", body.get("bullets", [])))
        return body if isinstance(body, list) else []

    fn = lambda_name or os.environ["UI_LAMBDA_NAME"]
    event = {
        "version": "2.0",
        "routeKey": "$default",
        "rawPath": "/api/v1/tailor/match",
        "rawQueryString": "",
        "headers": {"content-type": "application/json"},
        "queryStringParameters": {},
        "requestContext": {
            "http": {"method": "POST", "path": "/api/v1/tailor/match", "protocol": "HTTP/1.1"},
            "requestId": "preground-match",
        },
        "body": json.dumps({"job_description": job_description}),
        "isBase64Encoded": False,
    }
    try:
        result = _lambda_invoke(fn, event)
    except RuntimeError as exc:
        logger.warning("tailor/match unavailable via Lambda invoke (%s) — skipping", exc)
        return []
    body = json.loads(result.get("body", "[]"))
    if isinstance(body, dict):
        return cast(list[dict[str, Any]], body.get("matches", body.get("bullets", [])))
    matched: list[dict[str, Any]] = body if isinstance(body, list) else []
    return matched
