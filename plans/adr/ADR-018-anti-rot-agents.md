# ADR-018: Anti-rot Agents for the AI-DLC

**Status:** Accepted
**Date:** 2026-04-30
**Affected modules:** W-AIL
**Imported from:** njnewsroomproject ADR-015 (adapted for deploy-baba scope)

---

## Context

ADR-017 formalised the AI-DLC session lifecycle, including Stage 5 (Maintenance): update module Status fields, update `plans/INDEX.md`, refresh `.agent-cache/index.json`. In practice, Stage 5 is a manual checklist that gets skipped under time pressure. Without enforcement, `INDEX.md` Status fields drift, ADR claims diverge from actual code, and stale project memories mislead future sessions.

The operator requested maintenance agents scoped to **zero-cost** mechanisms: no CI token spend beyond normal session use, no Anthropic API calls outside Claude Code sessions.

---

## Decision

Introduce five maintenance artifacts under domain code `AIL`:

### Subagents (read-only auditors)

**`plan-doctor`** (`.claude/agents/plan-doctor.md`) — structural audit of the plan system:
- Status mismatches between `plans/INDEX.md` Module Status Table and individual module files.
- Stale work items (WIP with no recent git activity; DONE with no git evidence).
- Cache SHA drift vs `git rev-parse HEAD`.
- ADR ↔ module cross-reference gaps (Accepted ADR lists an "Affected modules" entry but that module's `.6 Cross-References` doesn't cite the ADR back).

**`drift-detector`** (`.claude/agents/drift-detector.md`) — semantic audit: diffs each Accepted ADR's Decision claims against actual code and infra. Produces draft DRL entry skeletons when divergence is found. Never writes files.

**Invariant: subagents are read-only.** They audit and report. Skills are the only writers.

### Skills (writers — each change requires user confirmation)

**`/plan-sync`** (`.claude/skills/plan-sync/SKILL.md`) — runs both subagents in parallel; applies safe auto-fixes (status table sync, missing ADR back-references); gates on user confirmation before writing DRL files or modifying ADR Status.

**`/cache-refresh`** (`.claude/skills/cache-refresh/SKILL.md`) — thin wrapper for `just cache-refresh` (which runs `cargo xtask cache refresh`, already implemented in `xtask/src/cache.rs`). Verifies idempotency. No duplicate logic — the skill is the session-time invocation point.

**`/memory-curate`** (`.claude/skills/memory-curate/SKILL.md`) — walks `~/.claude/projects/-Users-shantopagla-portfolio/memory/`, verifies project memories against current repo/infra state, proposes prunes per-file.

### Schedule

Two weekly routines (Monday morning) registered via `/schedule`:
- `dbb-plan-sync` → runs `/plan-sync` (Mon 09:00)
- `dbb-memory-curate` → runs `/memory-curate` (Mon 09:30, offset to avoid collision)

Both routines post findings and never auto-commit or auto-apply without user confirmation.

---

## Audit corpus

`plan-doctor` and `drift-detector` are scoped to the deploy-baba plan system: 22 ADRs (ADR-001 through ADR-022), 30 module plans (26 existing + 4 new: W-AIL, W-CI, W-WEB, W-DEV). The drift-detector's "Known falsifiable claims" block is tailored to this repo (see `.claude/agents/drift-detector.md`).

---

## Consequences

**Positive:**
- Stage 5 maintenance becomes one command (`/plan-sync`) instead of three manual edits.
- Cache stays fresh → cold-session startup cost remains O(3) tool calls as intended by ADR-017.
- ADR drift is caught before it misleads future sessions.
- Memory rot is caught weekly — stale project memories no longer mislead new sessions.

**Negative:**
- Two new agent definitions and three new skill files to maintain.
- `/plan-sync` can produce false positives (DONE item with no git evidence when the work landed in a parent path). Mitigation: the agent reports rather than auto-removes; user judgment resolves ambiguity.
- Weekly scheduled runs cost plan capacity. At low frequency this is negligible.

---

## Deferred

- `infra-reviewer` — audits OpenTofu changes for IAM over-permissioning and drift from infra ADRs. Present in njnewsroomproject `.claude/agents/` but not in the documented anti-rot set. Defer to W-AIL.5+.
- `ci-author` — drafts GitHub Actions workflow skeletons from plan modules. Defer to W-AIL.5+.

---

## Cross-References
- → ADR-017 (AI-DLC lifecycle this builds on)
- → `plans/cross-cutting/ai-dlc.md` (full lifecycle spec including Maintenance Agents section)
- → `plans/modules/ai-dlc.md` (W-AIL work items)
- → `plans/CONVENTIONS.md` (domain code `AIL`)
