# W-AUTH: Cognito Authentication & Admin Dashboard
**Path:** `services/ui/src/auth.rs`, `services/ui/src/middleware.rs`, `services/ui/src/routes/auth.rs`, `services/ui/src/routes/api/admin.rs`, `infra/cognito.tf` | **Status:** TODO
**Depends on:** W-UI (ui-service), W-OTF (infra) | **Depended on by:** —
→ ADR-008 (Cognito Authentication)

---

## W-AUTH.1 Purpose

Add AWS Cognito authentication to the portfolio so the admin (`baba-admin`) can edit all resume
artifacts via a protected `/dashboard` — without SSH or direct SQLite access.

**Scope:**
- **Public** — `GET /`, `/api/jobs`, `/api/competencies` remain unauthenticated (resume unchanged)
- **Protected** — `GET /dashboard` and all `PUT/POST/DELETE /api/admin/*` require a valid Cognito session
- **Entry point** — "Login" button in the nav → Cognito hosted UI → cookie session → `/dashboard`

→ ADR-008 for full rationale (cookie sessions, FromRef compat, dev-mode bypass, free tier)

---

## W-AUTH.2 Public API Surface

### Infrastructure outputs (`infra/outputs.tf`)

| Output | Description |
|--------|-------------|
| `cognito_user_pool_id` | Pool ID for Lambda env var + SSM |
| `cognito_client_id` | App client ID (public, no secret) |
| `cognito_domain` | Hosted UI domain prefix |

### Auth routes (`services/ui/src/routes/auth.rs`)

| Route | Behavior |
|-------|----------|
| `GET /auth/login` | 302 → Cognito hosted UI (`response_type=token&scope=openid` → `id_token` in fragment) |
| `GET /auth/callback` | Serve HTML page; JS extracts `id_token` from URL fragment, POSTs to `/auth/set-session` |
| `POST /auth/set-session` | Validate `id_token` JWT; set `auth_token` HttpOnly cookie; 200 or 401 |
| `GET /auth/logout` | Clear cookie; 302 → Cognito `/logout` |

### Admin CRUD API (`services/ui/src/routes/api/admin.rs`)

All routes require `require_auth` middleware. Returns JSON.

```
POST   /api/admin/jobs                      → 201 + created Job
PUT    /api/admin/jobs/:id                   → 200 + updated Job | 404
DELETE /api/admin/jobs/:id                   → 204 | 404

POST   /api/admin/jobs/:job_id/details       → 201 + created JobDetail
PUT    /api/admin/jobs/:job_id/details/:id   → 200 + updated JobDetail | 404
DELETE /api/admin/jobs/:job_id/details/:id   → 204 | 404

POST   /api/admin/competencies               → 201 + created Competency
PUT    /api/admin/competencies/:id           → 200 + updated Competency | 404
DELETE /api/admin/competencies/:id           → 204 | 404

POST   /api/admin/evidence                   → 201 + created Evidence
PUT    /api/admin/evidence/:id               → 200 + updated Evidence | 404
DELETE /api/admin/evidence/:id               → 204 | 404
```

### AppState (`services/ui/src/state.rs`)

```rust
#[derive(Clone, axum::extract::FromRef)]
pub struct AppState {
    pub db: Arc<Db>,
    pub auth: Arc<AuthConfig>,
}
```

`FromRef` ensures all existing handlers (`State(db): State<Arc<Db>>`) remain unchanged.

---

## W-AUTH.3 Implementation Notes

### 3.1 Infrastructure — `infra/cognito.tf`

Five HCL resources:

| Resource | Notes |
|----------|-------|
| `aws_cognito_user_pool.baba` | Password policy: min 12, upper+lower+num+sym; email auto-verified; deletion protection ACTIVE |
| `aws_cognito_user_pool_domain.baba` | Domain: `${var.project_name}-${var.environment}` → `deploy-baba-prod.auth.us-east-1.amazoncognito.com` |
| `aws_cognito_user_pool_client.baba_web` | Public client (no secret); **implicit grant flow**; callback/logout URLs for prod + localhost:3000 |
| `aws_cognito_user.baba_admin` | username: `baba-admin`; email: `var.admin_email`; temp password via sensitive variable |
| `data "http" "cognito_jwks"` | Fetches JWKS at deploy time; stored in `COGNITO_JWKS` Lambda env var — no runtime outbound calls |

New variables in `infra/variables.tf`:
- `admin_email` (string, default `"it@shantopagla.com"`)
- `cognito_temp_password` (string, sensitive — reset on first login)

New SSM parameters in `infra/ssm.tf` (under `/${project_name}/${environment}/`):
- `cognito-pool-id`, `cognito-client-id`, `cognito-domain`

New Lambda env vars in `infra/lambda.tf` `environment.variables`:
- `COGNITO_POOL_ID`, `COGNITO_CLIENT_ID`, `COGNITO_DOMAIN`, `COGNITO_REGION`, `APP_DOMAIN`, `COGNITO_JWKS`

