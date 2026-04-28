# W-SEC: AWS Secrets Manager Integration
**Service:** `xtask/` + `services/ui/` + `infra/` | **Status:** DONE (2026-04-03)
**Depends on:** W-OTF (tofu infra) | **Depended on by:** W-CTF (POW_SECRET)

## W-SEC.1 Purpose

Eliminate all secrets from:
- `infra/terraform.tfvars` (local file on disk)
- Lambda environment variables (visible in AWS console)
- Source code / hardcoded fallbacks

Replace with AWS Secrets Manager: secrets are stored encrypted, accessed via IAM,
and managed through xtask commands (`just secret-put`, `just secret-get`).

## W-SEC.2 Design

### Secrets to manage
| Secret name | SM path | Used by |
|-------------|---------|---------|
| `pow-secret` | `/deploy-baba/prod/pow-secret` | UI Lambda — PoW HMAC key |
| `anthropic-api-key` | `/deploy-baba/prod/anthropic-api-key` | UI Lambda — Anthropic API key for W-RST/W-RAG LLM calls |

### Infra changes (`infra/secrets.tf` — new file)
- `aws_secretsmanager_secret` — creates the secret resource
- `aws_secretsmanager_secret_version` — initial placeholder value (lifecycle.ignore_changes)
- `aws_iam_role_policy` attachment — `secretsmanager:GetSecretValue` for Lambda execution role
- Remove `POW_SECRET` from `aws_lambda_function.baba` environment block (`infra/lambda.tf`)
- Remove `pow_secret` variable from `infra/variables.tf` and `infra/terraform.tfvars`
- Add `POW_SECRET_ARN` env var (non-sensitive) so Lambda can locate the secret

### xtask commands (`xtask/src/secrets/`)
New module with clap subcommands:
- `cargo xtask secret put --name <name> --value <value> --profile <profile>`
  - Calls `aws_sdk_secretsmanager::Client::put_secret_value()`
  - Validates secret name against known list
- `cargo xtask secret get --name <name> --profile <profile>`
  - Returns secret value (stdout)
- `cargo xtask secret list --profile <profile>`
  - Lists all managed secrets with their SM paths

### justfile recipes
```just
secret-put NAME VALUE PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask secret put --name {{NAME}} --value {{VALUE}} --profile {{PROFILE}}

secret-get NAME PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask secret get --name {{NAME}} --profile {{PROFILE}}

secret-list PROFILE="default":
    just aws-check {{PROFILE}} && cargo xtask secret list --profile {{PROFILE}}
```

### Lambda runtime change (`services/ui/src/routes/contact.rs`)
Replace:
```rust
std::env::var("POW_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string())
```
With:
```rust
// In Lambda: fetch from Secrets Manager via POW_SECRET_ARN env var
// Locally: fall back to dev-secret-change-me (SM SDK call skipped when ARN absent)
async fn fetch_pow_secret() -> [u8; 32] {
    if let Ok(arn) = std::env::var("POW_SECRET_ARN") {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_secretsmanager::Client::new(&config);
        if let Ok(resp) = client.get_secret_value().secret_id(&arn).send().await {
            if let Some(val) = resp.secret_string() {
                return sha256_key(val);
            }
        }
    }
    sha256_key("dev-secret-change-me")
}
```
Use `OnceLock<tokio::sync::OnceCell<[u8; 32]>>` for async lazy initialization, or
fetch eagerly at Lambda cold start in `main.rs`.

### New workspace dep
`aws-sdk-secretsmanager = "1"` (workspace + services/ui)

## W-SEC.3 Work Items

| ID | Task | Status |
|----|------|--------|
| W-SEC.4.1 | Create `infra/secrets.tf` — SM secret + IAM policy + POW_SECRET_ARN env var | DONE |
| W-SEC.4.2 | Remove `pow_secret` var from `infra/variables.tf`, `infra/lambda.tf`, `terraform.tfvars` | DONE |
| W-SEC.4.3 | Add `aws-sdk-secretsmanager` to workspace Cargo.toml + services/ui + xtask | DONE |
| W-SEC.4.4 | Create `xtask/src/secret.rs` — put/get/list commands | DONE |
| W-SEC.4.5 | Add `secret-put`, `secret-get`, `secret-list` justfile recipes | DONE |
| W-SEC.4.6 | Update `contact.rs` to fetch POW_SECRET from SM at cold start via `init_pow_secret()` | DONE |
| W-SEC.4.7 | `just infra-apply` + `just secret-put pow-secret <value>` + `just lambda-deploy` | OPEN — deploy step |
| W-SEC.4.8 | Verify: submit contact form → challenge → solve → POST → email received | OPEN — W-CTF.4.12 |
| W-SEC.4.9 | Add `anthropic-api-key` SM resource + IAM policy + `ANTHROPIC_API_KEY_ARN` Lambda env var | DONE (2026-04-15) |
| W-SEC.4.10 | Load `anthropic-api-key` at cold start in `services/ui/src/main.rs` via `init_anthropic_key()` + OnceLock | DONE (2026-04-15) |
| W-SEC.4.11 | `just infra-apply PROFILE` + `just secret-put anthropic-api-key <KEY> PROFILE` + `just lambda-deploy PROFILE` | OPEN — provision step |

### Additional changes beyond original design
- `infra/secrets.tf` also includes `cognito-temp-password` SM secret (full secret audit)
- `infra/vpc-endpoints.tf` adds SM VPC Interface Endpoint (~$7.30/mo, 1 AZ)
- `infra/cognito.tf`: `temporary_password` now reads from SM secret version (not tfvar)
- `infra/lambda.tf`: `depends_on` includes `aws_iam_role_policy.lambda_secretsmanager`
- `xtask/src/secret.rs` at top-level (not `secrets/`), validates against `KNOWN_SECRETS`

## W-SEC.4 Cross-References
→ W-CTF.4.11 (blocked on W-SEC)
→ ADR-009 (API Gateway + PoW — uses POW_SECRET)
→ ADR-007 (OpenTofu manages SM resource)
