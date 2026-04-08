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

**Audit completed 2026-04-08** (W-SYNC.4.2 DONE):

| Table | Natural key | UNIQUE today? | Action |
|-------|-------------|---------------|--------|
| `jobs` | `slug` | YES — column `UNIQUE` in `001_create_jobs.sql` | none |
| `job_details` | composite `(job_id, sort_order)` | NO — only autoincrement `id` + non-unique `idx_job_details_job_id` | **added** `ux_job_details_job_sort` in `014_add_sync_unique_indexes.sql` |
| `competencies` | `slug` | YES — column `UNIQUE` in `002_create_competencies.sql` | none |
| `competency_evidence` | composite `(competency_id, job_id, sort_order)` | NO | **added** `ux_competency_evidence_comp_job_sort` in `014_add_sync_unique_indexes.sql` |
| `about_sections` | `slug` | YES — column `UNIQUE` in `008_create_about_sections.sql` | none |
| `social_links` | `platform` | YES — column `UNIQUE` in `010_create_social_links.sql` | none |

> **Re-ordering caveat:** `job_details.sort_order` and `competency_evidence.sort_order`
> are mutable from the dashboard and are part of the composite natural key. If the
> operator re-orders rows, emit explicit `DELETE FROM … WHERE (natural_key) = …;`
> statements **above** the upsert block. See `.claude/skills/sync-dashboard-data/SKILL.md`
> Phase 3 for the full template.

---

## W-SYNC.3 Workflow

Four phases — Phase 1 automated via `GET /api/admin/db-dump` (W-SYNC.4.3 DONE), full workflow in `/sync-dashboard-data` skill (W-SYNC.4.4 DONE):

### Phase 1 — Pull (automated — W-SYNC.4.3 DONE)

Obtain a consistent SQLite snapshot of the live EFS database via:

```bash
curl -b "auth_token=<token>" -o /tmp/baba-live.db \
     https://<cloudfront-host>/api/admin/db-dump
```

See `.claude/skills/sync-dashboard-data/SKILL.md` Phase 1 for full curl commands
(production Cognito-gated + local dev open).

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
| W-SYNC.4.2 | Audit UNIQUE constraints on natural keys across all dashboard-editable tables; fill in W-SYNC.2 table with findings | **DONE** | Audit complete 2026-04-08; preflight UNIQUE indexes added in `014_add_sync_unique_indexes.sql` |
| W-SYNC.4.3 | Choose and implement the live-DB pull path | **DONE** | **Option B** implemented: `GET /api/admin/db-dump` via `VACUUM INTO /tmp/baba-dump-<nanos>.db`; Cognito-gated via router-level `require_auth`; handler in `services/ui/src/routes/api/admin.rs::db_dump_handler`. **Option C rejected** — fixes `xtask` backup handler + bucket-name mismatch; more code churn, no auth reuse, S3 download adds latency vs direct HTTP. |
| W-SYNC.4.4 | Create `.claude/skills/sync-dashboard-data/SKILL.md` + register in `docs/skills.md` | **DONE** | Skill created 2026-04-08; registered in `docs/skills.md`; embeds ADR-010 upsert template, W-SYNC.2 natural-key table, all four workflow phases |
| W-SYNC.4.5 | Backfill: write the first upsert-style migration capturing current live-DB divergence from the committed seeds (all edits accumulated via `/dashboard` to date) | TODO | Operator step — requires deployed Lambda with `db-dump` route; run `/sync-dashboard-data` skill after deploy |
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

### db-dump endpoint smoke test (local dev, no auth)

```bash
just ui &
sleep 2
curl -fsS -o /tmp/baba-dump.db http://localhost:3000/api/admin/db-dump
file /tmp/baba-dump.db   # → "SQLite 3.x database"
sqlite3 /tmp/baba-dump.db "SELECT name FROM sqlite_master WHERE type='table';"
# Expected: jobs, job_details, competencies, competency_evidence, about_sections,
#           social_links, _migrations
sqlite3 /tmp/baba-dump.db "SELECT count(*) FROM jobs;"  # → an integer ≥ 0
kill %1
```

### UNIQUE index smoke test (migration 014)

```bash
rm -f deploy-baba.db
just dev &   # fresh DB → migration 014 applies
sleep 2 && kill %1
sqlite3 deploy-baba.db \
  "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'ux_%';"
# Expected:
#   ux_job_details_job_sort
#   ux_competency_evidence_comp_job_sort
```

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
