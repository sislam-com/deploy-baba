# DRL-2026-05-04-adr005-askama-claim

**Date:** 2026-05-04
**Topic:** ADR-005 rule 2 references Askama, which was removed by ADR-019
**Status:** Open

## Observation

ADR-005 (HTML-First UI Architecture) rule 2 states:
> "All HTML is rendered via Askama typed templates."

However, ADR-019 replaced Askama with Minijinja as the template engine. The codebase confirms Minijinja is in use — `askama` does not appear in any `Cargo.toml`.

## Impact

Low — ADR-005 is structurally sound except for the stale engine name. The principle (typed server-rendered templates) still holds; only the specific engine reference is wrong.

## Recommended Resolution

Update ADR-005 rule 2 to reference Minijinja instead of Askama, or add an addendum noting that ADR-019 supersedes the engine choice while preserving the architectural intent.
