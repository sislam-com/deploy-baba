# DRL-2026-05-04-adr005-askama-claim

**Date:** 2026-05-04
**Topic:** ADR-005 rule 2 references Askama, which was removed by ADR-019
**Status:** Resolved (2026-05-04)

## Observation

ADR-005 (Zero-Cost Philosophy) rule 2 states:
> "Compile-time template embedding — services/ui/ uses Askama..."

However, ADR-019 replaced all 15 Askama templates with a React/Vite SPA in `web/`. The `services/ui` crate now serves JSON only.

## Impact

Low — ADR-005 is structurally sound except for the stale engine reference. The zero-cost principle still holds via content-hashed static assets from S3/EFS.

## Resolution

ADR-005 rule 2 updated with ADR-019 supersession addendum. Zero-cost rationale preserved.
