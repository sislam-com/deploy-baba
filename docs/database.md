# Database and Migrations

Last updated: 2026-05-19

SQLite on EFS with numbered migrations and S3 backup. Zero monthly cost for the database layer.

## Engine

| Property | Value |
|----------|-------|
| Engine | SQLite 3 (via `rusqlite` with `bundled` feature) |
| Journal mode | WAL (concurrent readers, single writer) |
| Production path | `/mnt/db/app.db` on EFS |
| Local dev path | `deploy-baba.db` (from `stack.toml` or default) |
| Backup | S3 via EventBridge schedule (daily) |

The database is embedded in the Lambda binary — no external database server, no connection pooling, no RDS costs ([ADR-002](../plans/adr/ADR-002-sqlite-over-postgresql.md)).

## Migration System

28 numbered SQL files in `services/ui/migrations/`. Each migration is embedded at compile time via `include_str!()` in the `MIGRATIONS` array in `services/ui/src/db.rs`.

At startup, `run_migrations()` checks the `_migrations` table and applies any unapplied migrations in order. The runner is idempotent — re-running is always safe.

### Naming convention

```
NNN_description.sql
```

Three-digit zero-padded number, underscore, snake_case description. Use the `/add-migration` skill to scaffold new migrations.

### Migration categories

| Range | Category | Examples |
|-------|----------|---------|
| 001–007 | Schema + initial seed | jobs, competencies, personal_projects |
| 008–009 | About sections | about_sections table + seed data |
| 010–011 | Social links | social_links table + seed data |
| 012–014 | Alignment + sync indexes | LinkedIn profile alignment, unique indexes for upsert |
| 015–016 | Dashboard sync | ADR-010 upsert-based sync snapshots |
| 017 | RAG | FTS5 virtual table for full-text search |
| 018–021 | Resume + competency updates | AI positioning, me_summary, competency refresh |
| 022–023 | Challenges | challenges table + RAG evaluation |
| 024–025 | Resume polish | Cleanup + content refinement |
| 026 | Metrics | request_metrics, error_counts tables |
| 027–028 | Outcome-focused content | Description rewrites, resume consolidation |

## Schema Overview

### Resume / Career

- **`jobs`** — job experiences, keyed by `slug` (UNIQUE)
- **`job_details`** — per-job bullet points, keyed by composite `(job_id, sort_order)` via unique index
- **`competencies`** — skills/competencies, keyed by `slug` (UNIQUE)
- **`competency_evidence`** — evidence linking competencies to jobs, keyed by composite `(competency_id, job_id, sort_order)`

### About

- **`about_sections`** — about page content sections, keyed by `slug` (UNIQUE). Includes the `me-bio` row used for resume professional summary ([ADR-014](../plans/adr/ADR-014-resume-summary-from-db.md)).

### Social Links

- **`social_links`** — social media links displayed in nav, keyed by `platform` (UNIQUE)

### Challenges

- **`challenges`** — portfolio project showcases with descriptions and metadata

### RAG

- **`rag_index`** — FTS5 virtual table for full-text search across 7 portfolio corpora
- **`rag_eval`** — evaluation metrics for RAG query quality

### Metrics

- **`request_metrics`** — per-request latency and metadata ([ADR-025](../plans/adr/ADR-025-sqlite-metrics-collection.md))
- **`error_counts`** — aggregated error counts by category

### Internal

- **`_migrations`** — tracks which migrations have been applied (managed by the runner)

## Upsert Convention (ADR-010)

All seed and sync migrations use the INSERT ... ON CONFLICT ... DO UPDATE pattern ([ADR-010](../plans/adr/ADR-010-sqlite-upsert-seed-convention.md)):

```sql
INSERT INTO jobs (slug, company, title, start_date)
VALUES ('acme', 'Acme Corp', 'Engineer', '2024-01')
ON CONFLICT(slug)
DO UPDATE SET
    company    = EXCLUDED.company,
    title      = EXCLUDED.title,
    start_date = EXCLUDED.start_date;
```

Banned patterns: `INSERT OR IGNORE`, `INSERT OR REPLACE`, split `UPDATE` + `INSERT`.

For composite-key tables (`job_details`, `competency_evidence`), the `ON CONFLICT` clause targets the composite unique index columns.

## Dashboard Sync

When content is edited through the admin dashboard, those changes live only in the runtime database. To persist them as source-controlled migrations, use the `/sync-dashboard-data` skill, which follows a four-phase workflow: Pull (snapshot via `GET /api/admin/db-dump`) → Diff (against fresh seed) → Author (upsert migration) → Verify + Deploy.

See [plans/modules/dashboard-sync.md](../plans/modules/dashboard-sync.md) for the full workflow.

## Backup

An EventBridge rule triggers a daily SQLite backup to S3:
1. The UI Lambda's backup handler runs `VACUUM INTO` to create a consistent snapshot
2. The snapshot is uploaded to the S3 backup bucket with a date-stamped key
3. S3 lifecycle rules manage retention

Infrastructure: `infra/eventbridge.tf` (schedule), `infra/s3.tf` (backup bucket).

## Cross-References

- [ADR-002](../plans/adr/ADR-002-sqlite-over-postgresql.md) — SQLite over PostgreSQL
- [ADR-010](../plans/adr/ADR-010-sqlite-upsert-seed-convention.md) — Upsert seed convention
- [ADR-014](../plans/adr/ADR-014-resume-summary-from-db.md) — Resume summary from DB
- [ADR-025](../plans/adr/ADR-025-sqlite-metrics-collection.md) — SQLite-based metrics
- [plans/modules/dashboard-sync.md](../plans/modules/dashboard-sync.md) — Sync workflow
- [services.md](services.md) — UI Lambda (which runs the migration system)
