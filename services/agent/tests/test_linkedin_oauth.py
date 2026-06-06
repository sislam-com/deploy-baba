"""Tests for the LinkedIn OAuth2 flow endpoints."""

from __future__ import annotations

import time
from unittest.mock import AsyncMock, patch

import httpx
import pytest
from fastapi.testclient import TestClient

import linkedin_oauth
from linkedin_oauth import (
    _pending_states,
    _prune_expired_states,
    load_linkedin_credentials,
    router,
)
from models import LinkedInConnectionStatus


@pytest.fixture(autouse=True)
def _reset_module_state(monkeypatch: pytest.MonkeyPatch) -> None:
    """Reset module-level state between tests."""
    linkedin_oauth._client_id = ""
    linkedin_oauth._client_secret = ""
    linkedin_oauth._connection = LinkedInConnectionStatus(connected=False)
    _pending_states.clear()
    monkeypatch.setenv("LINKEDIN_CLIENT_ID", "test-client-id")
    monkeypatch.setenv("LINKEDIN_CLIENT_SECRET", "test-client-secret")
    monkeypatch.setenv("LINKEDIN_REDIRECT_BASE", "http://localhost:3000")
    monkeypatch.setenv("DASHBOARD_URL", "http://localhost:3000")
    load_linkedin_credentials()


@pytest.fixture()
def client() -> TestClient:
    from fastapi import FastAPI

    app = FastAPI()
    app.include_router(router)
    return TestClient(app)


# ── load_linkedin_credentials ──────────────────────────────────────────────


def test_load_from_env(monkeypatch: pytest.MonkeyPatch) -> None:
    linkedin_oauth._client_id = ""
    linkedin_oauth._client_secret = ""
    monkeypatch.setenv("LINKEDIN_CLIENT_ID", "env-id")
    monkeypatch.setenv("LINKEDIN_CLIENT_SECRET", "env-secret")
    load_linkedin_credentials()
    assert linkedin_oauth._client_id == "env-id"
    assert linkedin_oauth._client_secret == "env-secret"


def test_load_skips_sm_when_env_set(monkeypatch: pytest.MonkeyPatch) -> None:
    """Should not call Secrets Manager when env vars are present."""
    linkedin_oauth._client_id = ""
    linkedin_oauth._client_secret = ""
    monkeypatch.setenv("LINKEDIN_CLIENT_ID", "id")
    monkeypatch.setenv("LINKEDIN_CLIENT_SECRET", "secret")
    monkeypatch.setenv("LINKEDIN_SECRET_ARN", "arn:aws:secretsmanager:us-east-1:123:secret/test")
    with patch.dict("sys.modules", {"boto3": AsyncMock()}) as _:
        load_linkedin_credentials()
    assert linkedin_oauth._client_id == "id"
    assert linkedin_oauth._client_secret == "secret"


def test_load_no_config(monkeypatch: pytest.MonkeyPatch) -> None:
    linkedin_oauth._client_id = ""
    linkedin_oauth._client_secret = ""
    monkeypatch.delenv("LINKEDIN_CLIENT_ID", raising=False)
    monkeypatch.delenv("LINKEDIN_CLIENT_SECRET", raising=False)
    monkeypatch.delenv("LINKEDIN_SECRET_ARN", raising=False)
    load_linkedin_credentials()
    assert linkedin_oauth._client_id == ""
    assert linkedin_oauth._client_secret == ""


# ── State management ───────────────────────────────────────────────────────


def test_prune_expired_states() -> None:
    _pending_states["fresh"] = time.time() + 600
    _pending_states["stale"] = time.time() - 1
    _prune_expired_states()
    assert "fresh" in _pending_states
    assert "stale" not in _pending_states


# ── GET /auth-url ──────────────────────────────────────────────────────────


def test_auth_url_returns_linkedin_url(client: TestClient) -> None:
    resp = client.get("/api/v1/agent/linkedin/auth-url")
    assert resp.status_code == 200
    data = resp.json()
    assert "url" in data
    assert "state" in data
    assert data["url"].startswith("https://www.linkedin.com/oauth/v2/authorization")
    assert "client_id=test-client-id" in data["url"]
    assert "response_type=code" in data["url"]
    assert "scope=openid" in data["url"]


def test_auth_url_registers_state(client: TestClient) -> None:
    resp = client.get("/api/v1/agent/linkedin/auth-url")
    state = resp.json()["state"]
    assert state in _pending_states


def test_auth_url_503_when_not_configured(
    client: TestClient, monkeypatch: pytest.MonkeyPatch
) -> None:
    linkedin_oauth._client_id = ""
    resp = client.get("/api/v1/agent/linkedin/auth-url")
    assert resp.status_code == 503


# ── GET /callback ──────────────────────────────────────────────────────────


def test_callback_invalid_state(client: TestClient) -> None:
    resp = client.get(
        "/api/v1/agent/linkedin/callback",
        params={"code": "auth-code", "state": "bad-state"},
        follow_redirects=False,
    )
    assert resp.status_code == 400


