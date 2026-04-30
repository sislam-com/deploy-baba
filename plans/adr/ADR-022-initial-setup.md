# ADR-022: Developer First-Run Environment

**Status:** Accepted
**Date:** 2026-04-30
**Affected modules:** W-DEV, W-DX
**Imported from:** njnewsroomproject cross-cutting/initial-setup.md (elevated to ADR for deploy-baba scope)

---

## Context

There is no official first-run guide for the portfolio repo. The only reference is `docs/aws-setup.md` (AWS-specific steps) and the `CLAUDE.md` Agent Cache Protocol. A developer cloning the repo from scratch must discover the prerequisites by trial and error. Specific gaps:

- No idempotent script to bootstrap the OpenTofu state backend (S3 bucket + DynamoDB lock table).
- No checklist to verify prerequisites (`rustup`, `pnpm`, `tofu`, `cargo-lambda`, `aws sso`).
- No `.devcontainer/` for GitHub Codespaces or remote dev container parity.
- The SPA work (ADR-019) adds `pnpm` and Node 20 as new prerequisites that must be documented.

---

## Decision

Provide a first-run guide as an authoritative `plans/cross-cutting/initial-setup.md` document, backed by two idempotent scripts and a devcontainer definition.

### Scripts

`scripts/bootstrap-tfstate.sh` — idempotent; run once per AWS account. Creates:
- S3 bucket `deploy-baba-tfstate-${ACCOUNT_ID}` with versioning enabled.
- DynamoDB table `terraform-lock` (shared with other projects in the account; skips creation if already exists).

`scripts/dev-doctor.sh` — verifies all prerequisites are installed and prints a status table. Checks: `rustup`, `cargo-lambda`, `pnpm`, `node ≥20`, `tofu`, `aws sts get-caller-identity --profile deploy-baba` (verifies SSO session). Exits 0 if all green, 1 if any check fails (suitable as a `postCreateCommand`).

### Justfile recipes

```
just dev-doctor        # run scripts/dev-doctor.sh — verify prerequisites
just infra-bootstrap   # run scripts/bootstrap-tfstate.sh — one-time state backend setup
```

### Devcontainer

`.devcontainer/devcontainer.json` — enables GitHub Codespaces and VS Code Remote Container workflows:
- Base: `mcr.microsoft.com/devcontainers/rust:1`
- Features: `ghcr.io/devcontainers/features/node:1` (v20), `ghcr.io/devcontainers/features/aws-cli:1`, `ghcr.io/devcontainers/features/github-cli:1`
- `postCreateCommand`: `bash scripts/dev-doctor.sh && cargo build --workspace && pnpm --dir web install`
- Port forwards: 3000 (Lambda binary), 5173 (Vite dev server)

### First-run guide

`plans/cross-cutting/initial-setup.md` — step-by-step guide covering prerequisites, AWS SSO setup, one-time bootstrap, and local dev loop. Cross-references `docs/aws-setup.md` for IAM detail.

---

## Consequences

**Positive:**
- Cold clone to working local dev is a documented, reproducible procedure.
- `scripts/dev-doctor.sh` is suitable as a CI pre-step or Codespaces `postCreateCommand`.
- `scripts/bootstrap-tfstate.sh` can be run safely N times without side effects.
- `.devcontainer/` enables Codespaces — no local toolchain required for contributions.

**Negative:**
- `scripts/` directory adds maintenance surface (two small shell scripts).
- Devcontainer Docker image pull adds latency on first launch (~5 min Codespaces cold start).

---

## Cross-References
- → ADR-001 (justfile-only interface — `just dev-doctor` and `just infra-bootstrap`)
- → ADR-007 (OpenTofu — bootstrap-tfstate.sh sets up the OTF state backend)
- → ADR-019 (SPA replaces Askama — pnpm prerequisite documented here)
- → `plans/modules/dev-environment.md` (W-DEV work items)
- → `plans/cross-cutting/initial-setup.md` (the guide this ADR mandates)
- → `docs/aws-setup.md` (IAM setup detail)
