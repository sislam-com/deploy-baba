---
name: add-plan-module
description: Create a new plan module file following CONVENTIONS.md template, register the domain code, and update INDEX.md. Keeps the plan system as the single source of truth.
argument-hint: "[domain-code] [component-name]"
---

Add a new module to the `plans/` system. The plan system is the single source of truth per AGENTS.md — always keep it updated when adding new components.

## Steps

### 1. Choose a domain code

- Check `plans/CONVENTIONS.md` → "Domain Codes" table for existing codes
- Pick a short ALL-CAPS code (2–5 chars) that doesn't conflict
- Add a new row to the domain table in `plans/CONVENTIONS.md`:
  ```markdown
  | `XYZ` | my-component | `path/to/component/` |
  ```

### 2. Create the module file

Path: `plans/modules/<component-name>.md` (kebab-case, matches the crate/service directory name)

Template (from CONVENTIONS.md):

```markdown
# W-XYZ: <component-name>
**Crate:** `path/to/component/` | **Status:** TODO
**Coverage floor:** 80% | **Depends on:** W-CFG | **Depended on by:** W-UI

## W-XYZ.1 Purpose

What this component does and why it exists.

## W-XYZ.2 Public API Surface

Key types, traits, functions, or routes exposed.

## W-XYZ.3 Implementation Notes

Architectural choices, patterns used, anything non-obvious.

## W-XYZ.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-XYZ.4.1 | Scaffold the module | TODO | |

## W-XYZ.5 Test Strategy

Unit test approach, coverage targets, integration test hooks.

## W-XYZ.6 Cross-References

- → ADR-001 (justfile interface)
- → W-CFG (depends on)
- ← W-UI (depended on by)
```

### 3. Register in INDEX.md

File: `plans/INDEX.md` → "Module Status Table"

Add a new row:
```markdown
| my-component | W-XYZ | `path/to/component/` | TODO | Brief remaining work |
```

Place it in logical order (library crates first, then services, then infra/tooling).

### 4. Update agent cache

File: `.agent-cache/index.json`

- Add an entry under `crates` (or `services`) for the new component
- Update `git.sha` to current HEAD: run `git rev-parse HEAD`
- Update `last_updated` to today's date

### 5. Add to the P-queue if there's work to do

In `plans/INDEX.md` → "Remaining Work" section, add the work items under the appropriate priority (P0–P3).

## Conventions

- WBS IDs: `W-XYZ.4.N` for work items (section 4 = Work Items)
- Status: `TODO`, `WIP`, `DONE`, `BLOCKED`, `DROPPED`
- Cross-reference syntax: `→ W-CFG` (this depends on), `← W-UI` (this is depended on by)

## Key Files

- `plans/CONVENTIONS.md` — notation system and domain code registry
- `plans/INDEX.md` — module status table and priority queue
- `plans/modules/` — one file per component
- `.agent-cache/index.json` — agent cache (update after changes)
