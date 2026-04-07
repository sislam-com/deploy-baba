---
name: add-adr
description: Write a new Architecture Decision Record (ADR) using the project's numbering convention. Register it in INDEX.md and cross-reference affected module plans.
argument-hint: "[short-title]"
---

Document a significant architectural decision. ADRs are immutable records — once written, they are amended only with a new ADR (not edited in place).

## When to Write an ADR

- Choosing between two non-obvious technical approaches
- Overriding or superseding an existing ADR
- Decisions that will surprise future maintainers
- Constraints that cannot be inferred from the code

## Steps

### 1. Determine the next ADR number

Current highest: **ADR-009** (API Gateway HTTP API for POST /api/contact)

Check `plans/INDEX.md` → "ADR Index" table for the current list. Next number: **ADR-010**.

Format: zero-padded 3 digits → `ADR-010`

### 2. Create the ADR file

Path: `plans/adr/ADR-<NNN>-<short-title>.md`

- `<NNN>`: zero-padded 3 digits
- `<short-title>`: kebab-case, ≤5 words
- Example: `plans/adr/ADR-010-sqlite-wal-mode.md`

Template:

```markdown
# ADR-<NNN>: <Human Readable Title>

**Date:** YYYY-MM-DD
**Status:** Accepted | Supersedes ADR-XXX | Superseded by ADR-YYY
**Affected modules:** W-XXX, W-YYY

## Context

What situation or problem prompted this decision? What constraints exist?
Be specific — what options were considered?

## Decision

What was decided, and why? State it clearly in one sentence, then expand.

> We will use X instead of Y because Z.

## Consequences

### Positive
- Benefit 1
- Benefit 2

### Negative / Trade-offs
- Cost 1
- Cost 2

### Neutral
- Side effect that is neither good nor bad

## Alternatives Considered

| Option | Rejected because |
|--------|-----------------|
| Option A | ... |
| Option B | ... |

## Cross-References

- → ADR-NNN (if superseding)
- → W-XXX (affected module)
- → DRL-YYYY-MM-DD-topic (if triggered by an incident)
```

### 3. Register in INDEX.md

File: `plans/INDEX.md` → "ADR Index" table

Add a new row:
```markdown
| ADR-<NNN> | <Short human title> | W-XXX, W-YYY |
```

### 4. Cross-reference in affected module plans

For each module listed in "Affected modules":

File: `plans/modules/<module>.md` → section `W-XXX.6 Cross-References`

```markdown
→ ADR-<NNN> (<reason, e.g. "justfile interface">)
```

### 5. If superseding an existing ADR

- Update the old ADR file: change `**Status:** Accepted` → `**Status:** Superseded by ADR-<NNN>`
- Add to old ADR's Cross-References: `→ ADR-<NNN> (supersedes this)`
- Note the supersession in INDEX.md: append `(superseded by ADR-<NNN>)` in the old row

## Existing ADRs (for context)

| ID | Title |
|----|-------|
| ADR-001 | justfile Is the Only Interface |
| ADR-002 | SQLite Over PostgreSQL |
| ADR-003 | Lambda Function URL (No API Gateway) |
| ADR-004 | Dual-Mode Entry Point |
| ADR-005 | Zero-Cost Philosophy |
| ADR-006 | EFS + SQLite + S3 Backup |
| ADR-007 | OpenTofu Over Terraform |
| ADR-008 | Cognito Authentication for Admin Dashboard |
| ADR-009 | API Gateway HTTP API for POST /api/contact |

## Key Files

- `plans/adr/` — all ADR files
- `plans/INDEX.md` — ADR index table
- `plans/modules/` — cross-reference in affected module plans
