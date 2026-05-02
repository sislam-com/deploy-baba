# W-ABT: About Section
**Path:** `services/ui/src/routes/api/about.rs`, `services/ui/migrations/008-009` | **Status:** DONE (templates deleted D.5; data served via `/api/about/sections` JSON to React SPA)
**Coverage floor:** N/A | **Depends on:** W-UI, W-RSM (pattern reference) | **Depended on by:** —
**Migrations:** 008 (schema), 009 (seed) | **Admin API:** `POST/PUT/DELETE /api/admin/about` | **Dashboard:** `/dashboard/about`, `/dashboard/about/new`, `/dashboard/about/:slug`

## W-ABT.1 Purpose

Add two public "About" pages to the portfolio: `/about/me` (personal bio, skills, background)
and `/about/repo` (project description, tech stack, architecture decisions). Content is stored
in SQLite and rendered via Askama templates extending `base.html`.

## W-ABT.2 Public API Surface

| Route | Method | Auth | Description |
|-------|--------|------|-------------|
| `/about/me` | GET | Public | Personal bio, skills, engineering philosophy |
| `/about/repo` | GET | Public | Project overview, tech stack, architecture |

## W-ABT.3 Implementation Notes

### Schema

Single `about_sections` table with `page` column discriminator (`'me'` or `'repo'`):

```sql
CREATE TABLE IF NOT EXISTS about_sections (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    page       TEXT    NOT NULL,        -- 'me' or 'repo'
    slug       TEXT    NOT NULL UNIQUE,  -- e.g. 'me-bio', 'repo-stack'
    heading    TEXT    NOT NULL,
    body       TEXT    NOT NULL,
    icon       TEXT,                     -- optional
    sort_order INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_about_sections_page ON about_sections(page);
```

### Migrations

- `008_create_about_sections.sql` — schema + index
- `009_seed_about_sections.sql` — 4 "me" sections (bio, background, skills, philosophy) + 4 "repo" sections (overview, stack, architecture, crates)

### Route Handler Pattern

Follows `routes/resume.rs`: shared `query_sections(db, page)` helper returns `Vec<AboutSection>`.
Two handlers (`about_me`, `about_repo`) each call the helper with their page discriminator and
render their respective Askama template.

### UX

- Nav: single "About" link → `/about/me` (before "API Docs", no dropdown)
- Each about page has a server-rendered tab toggle (`<a>` tags) to switch between Me/Repo
- Active tab: `bg-cyan-600 text-white`; inactive: `text-gray-400 hover:text-white`
- Sections rendered as cards: `bg-gray-800/50 rounded-lg p-6 border border-gray-700`

## W-ABT.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-ABT.4.1 | Create `008_create_about_sections.sql` migration | DONE | Table + index |
| W-ABT.4.2 | Create `009_seed_about_sections.sql` migration | DONE | 4 "me" + 4 "repo" seed rows |
| W-ABT.4.3 | Register migrations in `db.rs` | DONE | Add 2 entries to `MIGRATIONS` array |
| W-ABT.4.4 | Create `routes/about.rs` with handlers | DONE | `about_me` + `about_repo` + `query_sections` helper |
| W-ABT.4.5 | Register module in `routes/mod.rs` | DONE | `pub mod about;` |
| W-ABT.4.6 | Add routes in `router.rs` | DONE | `/about/me`, `/about/repo` |
| W-ABT.4.7 | Create `about_me.html` template | DONE | Extends `base.html`, tab toggle |
| W-ABT.4.8 | Create `about_repo.html` template | DONE | Extends `base.html`, tab toggle |
| W-ABT.4.9 | Add "About" nav link in `base.html` | DONE | Before "API Docs" link |
| W-ABT.4.10 | Admin CRUD routes + dashboard pages | DONE | `POST/PUT/DELETE /api/admin/about`; dashboard list/new/detail handlers + templates |

### Implementation Order

1. W-ABT.4.1–4.3 (migrations + db.rs registration)
2. W-ABT.4.7–4.8 (templates)
3. W-ABT.4.4–4.6 (route handler + wiring)
4. W-ABT.4.9 (nav link)

### Files to Create

| File | Purpose |
|------|---------|
| `services/ui/migrations/008_create_about_sections.sql` | Schema + index |
| `services/ui/migrations/009_seed_about_sections.sql` | 8 seed rows |
| `services/ui/src/routes/about.rs` | Route handlers |
| `services/ui/templates/about_me.html` | /about/me template |
| `services/ui/templates/about_repo.html` | /about/repo template |

### Files to Modify

| File | Change |
|------|--------|
| `services/ui/src/db.rs` | Add 2 entries to `MIGRATIONS` array |
| `services/ui/src/routes/mod.rs` | Add `pub mod about;` |
| `services/ui/src/router.rs` | Add 2 `.route()` calls |
| `services/ui/templates/base.html` | Add "About" nav link before "API Docs" |

## W-ABT.5 Test Strategy

- Verify migrations apply cleanly on fresh DB (`just quality`)
- Verify GET `/about/me` returns 200 with seeded sections
- Verify GET `/about/repo` returns 200 with seeded sections
- Verify nav link renders on all pages
- Verify tab toggle navigates between the two about pages

## W-ABT.6 Cross-References

- → W-UI (ui-service framework)
- → W-RSM (resume pattern reference for DB queries + Askama templates)
- → ADR-002 (SQLite on EFS)
- → ADR-010 (upsert re-seed convention — about_sections already has UNIQUE(slug))
- → ADR-013 (React SPA — about page content served via JSON to SPA)
- → ADR-019 (SPA deploy pipeline — about content surfaced in deployed SPA)
- → W-SYNC (dashboard → migrations sync workflow)
