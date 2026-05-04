# DRL-2026-05-04-adr015-feature-flag-not-implemented

**Date:** 2026-05-04
**Topic:** ADR-015 rule 3 claims feature-flag adapter selection; actual architecture uses llm-proxy Lambda
**Status:** Open

## Observation

ADR-015 (LLM Provider Abstraction) rule 3 states:
> "Cargo feature flags in services/ui select the concrete adapter at compile time."

In practice, `services/ui/Cargo.toml` has no `llm-anthropic` feature flag. Instead, a separate `services/llm-proxy` Lambda handles LLM calls. The UI Lambda sends requests to llm-proxy over HTTP — adapter selection is an infrastructure concern (which Lambda is deployed), not a compile-time feature flag.

## Impact

Medium — the actual architecture is arguably better (cleaner separation, no vendor SDK in the VPC Lambda), but the ADR claim doesn't match reality. Anyone reading ADR-015 to understand how to add a new LLM provider would be misled about the wiring mechanism.

## Recommended Resolution

Update ADR-015 rule 3 to describe the llm-proxy Lambda architecture. The feature-flag pattern described in the ADR and in `plans/modules/llm-core.md` §W-LLM.3 should be revised to reflect the proxy-based adapter selection.
