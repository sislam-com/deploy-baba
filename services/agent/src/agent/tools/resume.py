"""Resume data retrieval tool — invokes the portfolio Rust Lambda."""

from __future__ import annotations

import json
import os
from typing import Any

import boto3
from langchain_core.tools import tool


def _invoke_ui_lambda(method: str, path: str, body: str | None = None) -> dict[str, Any]:
    """Invoke the UI Lambda via AWS Lambda SDK using Function URL event format."""
    client = boto3.client("lambda", region_name=os.environ.get("AWS_REGION", "us-east-1"))

    # Wrap in Lambda Function URL / API GW v2 HTTP event format for Rust handler routing
    event = {
        "version": "2.0",
        "routeKey": "$default",
        "rawPath": path,
        "rawQueryString": "",
        "headers": {"content-type": "application/json"},
        "queryStringParameters": {},
        "requestContext": {
            "http": {"method": method, "path": path, "protocol": "HTTP/1.1"},
            "requestId": "local-invoke",
        },
        "body": body,
        "isBase64Encoded": False,
    }

    response = client.invoke(
        FunctionName=os.environ["UI_LAMBDA_NAME"],
        InvocationType="RequestResponse",
        Payload=json.dumps(event).encode(),
    )
    result = json.loads(response["Payload"].read())
    if result.get("statusCode", 200) >= 400:
        raise RuntimeError(
            f"UI Lambda returned HTTP {result.get('statusCode')}: {result.get('body')}"
        )
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
