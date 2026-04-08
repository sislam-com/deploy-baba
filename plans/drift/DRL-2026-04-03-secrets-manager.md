# DRL-2026-04-03-secrets-manager — W-SEC Secrets Manager Migration

**Date:** 2026-04-03
**Domain:** W-SEC, W-CTF
**Trigger:** POW_SECRET was stored as plaintext in Lambda env var and `infra/terraform.tfvars`

## What Changed

| File | Change |
|------|--------|
| `Cargo.toml` | Added `aws-sdk-secretsmanager = "1"` to workspace deps |
| `xtask/Cargo.toml` | Wired `aws-sdk-secretsmanager` |
| `services/ui/Cargo.toml` | Wired `aws-sdk-secretsmanager` |
| `infra/secrets.tf` | New: 2 SM secrets (pow-secret, cognito-temp-password) + IAM policy |
| `infra/vpc-endpoints.tf` | Added SM VPC Interface Endpoint (`com.amazonaws.us-east-1.secretsmanager`) |
| `infra/lambda.tf` | `POW_SECRET` env var → `POW_SECRET_ARN`; added `lambda_secretsmanager` to depends_on |
| `infra/variables.tf` | Removed `pow_secret` and `cognito_temp_password` variables |
| `infra/cognito.tf` | `temporary_password` reads from `aws_secretsmanager_secret_version.cognito_temp_password_initial.secret_string` |
| `infra/terraform.tfvars` | Secret line removed; only comments remain |
| `xtask/src/secret.rs` | New: `SecretAction` enum + put/get/list commands |
| `xtask/src/main.rs` | Registered `mod secret` + `Secret` subcommand |
| `justfile` | Added `secret-put`, `secret-get`, `secret-list` recipes |
| `services/ui/src/routes/contact.rs` | Replaced sync `OnceLock` init with async `init_pow_secret()` |
| `services/ui/src/main.rs` | Calls `routes::contact::init_pow_secret().await` at cold start |

## Secrets Audit Result

| Value | Verdict | Action |
|-------|---------|--------|
| `POW_SECRET` | **MIGRATED** — HMAC signing key for PoW challenges | SM path: `deploy-baba/prod/pow-secret` |
| `cognito_temp_password` | **MIGRATED** — Cognito admin user bootstrap password | SM path: `deploy-baba/prod/cognito-temp-password` |
| `COGNITO_JWKS` | Safe — public JWKS data | No change |
| `COGNITO_CLIENT_ID` | Safe — public OAuth client ID | No change |
| All other env vars | Safe — infrastructure config | No change |

## Deploy Steps Remaining

1. `just infra-apply default` — creates SM secrets + VPC endpoint + IAM policy
2. `just secret-put pow-secret $(openssl rand -hex 32) default` — store real PoW key
3. `just secret-put cognito-temp-password <value> default` — record Cognito password
4. `just lambda-deploy default` — deploy new Lambda binary
5. Verify: `just secret-get pow-secret default` + submit contact form

## Notes

- SM secret names use `deploy-baba/prod/` prefix (matches infra `${var.project_name}/${var.environment}/`)
- `xtask/src/secret.rs` validates name against `KNOWN_SECRETS` list to prevent typos
- `init_pow_secret()` is idempotent via `OnceLock::get()` guard — safe to call multiple times
- VPC endpoint (~$7.30/mo, 1 AZ) required because UI Lambda is VPC-bound (for EFS) and has no NAT Gateway