### 3.2 Auth Module — `services/ui/src/auth.rs`

```rust
pub struct AuthConfig {
    pub pool_id: String,
    pub client_id: String,
    pub cognito_domain: String,  // e.g. "deploy-baba-prod.auth.us-east-1.amazoncognito.com"
    pub region: String,
    pub app_domain: String,      // e.g. "https://sislam.com"
    jwks_json: String,           // embedded from COGNITO_JWKS env var at deploy time
    pub dev_mode: bool,          // true when COGNITO_POOL_ID is unset
}

pub struct Claims { pub sub: String, pub email: String, pub username: String }

pub enum AuthError { ... }  // via thiserror
```

**Dev-mode bypass:** when `COGNITO_POOL_ID` is absent, `from_env()` returns a dev config where
`validate_token` always succeeds. All routes (including `/dashboard`) work locally without AWS.

**JWKS from env var:** `COGNITO_JWKS` is populated at deploy time by OpenTofu's `data "http"`
data source, which fetches JWKS from Cognito before Lambda is updated.  Lambda reads this string
at startup — no outbound network call needed.  `from_env()` and `validate_token` are both free
of I/O.  `validate_token` stays `async` for interface stability.

**Rationale:** Lambda runs in a VPC (for EFS) without a NAT Gateway.  The W-AUTH.4.20 lazy
fetch deferred the failure but didn't solve it — JWKS fetch still timed out on first auth
request.  Embedding JWKS at deploy time eliminates all outbound calls from Lambda.  Keys are
refreshed on each `just deploy` (which re-runs `tofu apply`).

### 3.3 Callback Flow (implicit grant)

```
Browser → GET /auth/login
        → 302 to Cognito hosted UI (response_type=id_token)
        → POST credentials to Cognito
        → 302 to GET /auth/callback#id_token=xxx&...  (fragment — NOT sent to server)
        → Lambda: return HTML page
        → JS: extract id_token from window.location.hash
        → JS: POST {"id_token": "..."} to /auth/set-session
        → Lambda: validate JWT (RS256 using jwks_json from env)
        → Lambda: set auth_token HttpOnly cookie
        → JS: redirect to /dashboard
```

Token exchange is client-side only; Lambda never makes outbound calls.

### 3.3 Middleware — `services/ui/src/middleware.rs`

Token extraction order:
1. `auth_token` HttpOnly cookie (primary — set by `/auth/callback`)
2. `Authorization: Bearer <token>` header (API fallback)

On validation failure:
- Browser request (`Accept: text/html` or no `Accept`) → 302 to Cognito login
- API request (`Accept: application/json`) → 401 JSON `{"error":"Unauthorized"}`

On success: `Claims` injected into request extensions via `.extensions_mut().insert(claims)`.

### 3.4 Cookie Security

| Attribute | Value | Rationale |
|-----------|-------|-----------|
| `HttpOnly` | true | Blocks JS access — XSS cannot steal token |
| `Secure` | true (prod) / false (dev) | HTTPS only in production |
| `SameSite` | Lax | Allows top-level nav, blocks CSRF |
| `Max-Age` | 3600 (1h) | Matches Cognito ID token TTL |
| `Path` | `/` | Available site-wide |

### 3.5 Dashboard — `services/ui/templates/dashboard.html`

Client-rendered master/detail layout:
- **Left sidebar:** accordion — Jobs → Job Details → Competencies → Evidence
- **Right panel:** editable form for selected item; submit via PUT to `/api/admin/*`
- **Data source:** existing public GET APIs (`/api/jobs`, `/api/competencies`)
- **Toast notifications:** success/error feedback without full page reload

Extends `base.html`; styled with Tailwind (same as rest of site).

### 3.6 Dependency Graph

```
W-AUTH.4.1 (cognito.tf)
    ├─► W-AUTH.4.2 (ssm.tf additions)
    ├─► W-AUTH.4.3 (lambda.tf env vars)
    └─► W-AUTH.4.4 (variables.tf + outputs.tf)

W-AUTH.4.5 (workspace deps)
    └─► W-AUTH.4.6 (auth.rs)
         ├─► W-AUTH.4.7 (state.rs)
         │    └─► W-AUTH.4.9 (main.rs)
         └─► W-AUTH.4.8 (middleware.rs)
              ├─► W-AUTH.4.10 (routes/auth.rs)
              ├─► W-AUTH.4.11 (routes/api/admin.rs)
              └─► W-AUTH.4.12 (routes/dashboard.rs)
                   ├─► W-AUTH.4.13 (dashboard.html)
                   ├─► W-AUTH.4.14 (base.html login button)
                   └─► W-AUTH.4.15 (router.rs wiring)
```

---

