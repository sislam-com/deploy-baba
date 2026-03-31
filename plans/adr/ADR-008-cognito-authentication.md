# ADR-008: Cognito Authentication for Admin Dashboard

**Date:** 2026-03-26
**Status:** Amended 2026-03-26 (W-AUTH.4.21 — implicit grant; JWKS from env)
**Affected Modules:** W-AUTH, W-UI, W-OTF

---

## Context

The portfolio at sislam.com is a fully public, unauthenticated site. All resume data lives in
SQLite on EFS. The sole admin (`baba-admin`) has no mechanism to edit resume artifacts without
SSH access or direct database manipulation.

The following options were considered:

1. **HTTP Basic Auth via Lambda Function URL** — simplest, but credentials travel in every
   request header, no logout, no session management. Rejected.

2. **Self-rolled JWT auth** — full control but requires building user management, password
   hashing, email verification, and token rotation from scratch. High maintenance burden.
   Rejected for a single-admin site.

3. **AWS Cognito hosted UI** — managed user pool, hosted login page, PKCE authorization
   code flow, standard OIDC tokens. Free tier: 10K MAUs/month (zero cost for one user).
   Minimal code: exchange code → validate JWT → set cookie. Selected.

4. **Third-party IdP (Auth0, Clerk)** — comparable UX but external dependency outside the AWS
   ecosystem. Unnecessary given existing AWS SSO familiarity. Not considered further.

---

## Decision

Use **AWS Cognito** with the hosted UI for authentication. A single user pool (`baba` pool)
contains one user (`baba-admin`). The web app uses the **implicit grant flow**:

```
Browser → GET /auth/login
        → 302 to Cognito hosted UI (response_type=token, scope=openid → returns id_token in fragment)
        → POST credentials to Cognito
        → 302 to GET /auth/callback#id_token=xxx  (fragment — not sent to server)
        → Lambda: return HTML page with inline JS
        → JS: extract id_token from window.location.hash
        → JS: POST {"id_token": "..."} to /auth/set-session
        → Lambda: validate JWT (RS256, JWKS from env var)
        → Set auth_token HttpOnly cookie; return 200
        → JS: redirect to /dashboard
```

**Amendment (W-AUTH.4.21, 2026-03-26):** Originally designed as PKCE authorization code flow.
Amended because Lambda runs in a VPC (for EFS) with no NAT Gateway — it cannot make outbound
HTTPS calls to either the Cognito token endpoint or the JWKS endpoint. The W-AUTH.4.20 lazy
JWKS fetch deferred the failure but didn't solve it. Switching to implicit grant eliminates all
outbound calls from Lambda.

**Session management:** HttpOnly/Secure/SameSite=Lax cookie (1h TTL, matches ID token).
No localStorage, no sessionStorage — XSS cannot steal the token.

**JWT validation:** App-layer RS256 validation using `jsonwebtoken` crate. JWKS is fetched
by OpenTofu at deploy time (`data "http" "cognito_jwks"`) and stored in the `COGNITO_JWKS`
Lambda env var. No network call at cold start or per-request.

**Lambda Function URL auth stays `NONE`** — Cognito operates at the application layer, not
the Lambda URL layer. CloudFront → Lambda URL authentication is unchanged (→ ADR-003).

---

## Consequences

**Positive:**
- Zero operational overhead — Cognito manages passwords, email verification, token rotation
- Free tier covers 10K MAU/month (single admin = ~0 cost)
- Standard OIDC flow — battle-tested browser compat, no custom login form to maintain
- HttpOnly cookie session is XSS-resistant; SameSite=Lax is CSRF-resistant
- JWKS embedded in env var: zero cold-start or per-request network overhead
- Dev-mode bypass (`COGNITO_POOL_ID` unset) keeps `just ui` working without AWS credentials
- No NAT Gateway required — Lambda stays in VPC for EFS access at zero extra cost

