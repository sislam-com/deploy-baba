# DRL-2026-05-09-rag-challenges-corpus

**Date:** 2026-05-09
**Severity:** low
**Affected modules:** W-RAG, W-CHL

## Summary

Plan-sync audit (2026-05-09) revealed that ADR-016 (RAG Architecture) documents 6 RAG corpora (Rust, HCL, plans, cache, OpenAPI, portfolio) but the codebase now implements 7 corpora including the challenges corpus. The challenges feature was fully implemented in the feat/challenges branch but was never documented in the plan system.

## Entries

| ID | Finding | Status | Resolution |
|----|---------|--------|-----------|
| DRL-CHL-1 | ADR-016 corpus table shows 6 corpora but code implements 7 (including challenges) | RESOLVED | Updated W-RAG module plan corpus table to include challenges as 7th corpus; added W-RAG.11.x work items documenting challenges integration |
| DRL-CHL-2 | W-CHL module plan missing from plan system | RESOLVED | Created `plans/modules/challenges.md` with full work items W-CHL.4.1–4.13 |
| DRL-CHL-3 | W-CHL domain code missing from CONVENTIONS.md | RESOLVED | Added W-CHL row to domain codes table in CONVENTIONS.md |
| DRL-CHL-4 | W-CHL missing from INDEX.md Module Status Table | RESOLVED | Added W-CHL row to INDEX.md Module Status Table with WIP status |
| DRL-CHL-5 | Challenges work items missing from INDEX.md remaining work | RESOLVED | Added W-CHL.4.10–4.13 to P2.5 Content Features section |
| DRL-CHL-6 | INDEX.md branch reference outdated (feat/llm-core vs feat/challenges) | RESOLVED | Updated branch reference to feat/challenges |
| DRL-CHL-7 | Agent cache stale (migration_count: 14 vs actual 22) | PENDING | Requires `just cache-refresh` to update cache |

## Lessons Learned

- New features should be documented in the plan system immediately after implementation
- Corpus additions to RAG system require updates to both ADR-016 and W-RAG module plan
- Domain codes must be added to CONVENTIONS.md when new modules are created
- Agent cache can become stale even when SHA matches - content drift vs SHA drift
- Plan-sync audit effectively detects missing modules, stale references, and corpus count mismatches

## Cross-References

- → ADR-016 (RAG Architecture — corpus documentation updated)
- → W-RAG (module plan updated with challenges corpus and work items)
- → W-CHL (new module plan created)
- → W-UI (challenges API routes)
- → W-WEB (challenges admin dashboard UI)
- → W-SYNC (dashboard → migrations sync workflow)
- → plans/INDEX.md (updated with W-CHL references)
- → plans/CONVENTIONS.md (updated with W-CHL domain code)