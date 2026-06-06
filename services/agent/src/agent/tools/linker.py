"""Presigned URL generation tool — creates a download link for an S3 object."""

from __future__ import annotations

import json
import os

import boto3
from langchain_core.tools import tool


@tool
def generate_presigned_url(s3_key: str, bucket: str) -> str:
    """Generate a presigned download URL for a cover letter PDF stored in S3.

    Creates a URL valid for 30 days that allows direct PDF download without
    authentication.

    Args:
        s3_key: The S3 object key (e.g. "cover-letters/2026-06-04/abc123.pdf").
        bucket: The S3 bucket name.
    """
    s3 = boto3.client("s3", region_name=os.environ.get("AWS_REGION", "us-east-1"))

    url = s3.generate_presigned_url(
        "get_object",
        Params={"Bucket": bucket, "Key": s3_key},
        ExpiresIn=30 * 24 * 3600,
    )

    return json.dumps({"download_url": url, "s3_key": s3_key, "expires_in_days": 30})
