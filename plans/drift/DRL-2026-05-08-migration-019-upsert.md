# DRL-2026-05-08-migration-019-upsert

**Date:** 2026-05-08
**Severity:** low
**Affected modules:** W-DB

## Summary

Dashboard sync assessment (2026-05-08) revealed that migration 019 (`019_add_me_summary.sql`) uses the banned `INSERT OR IGNORE` pattern instead of the ADR-010-compliant upsert pattern. The live DB is otherwise in sync with source migrations - no dashboard drift detected.

## Entries

| ID | Finding | Status | Resolution |
|----|---------|--------|-----------|
| DRL-MIG19-1 | Migration 019 uses `INSERT OR IGNORE` instead of ADR-010 upsert pattern | RESOLVED | Migration rewritten to use `ON CONFLICT(slug) DO UPDATE SET` pattern (2026-05-08) |

## Lessons Learned

- Always validate new migrations against ADR-010 (SQLite Upsert Re-Seed Convention) before deployment
- The sync-dashboard-data skill effectively catches both dashboard drift and migration pattern violations
- `INSERT OR IGNORE` is banned for seed/upsert migrations - must use explicit upsert pattern per ADR-010

## Correct Pattern

Migration 019 should be rewritten from:
```sql
INSERT OR IGNORE INTO about_sections (page, slug, heading, body, icon, sort_order) VALUES
('me', 'me-summary', 'Summary',
 'shantopagla — Full-Stack SaaS Engineer · Zero-cost Rust & AWS deployments',
 NULL, 0);
```

To ADR-010-compliant upsert:
```sql
INSERT INTO about_sections (page, slug, heading, body, icon, sort_order)
VALUES ('me', 'me-summary', 'Summary',
  'shantopagla — Full-Stack SaaS Engineer · Zero-cost Rust & AWS deployments',
  NULL, 0)
ON CONFLICT(slug) DO UPDATE SET
  page = EXCLUDED.page,
  heading = EXCLUDED.heading,
  body = EXCLUDED.body,
  icon = EXCLUDED.icon,
  sort_order = EXCLUDED.sort_order;
```

## Cross-References

- → ADR-010 (SQLite Upsert Re-Seed Convention)
- → W-DB (database module plan)
