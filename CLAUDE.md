# CLAUDE.md — Portfolio Project Instructions

This file provides guidance to Claude Code when working in this repository.
It includes global instructions (inlined from `~/CLAUDE.md`) plus project-specific context.

---

## ⚡ Agent Cache Protocol — READ THIS FIRST ON EVERY STARTUP

Before exploring any files, always run the cache check sequence:

```
1. Read `.agent-cache/index.json`          ← full project knowledge snapshot
2. Run: git rev-parse HEAD                 ← get current SHA
3. Compare SHA to index.json `git.sha`
```

**If SHAs match** → cache is fresh. Use `.agent-cache/index.json` as ground truth.
Skip re-reading Cargo.toml files, plans, crate structure, ADRs, and infra layout.
Only read source files for the specific task at hand.

**If SHAs differ** → cache is stale for changed components only.
Run: `git diff --name-only <cached_sha> HEAD` to find changed files.
Re-read only the files in changed components. Skip everything else.

**After any new discovery** → update `.agent-cache/index.json` with findings.
Update `git.sha`, `last_updated`, and the relevant component's `git_sha_at_scan`.

### What the cache contains

| Key | Contents |
|-----|----------|
| `project` | Name, status, tech stack, task runner |
| `crates.*` | Per-crate: description, dependencies, dependents, role, open issues |
| `services.ui` | Routes dir, framework (JSON API), auth, open issues |
| `infra` | OpenTofu files, AWS resources, open issues |
| `plans` | Priority queue (P0→P3), module status, ADRs |
| `database` | Engine, location, migration path |
| `key_commands` | All `just` commands |
| `known_patterns` | Error handling, async, templating conventions |
| `adrs` | All 6 architecture decisions at a glance |

### Cache management
- `just cache-status` — show cache age and staleness vs current HEAD
- `just cache-refresh` — re-scan the codebase and rewrite the cache
- `just cache-clear` — delete cache to force a full re-scan next session

---

---

## ⚖️ Engineering Principles — These Are Imperatives

**Follow these unconditionally when making any design, implementation, or architectural decision in this project.**

- **Plan deliberately, but assume plans will evolve.** Commit to a direction early enough to act, but treat every plan as a living document. Update `plans/INDEX.md` and module files as understanding improves — never let the plan drift silently from reality.

- **Validate dependencies early, and continuously refine through execution.** Don't defer integration risk. Wire up external boundaries (AWS, Cognito, SQLite, SES) in the earliest possible iteration and let real behavior drive plan updates.

- **Prefer boring infrastructure, explicit systems, and strong type guarantees over hidden complexity.** SQLite over RDS. OpenTofu HCL over dynamic abstractions. Rust types over runtime checks. If a solution is hard to explain, it's probably wrong.

- **Optimize for clarity, debuggability, and long-term maintainability.** Write code that is obvious to read six months later. Name things precisely. Avoid clever shortcuts that collapse under pressure. A longer-but-readable function beats a compact-but-opaque one every time.

- **Treat AI as a collaborator, not an oracle — design systems that verify, constrain, and observe its outputs.** Agent cache, plan modules, and ADRs exist to provide ground truth. Don't generate code in a vacuum; validate against the plan, run `just quality`, and confirm intent with the human.

- **Build feedback loops into every layer.** Compile-time: Rust types and `cargo clippy`. Test-time: `just dev` and coverage floors. Deploy-time: `just infra-plan` before apply. Runtime: CloudWatch logs via `just ui-logs`. User-level: observable state, not silent failures.

- **Every abstraction should earn its place — and justify its cost over time.** No xtask module without a justfile entry. No plan module without work items. No infra resource without an ADR or drift log. Challenge every layer of indirection: if it doesn't reduce real complexity, remove it.

---

## About Me

- **Name:** shantopagla
- **Email:** it@shantopagla.com
- **GitHub:** sislam-com
- **Primary Language:** Rust (also uses TypeScript/Node.js, Python)
- **Cloud Platform:** AWS (us-east-1)
- **OS:** macOS
- **Shell:** zsh
- **Package Managers:** cargo, npm (via nvm), pip (via pyenv), brew

## Development Environment

- **Rust:** Managed via rustup, cargo in `~/.cargo/bin`
- **Node.js:** Managed via nvm (`~/.nvm`)
- **Python:** Managed via pyenv (`~/.pyenv`)
- **Editor:** VS Code / Cursor
- **Git:** Default branch is `main`, uses git-lfs

## Coding Preferences

- Write clean, idiomatic code for the target language
- Prefer Rust's trait-based composition and zero-cost abstractions
- Use `thiserror` for error handling in Rust, not `anyhow` in library crates
- Follow existing project conventions and patterns
- Always run `cargo fmt` and `cargo clippy` before committing Rust code
- Use `just` commands when available in a project
- Prefer async/await patterns with tokio

## Git Conventions

