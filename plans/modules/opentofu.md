# W-OTF: opentofu
**Path:** `infra/` (HCL), `xtask/src/infra/` (wrapper) | **Status:** TODO
**Replaces:** W-TF (Terraform) | **Depends on:** W-XT | **Depended on by:** W-DX
→ ADR-007 (OpenTofu over Terraform)

---

## W-OTF.1 Purpose

Replace the `terraform` binary with `opentofu` (`tofu` CLI) for all infrastructure management.
OpenTofu is the Linux Foundation-governed, MPL-2.0 open-source fork of Terraform 1.5.x.

The change scope is intentionally minimal:
- **HCL files:** `required_version` updated; all resources unchanged
- **xtask:** `terraform.rs` renamed → `tofu.rs`; binary invocation `terraform` → `tofu`
- **justfile:** No changes (calls `cargo xtask infra …`, not `terraform` directly)
- **State:** S3 state format is compatible; no migration step needed

→ ADR-007 for full rationale and migration path

---

## W-OTF.2 Resource Inventory (inherited from W-TF)

Same 28 AWS resources, same file layout. No HCL resource blocks change.

| File | Resources | Changes |
|------|-----------|---------|
| `infra/main.tf` | Backend (S3+DynamoDB), provider | `required_version` only |
| `infra/variables.tf` | 9 variables | None |
| `infra/outputs.tf` | 5 outputs | None |
| `infra/lambda.tf` | Lambda function, Function URL, 2 permissions, CW log group | None |
| `infra/efs.tf` | EFS FS, access point, 3 mount targets, 2 SGs, 2 SG rules | None |
| `infra/s3.tf` | Backup bucket, versioning, encryption, lifecycle, public-access block | None |
| `infra/iam.tf` | Execution role, 2 managed attachments, 3 inline policies | None |
| `infra/ssm.tf` | 3 SSM parameters | None |
| `infra/eventbridge.tf` | Scheduled rule, target, Lambda permission | None |
| `infra/cdn.tf` | CloudFront distribution, 2 OACs, 6 Route53 records, origin request policy | Added `dev.${var.domain_name}` alias; switched cert to `aws_acm_certificate_validation.wildcard`; removed `acm_certificate_arn` variable; added `dev_a`+`dev_aaaa` Route53 records |
| `infra/acm.tf` | `aws_acm_certificate`, DNS validation records, `aws_acm_certificate_validation` | New file — manages `sislam.com + *.sislam.com` wildcard cert via DNS validation in Route53 |

---

## W-OTF.3 Implementation Notes

### 3.1 HCL Change — `infra/main.tf`

```hcl
# Before
terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
  ...
}

# After
terraform {
  required_version = ">= 1.6"   # OpenTofu first stable release
  required_providers {
    aws = {
      source  = "hashicorp/aws"  # OpenTofu registry resolves this natively
      version = "~> 5.0"
    }
  }
  ...
}
```

The `ManagedBy = "Terraform"` tag in `locals.common_tags` (main.tf) should also be updated to
`"OpenTofu"` for resource tagging accuracy.

### 3.2 xtask — `xtask/src/infra/terraform.rs` → `tofu.rs`

Rename file and update all internals:

```rust
// Before (terraform.rs)
fn make_cmd(dir: Option<String>, profile: Option<String>) -> (Command, String) {
    let mut cmd = Command::new("terraform");
    ...
}
pub async fn run_terraform_init(...) -> anyhow::Result<()> { ... }
pub async fn run_terraform_plan(...) -> anyhow::Result<()> { ... }
// etc.

// After (tofu.rs)
fn make_cmd(dir: Option<String>, profile: Option<String>) -> (Command, String) {
    let mut cmd = Command::new("tofu");
    ...
}
pub async fn run_tofu_init(...) -> anyhow::Result<()> { ... }
pub async fn run_tofu_plan(...) -> anyhow::Result<()> { ... }
// etc.
```

### 3.3 xtask — `xtask/src/infra/mod.rs`

Update `pub mod terraform;` → `pub mod tofu;` and all call sites:

