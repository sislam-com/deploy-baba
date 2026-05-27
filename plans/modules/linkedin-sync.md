# W-LINK: LinkedIn Sync Integration
**Service:** `services/ui/` + `infra/` + `web/` | **Status:** WIP
**Depends on:** W-AUTH (admin gating), W-SEC (secrets infra) | **Depended on by:** W-RST (resume tailoring comparison)

## W-LINK.1 Purpose

Compare LinkedIn profile data (positions, projects) against internal DB records to identify divergence between:
- What LinkedIn displays publicly
- What the tailored resume system uses internally
- What the admin has curated in the dashboard

Primary data source: LinkedIn's CSV data export (Settings > Data Privacy > Get a copy of your data).
Live API integration deferred — LinkedIn Profile API does not expose positions/projects/skills.

## W-LINK.2 Design

### Data flow
```
LinkedIn CSV export → paste/upload in dashboard → parse → linkedin_positions / linkedin_projects tables
                                                       → auto-match against jobs / challenges tables
                                                       → diff view in admin UI
```

### Tables
- `linkedin_positions` — imported positions with `mapped_job_id` FK + `sync_status`
- `linkedin_projects` — imported projects with `mapped_challenge_id` FK + `sync_status`
- `linkedin_sync_log` — import history metadata

### Sync statuses
- `unreviewed` — newly imported, not yet checked
- `synced` — mapped to internal record, fields match
- `diverged` — mapped but fields differ
- `linkedin_only` — no matching internal record
- `local_only` — internal record with no LinkedIn match

## W-LINK.3 Implementation Notes

- Admin endpoints under `/api/v1/admin/linkedin/` (Cognito-gated)
- CSV parser handles LinkedIn's `Mon YYYY` date format → `YYYY-MM`
- Auto-matching: company name (case-insensitive) + date overlap for positions; title similarity for projects
- Secret `linkedin-api-key` pre-wired in Secrets Manager per environment (dev/prod) but unused until API access granted

## W-LINK.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-LINK.4.1 | Add `linkedin-api-key` to `infra/secrets.tf` + IAM policy | TODO | Future-proofing; follows W-SEC pattern |
| W-LINK.4.2 | Register `linkedin-api-key` in `xtask/src/secret.rs` | TODO | |
| W-LINK.4.3 | Migration `032_create_linkedin_sync.sql` | TODO | 3 tables |
| W-LINK.4.4 | OpenAPI models in `api-openapi` | TODO | LinkedInPosition, LinkedInProject, etc. |
| W-LINK.4.5 | API endpoints in `services/ui/src/routes/api/linkedin.rs` | TODO | 10 endpoints |
| W-LINK.4.6 | Nest linkedin router in admin.rs | TODO | |
| W-LINK.4.7 | Dashboard nav + LinkedInSync.tsx | TODO | Main sync page with import + tabs |
| W-LINK.4.8 | LinkedInPositionDiff.tsx | TODO | Side-by-side position comparison |
| W-LINK.4.9 | LinkedInProjectDiff.tsx | TODO | Side-by-side project comparison |
| W-LINK.4.10 | Sync status badges on Jobs.tsx + Challenges.tsx | TODO | |
| W-LINK.4.11 | Dashboard home LinkedIn sync tile | TODO | |
| W-LINK.4.12 | CSV parser for LinkedIn export format | TODO | multipart/form-data upload |
| W-LINK.4.13 | Auto-matching algorithm | TODO | Company name + dates; title similarity |

## W-LINK.5 Test Strategy

- Unit: CSV parser date format conversion, auto-matching logic
- Integration: import → list → diff → map → status update flow
- UI: manual verification via `just ui` + `just web`

## W-LINK.6 Cross-References

- -> ADR-010 (upsert convention for re-imports)
- -> W-AUTH (require_auth middleware)
- -> W-SEC (Secrets Manager pattern)
- -> W-RSM (jobs table — mapped positions)
- -> W-CHL (challenges table — mapped projects)
