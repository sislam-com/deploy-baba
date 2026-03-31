# AWS Architecture — deploy-baba

**Region:** us-east-1 | **Updated:** 2026-03-26

---

## Production Topology

```
  HTTPS Request ──► CloudFront ──► Lambda Function URL ──► Lambda
  (sislam.com)      (cache: off)   (origin, HTTPS-only)

                    ┌──────────────────────────────┐
  HTTPS Request ──► │  Lambda Function URL          │  Free HTTPS endpoint
  (browser/curl)    │  (auth: NONE, CORS: enabled)  │  No API Gateway needed
                    └─────────────┬────────────────┘
                                  │ invokes
                    ┌─────────────▼────────────────┐
                    │  Lambda: deploy-baba-ui       │  Rust binary (Axum adapter)
                    │  Runtime: provided.al2023     │  aarch64, 256MB, ~5ms cold start
                    │  Timeout: 30s                 │
                    └────────┬────────────┬─────────┘
                             │ mount EFS  │ reads
                    ┌────────▼──────┐  ┌──▼───────────────────────────────┐
                    │  EFS          │  │  SSM Params                       │
                    │  /mnt/db/     │  │  /deploy-baba/*                   │
                    │  app.db       │  │  (config + cognito-pool-id, etc.) │
                    └────────┬──────┘  └──────────────────────────────────┘
                             │ scheduled backup
                    ┌────────▼──────┐
                    │  S3           │
                    │  backups/     │  EventBridge: daily
                    └───────────────┘

  Admin login flow (app-layer auth — Lambda Function URL auth stays NONE):

  Browser ─► GET /auth/login ─► 302 ─► Cognito Hosted UI
                                              │ (login form)
                                        POST credentials
                                              │
                                        302 to /auth/callback?code=xxx
                                              │
                    ┌─────────────────────────▼────────────────────┐
                    │  Lambda: POST /oauth2/token to Cognito       │
                    │  Validate ID token (RS256, JWKS cached)      │
                    │  Set auth_token cookie (HttpOnly, SameSite)  │
                    │  302 → /dashboard                            │
                    └──────────────────────────────────────────────┘

                    ┌──────────────────────────────────────────┐
  Cognito ────────► │  aws_cognito_user_pool "baba"            │
                    │  User: baba-admin (email verified)        │
                    │  Domain: deploy-baba-prod.auth.*          │
                    │  Client: baba_web (public, PKCE)          │
                    └──────────────────────────────────────────┘
```

---

## Option A: Lambda + Function URL (Recommended — near-zero cost)

**Decision rationale:** → ADR-003

```
                    ┌──────────────────────────────┐
  HTTPS Request ──► │  Lambda Function URL          │  Free HTTPS endpoint
                    │  (auth: NONE, CORS: enabled)  │
                    └─────────────┬────────────────┘
                                  │ invokes
                    ┌─────────────▼─────────────────┐
                    │  Lambda: deploy-baba-ui        │  (Rust binary via cargo-lambda)
                    │  Runtime: provided.al2023      │  aarch64, 256MB, ~5ms cold start
                    │  Timeout: 30s                  │
                    └────────┬─────────┬────────────┘
                             │ mount   │ reads
                    ┌────────▼──────┐  ┌──▼───────────────┐
                    │  EFS          │  │  SSM Params       │
                    │  /mnt/db/     │  │  /deploy-baba/*   │
                    └────────┬──────┘  └──────────────────┘
                             │ scheduled backup
                    ┌────────▼──────┐
                    │  S3 backups/  │  EventBridge: daily
                    └───────────────┘
```

**Cost breakdown:**

| Service | Free Tier | Typical Monthly (low traffic) |
|---------|-----------|-------------------------------|
| Lambda (256MB, <1M req/mo) | 1M req, 400K GB-sec/month | $0 |
| Lambda Function URL | Free (no added charge) | Free |
| EFS | 5GB free (first year) | ~$0.001 (tiny SQLite file) |
| S3 backup | 5GB free | ~$0.001 |
| ECR Public | Free | $0 |
| SSM Standard | Free | $0 |
| CloudWatch Logs | 5GB free | $0 |
| EventBridge | 14M events/month free | $0 |
| **Total** | | **~$0/month** (within free tier) |

