# DRL-2026-10-15 — dev-stack Port Shadowing (Vite proxy loop)

**Date:** 2026-10-15
**Modules:** W-DEV, W-WEB, W-AGT
**Status:** RESOLVED

## Entries

### DRL-DEVPORT-1: Vite silently drifted onto :3001 and proxied to itself
- **Claim:** `just dev-stack` serves the SPA on :3000 and proxies `/api` to the ui service on :3001.
- **Reality:** A foreign process (an unrelated local WhatsApp bridge) held `127.0.0.1:3000`. Vite had no `strictPort`, so it fell back to `127.0.0.1:3001`. macOS allows that bind next to the ui service's `*:3001` wildcard, so Vite shadowed the API on loopback. The proxy (`/api → localhost:3001`) then looped into Vite until ephemeral ports ran out, which produced `EADDRNOTAVAIL` / `ETIMEDOUT` on every `/api/*` request.
- **Fix:** `strictPort: true` and `127.0.0.1` proxy targets in `web/vite.config.ts`.
- **Status:** RESOLVED

### DRL-DEVPORT-2: Preflight port cleanup silently failed
- **Claim:** The dev-stack preflight frees ports 3000–3004 and 5555.
- **Reality:** `lsof -ti tcp:$port` also matched connected clients and returned a multi-line PID list. `kill "$pid"` failed, and `|| true` swallowed the failure. The same lookup could also have killed unrelated clients, such as browser tabs.
- **Fix:** Match `-sTCP:LISTEN` only and iterate PIDs, skipping any that already exited (a `uvicorn --reload` parent takes its worker down with it; under `set -euo pipefail` a failed lookup on the dead PID used to end the run silently). Kill only this stack's processes (repo cwd or a known command). Abort with a clear message when a foreign listener holds a port.
- **Status:** RESOLVED

### DRL-DEVPORT-3: Stale port documentation + inconsistent UI_SERVICE_URL default
- **Claim:** W-DEV documents Vite on :5173 and the Lambda binary on :3000. The agent reaches the ui service on :3001.
- **Reality:** The devcontainer and plan text were stale. `rag_sync.py` defaulted `UI_SERVICE_URL` to :3000 (through the Vite proxy), and dev-stack never exported it.
- **Fix:** Added a port registry to `plans/modules/dev-environment.md`, synced devcontainer `forwardPorts`, changed the `rag_sync.py` default to :3001, and exported `UI_SERVICE_URL` in dev-stack.
- **Status:** RESOLVED
