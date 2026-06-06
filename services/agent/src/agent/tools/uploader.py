"""S3 upload tool — uploads PDF bytes to the artifacts bucket."""

from __future__ import annotations

import base64
import hashlib
import json
import os
from datetime import date

import boto3
from langchain_core.tools import tool


@tool
def upload_to_s3(pdf_base64: str) -> str:
    """Upload a PDF cover letter to S3 for storage.

    Decodes base64 PDF bytes and uploads to the artifacts bucket under the
    cover-letters/ prefix with a content-hash key.

    Args:
        pdf_base64: Base64-encoded PDF bytes from the convert_to_pdf step.
    """
    pdf_bytes = base64.b64decode(pdf_base64)

    bucket = os.environ["ARTIFACTS_BUCKET"]
    today = date.today().isoformat()
    content_hash = hashlib.sha256(pdf_bytes).hexdigest()[:12]
    key = f"cover-letters/{today}/{content_hash}.pdf"

    s3 = boto3.client("s3", region_name=os.environ.get("AWS_REGION", "us-east-1"))
    s3.put_object(
        Bucket=bucket,
        Key=key,
        Body=pdf_bytes,
        ContentType="application/pdf",
    )

    return json.dumps({"s3_key": key, "bucket": bucket})
