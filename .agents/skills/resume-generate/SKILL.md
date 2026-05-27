---
name: resume-generate
description: Regenerate resume outputs (md, docx, pdf) from the SQLite DB, optionally updating the Professional Summary. Implements ADR-014 — DB is the single source of truth for all resume content including the Professional Summary.
argument-hint: "[--update-summary]"
---

Regenerate all resume outputs from the current SQLite DB state.
Implements ADR-014: `about_sections.me-bio` is the source of truth for the
Professional Summary; `generate.rs` loads it at generation time, polishes it
to third-person resume tone, and errors if the row is absent.

## When to Use

- After any dashboard edit to jobs, competencies, evidence, about_sections, or social_links.
- After accepting a new me-bio draft in the `/dashboard/about/me-bio` editor.
- As part of a pre-deploy checklist to ensure `target/resume/` is current.
- When the `polish_bio_to_summary()` text itself needs updating to reflect a repositioning.

---

## Step 1 — Verify DB has the me-bio row

```bash
sqlite3 deploy-baba.db "SELECT slug, substr(body,1,120) FROM about_sections WHERE slug='me-bio';"
```

Expected: one row. If missing, the generator will error with:
`about_sections row with slug='me-bio' not found — DB is missing required data`

---

## Step 2 — (Optional) Update the polished summary text

The polished Professional Summary is produced by `polish_bio_to_summary()` in
`xtask/src/resume/generate.rs`. It currently ignores the raw bio text and
returns a fixed third-person paragraph (v1 seam — easy to evolve later).

If the user's positioning has changed and they want new summary text:

1. Read the current bio from DB:
   ```bash
   sqlite3 deploy-baba.db "SELECT body FROM about_sections WHERE slug='me-bio';"
   ```
2. Draft a ~3-sentence third-person resume summary that captures:
   - Years of experience + core domain
   - Key delivery areas / industries
   - Current differentiation / AI-augmented angle
3. Update `polish_bio_to_summary()` in `xtask/src/resume/generate.rs`:
   ```rust
   fn polish_bio_to_summary(_raw_bio: &str) -> String {
       "## Professional Summary\n\n\
   <new polished text here>\n\n"
           .to_string()
   }
   ```
4. Run `cargo fmt && cargo clippy -p xtask` — must be clean before proceeding.

---

## Step 3 — Regenerate all outputs

```bash
just resume-generate
```

Expected output:
```
  Generating chronological resume...
  Written: target/resume/sharful-islam-resume-chronological.md
  Written: target/resume/sharful-islam-resume-chronological.docx
  Written: target/resume/sharful-islam-resume-chronological.pdf
  Generating functional resume...
  Written: target/resume/sharful-islam-resume-functional.md
  Written: target/resume/sharful-islam-resume-functional.docx
  Written: target/resume/sharful-islam-resume-functional.pdf
```

weasyprint CSS warnings are non-fatal — ignore them.

---

## Step 4 — Verify output

```bash
head -20 target/resume/sharful-islam-resume-functional.md
```

Check that the `## Professional Summary` paragraph matches the polished text.

Spot-check employment history is still intact:

```bash
grep "^###\|^-.*@" target/resume/sharful-islam-resume-chronological.md | head -20
```

---

## Step 5 — (Optional) Commit the regenerated files

If you want the `target/resume/` outputs versioned:

```bash
git add target/resume/
git commit -m "chore: regenerate resume from current DB state"
```

Note: `target/resume/` may be gitignored. Check with `git status` first.

---

## Architecture Notes (ADR-014)

| Component | Role |
|-----------|------|
| `about_sections.me-bio` (DB) | Raw first-person bio — serves `/about/me` page |
| `load_me_bio(conn)` | Reads `body` from DB; errors if row absent |
| `polish_bio_to_summary(raw)` | Transforms raw bio → third-person resume paragraph (v1: fixed text) |
| `generate_chronological / generate_functional` | Receive `summary: &str`; no longer reference `SUMMARY` const |
| `SUMMARY` const | **Deleted** — DB is sole source of truth |

The `polish_bio_to_summary` seam exists so future iterations can implement
actual text transformation (NLP, structured extraction, AI rewrite) without
changing the call site in `generate_resume`.

---

## Cross-References

- **ADR-014** `plans/adr/ADR-014-resume-summary-from-db.md` — decision record
- **W-RSM** `plans/modules/resume.md` — resume work-item tracking
- **`xtask/src/resume/generate.rs`** — implementation
- **`just resume-generate`** — justfile entry (L217)
