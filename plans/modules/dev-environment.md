# W-DEV: Developer Environment
**Path:** `scripts/`, `.devcontainer/`, `docs/`, `plans/cross-cutting/initial-setup.md` | **Status:** DONE
**Coverage floor:** n/a | **Depends on:** W-DX, W-OTF | **Depended on by:** W-CI (bootstrap-tfstate.sh used in one-time setup before CI can apply infra)

## W-DEV.1 Purpose

Provide an authoritative, reproducible first-run experience for a developer cloning the repo:
1. Prerequisite verification (`scripts/dev-doctor.sh`).
2. One-time idempotent infrastructure bootstrap (`scripts/bootstrap-tfstate.sh`).
3. A `.devcontainer/` for GitHub Codespaces / VS Code Remote Containers parity with local macOS.
4. A documented guide (`plans/cross-cutting/initial-setup.md`) that ties these together.

Motivated by the SPA work (ADR-019) adding `pnpm` + Node 20 as new prerequisites, and the CI/CD work (ADR-020) requiring a one-time infra bootstrap step before OIDC roles can be applied.

## W-DEV.2 Public Surface

| Artifact | Path | Purpose |
|---|---|---|
| `scripts/bootstrap-tfstate.sh` | `scripts/bootstrap-tfstate.sh` | Idempotent: creates tfstate S3 bucket + DynamoDB lock table |
| `scripts/dev-doctor.sh` | `scripts/dev-doctor.sh` | Verifies all prerequisites; exits 1 on failure |
| `devcontainer.json` | `.devcontainer/devcontainer.json` | Codespaces + VS Code Remote Container definition |
| `initial-setup.md` | `plans/cross-cutting/initial-setup.md` | Authoritative first-run guide |
| `just dev-doctor` | `justfile` | `bash scripts/dev-doctor.sh` |
| `just infra-bootstrap` | `justfile` | `bash scripts/bootstrap-tfstate.sh` |

## W-DEV.3 Implementation Notes

### `scripts/bootstrap-tfstate.sh`

Adapted from `~/njnewsroomproject/scripts/bootstrap-tfstate.sh`. Key behaviour:
- Reads `ACCOUNT_ID` from `aws sts get-caller-identity --query Account`.
- Creates `deploy-baba-tfstate-${ACCOUNT_ID}` bucket in `us-east-1` with versioning enabled. Skips if already exists.
- Creates `terraform-lock` DynamoDB table (PAY_PER_REQUEST, LockID string hash key). Skips if already exists — this table is shared with other projects in the account.
- Prints a status table (✓/✗ per action).
- Exits 0 if all green, 1 if any step fails.

```bash
#!/usr/bin/env bash
set -euo pipefail
PROFILE="${AWS_PROFILE:-deploy-baba}"
REGION="${AWS_REGION:-us-east-1}"
ACCOUNT_ID=$(aws sts get-caller-identity --profile "$PROFILE" --query Account --output text)
BUCKET="deploy-baba-tfstate-${ACCOUNT_ID}"
TABLE="terraform-lock"
# … create bucket + enable versioning + create table …
```

### `scripts/dev-doctor.sh`

Checks each tool and prints a colour status table. Checks:
- `rustup` — version
- `cargo-lambda` — version (required for Lambda builds)
- `pnpm` — version ≥8 (required for SPA; added by ADR-019)
- `node` — version ≥20
- `tofu` — version (OpenTofu binary)
- `aws sts get-caller-identity --profile deploy-baba` — SSO session active
- `.agent-cache/index.json` vs `git rev-parse HEAD` — cache freshness

### `.devcontainer/devcontainer.json`

```json
{
  "name": "deploy-baba",
  "image": "mcr.microsoft.com/devcontainers/rust:1",
  "features": {
    "ghcr.io/devcontainers/features/node:1": { "version": "20" },
    "ghcr.io/devcontainers/features/aws-cli:1": {},
    "ghcr.io/devcontainers/features/github-cli:1": {}
  },
  "postCreateCommand": "bash scripts/dev-doctor.sh && cargo build --workspace && pnpm --dir web install",
  "forwardPorts": [3000, 5173],
  "portsAttributes": {
    "3000": { "label": "Lambda (local)" },
    "5173": { "label": "Vite dev server" }
  },
  "customizations": {
    "vscode": {
      "extensions": ["rust-lang.rust-analyzer", "bradlc.vscode-tailwindcss", "dbaeumer.vscode-eslint"]
    }
  }
}
```

OpenTofu is not installed as a devcontainer feature (no official feature exists). Add via `postCreateCommand` `sudo apt-get install -y opentofu` or the opentofu install script.

## W-DEV.4 Work Items

| ID | Task | Status | Notes |
|---|---|---|---|
| W-DEV.4.1 | `scripts/bootstrap-tfstate.sh` | DONE | Idempotent; adapted from njnewsroomproject; just infra-bootstrap now calls this |
| W-DEV.4.2 | `scripts/dev-doctor.sh` | DONE | Checks rustup/cargo-lambda/node≥20/pnpm/tofu/AWS SSO/cache freshness |
| W-DEV.4.3 | `.devcontainer/devcontainer.json` | DONE | Node 20 + aws-cli + github-cli features + opentofu install |
| W-DEV.4.4 | `plans/cross-cutting/initial-setup.md` | DONE | Created in Phase A |
| W-DEV.4.5 | `just dev-doctor` recipe | DONE | `bash scripts/dev-doctor.sh` |
| W-DEV.4.6 | `just infra-bootstrap` recipe | DONE | `bash scripts/bootstrap-tfstate.sh` (updated from xtask call) |

## W-DEV.5 Test Strategy

- `bash scripts/bootstrap-tfstate.sh` — run twice; second run should print "already exists" for both resources and exit 0.
- `bash scripts/dev-doctor.sh` — run in a fresh Codespaces environment; all checks green after `postCreateCommand` completes.
- `just infra-bootstrap` — calls the script and passes through its exit code.
- `.devcontainer/` — open repo in GitHub Codespaces; `postCreateCommand` completes without error; `just dev` passes; Vite dev server starts on :5173; Lambda binary starts on :3000.

## W-DEV.6 Cross-References

- → ADR-001 (justfile-only interface — `just dev-doctor`, `just infra-bootstrap`)
- → ADR-007 (OpenTofu — bootstrap creates the OTF state backend)
- → ADR-019 (SPA replaces Askama — pnpm prerequisite documented here)
- → ADR-022 (first-run environment decision)
- → `docs/aws-setup.md` (IAM/SSO detail that initial-setup.md cross-references)
- → `plans/cross-cutting/initial-setup.md` (the guide this module creates)
