# ADR-007: OpenTofu Over Terraform

**Date:** 2026-03-25
**Status:** Accepted
**Affected Modules:** W-OTF (replaces W-TF), W-XT

---

## Context

The project currently uses HashiCorp Terraform (`terraform` binary) to manage all AWS infrastructure
in `infra/`. In August 2023, HashiCorp relicensed Terraform from MPL-2.0 to the Business Source
License (BUSL 1.1), which restricts use in competing services. The open-source community forked the
final MPL-licensed Terraform (1.5.x) as **OpenTofu**, now governed by the Linux Foundation.

OpenTofu is a drop-in replacement:
- Identical HCL syntax and provider protocol
- Compatible S3 backend and state format
- `hashicorp/aws` provider resolves from `registry.opentofu.org` under the same namespace
- CLI binary is `tofu` instead of `terraform`
- First stable release: 1.6.0 (January 2024); current: ~1.8.x

The xtask infra wrapper (`xtask/src/infra/terraform.rs`) spawns `terraform` as a subprocess.
Switching to OpenTofu requires only:
1. Replacing `terraform` with `tofu` in the subprocess command
2. Updating `required_version` in `main.tf`
3. Renaming the xtask module for clarity

No HCL resource definitions need to change. State is format-compatible.

---

## Decision

Replace the `terraform` binary dependency with `opentofu` (`tofu` CLI) for all infrastructure
management operations. The xtask infra module is refactored to call `tofu` and renamed accordingly.

---

## Consequences

**Positive:**
- MPL-2.0 license — no BUSL concerns for open-source publishing
- Active community development (Linux Foundation governance)
- Identical developer experience (`tofu plan`, `tofu apply`)
- No HCL changes required (100% syntax-compatible)
- State files are forward-compatible (OpenTofu can import existing Terraform state)

**Negative:**
- Requires `tofu` binary installation on dev machines and CI (brew install opentofu)
- Minor: `terraform` binary references in docs, error messages, and xtask must be updated

**Neutral:**
- Provider source `hashicorp/aws ~> 5.0` continues to work unchanged
- S3 backend configuration is identical

---

## Migration Path

For a fresh deployment (no existing Terraform state):
1. Install `tofu` (`brew install opentofu`)
2. Apply xtask changes (W-OTF.4.2–4.4)
3. Update `main.tf` `required_version` (W-OTF.4.3)
4. Run `just infra-bootstrap PROFILE` as before

For migration from existing Terraform-managed state:
1. The S3 state file (`deploy-baba/terraform.tfstate`) is directly consumable by `tofu`
2. Run `tofu init` to wire up the backend — no state migration step needed
3. Run `tofu plan` to verify zero diff before first `tofu apply`

---

## Cross-References

- → W-OTF (implementation plan)
- → W-XT (xtask infra module changes)
- → ADR-001 (justfile is the only interface — unchanged)
- → ADR-003 (Lambda Function URL — unchanged)
- → ADR-006 (EFS + S3 topology — unchanged)
