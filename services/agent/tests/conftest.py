"""Shared test fixtures for the agent service."""

from __future__ import annotations

import pytest


@pytest.fixture(autouse=True)
def _mock_env(monkeypatch: pytest.MonkeyPatch) -> None:
    """Set required environment variables for tests."""
    monkeypatch.setenv("AWS_REGION", "us-east-1")
    monkeypatch.setenv("UI_LAMBDA_NAME", "deploy-baba-dev-ui")
    monkeypatch.setenv("ARTIFACTS_BUCKET", "deploy-baba-artifacts-dev")
    monkeypatch.setenv("ANTHROPIC_API_KEY", "test-key-not-real")


SAMPLE_JD = """\
Senior Software Engineer — Platform Team

We're looking for an experienced software engineer to join our platform team.
You'll design and build scalable backend services, work with cloud infrastructure (AWS),
and mentor junior engineers. Experience with Rust, Python, and distributed systems preferred.
Strong communication skills and ability to work cross-functionally required.

Requirements:
- 5+ years of software engineering experience
- Experience with AWS (Lambda, ECS, S3, DynamoDB)
- Proficiency in at least two of: Rust, Python, TypeScript
- Experience with CI/CD pipelines and infrastructure as code
- Strong problem-solving and system design skills
"""
