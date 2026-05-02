# DRL-2026-05-02-bootstrap-terraform-docstring

**ADR:** ADR-007 | **Detected:** 2026-05-02 | **Severity:** Minor (doc-only)

## Divergence

`xtask/src/infra/bootstrap.rs` module-level doc comment (lines 3 and 5) still reads:

> "Creates the Terraform remote state backend … then runs `terraform init` to wire up the backend."

Line 12 also names the DynamoDB constant `LOCK_TABLE = "terraform-lock"`.

The actual implementation correctly calls `crate::infra::tofu::run_tofu_init()` (not the `terraform` binary). The xtask module was renamed `terraform.rs` → `tofu.rs` and all subprocess invocations updated to `tofu`, but the module-level prose was not updated in that pass.

## Impact

Documentation-level only. The `LOCK_TABLE` string governs the real DynamoDB table name in AWS — any future `infra-bootstrap` run will continue creating/referencing a table named `terraform-lock`, which is an identity inconsistency but not a runtime defect (the table name is stable and changing it would require a state migration).

A reader skimming `bootstrap.rs` for orientation would conclude `terraform` is still in use, contradicting ADR-007.

## Recommended Fix

1. Update the module doc comment: s/Terraform/OpenTofu/, s/`terraform init`/`tofu init`/.
2. Rename `LOCK_TABLE` from `"terraform-lock"` to `"opentofu-lock"` only if a matching infra rename is acceptable — otherwise leave the string as-is with an explanatory comment.