```rust
// Before
pub mod bootstrap;
pub mod terraform;
// ...
InfraAction::Init { dir, profile } => terraform::run_terraform_init(dir, profile).await,

// After
pub mod bootstrap;
pub mod tofu;
// ...
InfraAction::Init { dir, profile } => tofu::run_tofu_init(dir, profile).await,
```

### 3.4 xtask — `xtask/src/infra/bootstrap.rs`

One call site to update:

```rust
// Before
crate::infra::terraform::run_terraform_init(None, profile).await?;

// After
crate::infra::tofu::run_tofu_init(None, profile).await?;
```

### 3.5 Binary Detection

Add a `tofu_check()` helper (called from `bootstrap_account` and the plan/apply paths):

```rust
/// Verify `tofu` binary is available.
fn check_tofu_binary() -> anyhow::Result<()> {
    let output = Command::new("tofu").arg("version").output();
    match output {
        Ok(o) if o.status.success() => Ok(()),
        _ => Err(anyhow::anyhow!(
            "tofu binary not found. Install with: brew install opentofu"
        )),
    }
}
```

Call this at the top of `make_cmd`'s first use, or as a preflight in `run_tofu_init`.

### 3.6 State Migration (existing Terraform-managed infra)

If the S3 state bucket `deploy-baba-tfstate` already contains a `deploy-baba/terraform.tfstate`
written by Terraform, no migration is needed. OpenTofu reads the same state format:

```
# One-time: verify zero diff after switching
just infra-plan PROFILE
# If plan shows 0 changes → migration complete
```

If Terraform's `.terraform.lock.hcl` provider lock file exists in `infra/`, delete it first —
OpenTofu will regenerate it on `tofu init`:

```bash
rm infra/.terraform.lock.hcl
just infra-bootstrap PROFILE   # runs tofu init
```

### 3.7 CI / Installation

```bash
# macOS (dev)
brew install opentofu

# Linux (CI / cross-build container)
curl -fsSL https://get.opentofu.org/install-opentofu.sh | sh -s -- --install-method standalone
# or via apt/yum repos — see OpenTofu install docs
```

---

## W-OTF.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-OTF.4.1 | Install `tofu` binary locally | DONE | OpenTofu v1.11.5 confirmed via `tofu version` |
| W-OTF.4.2 | Rename `xtask/src/infra/terraform.rs` → `tofu.rs` | DONE | `run_tofu_*` functions; `tofu` binary; `terraform.rs` deleted |
| W-OTF.4.3 | Update `infra/main.tf` version constraint | DONE | `required_version = ">= 1.6"`; `ManagedBy = "OpenTofu"` |
| W-OTF.4.4 | Update `xtask/src/infra/mod.rs` + `bootstrap.rs` | DONE | `mod tofu`; all call sites updated |
| W-OTF.4.5 | Add `check_tofu_binary()` preflight | DONE | In `tofu.rs`; called at top of every `run_tofu_*` function |
| W-OTF.4.6 | Delete `infra/.terraform.lock.hcl` if present | N/A | Lock file was never committed (DRL-2026-03-25-opentofu entry 6) |
| W-OTF.4.7 | Run `just infra-bootstrap PROFILE` with tofu | DONE | `just infra-plan deploy-baba` runs clean (2026-05-01). Pre-existing HCL blockers fixed — see DRL-2026-05-01-infra-plan-blockers. |
| W-OTF.4.8 | Mark W-TF as superseded in INDEX.md | DONE | Done in plans/modules/terraform.md and INDEX.md |
| W-OTF.4.9 | Update docs — `terraform` → `tofu`/`OpenTofu` in prose | DONE | 9 files updated; see W-OTF.4.9 Detail below |

### W-OTF.4.9 Detail — Doc Update Audit (2026-03-26)

**Already clean (no action):**
- `README.md` — 0 terraform references
- `PLAN.md` — archived
- `plans/modules/terraform.md`, `plans/drift/DRL-*`, `plans/adr/ADR-*` — historical/intentional

**Files to update (prose only, 9 files):**