**Negative:**
- Additional AWS resource (Cognito user pool) adds complexity to `infra/cognito.tf`
- Implicit grant is less secure than PKCE code flow (token in URL fragment, though short-lived)
- JWKS keys embedded at deploy time — must re-deploy if Cognito rotates keys
- Admin must use Cognito hosted UI for password reset (no custom reset flow needed)

**Neutral:**
- `axum-extra` cookie extraction is the only new Axum dependency
- `FromRef` on `AppState` means zero changes to the 6+ existing handler files

---

## AppState Design — `FromRef` for Backward Compatibility

Rather than threading a new `AuthConfig` through every existing handler, `AppState` uses
Axum's `FromRef` derive:

```rust
#[derive(Clone, axum::extract::FromRef)]
pub struct AppState {
    pub db: Arc<Db>,
    pub auth: Arc<AuthConfig>,
}
```

Existing handlers continue to extract `State(db): State<Arc<Db>>` unchanged. Only new auth
and admin handlers extract `State<Arc<AuthConfig>>`. This is the idiomatic Axum pattern for
composing state from multiple independent components.

---

## Dev-Mode Bypass

When `COGNITO_POOL_ID` env var is absent, `AuthConfig::from_env()` returns a dev config with
`dev_mode: true`. The `require_auth` middleware skips token validation and injects a synthetic
`Claims { sub: "dev", email: "dev@localhost", username: "dev" }`.

This preserves the `just ui` inner loop (→ ADR-004) — the dashboard and all admin routes are
accessible locally without AWS credentials.

---

## Scope — Public Routes Unchanged

The public resume (`/`), API endpoints (`/api/jobs`, `/api/competencies`), and OpenAPI docs
remain fully unauthenticated. Authentication is additive — no existing routes are gated.

---

## Infrastructure

New resources in `infra/cognito.tf`:

| Resource | Notes |
|----------|-------|
| `aws_cognito_user_pool` | Password: min 12, upper+lower+num+sym; email verified; deletion protection; **self-sign-up disabled** |
| `aws_cognito_user_pool_domain` | `deploy-baba-prod.auth.us-east-1.amazoncognito.com` |
| `aws_cognito_user_pool_client` | Public implicit grant client; callbacks: prod + localhost:3000 |
| `aws_cognito_user` | `baba-admin`; temp password via sensitive variable; reset on first login |
| `data "http" "cognito_jwks"` | Fetches JWKS at deploy time; stored in `COGNITO_JWKS` Lambda env var |

Lambda env vars added to `infra/lambda.tf`:
`COGNITO_POOL_ID`, `COGNITO_CLIENT_ID`, `COGNITO_DOMAIN`, `COGNITO_REGION`, `APP_DOMAIN`, `COGNITO_JWKS`

---

## Cost

| Resource | Free Tier | Projected Monthly |
|----------|-----------|-------------------|
| Cognito User Pool | 10K MAU/month | $0 (1 admin user) |
| Cognito hosted UI | Included | $0 |
| reqwest JWKS call | N/A (1× per cold start) | $0 |
| **Total new cost** | | **$0** |

---

## Migration Path

1. Apply `infra/cognito.tf` via `just infra-apply PROFILE`
2. Set temp password for `baba-admin` and complete first-login password reset via Cognito hosted UI
3. Deploy updated Lambda via `just deploy PROFILE`
4. Navigate to `sislam.com/auth/login` → confirm redirect to Cognito hosted UI

No database migration required. No existing routes change.

---

## Cross-References

- → W-AUTH (implementation plan — all 18 work items)
- → ADR-003 (Lambda Function URL — auth=NONE unchanged)
- → ADR-004 (Dual-mode entry point — dev-mode bypass extends this pattern)
- → ADR-006 (EFS + SQLite — admin API writes to same SQLite DB)
- → `plans/cross-cutting/aws-architecture.md` — Cognito added to topology
- → `plans/cross-cutting/aws-setup-spec.md` — `cognito-idp:*` IAM permissions added
