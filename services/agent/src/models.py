"""Pydantic models for the agent service API."""

from __future__ import annotations

from pydantic import BaseModel, Field


class CoverLetterRequest(BaseModel):
    """Request to generate a tailored cover letter."""

    job_description: str = Field(
        ...,
        min_length=50,
        max_length=10000,
        description="The full job description text",
    )


class CoverLetterResponse(BaseModel):
    """Response containing the generated cover letter."""

    preview_html: str = Field(description="HTML content of the cover letter for inline preview")
    download_url: str = Field(description="S3 presigned URL for downloading the cover letter")
    summary: str = Field(description="Brief summary of how the cover letter was tailored")
