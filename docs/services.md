# Services

Last updated: 2026-05-19

deploy-baba runs three Lambda functions, each purpose-built for its workload.

```
                    ┌──────────────┐
                    │   Browser    │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │  CloudFront  │
                    │  CDN         │
                    └──┬───────┬───┘
           SPA assets  │       │  /api/*
                       ▼       ▼
                  ┌─────────────────┐
         ┌───────│   services/ui    │───────┐
         │       │   (main Lambda)  │       │
         │       │   VPC · EFS      │       │
         │       └────────┬─────────┘       │
         │                │                 │
    invoke()         SQLite/EFS        invoke()
         │                                  │
   ┌─────▼──────────┐              ┌────────▼────────┐
   │ services/email  │              │ services/       │
   │ (SES Lambda)    │              │ llm-proxy       │
   │ no VPC          │              │ no VPC          │
   └────────┬────────┘              └────────┬────────┘
            │                                │
       SES v2 API                    Anthropic API
```

## services/ui — Main Lambda

The portfolio backend. Serves the JSON API that the React SPA consumes.

| Property | Value |
|----------|-------|
| Binary | `deploy-baba-ui` |
| Framework | Axum + `lambda_http` adapter |
| Runtime | `provided.al2023`, aarch64, 256 MB |
| VPC | Yes (EFS mount required) |
| Database | SQLite on EFS (`/mnt/db/app.db`), 28 migrations |
| Auth | Cognito JWT RS256, HttpOnly session cookie ([ADR-008](../plans/adr/ADR-008-cognito-authentication.md)) |

**Dual-mode entry point** ([ADR-004](../plans/adr/ADR-004-dual-mode-entry-point.md)): detects at startup whether it's running in Lambda or locally. In Lambda, it uses the `lambda_http` adapter. Locally, it binds to a TCP socket for development.

### API Routes

| Route group | Path prefix | Auth | Handler file |
|-------------|-------------|------|-------------|
| Jobs | `/api/v1/jobs` | Public | `routes/api/jobs.rs` |
| Competencies | `/api/v1/competencies` | Public | `routes/api/competencies.rs` |
| About | `/api/v1/about` | Public | `routes/api/about.rs` |
| Social links | `/api/v1/social-links` | Public | `routes/api/social_links.rs` |
| Challenges | `/api/v1/challenges` | Public | `routes/api/challenges.rs` |
| Resume data | `/api/v1/resume` | Public | `routes/api/resume_data.rs` |
| Crates | `/api/v1/crates` | Public | `routes/api/crates.rs` |
| Stack | `/api/v1/stack` | Public | `routes/api/stack.rs` |
| Demo | `/api/v1/demo` | Public | `routes/api/demo.rs` |
| Ask (RAG) | `/api/ask` | Public | `routes/api/ask.rs` |
| Auth | `/api/auth/me` | Public | `routes/api/auth_me.rs` |
| Admin | `/api/admin/*` | Cognito | `routes/api/admin.rs` |

API versioning uses URL-based `/api/v1/` paths with deprecation headers ([ADR-024](../plans/adr/ADR-024-api-versioning-strategy.md)).

The admin API provides CRUD for all content types plus `GET /api/admin/db-dump` for SQLite snapshots (used by `/sync-dashboard-data`).

### OpenAPI Spec

Auto-generated dual spec ([ADR-012](../plans/adr/ADR-012-openapi-ssot.md)):
- `/docs` — public spec (admin operations stripped)
- `/api/openapi-admin.json` — full spec (Cognito-gated)

All 29 API models are defined in `crates/api-openapi/` (SSOT). The `services/ui` binary imports them — it never defines its own request/response types.

### Key files

- `services/ui/src/main.rs` — entry point with dual-mode detection
- `services/ui/src/router.rs` — Axum router assembly
- `services/ui/src/db.rs` — SQLite connection + migration runner
- `services/ui/src/auth.rs` — JWT verification + session middleware

## services/email — SES Lambda

Sends transactional emails triggered by the contact form.

| Property | Value |
|----------|-------|
| Binary | `email-lambda` |
| Runtime | `provided.al2023`, aarch64, 128 MB |
| VPC | No (needs direct internet access for SES) |
| Invocation | `aws_sdk_lambda::Client::invoke()` from UI Lambda |

The UI Lambda calls the email Lambda synchronously ([ADR-011](../plans/adr/ADR-011-emailer-response.md)) through a VPC interface endpoint (`com.amazonaws.us-east-1.lambda`). This avoids giving the VPC-bound UI Lambda direct internet access.

Two email types per submission:
1. **Admin notification** — sent to `contact-sislam@shantopagla.com`
2. **Submitter acknowledgement** — sent to the form submitter's address

Includes a honeypot check — bots that fill the hidden `website` field are silently dropped.

### Key files

- `services/email/src/main.rs` — single-file Lambda handler

## services/llm-proxy — LLM Routing Lambda

Routes LLM requests to the appropriate provider and executes tool-dispatch loops.

| Property | Value |
|----------|-------|
| Binary | `llm-proxy` |
| Runtime | `provided.al2023`, aarch64, 256 MB |
| VPC | No (needs direct internet access for Anthropic API) |
| Secrets | Anthropic API key from Secrets Manager |

The UI Lambda sends `AskProxyRequest` payloads to llm-proxy, which:
1. Initializes the Anthropic provider with the API key from Secrets Manager
2. Runs the agentic tool-dispatch loop (`run_agent_loop()` from `llm-core`)
3. Executes tool calls via HTTP callback to the UI Lambda (for DB queries etc.)
4. Returns the final `AskProxyResponse`

This architecture keeps the LLM API key out of the VPC-bound UI Lambda and allows provider-level changes without redeploying the main service.

### Key files

- `services/llm-proxy/src/main.rs` — Lambda handler + provider initialization
- `services/llm-proxy/src/tool_executor.rs` — HTTP-based tool execution
- `services/llm-proxy/src/tools.rs` — tool definitions

## Inter-Service Communication

```
Browser → CloudFront → UI Lambda (VPC)
                           ├── invoke() → Email Lambda (no VPC) → SES
                           └── invoke() → LLM Proxy (no VPC) → Anthropic API
                                              └── HTTP callback → UI Lambda (for tool results)
```

The VPC boundary is the key architectural constraint. The UI Lambda sits in a VPC for EFS access but cannot reach the internet directly. All outbound calls go through VPC endpoints:
- `com.amazonaws.us-east-1.lambda` — to invoke email and llm-proxy Lambdas
- `com.amazonaws.us-east-1.secretsmanager` — to read secrets at cold start
- `com.amazonaws.us-east-1.s3` — for S3 backup operations

## Cross-References

- [ADR-003](../plans/adr/ADR-003-lambda-function-url.md) — Lambda Function URL (no API Gateway)
- [ADR-004](../plans/adr/ADR-004-dual-mode-entry-point.md) — Dual-mode entry point
- [ADR-009](../plans/adr/ADR-009-api-gateway-pow-post.md) — API Gateway for POST /api/contact
- [ADR-011](../plans/adr/ADR-011-emailer-response.md) — Synchronous email Lambda invocation
- [ADR-023](../plans/adr/ADR-023-agentic-tool-dispatch.md) — Agentic tool-dispatch via llm-proxy
- [aws-setup.md](aws-setup.md) — Deployment instructions
