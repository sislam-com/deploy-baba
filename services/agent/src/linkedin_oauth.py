"""LinkedIn OAuth2 Authorization Code flow endpoints.

The admin initiates login from the dashboard; LinkedIn redirects back with an
auth code; we exchange it server-side for an access token, validate with
/v2/userinfo, and redirect the browser back to the dashboard.
"""

from __future__ import annotations

import json
import os
import secrets
import time

import httpx
from fastapi import APIRouter, HTTPException, Query
from fastapi.responses import RedirectResponse

from models import LinkedInAuthUrl, LinkedInConnectionStatus

router = APIRouter(prefix="/api/v1/agent/linkedin", tags=["linkedin-oauth"])

LINKEDIN_AUTH_URL = "https://www.linkedin.com/oauth/v2/authorization"
LINKEDIN_TOKEN_URL = "https://www.linkedin.com/oauth/v2/accessToken"
LINKEDIN_USERINFO_URL = "https://api.linkedin.com/v2/userinfo"

OAUTH_SCOPES = "openid profile email"

_client_id: str = ""
_client_secret: str = ""

# In-memory state store: {state_value: expiry_timestamp}
_pending_states: dict[str, float] = {}
STATE_TTL_SECONDS = 600

# Last successful connection (in-memory; lost on cold start — acceptable for v1)
_connection: LinkedInConnectionStatus = LinkedInConnectionStatus(connected=False)


def load_linkedin_credentials() -> None:
    """Load LinkedIn OAuth client_id/client_secret from env or Secrets Manager."""
    global _client_id, _client_secret

    _client_id = os.environ.get("LINKEDIN_CLIENT_ID", "")
    _client_secret = os.environ.get("LINKEDIN_CLIENT_SECRET", "")
    if _client_id and _client_secret:
        return

    arn = os.environ.get("LINKEDIN_SECRET_ARN")
    if not arn:
        return
    import boto3

    client = boto3.client("secretsmanager", region_name=os.environ.get("AWS_REGION", "us-east-1"))
    secret = client.get_secret_value(SecretId=arn)
    data = json.loads(secret["SecretString"])
    _client_id = data.get("client_id", "")
    _client_secret = data.get("client_secret", "")


def _get_redirect_uri() -> str:
    base = os.environ.get("LINKEDIN_REDIRECT_BASE", "http://localhost:3003")
    return f"{base}/api/v1/agent/linkedin/callback"


def _prune_expired_states() -> None:
    now = time.time()
    expired = [k for k, v in _pending_states.items() if v < now]
    for k in expired:
        del _pending_states[k]


@router.get("/auth-url", response_model=LinkedInAuthUrl)
async def get_auth_url() -> LinkedInAuthUrl:
    if not _client_id:
        raise HTTPException(status_code=503, detail="LinkedIn OAuth not configured")

    _prune_expired_states()

    state = secrets.token_urlsafe(32)
    _pending_states[state] = time.time() + STATE_TTL_SECONDS

    params = {
        "response_type": "code",
        "client_id": _client_id,
        "redirect_uri": _get_redirect_uri(),
        "state": state,
        "scope": OAUTH_SCOPES,
    }
    url = f"{LINKEDIN_AUTH_URL}?{'&'.join(f'{k}={v}' for k, v in params.items())}"
    return LinkedInAuthUrl(url=url, state=state)


@router.get("/callback")
async def oauth_callback(
    code: str = Query(...),
    state: str = Query(...),
) -> RedirectResponse:
    _prune_expired_states()

    if state not in _pending_states:
        raise HTTPException(status_code=400, detail="Invalid or expired state parameter")
    del _pending_states[state]

    if not _client_id or not _client_secret:
        raise HTTPException(status_code=503, detail="LinkedIn OAuth not configured")

    async with httpx.AsyncClient(timeout=15.0) as client:
        token_resp = await client.post(
            LINKEDIN_TOKEN_URL,
            data={
                "grant_type": "authorization_code",
                "code": code,
                "redirect_uri": _get_redirect_uri(),
                "client_id": _client_id,
                "client_secret": _client_secret,
            },
            headers={"Content-Type": "application/x-www-form-urlencoded"},
        )

        if token_resp.status_code != 200:
            raise HTTPException(
                status_code=502,
                detail=f"LinkedIn token exchange failed: {token_resp.text}",
            )

        token_data = token_resp.json()
        access_token = token_data.get("access_token", "")
        expires_in = token_data.get("expires_in", 0)

        if not access_token:
            raise HTTPException(status_code=502, detail="No access token in LinkedIn response")

        profile_resp = await client.get(
            LINKEDIN_USERINFO_URL,
            headers={"Authorization": f"Bearer {access_token}"},
        )

    global _connection
    if profile_resp.status_code == 200:
        profile = profile_resp.json()
        expires_at = str(int(time.time()) + expires_in) if expires_in else None
        _connection = LinkedInConnectionStatus(
            connected=True,
            name=profile.get("name"),
            email=profile.get("email"),
            picture_url=profile.get("picture"),
            token_expires_at=expires_at,
        )
    else:
        expires_at = str(int(time.time()) + expires_in) if expires_in else None
        _connection = LinkedInConnectionStatus(
            connected=True,
            token_expires_at=expires_at,
        )

    dashboard_base = os.environ.get("DASHBOARD_URL", "http://localhost:5173")
    return RedirectResponse(url=f"{dashboard_base}/dashboard/linkedin?connected=true")


@router.get("/status", response_model=LinkedInConnectionStatus)
async def connection_status() -> LinkedInConnectionStatus:
    if _connection.connected and _connection.token_expires_at:
        try:
            if int(_connection.token_expires_at) < int(time.time()):
                return LinkedInConnectionStatus(connected=False)
        except ValueError:
            pass
    return _connection


@router.post("/disconnect")
async def disconnect() -> dict[str, str]:
    global _connection
    _connection = LinkedInConnectionStatus(connected=False)
    return {"status": "disconnected"}
