# ADR-021: Automated Release Tagging via xtask

**Status:** Accepted
**Date:** 2026-04-30
**Affected modules:** W-CI, W-XT
**Imported from:** njnewsroomproject ADR-016 (adapted for deploy-baba scope)

---

## Context

There is no automated release tagging in the repo. Production is deployed manually from a developer laptop via `just deploy PROFILE`. Two problems:

1. No git-native audit trail of what code is running in prod — deployments are anonymous.
2. The prod gate is procedural (remember to deploy) not structural (automation enforces it).

With the CI/CD pipeline from ADR-020, every merge to `main` auto-deploys to dev. But promoting to prod still needs a principled version decision and a structural gate.

---

## Decision

**Two-step tagged release with conventional-commits versioning, implemented in `xtask/src/release/`.**

### Tag scheme

| Tag pattern | Who creates it | When |
|---|---|---|
| `dev-vX.Y.Z` | CI automatically (`deploy-dev.yml`) | After every successful dev deploy — the tag is a record; Lambda is already deployed before the tag is pushed |
| `vX.Y.Z` | Developer manually (`just release-promote`) | Before a prod release; triggers `deploy-prod.yml` |

`deploy-prod.yml` triggers on `push: tags: ['v*']` — no changes needed to that pattern.

### Versioning rule (conventional commits)

Bump kind derived from commit subjects since the last `dev-v*` tag:

| Subject pattern | Bump |
|---|---|
| Any type with `!:` marker, or body contains `BREAKING CHANGE:` | major |
| `feat:` | minor |
| `fix:`, `refactor:`, `perf:` | patch |
| `docs:`, `chore:`, `style:`, `test:`, `build:`, `ci:` | skip (patch if all-skip) |
| Anything else | patch (defensive default) |

Floor version: `xtask/Cargo.toml [package] version` (currently `0.1.0`). Git tags are the source of truth; `Cargo.toml` version fields are NOT bumped by the tool.

### Implementation

Logic lives in `xtask/src/release/` (Rust). Locally runnable, testable, identical between laptop and CI.

```bash
just release-next                    # dry-run: print next version
just release-tag KIND=dev [PUSH=1]   # create dev-vX.Y.Z tag (CI uses this)
just release-promote [PUSH=1]        # create vX.Y.Z from latest dev-v* (developer uses this)
```

### Loop safety

`deploy-dev.yml` triggers on `workflow_run: workflows: [CI]`. Tag pushes do **not** retrigger CI (CI triggers on `push: branches` and `pull_request`). `dev-v*` tags do not match `deploy-prod.yml` (`'v*'` matches `v0.1.0` but not `dev-v0.1.0`). No infinite loop is possible.

### Idempotency

`release tag` is a no-op if the tag already exists at HEAD (two concurrent CI runs for the same commit). It errors if the tag exists at a different commit.

### Human-in-the-loop for prod

Two explicit gates before code reaches prod:
1. **Developer intent:** `just release-promote --push` creates the `vX.Y.Z` tag and pushes it.
2. **Manual approval:** `deploy-prod.yml` is gated by the GitHub `production` environment (Required Reviewers setting). The workflow queues and does not run until a reviewer clicks Approve in the GitHub UI.

---

## Consequences

- Every dev deploy is auditable: `git tag --list 'dev-v*'` shows the full deployment history.
- Prod deployments require two deliberate human actions — no accidental prod deploy.
- Version bumps are automatic and grounded in commit subjects — no manual version decision required.
- `xtask` gains a meaningful `release` subcommand (partially closes W-XT.4.2 by giving dead capacity real work).
- `deploy-dev.yml` requires `contents: write` permission and `fetch-depth: 0` checkout.

---

## Cross-References
- → ADR-001 (justfile-only interface — `just release-*` recipes)
- → ADR-020 (CI OIDC — `deploy-dev.yml` calls `xtask release tag`)
- → `plans/modules/ci.md` (W-CI work items include the xtask release subcommand)
- → `plans/modules/xtask.md` (W-XT — xtask/src/release/ added here)
