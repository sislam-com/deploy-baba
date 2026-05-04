# W-AIL: AI Development Lifecycle
**Path:** `.claude/` | **Status:** DONE
**Coverage floor:** n/a | **Depends on:** W-DX | **Depended on by:** all modules

## W-AIL.1 Purpose

Provide session-time agents and maintenance skills that keep the plan system, agent cache, and persistent memory accurate over time. Prevents rot in `plans/INDEX.md`, module Status fields, `.agent-cache/index.json`, and `memory/project_*.md` without requiring CI spend or Anthropic API token budget beyond normal session use.

## W-AIL.2 Public Surface

**Subagents (read-only auditors — never edit files):**

| Name | File | Purpose |
|---|---|---|
| `plan-doctor` | `.claude/agents/plan-doctor.md` | Structural audit: status table mismatches, stale work items, cache SHA drift, ADR↔module cross-reference gaps |
| `drift-detector` | `.claude/agents/drift-detector.md` | Semantic audit: ADR Decision claims vs actual code/infra; drafts DRL entry skeletons when divergence found |

**Skills (writers — every change requires user confirmation):**

| Command | File | Purpose |
|---|---|---|
| `/plan-sync` | `.claude/skills/plan-sync/SKILL.md` | Runs both subagents in parallel; applies safe fixes; gates on user before DRL/ADR status writes |
| `/cache-refresh` | `.claude/skills/cache-refresh/SKILL.md` | Wraps `just cache-refresh` (xtask/src/cache.rs); verifies idempotency |
| `/memory-curate` | `.claude/skills/memory-curate/SKILL.md` | Verifies project memories against repo/infra state; proposes prunes per-file |

## W-AIL.3 Implementation Notes

**Subagent format:** Frontmatter (`name`, `description`, `tools`, `model`) + markdown body. Project-scoped under `.claude/agents/`. Model: `sonnet`.

**Skill format:** Frontmatter (`name`, `description`) + markdown body with step-by-step instructions. Project-scoped under `.claude/skills/<name>/SKILL.md`.

**Invariant:** Subagents are read-only. They call `Bash`, `Read`, `Grep`, `Glob` but never `Write` or `Edit`. Skills hold the write authority and gate on user confirmation before any destructive or DRL-creating step.

**Cache reuse:** The `/cache-refresh` skill does NOT reimplement `xtask/src/cache.rs`. It is a thin invocation wrapper: calls `just cache-refresh` and then runs it a second time to verify idempotency (second run must produce no diff). This keeps the single source of truth in xtask.

**Audit corpus:** `plan-doctor` and `drift-detector` cover 22 ADRs (ADR-001 through ADR-022), 30 module plans, and 8 cross-cutting docs. The drift-detector's falsifiable claims list lives in `.claude/agents/drift-detector.md` and is scoped to this repo's specific ADRs.

**Schedule:** Two weekly crons registered via `/schedule`:
- `dbb-plan-sync` — Mon 09:00, runs `/plan-sync`
- `dbb-memory-curate` — Mon 09:30 (offset), runs `/memory-curate`

**Domain code:** `AIL` (registered in ADR-017, first used here).

## W-AIL.4 Work Items

| ID | Task | Status | Notes |
|---|---|---|---|
| W-AIL.4.1 | `plan-doctor` subagent | TODO | `.claude/agents/plan-doctor.md` — port from njnewsroomproject; retarget component paths and ADR corpus |
| W-AIL.4.2 | `drift-detector` subagent | TODO | `.claude/agents/drift-detector.md` — port + replace claims block with portfolio ADRs 001–022 |
| W-AIL.4.3 | `/plan-sync` skill | TODO | `.claude/skills/plan-sync/SKILL.md` — port verbatim |
| W-AIL.4.4 | `/cache-refresh` skill | TODO | `.claude/skills/cache-refresh/SKILL.md` — wrap `just cache-refresh`, not duplicate |
| W-AIL.4.5 | `/memory-curate` skill | TODO | `.claude/skills/memory-curate/SKILL.md` — port verbatim |
| W-AIL.4.6 | ADR-018 | DONE | `plans/adr/ADR-018-anti-rot-agents.md` ✓ |
| W-AIL.4.7 | Weekly schedule wiring | TODO | Run `/schedule` to create `dbb-plan-sync` + `dbb-memory-curate` routines |

## W-AIL.5 Test Strategy

Manual verification (no automated test suite — these are agent/skill definitions, not compiled code):

1. **plan-doctor dry run:** `Agent(subagent_type="plan-doctor", prompt="Run full audit")` — confirm four sections in output, each with content or "None."
2. **drift-detector scoped:** `Agent(subagent_type="drift-detector", prompt="audit ADR-015")` — confirm `crates/llm-core` claims are verified or diverged with evidence.
3. **cache-refresh idempotency:** Run `/cache-refresh` twice; confirm second pass produces no file diff.
4. **plan-sync picks up current drift:** Run `/plan-sync` after any session that touched infra or plans; confirm it proposes the correct status update.
5. **memory-curate verification:** Run `/memory-curate project`; confirm it checks project memory entries against current repo state.

## W-AIL.6 Cross-References

- → ADR-017 (AI-DLC lifecycle this builds on)
- → ADR-018 (decision record for these agents)
- → `plans/cross-cutting/ai-dlc.md` (full lifecycle spec)
- → `plans/CONVENTIONS.md` (domain code AIL)
- → `.agent-cache/index.json` (live cache that `/cache-refresh` maintains)
- → `xtask/src/cache.rs` (cache refresh implementation — reused, not duplicated)
- → `~/.claude/projects/-Users-shantopagla-portfolio/memory/` (store that `/memory-curate` audits)
