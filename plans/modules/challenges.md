# W-CHL: Challenges
**Path:** `services/ui/src/routes/api/challenges.rs`, `services/ui/migrations/022`, `web/src/routes/dashboard/Challenges.tsx` | **Status:** DONE (basic CRUD DONE; RAG integration DONE; public pages DONE; search/filter DONE; evaluation metrics deferred)
**Coverage floor:** N/A | **Depends on:** W-UI, W-RAG, W-WEB | **Depended on by:** —
**Migrations:** 022 (schema + seed) | **Admin API:** `GET /api/challenges`, `GET /api/challenges/{slug}`, `GET /api/jobs/{slug}/challenges` | **Dashboard:** `/dashboard/challenges`, `/dashboard/challenges/:id`

## W-CHL.1 Purpose

Add a "Challenges" feature to showcase portfolio projects and technical challenges. Each challenge represents a significant project or technical problem solved, with rich metadata including tech stack, category, job linkage, and detailed descriptions. Content is stored in SQLite and served as JSON via API endpoints to the React SPA admin dashboard. Challenges are integrated into the RAG system as the 7th corpus for AI-powered Q&A about portfolio projects.

## W-CHL.2 Public API Surface

| Route | Method | Auth | Description |
|-------|--------|------|-------------|
| `/api/challenges` | GET | Public | List all challenges ordered by sort_order |
| `/api/challenges/{slug}` | GET | Public | Get single challenge by slug |
| `/api/jobs/{slug}/challenges` | GET | Public | Get challenges linked to a specific job |

## W-CHL.3 Implementation Notes

### Schema

Single `challenges` table with optional job linkage and categorization:

```sql
CREATE TABLE IF NOT EXISTS challenges (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    slug              TEXT    NOT NULL UNIQUE,
    title             TEXT    NOT NULL,
    job_id            INTEGER REFERENCES jobs(id),
    description       TEXT    NOT NULL,
    short_description TEXT,
    tech_stack        TEXT,
    category          TEXT,
    url               TEXT,
    image_url         TEXT,
    featured          INTEGER NOT NULL DEFAULT 0,
    sort_order        INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_challenges_job_id ON challenges(job_id);
```

### Migrations

- `022_create_challenges.sql` — schema + index + 4 seeded challenges (deploy-baba portfolio, 27-step platform deployment, Scala multi-tenancy, RAG grounding system)

### Route Handler Pattern

Follows `routes/api/jobs.rs` pattern: `row_to_challenge()` helper converts SQLite rows to Challenge structs with comma-separated tech_stack parsing. Three handlers (`list_challenges`, `get_challenge`, `list_challenges_for_job`) return JSON consumed by React SPA.

### RAG Integration

Challenges are integrated as the 7th RAG corpus:
- `crates/rag-core/src/portfolio.rs`: `get_challenges_summary()` method for live-data retrieval
- `crates/rag-core/src/chunk/portfolio.rs`: `challenge_to_prose()` function converts challenge entities to prose chunks
- `crates/rag-core/src/hybrid.rs`: "challenge"/"challenges" keyword triggers for filtered retrieval
- Challenges linked to jobs enable cross-corpus queries (challenges → jobs → competencies)

### UX

- Admin dashboard: list view with featured badges, detail view with full metadata
- Category badges: fullstack, platform, ai (displayed in admin UI)
- Featured flag: highlights important challenges in list view
- Sort order: manual ordering for display priority

