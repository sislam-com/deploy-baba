---
name: plan-sync
description: Run plan-doctor and drift-detector in parallel, then apply safe fixes to plans/INDEX.md, module Status fields, and ADR back-references. Stop before writing DRL files or changing ADR Status — those require user confirmation. Use at end of any session that touched code or plans.
---

# /plan-sync

Audits and repairs the plan system. Safe writes only; destructive changes always require your confirmation.

## What it does

1. Runs `plan-doctor` and `drift-detector` subagents in parallel.
2. Applies safe auto-fixes:
   - Syncs `plans/INDEX.md` Module Status Table to match module file Status fields (module file wins if git log shows more recent activity there).
   - Inserts missing ADR back-references into module W-`<CODE>`.6 Cross-References sections.
3. Stops and shows you the drift report before writing any DRL files or flipping an ADR from Proposed → Accepted.
4. Prints unfixed findings (semantic drift, stale-but-ambiguous items) for your review.

## How to run

Just invoke `/plan-sync` — no arguments needed. Optionally scope the drift check:

```
/plan-sync              # full audit + auto-fix
/plan-sync ADR-015      # scope drift-detector to one ADR only
```

## Implementation steps

When this skill is invoked:

### Step 0 — Scope to changed files (cost optimization)

Before launching subagents, determine what actually changed:

1. Read `.agent-cache/index.json` to get the cached `git.sha`.
2. Run: `git diff --name-only <cached_sha> HEAD` (or `HEAD~5 HEAD` if cache is missing).
3. Filter the changed files to identify:
   - **changed_modules**: plan module files in `plans/modules/` that changed
   - **changed_source**: source paths (`crates/`, `services/`, `infra/`, `web/`, `xtask/`) that changed
   - **changed_adrs**: ADR files in `plans/adr/` that changed
4. Map changed source paths to their plan domain codes (e.g. `services/ui/` → W-UI, `crates/rag-core/` → W-RAG).
5. Build the scoped audit lists:
   - **modules_to_check**: union of changed_modules + modules whose source paths changed
   - **adrs_to_check**: union of changed_adrs + ADRs referenced by modules_to_check

**If no plan-relevant files changed** (no plans/, no source, no infra, no .claude/):
Print "No plan-relevant changes since last cache — skipping audit." and stop.

### Step 1 — Scoped parallel audit

Launch two subagents in a single message (parallel), passing the scoped lists:
```
Agent(subagent_type="plan-doctor",   prompt="Audit only these modules: <modules_to_check>. Check their status, work items, cache drift, and ADR back-references. Skip all other modules.")
Agent(subagent_type="drift-detector", prompt="Audit only these ADRs: <adrs_to_check>. Skip all others.")
```

If `adrs_to_check` is empty, skip the drift-detector agent entirely.
If a specific ADR was passed as an argument (e.g. `/plan-sync ADR-015`), use that as the sole scope for drift-detector.

Wait for both to complete. Collect their markdown reports.

### Step 1.5 — Early exit

If both agents report zero findings, print "Plan system clean — no fixes needed." and stop.
Do not proceed to Steps 2-6.

### Step 2 — Auto-fix: status table

Read `plans/INDEX.md`. For each module row where plan-doctor reported a status mismatch:
- Run `git log -1 --format="%ci" -- plans/modules/<m>.md` and `git log -1 --format="%ci" -- <component path>` to find which was updated more recently.
- If the module file is more recent, update `plans/INDEX.md` to match it.
- If INDEX.md is more recent (rare), update the module file Status field.
- Show the user the diff before writing.

### Step 3 — Auto-fix: ADR back-references

For each gap reported by plan-doctor ("ADR-NNN not back-referenced in W-<CODE>.6"):
- Read the module file's W-`<CODE>`.6 Cross-References section.
- Append the missing ADR line: `- → ADR-NNN` (following the existing reference syntax from `plans/CONVENTIONS.md`).
- Write inline — no prompt needed for this safe insert.

### Step 4 — Gate: DRL files

If drift-detector found any divergences:
- Show the user each proposed DRL entry (file name + body skeleton).
- Ask: "Create these DRL files? (y/n for each)"
- Only create the ones the user approves.

DRL files go in `plans/drift/DRL-YYYY-MM-DD-<topic>.md` using today's date.

### Step 5 — Gate: cache refresh

If plan-doctor reported cache SHA drift:
- Ask: "Cache is stale — run /cache-refresh now?"
- If yes, invoke `/cache-refresh`.

### Step 6 — Unfixed findings

Print a summary of any findings that require manual action:
- Stale WIP items with no recent activity (needs a human decision: resume or drop?)
- DONE items with no git evidence (needs investigation)
- Semantic drift where code changed but ADR claim is still arguably correct (judgment call)

## What it never does

- Never commits changes.
- Never modifies ADR Status fields without explicit user confirmation.
- Never deletes plan files.
- Never touches source code, infra, or CI files.
