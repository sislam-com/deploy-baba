# DRL-2026-05-02-contact-response-dual-definition

**ADR:** ADR-012 | **Detected:** 2026-05-02 | **Severity:** Medium

## Divergence

`services/ui/src/routes/contact.rs` defines three private/public Rust structs:
- `ChallengeResponse` (line 189)
- `ContactSubmitRequest` (line 197)
- `ContactResponse` (line 221)

Canonical definitions of all three types live in `crates/api-openapi/src/models/contact.rs` and implement `ApiModel + ToSchema`. The local copies in `contact.rs` do not implement `ToSchema` and are not registered in `ALL_MODELS`.

The `#[utoipa::path]` annotations added in 2026-05-02 correctly reference `api_openapi::models::*` for spec generation (so the spec is accurate), but the handler code constructs and returns the local structs at runtime.

ADR-012 Decision rule: "No request/response struct may be defined in `services/ui`."

## Impact

- If a field is added to the canonical `api_openapi::models::ChallengeResponse` (e.g., adding a `version` field), the local copy silently omits it and the HTTP response diverges from the documented spec.
- The two codepaths (spec vs. handler) can drift independently with no compile-time warning.

## Recommended Fix

Remove the local struct definitions from `contact.rs`. Import and use `api_openapi::models::{ChallengeResponse, ContactSubmitRequest, ContactResponse}` directly in the handler. The structs have the same wire shape — this is a mechanical substitution.
