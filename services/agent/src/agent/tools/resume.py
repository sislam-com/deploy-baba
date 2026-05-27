"""Resume data retrieval tool — invokes the portfolio Rust Lambda."""

from __future__ import annotations

import json
import os
from typing import Any

import boto3
from langchain_core.tools import tool


def _invoke_ui_lambda(method: str, path: str, body: str | None = None) -> dict[str, Any]:
    """Invoke the UI Lambda via AWS Lambda SDK."""
    client = boto3.client("lambda", region_name=os.environ.get("AWS_REGION", "us-east-1"))
    payload = {
        "method": method,
        "path": path,
        "headers": {"content-type": "application/json"},
        "query": {},
        "body": body,
        "auth_context": None,
    }
    response = client.invoke(
        FunctionName=os.environ["UI_LAMBDA_NAME"],
        InvocationType="RequestResponse",
        Payload=json.dumps(payload).encode(),
    )
    result = json.loads(response["Payload"].read())
    if result.get("status_code", 200) >= 400:
        raise RuntimeError(f"UI Lambda returned {result.get('status_code')}: {result.get('body')}")
    parsed: dict[str, Any] = json.loads(result.get("body", "{}"))
    return parsed


@tool
def retrieve_resume_data() -> str:
    """Retrieve resume data (jobs, competencies, tech stack) from the portfolio database.

    Returns JSON with the candidate's work history, skills, and competency evidence.
    Use this data as grounding context for cover letter generation.
    """
    data = _invoke_ui_lambda("GET", "/api/v1/resume")
    return json.dumps(data, indent=2)
