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


class LinkedInAuthUrl(BaseModel):
    """LinkedIn OAuth2 authorization URL with CSRF state."""

    url: str = Field(description="Full LinkedIn authorization URL to redirect the browser to")
    state: str = Field(description="CSRF state parameter for callback validation")


class LinkedInConnectionStatus(BaseModel):
    """Current state of the LinkedIn OAuth2 connection."""

    connected: bool = Field(description="Whether a valid LinkedIn access token exists")
    name: str | None = Field(default=None, description="LinkedIn profile display name")
    email: str | None = Field(default=None, description="LinkedIn profile email")
    picture_url: str | None = Field(default=None, description="LinkedIn profile photo URL")
    token_expires_at: str | None = Field(
        default=None, description="Unix timestamp when the token expires"
    )
