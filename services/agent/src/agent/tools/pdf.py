"""PDF conversion tool — invokes the standalone PDF Lambda."""

from __future__ import annotations

import json
import os

import boto3
from langchain_core.tools import tool


@tool
def convert_to_pdf(cover_letter_html: str) -> str:
    """Convert an HTML cover letter to PDF format.

    Invokes the PDF Lambda service to render HTML as a properly formatted PDF document.

    Args:
        cover_letter_html: The generated cover letter as HTML content.
    """
    function_name = os.environ["PDF_LAMBDA_NAME"]
    client = boto3.client("lambda", region_name=os.environ.get("AWS_REGION", "us-east-1"))

    # Wrap payload in API Gateway REST proxy event format for Mangum routing
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
        "body": json.dumps({"html": cover_letter_html}),
        "isBase64Encoded": False,
        "requestContext": {
            "resourcePath": "/convert",
            "httpMethod": "POST",
            "path": "/convert",
            "stage": "prod",
            "requestId": "local-invoke",
            "identity": {"sourceIp": "127.0.0.1"},
        },
    }

    response = client.invoke(
        FunctionName=function_name,
        InvocationType="RequestResponse",
        Payload=json.dumps(event).encode(),
    )

    result = json.loads(response["Payload"].read())

    if response.get("FunctionError"):
        raise RuntimeError(f"PDF Lambda error: {result}")

    # Mangum returns API Gateway response format
    if result.get("statusCode", 200) >= 400:
        raise RuntimeError(f"PDF Lambda HTTP {result.get('statusCode')}: {result.get('body')}")

    body = json.loads(result.get("body", "{}"))
    pdf_base64 = body.get("pdf_base64", "")

    if not pdf_base64:
        raise RuntimeError("PDF Lambda returned empty result")

    return json.dumps({"pdf_base64": pdf_base64})