After free tier expiry: ~$0–1/month at low traffic.

**Implementation notes:**
- Use `cargo-lambda` for cross-compilation and packaging
- `lambda_http` crate adapts Axum router to Lambda HTTP events
- Lambda reads SQLite via EFS mount at `/mnt/db/deploy-baba.db`
- EFS access point scoped to `/deploy-baba` directory

---

## Option B: ECS Fargate Spot (always-on, ~$5-7/month)

```
  HTTP Request  ──► NLB ($0.008/LCU-hr) ──► ECS Fargate Spot task
                                              (0.25 vCPU, 0.5GB)
                                              │ mount EFS
                                              └► SQLite ──► S3 backup
```

**Cost breakdown:**

| Service | Monthly |
|---------|---------|
| ECS Fargate Spot (0.25vCPU, 0.5GB) | ~$1.50 |
| NLB | ~$5.50 |
| EFS | ~$0.001 |
| S3 backup | ~$0.001 |
| ECR Public | $0 |
| SSM | $0 |
| **Total** | **~$7/month** |

**Mode selection:** Set in `stack.toml`:
```toml
[deploy]
mode = "lambda"    # or "ecs-fargate-spot"
```

---

## OpenTofu Resources (32 total after W-AUTH)

Managed in `infra/`:

| Resource | File | Notes |
|----------|------|-------|
| `aws_lambda_function` | `lambda.tf` | aarch64, provided.al2023 |
| `aws_lambda_function_url` | `lambda.tf` | auth=NONE, CORS enabled |
| `aws_lambda_permission` | `lambda.tf` | allow CloudFront invoke |
| `aws_efs_file_system` | `efs.tf` | |
| `aws_efs_access_point` | `efs.tf` | scoped to /deploy-baba |
| `aws_efs_mount_target` | `efs.tf` | per subnet |
| `aws_security_group` (efs) | `efs.tf` | NFS port 2049 |
| `aws_security_group` (lambda) | `efs.tf` | |
| `aws_security_group_rule` ×2 | `efs.tf` | cross-SG rules (no cycle) |
| `aws_s3_bucket` (backup) | `s3.tf` | |
| `aws_s3_bucket` (tfstate) | `s3.tf` | OpenTofu backend |
| `aws_cloudwatch_event_rule` | `eventbridge.tf` | daily backup schedule |
| `aws_cloudwatch_log_group` | `lambda.tf` | |
| `aws_iam_role` | `iam.tf` | Lambda execution role |
| `aws_iam_role_policy` ×3 | `iam.tf` | EFS, S3, SSM |
| `aws_iam_role_policy_attachment` ×2 | `iam.tf` | basic execution, VPC |
| `aws_cloudfront_distribution` | `cdn.tf` | custom domain |
| `aws_route53_record` ×2 | `cdn.tf` | apex + www |
| `aws_cognito_user_pool` | `cognito.tf` | **W-AUTH** — baba user pool |
| `aws_cognito_user_pool_domain` | `cognito.tf` | **W-AUTH** — hosted UI domain |
| `aws_cognito_user_pool_client` | `cognito.tf` | **W-AUTH** — public PKCE client |
| `aws_cognito_user` | `cognito.tf` | **W-AUTH** — baba-admin user |

---

## Cross-References
- → ADR-002 (SQLite over PostgreSQL)
- → ADR-003 (Lambda Function URL)
- → ADR-006 (EFS + SQLite + S3 topology)
- → ADR-008 (Cognito Authentication — W-AUTH)
- → `plans/modules/terraform.md` — W-TF implementation
- → `plans/modules/auth.md` — W-AUTH Cognito implementation
- → `plans/modules/ui-service.md` — W-UI Lambda entry point
- → `plans/cross-cutting/aws-setup-spec.md` — IAM policy + profile config
- → `plans/drift/DRL-2026-03-18-terraform.md` — first-run fixes