## W-AUTH.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-AUTH.4.1 | Create `infra/cognito.tf` | TODO | User pool, domain, client, admin user |
| W-AUTH.4.2 | Add Cognito SSM params to `infra/ssm.tf` | TODO | 3 params under `cognito-*` prefix |
| W-AUTH.4.3 | Add Cognito env vars to `infra/lambda.tf` | TODO | 5 env vars: POOL_ID, CLIENT_ID, DOMAIN, REGION, APP_DOMAIN |
| W-AUTH.4.4 | Add variables + outputs to `infra/variables.tf`, `infra/outputs.tf` | TODO | `admin_email`, `cognito_temp_password` (sensitive); 3 outputs |
| W-AUTH.4.5 | Add workspace deps (jsonwebtoken, reqwest, axum-extra) | TODO | `Cargo.toml` workspace + `services/ui/Cargo.toml` |
| W-AUTH.4.6 | Create `services/ui/src/auth.rs` | TODO | AuthConfig, JWKS fetch, JWT RS256 validation, dev-mode bypass |
| W-AUTH.4.7 | Create `services/ui/src/state.rs` | TODO | AppState with `FromRef`; zero changes to existing handlers |
| W-AUTH.4.8 | Create `services/ui/src/middleware.rs` | TODO | `require_auth` — cookie/header extraction, 302 vs 401 branching |
| W-AUTH.4.9 | Update `services/ui/src/main.rs` | TODO | Init AuthConfig, construct AppState, pass to router::build |
| W-AUTH.4.10 | Create `services/ui/src/routes/auth.rs` | TODO | login redirect, callback (code exchange + cookie), logout |
| W-AUTH.4.11 | Create `services/ui/src/routes/api/admin.rs` | TODO | Full CRUD: POST/PUT/DELETE for jobs, job_details, competencies, evidence |
| W-AUTH.4.12 | Create `services/ui/src/routes/dashboard.rs` | TODO | Minimal handler — serves DashboardTemplate (Askama) |
| W-AUTH.4.13 | Create `services/ui/templates/dashboard.html` | TODO | Tailwind master/detail; JS fetches public APIs; submits to admin API |
| W-AUTH.4.14 | Add login button to `services/ui/templates/base.html` nav | TODO | `<a href="/auth/login">Login</a>` alongside existing nav links |
| W-AUTH.4.15 | Wire all new routes into `services/ui/src/router.rs` | TODO | `/auth/*`, `/dashboard` (protected), `/api/admin/*` (protected) |
| W-AUTH.4.16 | Create `plans/modules/auth.md` | DONE | This file |
| W-AUTH.4.17 | Create `plans/adr/ADR-008-cognito-authentication.md` | DONE | Decision record |
| W-AUTH.4.18 | Update `plans/INDEX.md` + cross-cutting files | DONE | W-AUTH row, ADR-008, Cognito topology + IAM |
| W-AUTH.4.19 | Add OpenAPI security scheme + admin endpoint docs | DONE | cookieAuth/bearerAuth, 12 admin paths, ToSchema on input types |
| W-AUTH.4.20 | Fix Lambda 504 — lazy JWKS fetch with 5s timeout | SUPERSEDED | Deferred fetch still failed (VPC has no NAT Gateway). Replaced by W-AUTH.4.21. |
| W-AUTH.4.21 | Fix Cognito callback 504 — implicit grant + JWKS from env var | DONE | `allowed_oauth_flows=["implicit"]`; `allow_admin_create_user_only=true` (no self-signup); `data "http" cognito_jwks`; `COGNITO_JWKS` env var; HTML callback page + `/auth/set-session` endpoint; zero Lambda outbound calls |

---

## W-AUTH.5 Test Strategy

1. **Dev-mode smoke test:** `just ui` — navigate to `/dashboard` without Cognito env vars; verify page loads (dev bypass active)
2. **Login flow (local):** set `COGNITO_POOL_ID` etc. in `.env`; hit `/auth/login`; verify redirect to Cognito hosted UI
3. **Callback validation:** mock JWKS endpoint; call `/auth/callback?code=xxx`; verify `auth_token` cookie set
4. **Middleware enforcement:** request `GET /dashboard` without cookie → 302 to login; with valid cookie → 200
5. **Admin CRUD (unit):** test each handler in `admin.rs` with in-memory SQLite; verify 201/200/204/404 responses
6. **XSS resistance:** cookie `HttpOnly` — verify `document.cookie` doesn't contain `auth_token` in browser
7. **CSRF resistance:** SameSite=Lax — cross-origin POST doesn't include cookie

---

## W-AUTH.6 Cross-References

- → ADR-008 (Cognito Authentication — decision record)
- → ADR-003 (Lambda Function URL — auth=NONE preserved; Cognito is app-layer, not Lambda URL auth)
- → ADR-004 (Dual-mode entry point — dev-mode bypass extends this pattern)
- → W-RSM (resume module — public GET APIs reused by dashboard JS)
- → W-UI (ui-service — AppState refactor; router.rs changes)
- → W-OTF (opentofu — infra changes in Phases 1 and 2)
- → `plans/cross-cutting/aws-architecture.md` — Cognito added to topology
- → `plans/cross-cutting/aws-setup-spec.md` — `cognito-idp:*` IAM permissions