def test_callback_success(client: TestClient) -> None:
    valid_state = "test-state-abc"
    _pending_states[valid_state] = time.time() + 600

    token_response = httpx.Response(
        200,
        json={"access_token": "li-token-123", "expires_in": 5184000},
    )
    profile_response = httpx.Response(
        200,
        json={
            "name": "Test User",
            "email": "test@example.com",
            "picture": "https://example.com/photo.jpg",
        },
    )

    with patch("linkedin_oauth.httpx.AsyncClient") as mock_client_cls:
        mock_client = AsyncMock()
        mock_client.post = AsyncMock(return_value=token_response)
        mock_client.get = AsyncMock(return_value=profile_response)
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client_cls.return_value = mock_client

        resp = client.get(
            "/api/v1/agent/linkedin/callback",
            params={"code": "auth-code-123", "state": valid_state},
            follow_redirects=False,
        )

    assert resp.status_code == 307
    assert "dashboard/linkedin?connected=true" in resp.headers["location"]
    assert valid_state not in _pending_states
    assert linkedin_oauth._connection.connected is True
    assert linkedin_oauth._connection.name == "Test User"
    assert linkedin_oauth._connection.email == "test@example.com"


def test_callback_token_exchange_failure(client: TestClient) -> None:
    valid_state = "test-state-fail"
    _pending_states[valid_state] = time.time() + 600

    token_response = httpx.Response(400, text="invalid_grant")

    with patch("linkedin_oauth.httpx.AsyncClient") as mock_client_cls:
        mock_client = AsyncMock()
        mock_client.post = AsyncMock(return_value=token_response)
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client_cls.return_value = mock_client

        resp = client.get(
            "/api/v1/agent/linkedin/callback",
            params={"code": "bad-code", "state": valid_state},
            follow_redirects=False,
        )

    assert resp.status_code == 502


def test_callback_no_access_token(client: TestClient) -> None:
    valid_state = "test-state-notoken"
    _pending_states[valid_state] = time.time() + 600

    token_response = httpx.Response(200, json={"expires_in": 3600})

    with patch("linkedin_oauth.httpx.AsyncClient") as mock_client_cls:
        mock_client = AsyncMock()
        mock_client.post = AsyncMock(return_value=token_response)
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client_cls.return_value = mock_client

        resp = client.get(
            "/api/v1/agent/linkedin/callback",
            params={"code": "code", "state": valid_state},
            follow_redirects=False,
        )

    assert resp.status_code == 502


def test_callback_profile_failure_still_connects(client: TestClient) -> None:
    """If userinfo fails, we still mark as connected (just without profile data)."""
    valid_state = "test-state-noprofile"
    _pending_states[valid_state] = time.time() + 600

    token_response = httpx.Response(200, json={"access_token": "token", "expires_in": 3600})
    profile_response = httpx.Response(401, text="Unauthorized")

    with patch("linkedin_oauth.httpx.AsyncClient") as mock_client_cls:
        mock_client = AsyncMock()
        mock_client.post = AsyncMock(return_value=token_response)
        mock_client.get = AsyncMock(return_value=profile_response)
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client_cls.return_value = mock_client

        resp = client.get(
            "/api/v1/agent/linkedin/callback",
            params={"code": "code", "state": valid_state},
            follow_redirects=False,
        )

    assert resp.status_code == 307
    assert linkedin_oauth._connection.connected is True
    assert linkedin_oauth._connection.name is None


# ── GET /status ────────────────────────────────────────────────────────────


def test_status_disconnected(client: TestClient) -> None:
    resp = client.get("/api/v1/agent/linkedin/status")
    assert resp.status_code == 200
    data = resp.json()
    assert data["connected"] is False


def test_status_connected(client: TestClient) -> None:
    linkedin_oauth._connection = LinkedInConnectionStatus(
        connected=True,
        name="Shanto",
        email="test@test.com",
        token_expires_at=str(int(time.time()) + 3600),
    )
    resp = client.get("/api/v1/agent/linkedin/status")
    data = resp.json()
    assert data["connected"] is True
    assert data["name"] == "Shanto"


def test_status_expired_token(client: TestClient) -> None:
    linkedin_oauth._connection = LinkedInConnectionStatus(
        connected=True,
        name="Shanto",
        token_expires_at=str(int(time.time()) - 100),
    )
    resp = client.get("/api/v1/agent/linkedin/status")
    data = resp.json()
    assert data["connected"] is False


# ── POST /disconnect ───────────────────────────────────────────────────────


def test_disconnect(client: TestClient) -> None:
    linkedin_oauth._connection = LinkedInConnectionStatus(connected=True, name="Shanto")
    resp = client.post("/api/v1/agent/linkedin/disconnect")
    assert resp.status_code == 200
    assert resp.json()["status"] == "disconnected"
    assert linkedin_oauth._connection.connected is False
