"""S3 artifact storage tool — upload cover letter and generate presigned URL."""

from __future__ import annotations

import hashlib
import json
import os
from datetime import date

import boto3
from langchain_core.tools import tool


@tool
def save_artifact(cover_letter_html: str) -> str:
    """Save the generated cover letter to S3 and return a download URL.

    Uploads the HTML cover letter to S3 and generates a presigned download URL
    valid for 30 days.

    Args:
        cover_letter_html: The generated cover letter as HTML content.
    """
    bucket = os.environ["ARTIFACTS_BUCKET"]
    today = date.today().isoformat()
    content_hash = hashlib.sha256(cover_letter_html.encode()).hexdigest()[:12]
    key = f"cover-letters/{today}/{content_hash}.html"

    s3 = boto3.client("s3", region_name=os.environ.get("AWS_REGION", "us-east-1"))
    s3.put_object(
        Bucket=bucket,
        Key=key,
        Body=cover_letter_html.encode(),
        ContentType="text/html",
    )

    url = s3.generate_presigned_url(
        "get_object",
        Params={"Bucket": bucket, "Key": key},
        ExpiresIn=30 * 24 * 3600,
    )

    return json.dumps({"download_url": url, "s3_key": key})