- Commit messages: concise, imperative mood ("Add feature" not "Added feature")
- Default branch: `main`
- Use conventional commit prefixes when appropriate: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`

## AWS & Infrastructure

- Default region: `us-east-1`
- Uses AWS SSO for authentication
- Infrastructure managed via OpenTofu (generated from Rust types)
- Deployment philosophy: zero-cost first, scale up only when needed

## Security Notes

- Never commit credentials, API keys, or secrets to git
- Use environment variables or AWS Secrets Manager for sensitive values
- Check `.gitignore` before staging files

---

## Project: deploy-baba

Zero-cost Rust portfolio and deployment automation platform hosted on AWS Lambda.

### Workspace Structure

```
portfolio/
├── crates/           # 10 library crates (pure Rust, no binaries)
├── services/ui/      # Lambda binary (the deployed service)
├── xtask/            # Internal CLI — do NOT call directly
├── examples/         # 4 example binaries
├── infra/            # OpenTofu (Lambda + EFS + S3 + EventBridge + CloudFront)
├── plans/            # Modular plan system (see plans/INDEX.md)
├── stack.toml        # Local-only config (copy from stack.example.toml)
└── justfile          # The only interface — use `just` commands
```

### Task Runner — `just` is the only interface (ADR-001)

Never call `cargo xtask` directly. All commands go through `just`.

Key commands:
- `just dev` — inner development loop
- `just quality` — full quality gate (fmt + clippy + test)
- `just lambda-deploy PROFILE` — build + upload Lambda zip (no infra changes)
- `just ui` — run local UI server
- `just lambda-build` — build Lambda binary (uses cargo-lambda for aarch64)
- `just infra-plan PROFILE` / `just infra-apply PROFILE` — OpenTofu plan/apply
- `just secret-put NAME VALUE PROFILE` — write secret to AWS Secrets Manager (W-SEC, TODO)
- `just secret-get NAME PROFILE` — read secret from AWS Secrets Manager (W-SEC, TODO)

### Architecture Decisions

- **ADR-001:** justfile-only interface — xtask is internal plumbing, never invoked directly
- **ADR-002:** SQLite on EFS + S3 backup — no PostgreSQL, no RDS
- **ADR-003:** Lambda Function URL — no API Gateway (exception: ADR-009)
- **ADR-004:** Dual-mode entry point — runtime env var detection (local vs Lambda)
- **ADR-007:** OpenTofu over Terraform — `tofu` CLI binary, MPL-2.0
- **ADR-008:** Cognito hosted UI auth — implicit grant, JWKS from env, HttpOnly cookie, dev-mode bypass
- **ADR-009:** API Gateway HTTP API for `POST /api/contact` only — OAC body hash workaround
- **ADR-024:** API Versioning Strategy — URL-based versioning with Function URL routing; deprecation headers
- **ADR-025:** SQLite-Based Metrics Collection — Zero-cost observability via SQLite metrics tables
- **ADR-026:** Code-Level Resilience Patterns — In-memory rate limiting; retry; circuit breaker
- **ADR-027:** Module-Based Service Decomposition — Logical separation within single Lambda

### Stack Config (`stack.toml`)

Local-only config file — not committed to git. Contains:
- Project metadata, deploy mode, database path
- Observability settings, AWS profile

Copy `stack.example.toml` to `stack.toml` to get started. No external dependencies or remote service URLs.

### Secrets Policy

**All secrets must live in AWS Secrets Manager** (W-SEC). Never store secrets in:
- Source code or hardcoded fallbacks (except `dev-*` local-only defaults)
- Lambda environment variables (visible in AWS console)
- Committed files of any kind

Use `just secret-put NAME VALUE PROFILE` to write secrets, `just secret-get NAME PROFILE` to read.
Lambda reads secrets via `POW_SECRET_ARN` env var at cold start. See `plans/modules/secrets-manager.md`.

### Plan System

Entry point: `plans/INDEX.md` — lists all modules, ADRs, cross-cutting concerns, and drift logs.
**The plan system is the single source of truth for project state.** Keep it updated.

Structure under `plans/`:
- `modules/` — per-component plans (34 modules incl. ai-dlc, ci, web, dev-environment, api-versioning, observability, resilience, module-decomposition)
- `adr/` — architecture decision records (ADR-001 through ADR-027)
- `cross-cutting/` — 11 shared concern files (incl. ai-dlc.md, initial-setup.md, zero-cost-microservices.md)
- `drift/` — drift logs (format: `DRL-YYYY-MM-DD-topic`)

AI-DLC session lifecycle: `plans/cross-cutting/ai-dlc.md` — covers the 6 stages (Startup → Planning → Implementation → Verification → Maintenance → Commit). Run `/plan-sync` at the end of any implementation session to sync module Status fields, INDEX.md, and ADR back-references.

Current status: ~93% complete. Remaining work listed in P0.1–P3 sections of `plans/INDEX.md`.

### Cross-Session Memory

Claude's auto-memory for this project: `~/.claude/projects/-Users-shantopagla-portfolio/memory/`
