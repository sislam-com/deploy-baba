# DRL-2026-05-02-askama-workspace-orphan

**ADR:** ADR-019 | **Detected:** 2026-05-02 | **Severity:** Minor (cleanup)
**Status:** Resolved (2026-05-04)

## Divergence

`/Users/shantopagla/portfolio/Cargo.toml` `[workspace.dependencies]` still declared:
```toml
askama = "0.12"
askama_axum = "0.4"
```
No workspace member crate depended on either entry. The Askama template engine was removed from `services/ui/Cargo.toml` as part of the ADR-019 SPA migration (Phase D.5, 2026-04-30), but the workspace-level declarations were not pruned.

Additionally, `web/tsconfig.json` does not contain `"strict": true` — that setting lives in `web/tsconfig.app.json:14`. TypeScript strict mode is enforced in practice via project references, but the ADR's stated file location is inaccurate.

## Impact

- Orphaned workspace deps: no compile error, but misleads readers and bloats metadata.
- tsconfig claim: drift-detector would false-negative if checking `tsconfig.json` directly.

## Resolution

1. Removed `askama` and `askama_axum` from `[workspace.dependencies]` in root `Cargo.toml`.
2. tsconfig reference: deferred (ADR-019 wording update tracked separately).