| File | Line(s) | Current | Target | Notes |
|------|---------|---------|--------|-------|
| `CLAUDE.md` | 47 | `Infrastructure managed via Terraform` | `via OpenTofu` | |
| `CLAUDE.md` | 70 | `# Terraform (Lambda + EFS + …)` | `# OpenTofu (…)` | |
| `CLAUDE.md` | 86 | `Terraform plan/apply` | `OpenTofu plan/apply` | |
| `docs/aws-setup.md` | 28 | `full Terraform provisioning` | `full OpenTofu provisioning` | IAM SIDs unchanged (deployed names) |
| `docs/architecture.md` | 72 | `Terraform, deployment` | `OpenTofu, deployment` | |
| `justfile` | 104 | `reads from Terraform outputs` | `OpenTofu outputs` | comment only |
| `justfile` | 136 | `Infrastructure (Terraform)` | `Infrastructure (OpenTofu)` | comment only |
| `justfile` | 154 | `Show Terraform outputs` | `Show OpenTofu outputs` | comment only |
| `plans/modules/xtask.md` | 11 | `Wraps cargo, terraform, AWS SDK` | `cargo, tofu, AWS SDK` | |
| `plans/modules/xtask.md` | 43-45 | file tree: `terraform.rs` | `tofu.rs`; update descriptions | |
| `plans/modules/xtask.md` | 76 | `Run terraform -chdir=infra init` | `tofu -chdir=infra init` | |
| `plans/modules/xtask.md` | 97 | `Terraform subprocess pattern` | `OpenTofu subprocess pattern` | |
| `plans/cross-cutting/dependency-graph.md` | 99 | `terraform \| brew install terraform` | `tofu \| brew install opentofu` | |
| `plans/cross-cutting/aws-architecture.md` | 117 | `Terraform Resources (28 total)` | `OpenTofu Resources (28 total)` | |
| `plans/INDEX.md` | 113 | `# Terraform (Lambda + EFS + …)` | `# OpenTofu (…)` | |
| `plans/modules/dx-justfile.md` | 71 | `Infrastructure (Terraform)` | `Infrastructure (OpenTofu)` | |

**Intentionally NOT changed:**
- AWS resource names: DynamoDB `terraform-lock`, S3 key `terraform.tfstate`, IAM SIDs — deployed names
- HCL `terraform {}` block — syntax, not product name
- Archived/historical: PLAN.md, terraform.md, drift logs, ADRs

### Dependency Order
```
W-OTF.4.1 (install binary)
    └─► W-OTF.4.2 (rename xtask module)
    └─► W-OTF.4.3 (update main.tf)
         └─► W-OTF.4.4 (update mod.rs + bootstrap.rs)
              └─► W-OTF.4.5 (add binary check)
                   └─► W-OTF.4.6 (delete old lock file)
                        └─► W-OTF.4.7 (smoke test)
                             └─► W-OTF.4.8 + W-OTF.4.9
```

---

## W-OTF.5 Test Strategy

1. **Compile check:** `cargo build --package xtask` passes after module rename
2. **Binary check:** `tofu version` prints version string (≥ 1.6.0)
3. **Plan parity:** `just infra-plan PROFILE` shows 0 changes vs existing state
4. **Bootstrap smoke test:** `just infra-bootstrap PROFILE` completes without error on a clean account
5. **Lint:** `cargo clippy --package xtask` passes (no dead code from renamed functions)

No coverage floor (internal tooling), consistent with W-XT strategy.

---

## W-OTF.6 Cross-References

- → ADR-007 (decision record)
- → W-TF (superseded module — same HCL, replaced binary)
- → W-XT (xtask crate being modified)
- → W-DX (justfile commands unchanged; docs need prose update)
- → `plans/drift/DRL-2026-03-25-opentofu.md` — observed drift and already-fixed items
- → `plans/cross-cutting/aws-architecture.md` — topology unchanged
- → `plans/cross-cutting/aws-setup-spec.md` — IAM permissions unchanged
- → ADR-008 (Cognito infra resources managed by OpenTofu)
- → ADR-009 (API Gateway infra managed by OpenTofu)
- → ADR-019 (SPA deploy pipeline — S3 + CloudFront infra managed by OpenTofu)
- → ADR-020 (wildcard ACM cert — managed by OpenTofu)