## W-CHL.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-CHL.4.1 | Create `022_create_challenges.sql` migration | DONE | Schema + index + 4 seeded challenges |
| W-CHL.4.2 | Register migration in `db.rs` | DONE | Add to MIGRATIONS array |
| W-CHL.4.3 | Create `routes/api/challenges.rs` with handlers | DONE | `list_challenges`, `get_challenge`, `list_challenges_for_job`, `row_to_challenge` |
| W-CHL.4.4 | Register module in `routes/api/mod.rs` | DONE | `pub mod challenges;` |
| W-CHL.4.5 | Add routes in `routes/api/mod.rs` | DONE | `/api/challenges/*` routes |
| W-CHL.4.6 | Create admin dashboard `Challenges.tsx` | DONE | List view with featured badges |
| W-CHL.4.7 | Create admin dashboard `ChallengeDetail.tsx` | DONE | Detail view with full metadata |
| W-CHL.4.8 | Add Challenge model to `api-openapi` | DONE | Added to models/portfolio.rs |
| W-CHL.4.9 | Implement challenges RAG corpus integration | DONE | `get_challenges_summary()`, `challenge_to_prose()`, keyword triggers |
| W-CHL.4.10 | Complete admin CRUD (edit/delete forms) | DONE | Admin endpoints (POST/PUT/DELETE /api/admin/challenges) implemented in admin.rs; React forms (new/edit/delete) implemented in ChallengeDetail.tsx |
| W-CHL.4.11 | Add challenges to public portfolio pages | DONE | Challenges already integrated into Home.tsx with ChallengeCard component; accessible via "Challenges" tab; fetched via /api/resume endpoint |
| W-CHL.4.12 | Implement challenges-specific evaluation metrics | DEFERRED | RAG evaluation for challenges corpus - defer to broader RAG evaluation framework (P2/P3) |
| W-CHL.4.13 | Implement challenges search/filter on public site | DONE | Added category dropdown filter and featured checkbox filter to challenges view in Home.tsx; filters are client-side with reactive state |

### Implementation Order

1. W-CHL.4.1–4.5 (migrations + API routes)
2. W-CHL.4.6–4.7 (admin dashboard UI)
3. W-CHL.4.8–4.9 (OpenAPI + RAG integration)
4. W-CHL.4.10–4.13 (remaining features)

### Files to Create

| File | Purpose |
|------|---------|
| `services/ui/migrations/022_create_challenges.sql` | Schema + index + seed data |
| `services/ui/src/routes/api/challenges.rs` | API route handlers |
| `web/src/routes/dashboard/Challenges.tsx` | Admin list view |
| `web/src/routes/dashboard/ChallengeDetail.tsx` | Admin detail view |

### Files to Modify

| File | Change |
|------|--------|
| `services/ui/src/db.rs` | Add migration to MIGRATIONS array |
| `services/ui/src/routes/api/mod.rs` | Add `pub mod challenges;` + routes |
| `crates/api-openapi/src/models/portfolio.rs` | Add Challenge struct |
| `crates/rag-core/src/portfolio.rs` | Add `get_challenges_summary()` to PortfolioDataProvider |
| `crates/rag-core/src/chunk/portfolio.rs` | Add `challenge_to_prose()` function |
| `crates/rag-core/src/hybrid.rs` | Add "challenge"/"challenges" keyword triggers |

## W-CHL.5 Test Strategy

- Verify migration applies cleanly on fresh DB (`just quality`)
- Verify GET `/api/challenges` returns 200 with seeded challenges
- Verify GET `/api/challenges/{slug}` returns 200 for valid slugs, 404 for invalid
- Verify GET `/api/jobs/{slug}/challenges` returns challenges linked to specific job
- Verify admin dashboard renders challenge list with featured badges
- Verify RAG retrieval includes challenges corpus for relevant queries
- Verify tech_stack parsing (comma-separated to array)

## W-CHL.6 Cross-References

- → W-UI (ui-service framework + API routes)
- → W-WEB (React admin dashboard)
- → W-RAG (7th corpus integration; PortfolioDataProvider trait)
- → ADR-002 (SQLite on EFS)
- → ADR-010 (upsert re-seed convention — challenges uses UNIQUE(slug))
- → ADR-016 (RAG Architecture — challenges as 7th corpus)
- → W-SYNC (dashboard → migrations sync workflow)
- → W-RSM (job linkage via job_id foreign key)