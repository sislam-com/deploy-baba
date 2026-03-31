# W-RSM: Resume — Career Timeline & Capabilities UI
**Path:** `services/ui/` (migrations + routes + template) | **Status:** DONE
**Coverage floor:** N/A (binary) | **Depends on:** W-UI, ADR-002, ADR-003

---

## W-RSM.1 Purpose

Career timeline and functional/competency view for the portfolio — the first real
data-backed feature in the UI service, fulfilling ADR-002 (SQLite on EFS).

This is the **home page** (`/`). Two views behind a single route:
- **Timeline** — reverse-chronological job list with expandable detail bullets
- **Capabilities** — competency cards with cross-referenced evidence from jobs

---

## W-RSM.2 Data Model

### `jobs` (master)
slug (UNIQUE), company, title, location, start\_date, end\_date (NULL = Present),
summary, tech\_stack (comma-separated), sort\_order (0 = most recent)

### `job_details` (detail)
job\_id FK, detail\_text, category (`achievement` | `responsibility` | `sub-engagement`),
sort\_order

### `competencies`
slug (UNIQUE), name, description, icon (emoji), sort\_order

### `competency_evidence` (many-to-many: competencies ↔ job_details)
competency\_id FK, job\_id FK, detail\_id FK (nullable), highlight\_text (optional override),
sort\_order

---

## W-RSM.3 File Structure

```
services/ui/
├── migrations/
│   ├── 001_create_jobs.sql              # Schema: jobs + job_details
│   ├── 002_create_competencies.sql      # Schema: competencies + competency_evidence
│   ├── 003_seed_jobs.sql                # 9 positions (personal-projects → openpages)
│   ├── 004_seed_job_details.sql         # All accomplishment bullets + sub-engagements
│   ├── 005_seed_competencies.sql        # 6 competency categories
│   ├── 006_seed_competency_evidence.sql # Cross-references
│   └── 007_seed_personal_projects.sql   # Personal projects job details seed
├── src/
│   ├── db.rs                            # Db struct, open(), migration runner
│   └── routes/
│       ├── resume.rs                    # GET / → Askama handler (home page)
│       └── api/
│           ├── jobs.rs                  # GET /api/jobs, GET /api/jobs/{slug}
│           └── competencies.rs          # GET /api/competencies, GET /api/competencies/{slug}
└── templates/
    └── resume.html                      # Dual-view: timeline + capabilities toggle

xtask resume generation (W-XT.4.5):
xtask/src/resume/
├── mod.rs                               # CLI subcommand: resume generate | resume upload
├── generate.rs                          # Reads DB → builds resume struct → serializes to JSON/PDF
└── upload.rs                            # Uploads generated resume artifact to S3
```

---

## W-RSM.4 API Surface

```
GET  /                                    Dual-view HTML home page (Askama server-rendered)
GET  /api/jobs                            [{slug, company, title, dates, summary, tech_stack}, ...]
GET  /api/jobs/{slug}                     Job + detail bullets grouped by category
GET  /api/competencies                    [{slug, name, description, icon}, ...]
GET  /api/competencies/{slug}             Competency + evidence items (highlight or detail_text)
```

Query parameter: `GET /api/jobs?view=chronological` (functional view reserved for future).
URL state: `/?view=capabilities` — updated via `history.pushState`.

---

## W-RSM.5 Migration Runner (`db.rs`)

- Migrations embedded at compile time via `include_str!` (safe for Lambda, no FS access needed)
- Tracks applied migrations in `_migrations` table (name UNIQUE)
- Idempotent: `CREATE TABLE IF NOT EXISTS`, `INSERT OR IGNORE`
- `007_seed_personal_projects.sql` is now present — runner picks up new migrations automatically, no Rust changes needed
- WAL mode + `PRAGMA foreign_keys=ON` enabled at open time

---

## W-RSM.6 State Threading

`Arc<Db>` is created in `main.rs` and passed to `router::build(db)`. Applied via
`.with_state(db)` on the top-level router. Sub-routers return `Router<Arc<Db>>` —
handlers that don't use state work unchanged; handlers that do use
`State(db): State<Arc<Db>>`.

---

## W-RSM.7 Seed Data Summary

| Slug | Company | Dates |
|------|---------|-------|
| personal-projects | Personal Projects | 2024–Present |
| scala-computing | Scala Computing | 2019–2026 |
| sunbird-dcim | Sunbird DCIM | 2016–2019 |
| falconstor | FalconStor Software | 2014–2016 |
| galaxe-solutions | GalaxE.Solutions | 2010–2014 |
| independent-contractor | Independent Contractor | 2008–2010 |
| wbgo | Newark Public Radio (WBGO) | 2002–2008 |
| logistics-com | Logistics.com | 2001 |
| openpages | Openpages Inc. | 2000–2001 |

GalaxE sub-engagements (Coach, GSI Commerce, TrueAction) stored as `job_details`
with `category = 'sub-engagement'`.

6 competency categories: `platform-architecture`, `cloud-infrastructure`,
`frontend-engineering`, `ai-augmented-dev`, `technical-leadership`, `saas-product`.

---

## W-RSM.8 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-RSM.8.1 | Add `view=functional` grouping to `GET /api/jobs` | TODO | Returns jobs nested under competency headings |
| W-RSM.8.2 | Cross-navigation deep-links (capabilities → timeline anchor) | TODO | Currently scrolls to job card; could expand it automatically |
| W-RSM.8.3 | Print/PDF stylesheet for `/resume` | TODO | `@media print` CSS |

---

## W-RSM.9 Cross-References
- → ADR-002 (SQLite on EFS — this is the first real data source)
- → ADR-003 (Lambda Function URL — no API Gateway)
- → ADR-004 (dual-mode entry point — DB path from `DB_PATH` env var)
- → W-UI (base route surface, router, state architecture)
- → `plans/cross-cutting/aws-architecture.md` (EFS mount for SQLite on Lambda)
