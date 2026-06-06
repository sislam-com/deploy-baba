"""LinkedIn OAuth2 Authorization Code flow endpoints.

The admin initiates login from the dashboard; LinkedIn redirects back with an
auth code; we exchange it server-side for an access token, validate with
/v2/userinfo, and redirect the browser back to the dashboard.

Tokens are persisted to the UI service's SQLite DB so they survive cold starts.
"""

from __future__ import annotations

import json
import logging
import os
import secrets
import time

import httpx
from fastapi import APIRouter, HTTPException, Query
from fastapi.responses import RedirectResponse

from models import LinkedInAuthUrl, LinkedInConnectionStatus

logger = logging.getLogger(__name__)

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

_connection: LinkedInConnectionStatus = LinkedInConnectionStatus(connected=False)


def _ui_base() -> str:
    return os.environ.get("UI_SERVICE_URL", "http://localhost:3001")


async def _persist_token(
    access_token: str, expires_at: int, name: str | None, email: str | None, picture_url: str | None,
) -> None:
    """Store the OAuth token in the UI service's DB for cold-start recovery."""
    try:
        async with httpx.AsyncClient(timeout=5.0) as client:
            await client.put(
                f"{_ui_base()}/api/v1/admin/linkedin/oauth-token",
                json={
                    "access_token": access_token,
                    "expires_at": expires_at,
                    "name": name,
                    "email": email,
                    "picture_url": picture_url,
                },
            )
    except Exception:
        logger.warning("Failed to persist LinkedIn token to UI service", exc_info=True)


async def _delete_persisted_token() -> None:
    """Clear the persisted OAuth token from the UI service's DB."""
    try:
        async with httpx.AsyncClient(timeout=5.0) as client:
            await client.delete(f"{_ui_base()}/api/v1/admin/linkedin/oauth-token")
    except Exception:
        logger.warning("Failed to delete LinkedIn token from UI service", exc_info=True)


async def _restore_token() -> None:
    """On startup, restore connection state from the UI service's persisted token."""
    global _connection
    try:
        async with httpx.AsyncClient(timeout=5.0) as client:
            resp = await client.get(f"{_ui_base()}/api/v1/admin/linkedin/oauth-token")
            if resp.status_code == 200:
                data = resp.json()
                if data.get("connected"):
                    _connection = LinkedInConnectionStatus(
                        connected=True,
                        name=data.get("name"),
                        email=data.get("email"),
                        picture_url=data.get("picture_url"),
                        token_expires_at=data.get("token_expires_at"),
                    )
                    logger.info("Restored LinkedIn connection for %s", data.get("name"))
    except Exception:
        logger.info("No persisted LinkedIn token found (UI service not reachable)")


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
    try:
        import boto3

        client = boto3.client("secretsmanager", region_name=os.environ.get("AWS_REGION", "us-east-1"))
        secret = client.get_secret_value(SecretId=arn)
        data = json.loads(secret["SecretString"])
        _client_id = data.get("client_id", "")
        _client_secret = data.get("client_secret", "")
    except (json.JSONDecodeError, KeyError, Exception) as e:
        logger.warning("Failed to load LinkedIn credentials from %s: %s", arn, e)


def _get_redirect_uri() -> str:
    base = os.environ.get("LINKEDIN_REDIRECT_BASE", "http://localhost:3000")
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
    expires_at = int(time.time()) + expires_in if expires_in else 0
    expires_at_str = str(expires_at) if expires_at else None

    if profile_resp.status_code == 200:
        profile = profile_resp.json()
        name = profile.get("name")
        email = profile.get("email")
        picture_url = profile.get("picture")
        _connection = LinkedInConnectionStatus(
            connected=True,
            name=name,
            email=email,
            picture_url=picture_url,
            token_expires_at=expires_at_str,
        )
    else:
        name = None
        email = None
        picture_url = None
        _connection = LinkedInConnectionStatus(
            connected=True,
            token_expires_at=expires_at_str,
        )

    await _persist_token(access_token, expires_at, name, email, picture_url)

    dashboard_base = os.environ.get("DASHBOARD_URL", "http://localhost:3000")
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
    await _delete_persisted_token()
    return {"status": "disconnected"}
