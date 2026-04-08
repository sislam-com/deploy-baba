---
name: add-drift-log
description: Create a drift log (DRL) documenting an incident, unexpected finding, or post-mortem. Register it in INDEX.md and cross-reference affected modules.
argument-hint: "[topic]"
---

Document an incident, gap, or unexpected deviation from the plan. Drift logs (DRLs) are the project's post-mortem / lessons-learned format.

## When to Use

- After a production incident or unexpected failure
- When discovering code gaps during implementation
- After a significant course correction or reversal of a prior decision
- When something in the plan was wrong and needed updating

## Steps

### 1. Create the drift log file

Path: `plans/drift/DRL-<YYYY-MM-DD>-<topic>.md`

- Date: today's date in `YYYY-MM-DD` format
- Topic: kebab-case, 2–4 words (e.g. `opentofu-migration`, `function-url-auth`, `contact-form`)
- Example: `plans/drift/DRL-2026-04-07-email-lambda-timeout.md`

Template:

```markdown
# DRL-<YYYY-MM-DD>-<topic>

**Date:** YYYY-MM-DD
**Severity:** low | medium | high
**Affected modules:** W-XXX, W-YYY

## Summary

One paragraph describing what happened.

## Entries

| ID | Finding | Status | Resolution |
|----|---------|--------|-----------|
| DRL-<TOPIC>-1 | Description of the gap/issue | RESOLVED / OPEN | What was done or needs doing |

## Lessons Learned

- Bullet points of what to do differently

## Cross-References

- → W-XXX (affected module plan)
- → ADR-NNN (if an ADR was created or changed as a result)
```

### 2. Register in INDEX.md

File: `plans/INDEX.md` → "Drift Log Index" table

Add a new row:
```markdown
| DRL-<YYYY-MM-DD>-<topic> | <YYYY-MM-DD> | <Human readable topic> | <N> entries |
```

### 3. Cross-reference affected modules

For each affected module plan (e.g. `plans/modules/opentofu.md`):

- In section `W-XXX.6 Cross-References`, add:
  ```
  → DRL-<YYYY-MM-DD>-<topic>
  ```

### 4. Update open work items if needed

If the drift log surfaces new work:
- Add `W-XXX.4.N` items to the affected module file
- Add to `plans/INDEX.md` priority queue under appropriate P-level

## Existing Drift Logs (for context / numbering reference)

| File | Date | Topic |
|------|------|-------|
| DRL-2026-03-18-terraform | 2026-03-18 | Terraform first-run gaps |
| DRL-2026-03-18-xtask | 2026-03-18 | xtask/justfile gaps |
| DRL-2026-03-25-opentofu | 2026-03-25 | OpenTofu migration audit |
| DRL-2026-03-27-function-url-auth | 2026-03-27 | Lambda Function URL auth incident |
| DRL-2026-04-03-contact-form | 2026-04-03 | Contact Form + SES implementation |
| DRL-2026-04-03-pow-apigateway | 2026-04-03 | POST+PoW via API Gateway |

## Key Files

- `plans/drift/` — all drift log files
- `plans/INDEX.md` — drift log index table
- `plans/CONVENTIONS.md` — DRL naming rules
