---
name: cache-refresh
description: Re-derive .agent-cache/index.json from current repo state by invoking the existing xtask cache refresh command. Verifies idempotency. Use whenever cache SHA diverges from HEAD or after a session that touched multiple components.
---

# /cache-refresh

Refreshes `.agent-cache/index.json` by calling the existing `xtask cache refresh` implementation (`xtask/src/cache.rs`). This skill is a thin invocation wrapper — it does NOT re-implement the cache logic.

## How to run

```
/cache-refresh          # update all components
```

## Implementation steps

When this skill is invoked:

### Step 1 — Run the existing cache refresh

```bash
just cache-refresh
```

This calls `cargo run -q -p xtask -- cache refresh`, which:
- Reads current git SHA via `git rev-parse HEAD`.
- Reads status fields from each `plans/modules/*.md`.
- Reads per-component latest git SHAs.
- Rewrites `.agent-cache/index.json` with updated `git.sha`, `last_updated`, per-component `git_sha_at_scan`, and status map.
- Preserves all other fields (descriptions, files arrays, `pending_operator_actions`).

### Step 2 — Verify idempotency

Run a second time:
```bash
just cache-refresh
```

Then check if the file changed:
```bash
git diff .agent-cache/index.json
```

If there is a diff on the second run, that is a bug in the cache refresh logic — report it to the user.

### Step 3 — Report

Show the user a one-line summary: "Cache refreshed: git.sha updated to `<sha>`, last_updated set to `<timestamp>`."

If idempotency check showed no diff: "Idempotency verified — second run produced no changes."

## What it never does

- Never modifies source code, plans, infra, or CI files.
- Never commits the updated cache file.
- Never calls `cargo`, `pnpm`, or `tofu` outside of `just cache-refresh`.
- Never re-implements the cache logic from `xtask/src/cache.rs` inline.
