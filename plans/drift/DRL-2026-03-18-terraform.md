# DRL-2026-03-18-terraform: Terraform First-Run Drift Log

**Date:** 2026-03-18 | **Phase:** 6 (Terraform + End-to-End Deploy)
**Source:** First real `terraform init` + `plan` + `apply` run
**Affected modules:** W-TF, W-XT

All items below represent gaps between the plan and actual state.
Items marked [FIXED] were resolved in-session. Items marked [OPEN] remain.

---

## Entry 1: Backend bootstrap not automated — S3 bucket and DynamoDB table missing

**Plan:** `just infra-bootstrap` creates S3 state bucket + SSM sentinel.
**Reality:** `infra-bootstrap` was not yet implemented in xtask. S3 bucket
(`deploy-baba-tfstate`) and DynamoDB lock table (`terraform-lock`) were absent,
causing `terraform init` to fail immediately.

**Fix applied [OPEN]:** Manually bootstrapped via AWS CLI:
```bash
aws s3api create-bucket --bucket deploy-baba-tfstate --region us-east-1
aws s3api put-bucket-versioning --bucket deploy-baba-tfstate --versioning-configuration Status=Enabled
aws s3api put-bucket-encryption ...  # AES256
aws s3api put-public-access-block ... # block all public access
aws dynamodb create-table --table-name terraform-lock \
  --attribute-definitions AttributeName=LockID,AttributeType=S \
  --key-schema AttributeName=LockID,KeyType=HASH \
  --billing-mode PAY_PER_REQUEST
```

**Required action (W-XT.4.3):** Implement `xtask/src/infra/bootstrap.rs` to create
S3 bucket + DynamoDB table + SSM sentinel atomically. See DRL-2026-03-18-xtask Entry 1.

**Also:** Plan says bucket name is `deploy-baba-tfstate-<account-id>` but `infra/main.tf`
uses `deploy-baba-tfstate` (no suffix). Keep in sync.

---

## Entry 2: Security group cycle in `infra/efs.tf` [FIXED]

**Plan:** EFS SG allows NFS ingress from Lambda SG; Lambda SG egresses to EFS SG.
**Reality:** Both `aws_security_group.efs` and `aws_security_group.lambda_efs` referenced
each other inline (ingress/egress blocks), creating a Terraform cycle:
`Error: Cycle: aws_security_group.efs, aws_security_group.lambda_efs`

**Fix applied:** Extracted cross-SG rules into separate resources:
```hcl
resource "aws_security_group_rule" "efs_ingress_from_lambda" { ... }
resource "aws_security_group_rule" "lambda_egress_to_efs"    { ... }
```

**Lesson:** Security groups for EFS/Lambda must use separate `aws_security_group_rule`
resources for cross-references. Inline rules create a cycle when two SGs reference each other.

---

## Entry 3: `aws_efs_mount_target` does not support `tags` [FIXED]

**Reality:** AWS provider rejects `tags` on `aws_efs_mount_target`.
`Error: Argument named "tags" is not expected here`

**Fix applied:** Removed `tags` block from `aws_efs_mount_target.baba_db`.

---

## Entry 4: `lambda.tf` `depends_on` referenced non-existent `aws_iam_role_policy_attachment` resources [FIXED]

**Reality:** `lambda.tf` listed `aws_iam_role_policy_attachment.lambda_efs/s3/ssm`
in `depends_on`, but `iam.tf` defines those as `aws_iam_role_policy` (inline policies,
not attachments). Also missing `lambda_vpc` from dependency list.

**Fix applied:**
```hcl
depends_on = [
  aws_cloudwatch_log_group.lambda,
  aws_iam_role_policy_attachment.lambda_logs,
  aws_iam_role_policy_attachment.lambda_vpc,
  aws_iam_role_policy.lambda_efs,
  aws_iam_role_policy.lambda_s3,
  aws_iam_role_policy.lambda_ssm,
]
```

---

## Entry 5: Lambda zip missing — `filebase64sha256` fails at plan time [OPEN]

**Plan:** `just deploy` runs `cargo lambda build --release` and produces the zip.
**Reality:** `infra/variables.tf` defaults `lambda_code_path = "./build/lambda.zip"`.
Terraform's `filebase64sha256()` is evaluated at **plan time**, so `terraform plan`
fails if the file doesn't exist before any deploy has ever run.

**Options:**
- **Option A (preferred):** Build the Lambda zip before `terraform plan/apply`. Add
  `just lambda-build` recipe; wire into `just infra-apply` as prerequisite.
- **Option B:** Use `try()` or `null_resource`/`external` data source — fragile.

**Also:** `cargo-lambda` was not installed. Must be added to bootstrap docs.

**Required actions (W-DX.5):**
- Add `cargo install cargo-lambda` to new developer setup docs
- `just lambda-build` wraps `cargo lambda build --release`
- `just infra-apply` must depend on `just lambda-build`

---

## Entry 6: Terraform provider deprecation warnings [RESOLVED]

| File | Warning | Status | Fix |
|------|---------|--------|-----|
| `infra/eventbridge.tf:6` | `is_enabled` deprecated — use `state` | RESOLVED | Fixed: `state = "ENABLED"` already in code. Confirmed by DRL-2026-03-25-opentofu. |
| `infra/s3.tf:41` | lifecycle rule missing `filter {}` block | RESOLVED | Fixed: `filter {}` already present in code. Confirmed by DRL-2026-03-25-opentofu. |

Both items were fixed in code before this log was written.
See DRL-2026-03-25-opentofu for confirmation of W-TF.4.1 and W-TF.4.2 resolution.

---

## Phase 6 Checklist (Updated)

- [x] `infra/cdn.tf` — CloudFront distribution + Route53 records
- [x] `infra/outputs.tf` — cloudfront_distribution_id, cloudfront_domain_name, site_url
- [ ] **W-XT.4.3** — Implement `xtask/src/infra/bootstrap.rs` (S3 + DynamoDB + SSM)
- [ ] **W-DX.5** — Add `just lambda-build`; wire into `just infra-apply`
- [x] **W-TF.4.1** — Fix `eventbridge.tf`: replace `is_enabled` with `state = "ENABLED"` — RESOLVED
- [x] **W-TF.4.2** — Fix `s3.tf` lifecycle rules: add `filter {}` block — RESOLVED
- [ ] `terraform plan` clean (zero errors, zero warnings)
- [ ] `terraform apply` — 28 resources created
- [ ] CloudFront propagates (~5–15 min)
- [ ] `curl -I https://sislam.com` → 200
- [ ] `curl https://sislam.com/health` → `{"status":"ok",...}`
