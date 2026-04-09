# ADR-014: Resume Professional Summary Sourced from DB (about_sections.me-bio)

**Date:** 2026-04-09
**Status:** Accepted
**Affected modules:** W-RSM, W-XT

## Context

`xtask/src/resume/generate.rs` contained a hardcoded `SUMMARY` constant (lines 21–29)
that defined the Professional Summary section for both the chronological and functional
resumes. The constant was never connected to the DB — it drifted silently while the
`about_sections.me-bio` row (which serves the `/about/me` public page) was kept
up-to-date via the admin dashboard.

After a meaningful repositioning edit to `me-bio` (AI-augmented SaaS/PaaS engineer,
hierarchy-problem focus, portfolio portal as AI playground), the disconnect became
visible: regenerating the resume would not pick up the new content at all.

The `me-bio` row is already the canonical author-voice bio for the site. The resume
summary is a derivative of that text — tighter, third-person, resume-toned — but
sourced from the same positioning intent.

## Decision

> Delete the hardcoded `SUMMARY` const. Load the raw bio from `about_sections.me-bio`
> at generation time and transform it into a polished Professional Summary. The generator
> errors (no fallback) when the DB row is absent.

Specific rules:

1. `load_me_bio(conn: &Connection) -> anyhow::Result<String>` reads
   `SELECT body FROM about_sections WHERE slug = 'me-bio'`. Returns an error
   (not a default string) when the row is missing — the DB is required, not optional.

2. `polish_bio_to_summary(raw_bio: &str) -> String` produces the third-person
   resume paragraph. v1 returns a fixed polished string (the `_raw_bio` argument is
   accepted but unused). The seam exists so future iterations can implement real
   text transformation without changing the call site.

3. `generate_chronological` and `generate_functional` accept `summary: &str` as an
   explicit argument instead of referencing a module-level constant.

4. The `/about/me` page is unaffected — `me-bio.body` remains first-person conversational
   in the DB. No DB write occurs during resume generation.

5. Updating the polished summary text requires editing `polish_bio_to_summary()` in
   `generate.rs`, running `cargo fmt && cargo clippy -p xtask`, and re-running
   `just resume-generate`.

## Consequences

### Positive
- The resume and `/about/me` page share the same positioning source (`me-bio`).
  A dashboard edit to the bio is the single trigger for a resume refresh.
- Generator fails loudly on missing data rather than silently emitting stale content.
- `polish_bio_to_summary` seam keeps transformation logic co-located and easy to evolve
  (e.g., AI rewrite, structured extraction) without touching the generation call sites.

### Negative / Trade-offs
- v1 of `polish_bio_to_summary` ignores the raw bio content; the polished text is still
  hand-written. A bio edit will not automatically propagate to the resume paragraph
  until `polish_bio_to_summary` is updated and `just resume-generate` is rerun.
- Anyone running `just resume-generate` against a DB that lacks `about_sections.me-bio`
  will see an error rather than a degraded-but-working resume.

### Neutral
- The `HEADER`, `EDUCATION`, and contact line remain hardcoded constants — they have no
  corresponding DB table yet.
- `target/resume/` outputs should be regenerated after any dashboard edit to jobs,
  competencies, evidence, or `me-bio`. See the `/resume-generate` skill.

## Alternatives Considered

| Option | Rejected because |
|--------|-----------------|
| Keep `SUMMARY` const, update manually | Silent drift between bio and resume; defeats the SSOT principle |
| Write polished text back to DB (new column) | Adds schema complexity for a derivative of existing data |
| AI rewrite at generation time (Claude API call) | Adds network dependency and non-determinism to a local CLI tool |
| Fallback to `SUMMARY` const when DB row missing | Masks missing data; contradicts "errors over silent failures" principle |

## Cross-References

- → W-RSM (resume work-item tracking)
- → W-XT (`xtask/src/resume/generate.rs` implementation)
- → ADR-002 (SQLite as single source of truth for application data)
- → ADR-010 (upsert convention — ensures `me-bio` row survives re-deploys)
- → `/resume-generate` skill (`.claude/skills/resume-generate/SKILL.md`)
