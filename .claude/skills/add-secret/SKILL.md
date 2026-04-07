---
name: add-secret
description: Add a new AWS Secrets Manager secret following the W-SEC policy. Covers xtask registration, infra HCL, IAM policy, Lambda env wiring, and the put command.
argument-hint: "[secret-name]"
---

Add a new managed secret following the W-SEC secrets policy. **Never** store secrets in Lambda environment variables (visible in AWS console), source code, or committed files.

## The W-SEC Policy

All secrets → AWS Secrets Manager. Lambda reads them via ARN at cold start. The `POW_SECRET` (pow-secret) is the reference implementation.

## Steps

### 1. Register in xtask/src/secret.rs

File: `xtask/src/secret.rs`

Add the secret name to the `KNOWN_SECRETS` list (or equivalent constant/enum):
```rust
pub const KNOWN_SECRETS: &[&str] = &[
    "pow-secret",
    "<your-secret-name>",   // ← add here
];
```

### 2. Add the Secrets Manager resource in infra/secrets.tf

File: `infra/secrets.tf`

```hcl
resource "aws_secretsmanager_secret" "<secret_name_underscored>" {
  name                    = "<secret-name>"
  recovery_window_in_days = 0  # allow immediate deletion in dev
}

resource "aws_secretsmanager_secret_version" "<secret_name_underscored>_placeholder" {
  secret_id     = aws_secretsmanager_secret.<secret_name_underscored>.id
  secret_string = "PLACEHOLDER_SET_WITH_JUST_SECRET_PUT"

  lifecycle {
    ignore_changes = [secret_string]  # managed by just secret-put, not tofu
  }
}
```

### 3. Add IAM policy for Lambda to read the secret

File: `infra/iam.tf` (or wherever `aws_iam_role_policy` for Lambda is defined)

```hcl
{
  "Effect": "Allow",
  "Action": ["secretsmanager:GetSecretValue"],
  "Resource": [aws_secretsmanager_secret.<secret_name_underscored>.arn]
}
```

### 4. Wire the ARN as a Lambda environment variable

File: `infra/lambda.tf`

```hcl
environment {
  variables = {
    # existing vars...
    <SECRET_NAME_UPPER>_ARN = aws_secretsmanager_secret.<secret_name_underscored>.arn
  }
}
```

### 5. Read the ARN in the Lambda startup code

File: `services/ui/src/main.rs` or the config struct

```rust
let secret_arn = std::env::var("<SECRET_NAME_UPPER>_ARN")
    .expect("<SECRET_NAME_UPPER>_ARN env var required");
let secret_value = fetch_secret(&secret_arn).await?;
```

Follow the same pattern as `POW_SECRET_ARN` in `services/ui/src/routes/contact.rs`.

### 6. Apply infra and write the secret value

```bash
just infra-plan <PROFILE>               # review before applying
just infra-apply <PROFILE>              # create SM resource + update Lambda env
just secret-put <secret-name> <VALUE> <PROFILE>   # write the actual secret
just lambda-deploy <PROFILE>            # redeploy to pick up new ARN env var
```

### 7. Update plan docs

- Add the secret to `plans/modules/secrets-manager.md` (W-SEC section)
- If this is for a new feature, reference in the feature's module plan

## Reference Implementation

`pow-secret` (PoW HMAC key):
- `xtask/src/secret.rs` — KNOWN_SECRETS
- `infra/secrets.tf` — SM resource
- `infra/iam.tf` — Lambda read policy
- `infra/lambda.tf` — `POW_SECRET_ARN` env var
- `services/ui/src/routes/contact.rs` — reads secret at cold start

## Key Files

- `xtask/src/secret.rs` — secret name registry + CLI commands
- `infra/secrets.tf` — SM resource definitions
- `infra/iam.tf` — Lambda IAM policies
- `infra/lambda.tf` — Lambda env vars
- `plans/modules/secrets-manager.md` — W-SEC plan module
