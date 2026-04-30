---
name: memory-curate
description: Walk ~/.claude/projects/-Users-shantopagla-portfolio/memory/, verify project memories against current repo and infra state, and propose prunes with per-file confirmation. Use after major milestones or on weekly schedule (dbb-memory-curate).
---

# /memory-curate

Audits Claude's persistent memory for the deploy-baba project. Catches stale project memories before they mislead future sessions.

## How to run

```
/memory-curate                  # check all memory types
/memory-curate project          # check only project memories (fastest, most decay-prone)
/memory-curate feedback         # check only feedback memories
```

## Memory location

`~/.claude/projects/-Users-shantopagla-portfolio/memory/`

Memory types: `user/`, `feedback/`, `project/`, `reference/`.

## Implementation steps

When this skill is invoked:

### Step 1 — Inventory memories

List all files under the memory directory:
```bash
find ~/.claude/projects/-Users-shantopagla-portfolio/memory/ -name "*.md" | sort
```

Filter by type argument if provided.

### Step 2 — Read MEMORY.md index

Read `~/.claude/projects/-Users-shantopagla-portfolio/memory/MEMORY.md` to understand what each file is supposed to contain.

### Step 3 — Verify project memories (highest priority)

For each `project/*.md` file:
1. Read the file.
2. Extract the factual claim (the "fact" at the top, before **Why:** and **How to apply:**).
3. Verify the claim against current repo state:
   - If it names a file path: check the file exists (`ls <path>`).
   - If it names a function or struct: grep for it (`grep -r '<name>' services/ crates/ xtask/`).
   - If it describes module Status (e.g. "W-AUTH is DONE"): check `plans/INDEX.md`.
   - If it describes a pending deploy action: check git log for recent commits on the relevant path.
4. If the memory contradicts current state, mark it "STALE".
5. If the memory is still accurate but references something that has been superseded, mark it "OUTDATED".

### Step 4 — Verify reference memories

For each `reference/*.md` file:
- If it names a Linear project: cannot verify without Linear MCP — skip with a note.
- If it names an AWS resource: cannot verify without live AWS session — skip with a note.
- If it names a local path or justfile recipe: verify it exists.

### Step 5 — Review feedback and user memories (lightest verification)

For each `feedback/*.md` and `user/*.md` file:
- Check that the rule still makes sense in the current project context.
- Flag only if the project has changed in a way that clearly invalidates the feedback.

### Step 6 — Present findings and gate

For each memory flagged as STALE or OUTDATED:
1. Show the user:
   - File path
   - Current content (brief)
   - Why it was flagged (what check failed)
   - Proposed action: DELETE, UPDATE <new content>, or KEEP

2. Ask for confirmation per file: "Delete/update/keep this memory?"

3. Only act on explicit user approval:
   - DELETE: `rm <file>` + remove the entry from MEMORY.md index.
   - UPDATE: `Edit` the file with the corrected content.
   - KEEP: leave unchanged.

### Step 7 — Summary

Print: "Memory curate complete. N files checked, M flagged, K actions taken (L deletions, J updates)."

## What it never does

- Never deletes or edits a memory file without explicit user confirmation per file.
- Never modifies source code, plans, infra, CI, or the agent cache.
- Never commits changes.
- Never flags feedback memories as stale just because a task mentioned in them is complete — feedback is about collaboration style, not task state.
