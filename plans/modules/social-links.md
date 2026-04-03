# W-SL: Social Links

## Status: DONE

## Summary

DB-managed social links displayed in the top nav. Replaces the hardcoded GitHub anchor in `base.html`.

## Table

```sql
CREATE TABLE social_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    platform TEXT NOT NULL UNIQUE,
    url TEXT NOT NULL,
    icon TEXT,
    label TEXT NOT NULL,
    visible INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0
);
```

Seeded with LinkedIn (sort_order=0) and GitHub (sort_order=1).

## Files

| File | Role |
|------|------|
| `migrations/010_create_social_links.sql` | Schema |
| `migrations/011_seed_social_links.sql` | Seed data |
| `src/db.rs` | `SocialLink` struct + `load_social_links()` helper |
| `src/routes/resume.rs` | `social_links` field on `ResumeTemplate` |
| `src/routes/about.rs` | `social_links` field on both about templates |
| `src/routes/dashboard.rs` | `social_links` field on all dashboard templates + list/new/detail handlers |
| `src/routes/api/admin.rs` | `POST/PUT/DELETE /api/admin/social-links` |
| `src/router.rs` | Dashboard routes wired |
| `templates/base.html` | Nav loop replaces hardcoded GitHub link |
| `templates/dashboard_social_links_list.html` | List view |
| `templates/dashboard_social_link_detail.html` | Create/edit form |
| `templates/dashboard_home.html` | Social Links count tile |

## Dashboard Routes

- `GET /dashboard/social-links` — list
- `GET /dashboard/social-links/new` — new form
- `GET /dashboard/social-links/:id` — edit form (id-based, not platform slug)

## Admin API Routes

- `POST /api/admin/social-links` — create
- `PUT /api/admin/social-links/:id` — update
- `DELETE /api/admin/social-links/:id` — delete
