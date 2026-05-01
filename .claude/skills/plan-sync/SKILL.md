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

### Step 1 — Parallel audit

Launch two subagents in a single message (parallel):
```
Agent(subagent_type="plan-doctor",   prompt="Run full audit")
Agent(subagent_type="drift-detector", prompt="<scope from args, or 'full sweep'>")
```

Wait for both to complete. Collect their markdown reports.

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
