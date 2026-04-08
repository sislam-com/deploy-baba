# ADR-010: SQLite Upsert as the Canonical Re-Seed Convention

**Date:** 2026-04-08
**Status:** Accepted
**Affected modules:** W-RSM, W-ABT, W-SL, W-CTF, W-XT, W-UI, W-SYNC

---

## Context

The `/dashboard` admin UI mutates DB content directly against the live SQLite file on
EFS (`/mnt/db/baba.db`). Those edits drift away from source: the "ground truth" seeds
live under `services/ui/migrations/` and are compile-time embedded in the Lambda binary
via the `MIGRATIONS` array in `services/ui/src/db.rs:6-55`.

The migration runner (`db.rs:95-128`) applies each `MIGRATIONS` entry exactly once,
tracked by name in a `_migrations` table. Editing an already-applied seed file is a
**silent no-op**. The only way to push new content to a live database is a new numbered
migration.

Across the 12 existing migrations, three conflicting re-seed styles have been used:

1. **Plain `INSERT INTO … VALUES (…)`** (e.g. `003_seed_jobs.sql`,
   `011_seed_social_links.sql`) — only works because the runner skips applied migrations
   by name; fails if re-run against a seeded DB.
2. **`INSERT OR IGNORE INTO …`** (e.g. `009_seed_about_sections.sql`) — idempotent
   insert, but cannot update existing rows; dashboard edits are silently dropped on
   re-seed.
3. **`UPDATE … WHERE <key>; INSERT OR IGNORE INTO …` split** (e.g.
   `012_update_about_sections.sql`) — covers updates but is not atomic per row, duplicates
   the column list across both halves, and requires two statements per row.

A single canonical convention is needed so that future migration authors (and the
planned `/sync-dashboard-data` skill, W-SYNC.4.4) have one unambiguous pattern to
follow.

---

## Decision

> All future re-seed and dashboard-sync migrations shall use the SQLite `INSERT … ON
> CONFLICT(…) DO UPDATE SET …` upsert form.

```sql
INSERT INTO <table> (col1, col2, …, <natural_key>)
VALUES (…)
ON CONFLICT(<natural_key>) DO UPDATE SET
    col1 = excluded.col1,
    col2 = excluded.col2,
    …;
```

Rules:

- The natural key column is **not** listed in the `DO UPDATE SET` clause — it is the
  conflict-match target, not a mutation target.
- One atomic statement per row (or multi-row `VALUES` list for the same table).
- The `VALUES` tuple is the single source of truth for a row's column values; the
  `DO UPDATE` clause rebinds from `excluded.*`.
- Natural keys must be stable text columns (not autoincrement `id`) with a `UNIQUE`
  constraint. Composite `UNIQUE` indexes are permitted.
- **`INSERT OR IGNORE`, `INSERT OR REPLACE`, and split `UPDATE` + `INSERT OR IGNORE`
  are banned in all new migrations.**

### Worked Example

```sql
INSERT INTO about_sections (page, slug, heading, body, icon, sort_order)
VALUES
    ('me', 'me-intro', 'Hello', 'Updated body text.', NULL, 1),
    ('me', 'me-skills', 'Skills', 'Rust, AWS, SQLite.', NULL, 2)
ON CONFLICT(slug) DO UPDATE SET
    page       = excluded.page,
    heading    = excluded.heading,
    body       = excluded.body,
    icon       = excluded.icon,
    sort_order = excluded.sort_order;
```

---

## Consequences

### Positive

- Single canonical pattern — no ambiguity across migration authors or tooling.
- Atomic per row — no partial update possible.
- Column list appears in one place (the `VALUES` tuple); the `DO UPDATE` clause is a
  mechanical mirror.
- Works identically on a fresh DB (acts as `INSERT`) and a seeded DB (acts as `UPDATE`).
- Migration file is roughly half the size of the split `UPDATE` + `INSERT OR IGNORE`
  style.
- Review burden drops — reviewers check one list, not two.

### Negative / Trade-offs

- Requires `UNIQUE` constraints on natural keys. Tables that currently lack them
  (`job_details`, `competency_evidence`) must add composite `UNIQUE` indexes in the same
  migration before the first upsert block runs.
- Deletes are not covered by upsert. Removed rows must be expressed as explicit
  `DELETE FROM <table> WHERE <natural_key> = '…';` statements placed **above** the
  upsert block in the migration file.

### Neutral

- Existing migrations (001–012) are **not** rewritten — ADRs apply to new work only.
  `012_update_about_sections.sql` remains as a historical record of the prior split
  convention; it is not a template for future work.

---

## Alternatives Considered

| Option | Rejected because |
|--------|-----------------|
| `INSERT OR IGNORE` | Cannot update existing rows; dashboard edits would be silently lost on re-seed |
| `INSERT OR REPLACE` | Deletes and re-inserts the row → breaks foreign keys referencing `id`, drops timestamp defaults, loses audit trail |
| `UPDATE … WHERE; INSERT OR IGNORE` split (012 style) | Not atomic per row; column list duplicated across both halves; two statements per row |
| Generated code (`xtask seed dump <table>`) | Worth doing later (W-SYNC.4.6); the convention must be decided first so generated code has something to emit |
| Runtime serialisation to JSON + loader on cold boot | Reintroduces the live-DB / source split at a different layer; does not satisfy the review-on-Git requirement |

---

## Cross-References

- → ADR-002 (SQLite Over PostgreSQL — foundational storage decision)
- → ADR-006 (EFS + SQLite + S3 Backup — live DB location)
- → W-RSM (jobs and competency tables use this convention going forward)
- → W-ABT (about_sections was the first partial attempt; superseded here)
- → W-SL (social_links upserts follow this convention)
- → W-XT (xtask read-only loader pattern reused for future dump tooling)
- → W-SYNC (dashboard → migrations sync workflow module that operationalises this ADR)
