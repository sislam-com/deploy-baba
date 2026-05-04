# DRL-2026-05-03-rustsec-webpki-cves

**Date:** 2026-05-03
**Status:** RESOLVED
**Domain:** W-XT (audit step), W-SEC (TLS chain)
**Discovered by:** `cargo audit` (step 5 of `just quality`)

---

## Observations

`cargo audit` reported **3 vulnerabilities** in `rustls-webpki 0.101.7`:

| Advisory | Summary |
|----------|---------|
| RUSTSEC-2026-0098 | Name constraints incorrectly accept URI names |
| RUSTSEC-2026-0099 | Name constraints accepted for wildcard certificates |
| RUSTSEC-2026-0104 | Reachable panic in CRL parsing |

Provenance (from `Cargo.lock`):
```
aws-config 1.8.16
  → aws-smithy-http-client 1.1.12
    → hyper-rustls 0.24.2
      → rustls 0.21.12
        → rustls-webpki 0.101.7  ← VULNERABLE
```

The workspace's `reqwest 0.12` chain already used `rustls 0.23` / `rustls-webpki 0.103` (unaffected). Both chains coexisted in the lockfile, causing the old chain to appear in the audit.

A separate unmaintained advisory `RUSTSEC-2024-0370` (proc-macro-error 1.0.4, via `utoipa-gen 4.3.1`) was present as an allowed warning — it did not fail the gate.

---

## Root Cause

All `aws-sdk-*` workspace dependencies used implicit default features, which includes `hyper-rustls` 0.24 (the old TLS backend). `aws-smithy-http-client 1.1.12`'s default features pull in `hyper-rustls 0.24` → `rustls 0.21` → `rustls-webpki 0.101.7`.

---

## Fix Applied

Added `default-features = false` to all `aws-sdk-*` entries in `Cargo.toml` `[workspace.dependencies]`, with explicit `features = ["default-https-client", "rt-tokio"]` (and `"sigv4a"`, `"http-1x"`, `"behavior-version-latest"` where already required):

```toml
aws-sdk-sts        = { version = "1", default-features = false, features = ["sigv4a", "default-https-client", "rt-tokio"] }
aws-sdk-ssm        = { version = "1", default-features = false, features = ["default-https-client", "rt-tokio"] }
aws-sdk-s3         = { version = "1", default-features = false, features = ["sigv4a", "http-1x", "default-https-client", "rt-tokio"] }
aws-sdk-lambda     = { version = "1", default-features = false, features = ["default-https-client", "rt-tokio"] }
aws-sdk-ecs        = { version = "1", default-features = false, features = ["default-https-client", "rt-tokio"] }
aws-sdk-ecr        = { version = "1", default-features = false, features = ["default-https-client", "rt-tokio"] }
aws-sdk-efs        = { version = "1", default-features = false, features = ["default-https-client", "rt-tokio"] }
aws-sdk-dynamodb   = { version = "1", default-features = false, features = ["default-https-client", "rt-tokio"] }
aws-sdk-sesv2      = { version = "1", default-features = false, features = ["behavior-version-latest", "sigv4a", "default-https-client", "rt-tokio"] }
aws-sdk-secretsmanager = { version = "1", default-features = false, features = ["default-https-client", "rt-tokio"] }
```

`aws-config` retains its default features (does not pull the old hyper-rustls chain independently).

**Effect:** `hyper-rustls 0.24`, `rustls 0.21.12`, and `rustls-webpki 0.101.7` are no longer in `Cargo.lock`. Only `rustls-webpki 0.103.13` remains (from reqwest's chain).

---

## Verification

```
$ grep -A2 '^name = "rustls-webpki"' Cargo.lock
name = "rustls-webpki"
version = "0.103.13"            ← only entry, no 0.101.x

$ cargo audit
0 vulnerabilities found
1 warning: proc-macro-error (RUSTSEC-2024-0370) — allowed, not a failure
```

`just quality` exits 0 including the audit step.

---

## Out of Scope

- `proc-macro-error` RUSTSEC-2024-0370 (unmaintained, via `utoipa-gen 4.3.1`) — deferred to W-UI.4.1 (`utoipa` 4 → 5 migration). Not a vulnerability; does not fail the gate.

---

## Cross-References
- → `Cargo.toml` — `[workspace.dependencies]` aws-sdk-* entries
- → `Cargo.lock` — `rustls-webpki` (now single entry at 0.103.13)
- → `plans/modules/xtask.md` — W-XT.5 audit step
- → `plans/cross-cutting/quality-gates.md` — `cargo audit` policy
