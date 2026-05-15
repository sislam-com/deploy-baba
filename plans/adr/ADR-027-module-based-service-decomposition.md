# ADR-027: Module-Based Service Decomposition

**Date:** 2026-05-14
**Status:** Proposed
**Affected modules:** W-MOD, W-UI, W-RAG, W-AUTH

## Context

The current Lambda service is a monolithic binary with all handlers in a single codebase. As the service grows (portfolio data, RAG, admin dashboard, auth), the lack of clear boundaries makes testing difficult and obscures dependencies. Traditional microservices would extract these into separate Lambdas immediately, but this introduces infrastructure complexity and cost that may not be justified at current scale.

Constraints:
- Must not introduce infrastructure changes initially (ADR-005: zero-cost first)
- Must provide clear module boundaries for independent testing
- Must enable future extraction to separate Lambdas if scaling needs require
- Must maintain existing deployment pipeline (single Lambda zip)
- Must follow zero-cost philosophy (scale up only when needed)

## Decision

> We will implement logical module separation within the single Lambda service using Rust module organization. Each domain (portfolio, RAG, admin, auth) becomes a separate module with its own router, error types, state management, and metrics. This provides clear boundaries and independent testing while maintaining a single deployment unit. Future extraction to separate Lambdas is straightforward if scale requires it.

Specific rules:
1. **Module trait definition**: `ModuleRouter` trait defines interface for all modules
2. **Logical separation**: Each module in `services/ui/src/modules/{domain}/`
3. **Independent testing**: Each module has isolated test suite
4. **Module-specific metrics**: Per-module metric prefixes for observability
5. **Module-specific rate limits**: Different limits per module (stricter for LLM calls)
6. **Router composition**: Main router composes module routers
7. **Future extraction path**: Modules can be moved to separate binary crates when needed

## Consequences

### Positive
- Clear module boundaries without infrastructure cost
- Independent testing per module (faster test runs)
- Module-specific metrics and rate limits
- Future extraction path to separate Lambdas is straightforward
- Reduced cognitive load (smaller, focused modules)
- Easier onboarding (new developers focus on specific modules)

### Negative / Trade-offs
- Initial refactoring effort to extract modules
- Some code duplication across modules (mitigated by shared utilities)
- Module boundaries may need adjustment over time
- Single Lambda still scales as a unit (no independent scaling yet)

### Neutral
- Module extraction to separate Lambdas is optional, not required
- Can extract modules incrementally based on scaling needs
- Shared utilities can be extracted to separate crate if needed
- Module boundaries may evolve with requirements

## Alternatives Considered

| Option | Rejected because |
|--------|-----------------|
| Immediate microservices (separate Lambdas) | Infrastructure complexity/cost not justified at current scale |
| Feature folders only | No clear boundaries, shared state makes testing difficult |
| Monorepo with separate services | Overkill for current needs, adds deployment complexity |
| Domain-driven design with bounded contexts | Too theoretical, module separation is sufficient |
| No decomposition | Monolithic codebase becomes unmaintainable as it grows |

## Cross-References

- → W-MOD (implementation module)
- → ADR-005 (Zero-cost philosophy — scale up only when needed)
- → ADR-003 (Lambda Function URL — future multi-Lambda routing)
- → W-UI (router composition from modules)
- → W-VER (version-aware module routing)
- → W-OBS (module-specific metrics)
- → W-RES (module-specific rate limits)
- → W-RAG (RAG module extraction)
- → W-AUTH (auth module extraction)
