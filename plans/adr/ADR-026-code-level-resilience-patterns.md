# ADR-026: Code-Level Resilience Patterns

**Date:** 2026-05-14
**Status:** Proposed
**Affected modules:** W-RES, W-UI, W-LLM

## Context

The current Lambda service has no resilience patterns. Transient failures (network timeouts, LLM API rate limits) result in immediate errors to clients. There is no rate limiting, making the service vulnerable to abuse. External dependencies (LLM API, email Lambda) have no circuit breaking, allowing cascading failures.

Constraints:
- Must not introduce external dependencies (Redis, etc.) for resilience state
- Must follow zero-cost philosophy (ADR-005: no additional infrastructure)
- Must use Rust ecosystem solutions (Tower middleware, standard library)
- Must protect against LLM API cost runaway (rate limiting)
- Must handle transient failures gracefully (retry logic)

## Decision

> We will implement code-level resilience patterns using Tower middleware and standard library primitives. In-memory rate limiting protects against abuse, retry with exponential backoff handles transient failures, and circuit breakers prevent cascading failures to external dependencies. Request validation catches malformed input early.

Specific rules:
1. **In-memory rate limiting**: Sliding window per client IP + endpoint, stored in `HashMap<String, Vec<Instant>>`
2. **Retry with exponential backoff**: Tower RetryLayer, max 3 retries, transient error detection only
3. **Circuit breaker**: Atomic state for external LLM calls, opens after 5 consecutive failures
4. **Request validation**: validator crate for struct-level validation, early rejection
5. **Middleware stack**: Ordered as metrics → rate limit → retry → validation → handlers
6. **Per-endpoint limits**: Stricter limits for expensive endpoints (LLM calls)
7. **No external state**: All resilience state in-memory, resets on Lambda restart

## Consequences

### Positive
- Zero infrastructure cost (pure code implementation)
- Protects against LLM cost runaway (rate limiting)
- Handles transient failures gracefully (retry logic)
- Prevents cascading failures (circuit breaker)
- Early rejection of malformed requests (validation)
- Standard Rust middleware patterns (Tower ecosystem)

### Negative / Trade-offs
- In-memory state lost on Lambda restart (acceptable for stateless design)
- Rate limiting per Lambda instance (not distributed, acceptable for portfolio scale)
- No persistent circuit breaker state (reopens on restart, acceptable)
- Middleware stack adds request processing overhead
- Manual transient error detection required

### Neutral
- Rate limiting can be made distributed later if needed (Redis/DynamoDB)
- Circuit breaker state can be persisted to SQLite if needed
- Validation rules can be extended with custom validators
- Middleware order can be adjusted per endpoint requirements

## Alternatives Considered

| Option | Rejected because |
|--------|-----------------|
| AWS WAF | Recurring cost (~$5/month), overkill for portfolio scale |
| API Gateway throttling | Violates ADR-003 (no API Gateway), adds cost |
| Redis for rate limiting | Additional managed service cost, violates ADR-002 |
| External circuit breaker library | Adds dependency complexity, Tower is sufficient |
| Chaos engineering tools | Overkill for portfolio, manual testing sufficient |

## Cross-References

- → W-RES (implementation module)
- → ADR-005 (Zero-cost philosophy — no external dependencies)
- → ADR-003 (Lambda Function URL — no API Gateway features)
- → W-UI (middleware stack integration)
- → W-LLM (circuit breaker for LLM proxy calls)
- → W-OBS (metrics for circuit breaker decisions)
- → W-MOD (module-specific rate limits)
- → ADR-023 (Agentic tool dispatch — external LLM dependency)
