---
name: sync-dashboard-data
description: Bridge /dashboard admin UI edits back to source-tree migrations using the ADR-010 SQLite upsert convention (W-SYNC). Run this before any deploy that would otherwise wipe accumulated dashboard edits.
argument-hint: ""
---

Bridge dashboard edits back to source migrations so content survives a fresh deploy.
Implements the W-SYNC four-phase workflow (Pull → Diff → Author → Verify+Deploy) using ADR-010 upserts.

## When to Use

- The live EFS DB has diverged from `services/ui/migrations/` (dashboard edits not yet captured).
- Before a `just lambda-deploy` that would wipe accumulated dashboard content.
- As a regular hygiene step to keep source migrations in sync with production data.

---

## Natural-Key Audit Table (W-SYNC.2 — confirmed 2026-04-08)

| Table | Natural key | UNIQUE constraint | Status |
|-------|-------------|-------------------|--------|
| `jobs` | `slug` | column UNIQUE (`001_create_jobs.sql`) | Ready |
| `job_details` | composite `(job_id, sort_order)` | index `ux_job_details_job_sort` (migration 014) | Ready |
| `competencies` | `slug` | column UNIQUE (`002_create_competencies.sql`) | Ready |
| `competency_evidence` | composite `(competency_id, job_id, sort_order)` | index `ux_competency_evidence_comp_job_sort` (migration 014) | Ready |
| `about_sections` | `slug` | column UNIQUE (`008_create_about_sections.sql`) | Ready |
| `social_links` | `platform` | column UNIQUE (`010_create_social_links.sql`) | Ready |

> **Re-ordering caveat:** `job_details.sort_order` and `competency_evidence.sort_order`
> are part of the natural key. If you re-ordered rows in the dashboard, emit explicit
> `DELETE FROM <table> WHERE <natural_key_cols> = …;` statements **above** the upsert
> block. See Phase 3 below.

---

## Phase 1 — Pull (automated via `GET /api/admin/db-dump`)

The `db_dump_handler` in `services/ui/src/routes/api/admin.rs` returns a consistent
SQLite snapshot via `VACUUM INTO`. It is Cognito-gated in production and open in dev mode.

> **CRITICAL — never use `cp deploy-baba.db /tmp/baba-live.db`.**
> The DB runs in WAL mode (`PRAGMA journal_mode=WAL`). A plain `cp` of the main
> `.db` file can miss writes still buffered in the `-wal` sidecar, producing a stale
> snapshot that matches the seed DB and hides real drift. **Always pull via the
> `db-dump` endpoint** which issues `VACUUM INTO` — this checkpoints the WAL and
> produces a single, consistent file with all committed writes.

### Production (Cognito auth required)

```bash
# Obtain your auth_token cookie from the browser (DevTools → Application → Cookies)
curl -b "auth_token=<your-token>" \
     -H "Accept: application/octet-stream" \
     -o /tmp/baba-live.db \
     https://<cloudfront-host>/api/admin/db-dump

# Verify the download
sqlite3 /tmp/baba-live.db "SELECT count(*) FROM jobs;"
```

### Local dev (no Cognito)

If `just ui` is already running, use it directly. If not, start it first:

```bash
# If server is NOT already running:
just ui &
sleep 2

# Pull snapshot via db-dump (always — even when server is already running):
curl -fsS -o /tmp/baba-live.db http://localhost:3000/api/admin/db-dump
file /tmp/baba-live.db   # → "SQLite 3.x database"
sqlite3 /tmp/baba-live.db "SELECT count(*) FROM jobs;"

# Kill server only if YOU started it above:
# kill %1
```

**Verify snapshot freshness:** Before proceeding to Phase 2, check that the
row counts in `/tmp/baba-live.db` match what you see in the dashboard UI.
If counts are off, the server may not be running — start it and re-pull.

---

## Phase 2 — Diff

Compare the pulled DB against a fresh local DB seeded from source migrations:

```bash
# Start with a clean local DB
rm -f deploy-baba.db
just dev &   # starts on :3000; Ctrl-C after "listening"
sleep 2 && kill %1

# Dump both DBs per-table and diff
for table in jobs job_details competencies competency_evidence about_sections social_links; do
    sqlite3 /tmp/baba-live.db ".dump $table" > /tmp/live-${table}.sql
    sqlite3 deploy-baba.db    ".dump $table" > /tmp/seed-${table}.sql
    echo "=== $table ==="
    diff /tmp/seed-${table}.sql /tmp/live-${table}.sql || true
done
```

For each table, classify changes as:
- **INSERT** — new natural key in live DB not in seeds
- **UPDATE** — same key, different column values
- **DELETE** — key exists in seeds but gone from live DB

---

## Phase 3 — Author the Migration

Scaffold the migration file:

```
/add-migration NNN_sync_dashboard_<YYYY-MM-DD>
```

This delegates to the `/add-migration` skill which creates `services/ui/migrations/NNN_*.sql`
and appends the `include_str!` entry to `services/ui/src/db.rs`.

### ADR-010 Upsert Template

For each table with changes, write one block:

```sql
-- DELETE rows removed since last sync (place ABOVE upsert block)
DELETE FROM <table> WHERE <natural_key_col> = '<value>';

-- Upsert rows added/changed since last sync (ADR-010)
INSERT INTO <table> (<col1>, <col2>, …)
VALUES
    (<val1>, <val2>, …),
    (<val1>, <val2>, …)
ON CONFLICT(<natural_key_col_or_cols>)
DO UPDATE SET
    <col1> = EXCLUDED.<col1>,
    <col2> = EXCLUDED.<col2>,
    …;
```

**Composite key tables** (`job_details`, `competency_evidence`):

```sql
INSERT INTO job_details (job_id, detail_text, category, sort_order)
VALUES (1, 'Led migration to Rust', 'Engineering', 1)
ON CONFLICT(job_id, sort_order)
DO UPDATE SET
    detail_text = EXCLUDED.detail_text,
    category    = EXCLUDED.category;
```

```sql
INSERT INTO competency_evidence (competency_id, job_id, detail_id, highlight_text, sort_order)
VALUES (2, 1, NULL, 'Reduced latency by 40%', 1)
ON CONFLICT(competency_id, job_id, sort_order)
DO UPDATE SET
    detail_id      = EXCLUDED.detail_id,
    highlight_text = EXCLUDED.highlight_text;
```

> **Re-ordering:** If rows were re-ordered in the dashboard, add explicit `DELETE`
> statements above the upsert block so the old `(job_id, sort_order)` or
> `(competency_id, job_id, sort_order)` tuples are removed before the upsert runs.

---

## Phase 4 — Verify + Deploy

```bash
# INSERT path: fresh DB
rm -f deploy-baba.db
just dev    # migration applies all rows as INSERT (no conflicts)

# UPDATE path: seed an intentionally divergent local DB, then re-run
sqlite3 deploy-baba.db "UPDATE jobs SET company='Old Co' WHERE slug='acme';"
just dev    # same migration applies as UPDATE (ON CONFLICT path fires)

# Deploy
just lambda-deploy <profile>
```

---

## Cross-References

- **ADR-010** `plans/adr/ADR-010-sqlite-upsert-seed-convention.md` — canonical upsert pattern
- **W-SYNC** `plans/modules/dashboard-sync.md` — full work-item tracking
- **`/add-migration`** `.claude/skills/add-migration/SKILL.md` — migration scaffolding
- **Handler** `services/ui/src/routes/api/admin.rs` → `db_dump_handler`
- **MIGRATIONS array** `services/ui/src/db.rs:6-128`
