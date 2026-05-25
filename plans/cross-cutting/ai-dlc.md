# AI Development Lifecycle (AI-DLC) — deploy-baba

**Last updated:** 2026-04-30

This document describes how Claude Code (AI agents) participates in the development lifecycle for this repo. It is authoritative for any Claude session and supplements `CLAUDE.md`.

Adapted from `~/njnewsroomproject/plans/cross-cutting/ai-dlc.md` (the reference implementation).

---

## Role of AI in This Project

Claude Code acts as the primary development interface. It:
- Reads and updates the plan system (`plans/`)
- Runs the justfile commands (`just dev`, `just web-test`, `just quality`, etc.)
- Implements code, writes tests, and performs quality checks
- Manages the agent cache (`.agent-cache/index.json`)
- Persists cross-session knowledge in `~/.claude/projects/-Users-shantopagla-portfolio/memory/`

The operator (Shanto) approves plans, provides direction, reviews diffs, and triggers deployments.

---

## Session Lifecycle

### 1. Startup (every session)

```
1. Read .agent-cache/index.json           ← full project snapshot
2. git rev-parse HEAD                     ← current SHA
3. Compare SHA to index.json.git.sha
   - Match  → cache fresh; skip re-reads
   - Differ → git diff --name-only <sha> HEAD → re-read changed files only
4. Check memory: ~/.claude/projects/-Users-shantopagla-portfolio/memory/MEMORY.md
```

### 2. Planning (for non-trivial tasks)

Non-trivial = any change spanning more than one component, or touching infra/CI.

```
1. EnterPlanMode
2. Read `.agent-cache/index.json` — structural knowledge already cached
3. Query local MCP (`mcp-rs` resources, `portfolio-rag` tools) for context
4. Fall back to direct Read/Bash/grep only if MCP is unavailable
5. Design approach inline — identify files, note risks
6. Ask AskUserQuestion if approach is unclear
7. Write plan file to ~/.claude/plans/<slug>.md
8. ExitPlanMode → user approves
```

**Token budget:** 20k tokens max per request. Never spawn Explore or Plan subagents — they burn 80–110k tokens each. Use the local MCP pipeline first, then direct reads.

Trivial tasks (single-file typo, dependency bump, doc touch) may skip plan mode.

### 3. Implementation

```
1. Execute plan items in order
2. Run quality gates after each logical chunk (not after every file):
   just dev          ← Rust: fmt + clippy + test
   just web-test     ← Vitest (once web/ exists)
   just web-typecheck
3. If tests fail: fix before proceeding to the next item
```

### 4. Verification

Before reporting done:
```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
pnpm --dir web run test     # once web/ exists
pnpm --dir web run typecheck
```
For infra changes (no AWS credentials required in CI):
```bash
tofu -chdir=infra fmt -check -recursive
tofu -chdir=infra init -backend=false
tofu -chdir=infra validate -var="environment=prod"
```

### 5. Plan + cache maintenance

After implementation:
1. Update the relevant module's `Status` and work-item table in `plans/modules/<module>.md`.
2. Update `plans/INDEX.md` module status table.
3. Update `.agent-cache/index.json`: set `git.sha`, `last_updated`, and the affected component's `git_sha_at_scan`.

Run `/plan-sync` to automate Steps 1–3 (requires W-AIL.4.1–4.5 complete).

### 6. Commit (only when user asks)

Follow conventional commits (`feat:`, `fix:`, `chore:`, etc.). Never commit without user approval.

---

## Agent Cache Contract

`.agent-cache/index.json` is the session-to-session memory for project structure. It must be kept accurate.

**Fields that must stay current:**
- `git.sha` — HEAD SHA at last cache refresh
- `last_updated` — ISO timestamp
- Each component's `git_sha_at_scan` — SHA when that component was last scanned

**When to update:** After any implementation session that touches a component. Run `just cache-refresh` or `/cache-refresh`.

**Invalidation rule:** If `git.sha` != `git rev-parse HEAD`, diff the changed paths and re-scan only those components.

---

## Memory System

Claude's persistent memory lives in `~/.claude/projects/-Users-shantopagla-portfolio/memory/`.

Types used in this project:
- `user/` — Shanto's preferences and working style
- `feedback/` — corrections and validated approaches
- `project/` — current initiative state (decays fast; verify against git before trusting)
- `reference/` — pointers to external systems (AWS, GitHub)

Do not save: code patterns, file paths, git history, debugging recipes. These belong in the code or commit messages.

---

## Quality Gates

| Gate | Command | When |
|---|---|---|
| Rust format | `cargo fmt --all --check` | CI + pre-merge |
| Rust lint | `cargo clippy --workspace --all-targets -D warn` | CI + pre-merge |
| Rust tests | `cargo test --workspace` | CI + pre-merge |
| Security audit | `cargo audit` | CI weekly + pre-deploy |
| Web type check | `pnpm --dir web run typecheck` | CI + pre-merge (once web/ exists) |
| Web tests | `pnpm --dir web run test` | CI + pre-merge |
| Web build | `pnpm --dir web run build` | CI + pre-merge |
| Tofu format | `tofu -chdir=infra fmt -check -recursive` | CI + pre-merge |
| Tofu validate | `tofu -chdir=infra validate` | CI + pre-merge |

---

## Deployment Integration

Two deployment paths:
1. **Automated (CI):** merge to `main` → `deploy-dev.yml` auto-deploys to dev + tags `dev-vX.Y.Z`.
2. **Promoted (developer):** `just release-promote --push` → creates `vX.Y.Z` tag → `deploy-prod.yml` queues for manual approval.

Claude never pushes to the remote or tags releases without explicit user instruction.

---

## Maintenance Agents (ADR-018)

Stage 5 (Maintenance) is supported by five artifacts under domain code `AIL`. See `plans/modules/ai-dlc.md` for full detail.

**Subagents (read-only — never edit files):**

| Agent | File | Trigger |
|---|---|---|
| `plan-doctor` | `.claude/agents/plan-doctor.md` | Via `/plan-sync` or on demand |
| `drift-detector` | `.claude/agents/drift-detector.md` | Via `/plan-sync` or on demand |

**Skills (writes gated on user confirmation):**

| Command | File | When to use |
|---|---|---|
| `/plan-sync` | `.claude/skills/plan-sync/SKILL.md` | End of any session touching code or plans |
| `/cache-refresh` | `.claude/skills/cache-refresh/SKILL.md` | When cache SHA diverges from HEAD |
| `/memory-curate` | `.claude/skills/memory-curate/SKILL.md` | After major milestones; weekly via schedule |

**Schedule:** `dbb-plan-sync` + `dbb-memory-curate` run weekly (Monday). Register with `/schedule`.

**Invariant:** Subagents audit; skills write. No agent commits without explicit user instruction.

---

## Cross-References
- → `CLAUDE.md` (Agent Cache Protocol, startup instructions, justfile commands)
- → `.agent-cache/index.json` (live project snapshot)
- → `plans/CONVENTIONS.md` (domain codes, status codes, ADR/DRL format)
- → `plans/adr/ADR-017-ai-dlc.md` (ADR for this decision)
- → `plans/adr/ADR-018-anti-rot-agents.md` (ADR for maintenance agents)
- → `plans/modules/ai-dlc.md` (W-AIL work items)
- → `~/.claude/projects/-Users-shantopagla-portfolio/memory/MEMORY.md`
