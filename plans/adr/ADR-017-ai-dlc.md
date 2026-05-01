# ADR-017: AI-Assisted Development Lifecycle (AI-DLC)

**Status:** Accepted
**Date:** 2026-04-30
**Affected modules:** all (W-AIL owns the artifacts; every other module is subject to the lifecycle)
**Imported from:** njnewsroomproject ADR-014 (adapted for deploy-baba scope)

---

## Context

Claude Code is the primary development agent for this repo. Sessions with Claude have a recurring startup cost: re-reading Cargo.toml files, plan documents, and infra layout takes 10–20 tool calls when starting cold. The repo already has `.agent-cache/index.json` (SHA-keyed project snapshot) and the Agent Cache Protocol in `CLAUDE.md`, but there is no formalised convention for:

- What quality gates a Claude session must run before declaring work done.
- How the plan system (INDEX.md module Status fields, ADR claims) gets updated after implementation.
- What structured flow non-trivial multi-component tasks follow.

Without a formalised lifecycle, each session re-invents the process and Stage 5 (Maintenance) gets skipped, causing `INDEX.md` to drift from the codebase.

---

## Decision

**Formalise the AI-DLC.** The lifecycle is documented in `plans/cross-cutting/ai-dlc.md` and covers six stages:

1. **Startup** — agent cache check (`.agent-cache/index.json` vs. `git rev-parse HEAD`) to avoid cold re-reads. Documented in `CLAUDE.md` Agent Cache Protocol.
2. **Planning** — plan mode (`EnterPlanMode` → Explore/Plan agents → `ExitPlanMode`) for non-trivial tasks (any change spanning > 1 component, or touching infra/CI). Trivial tasks (single-file typo, dep bump, doc touch) skip plan mode.
3. **Implementation** — execute plan items; run quality gates after each logical chunk (not after every file): `just dev` (fmt + lint + test) for Rust; `just web-test` and `just web-typecheck` for SPA; `tofu validate` for infra changes.
4. **Verification** — mandatory gate before reporting done: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `pnpm --dir web run test`, `pnpm --dir web run typecheck`, `tofu -chdir=infra validate`.
5. **Maintenance** — update module Status in `plans/modules/*.md`, update `plans/INDEX.md`, refresh `.agent-cache/index.json`. Anti-rot agents from ADR-018 catch skipped maintenance.
6. **Commit** — only on explicit user instruction; conventional commits (`feat:`, `fix:`, etc.). Never commit without user approval.

Domain code `AIL` is reserved for AI lifecycle work items in module spec files.

Skills in use or planned:
- `/plan-sync` — ADR-018 anti-rot orchestrator (run at end of any implementation session).
- `/cache-refresh` — thin wrapper for `xtask cache refresh` (run when cache SHA diverges from HEAD).
- `/memory-curate` — audits cross-session memory for stale project facts.
- Weekly scheduled routines: `dbb-plan-sync` (Mon 09:00), `dbb-memory-curate` (Mon 09:30).

---

## Consequences

**Positive:**
- Consistent, auditable session behaviour regardless of Claude model version.
- Startup cost reduced from O(20 tool calls) to O(3) when cache is fresh.
- User never needs to re-explain project conventions mid-session.
- Plan files provide a written record of why architectural choices were made.

**Negative:**
- Plan mode adds latency (~2–3 turns) before implementation begins. Acceptable for non-trivial tasks; trivial tasks are exempt.
- Agent cache can go stale if a commit is made outside a Claude session. Mitigation: cache check on every startup catches this within one tool call.

---

## Cross-References
- → `plans/cross-cutting/ai-dlc.md` (full lifecycle spec)
- → `CLAUDE.md` (Agent Cache Protocol + startup instructions)
- → `plans/CONVENTIONS.md` (domain code `AIL`)
- → ADR-018 (anti-rot agents that enforce Stage 5)
- → `plans/modules/ai-dlc.md` (W-AIL work items)
