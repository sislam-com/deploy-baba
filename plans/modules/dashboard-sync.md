# W-SYNC: Dashboard → Migrations Sync
**Path:** `services/ui/migrations/`, `services/ui/src/db.rs`, `.claude/skills/` | **Status:** TODO
**Coverage floor:** N/A | **Depends on:** W-UI, W-RSM, W-ABT, W-SL, ADR-010 | **Depended on by:** —

---

## W-SYNC.1 Purpose

Bridge dashboard edits back to source-tree migrations so content survives a fresh
deploy. The `/dashboard` admin UI mutates live EFS SQLite directly
(`/mnt/db/baba.db`); without this workflow, those edits are lost when the Lambda
cold-boots against the embedded seed migrations.

The canonical re-seed pattern for all migrations produced by this workflow is the
SQLite upsert form — `INSERT … ON CONFLICT(…) DO UPDATE SET …` — as specified in
**ADR-010**.

---

## W-SYNC.2 Scope — Editable Tables and Natural Keys

The following tables are writable via `/dashboard` admin UI or `POST/PUT/DELETE
/api/admin/*` endpoints. Each row is audited for UNIQUE constraints that the ADR-010
upsert form requires.

| Table | Natural key | UNIQUE today? | Action |
|-------|-------------|---------------|--------|
| `jobs` | `slug` | verify in `001_create_jobs.sql` | preflight |
| `job_details` | composite `(job_id, sort_order)` | likely no — autoincrement `id` only | add composite UNIQUE in first sync migration |
| `competencies` | `slug` | verify in `002_create_competencies.sql` | preflight |
| `competency_evidence` | composite `(competency_id, job_id, sort_order)` | likely no | add composite UNIQUE in first sync migration |
| `about_sections` | `slug` | UNIQUE in `008_create_about_sections.sql` | good |
| `social_links` | `platform` | UNIQUE in `010_create_social_links.sql` | good |

> **Note:** The first sync migration for any table that lacks a UNIQUE constraint must
> add the composite index **before** the first upsert block runs. See ADR-010 for the
> full constraint requirement.

---

## W-SYNC.3 Workflow

Four phases, manual today, partially automated when W-SYNC.4.3 and W-SYNC.4.4 land:

### Phase 1 — Pull

Obtain a local copy of the live EFS SQLite database. Manual today:
- SSH/exec into the Lambda environment and copy `/mnt/db/baba.db` locally, **or**
- Use the future `GET /api/admin/db-dump` route (W-SYNC.4.3 Option B), **or**
- Fix the EventBridge→Lambda backup handler + S3 bucket name mismatch and pull the
  S3 backup (W-SYNC.4.3 Option C).

### Phase 2 — Diff

Compare the pulled DB state against the committed seed migrations:
- Open the pulled DB with `sqlite3` and inspect each editable table.
- Compare against the last committed `00N_seed_*.sql` or `00N_update_*.sql` file for
  that table.
- Note: inserts (new rows), updates (changed values), and deletes (removed rows).

### Phase 3 — Author

Scaffold the new migration with `/add-migration`, then write:
1. Any `CREATE UNIQUE INDEX IF NOT EXISTS …` statements needed (prefix, before upsert).
2. `DELETE FROM <table> WHERE <natural_key> = '…';` for each removed row (above upsert).
3. One `INSERT INTO … ON CONFLICT(…) DO UPDATE SET …;` block per table, per ADR-010.

### Phase 4 — Verify + Deploy

```
just dev          # fresh DB → migration applies as INSERT
# manually seed a pre-seeded DB with divergent content
just dev          # same migration applies as UPDATE
just lambda-deploy <profile>
```

---

## W-SYNC.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-SYNC.4.1 | Adopt ADR-010 (upsert re-seed convention) | DONE | Convention only; no code; ADR-010 committed 2026-04-08 |
| W-SYNC.4.2 | Audit UNIQUE constraints on natural keys across all dashboard-editable tables; fill in W-SYNC.2 table with findings | TODO | Read `00N_create_*.sql` files; update action column with confirmed status |
| W-SYNC.4.3 | Choose and implement the live-DB pull path: **Option B** (`GET /api/admin/db-dump` streaming `VACUUM INTO` copy, Cognito-gated) or **Option C** (fix EventBridge→Lambda backup handler + bucket-name mismatch in `xtask/src/database/{backup,restore}.rs` vs `infra/s3.tf`) | TODO | Recommendation: Option B — smallest footprint, reuses existing Cognito auth layer; record rejection of Option C if B is chosen |
| W-SYNC.4.4 | Create `.claude/skills/sync-dashboard-data/SKILL.md` + register in `docs/skills.md`. Delegates scaffolding to `/add-migration`; embeds ADR-010 upsert template; lists W-SYNC.2 natural-key table for preflight; cites W-SYNC.4.3 as the automated pull source | TODO | Execution step — deferred; skill authoring follows W-SYNC.4.3 |
| W-SYNC.4.5 | Backfill: write the first upsert-style migration capturing current live-DB divergence from the committed seeds (all edits accumulated via `/dashboard` to date) | TODO | Blocked on W-SYNC.4.3 (need a DB copy) |
| W-SYNC.4.6 | Optional: `xtask seed dump <table> --db-path <file>` command that emits upsert SQL for a single table from a local DB copy, reusing `xtask/src/resume/generate.rs` read-only loader pattern | TODO | Nice-to-have; defer until W-SYNC.4.5 proves the hand-authored path is too slow |
| W-SYNC.4.7 | Optional: extend `just dev` / CI to diff the live EFS DB against applied seed migrations and fail loudly if drift exists | TODO | Observability layer; makes "edit was never synced" state visible |

---

## W-SYNC.5 Test Strategy

- **INSERT path:** `just dev` against a freshly deleted DB → new upsert migration
  applies all rows as `INSERT`.
- **UPDATE path:** `just dev` against a pre-seeded DB with intentionally divergent
  content (edit one value manually in sqlite3) → same migration applies as `UPDATE`.
- **Read-back smoke test:** `cargo xtask resume generate --db-path fresh.db` — confirms
  the seeded rows are queryable via the read-only loader.
- **Pull path (W-SYNC.4.3):** add a cross-cutting integration test once the route or
  S3 backup handler lands; test that the downloaded file is a valid SQLite database.

---

## W-SYNC.6 Cross-References

- → ADR-002 (SQLite over PostgreSQL — foundational storage decision)
- → ADR-006 (EFS + SQLite + S3 backup — live DB location and backup plumbing)
- → ADR-010 (upsert re-seed convention — the canonical pattern this module operationalises)
- → W-RSM (jobs, job_details, competencies, competency_evidence — source tables)
- → W-ABT (about_sections — source table; UNIQUE(slug) already in place)
- → W-SL (social_links — source table; UNIQUE(platform) already in place)
- → W-CTF (contact tables — convention applies even if not editable via dashboard today)
- → W-XT (xtask plumbing reused for future dump tooling; also tracks the
  `deploy-baba-backups` bucket name mismatch vs `infra/s3.tf`)
- → `services/ui/src/db.rs:6-128` (`MIGRATIONS` array and idempotent runner)
- → `services/ui/migrations/012_update_about_sections.sql` (superseded split convention)
- → `.claude/skills/add-migration/SKILL.md` (scaffolding dependency for W-SYNC.4.4)
- → `xtask/src/resume/generate.rs` (read-only SQLite loader pattern for W-SYNC.4.6)
