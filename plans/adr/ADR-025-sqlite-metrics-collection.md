# ADR-025: SQLite-Based Metrics Collection

**Date:** 2026-05-14
**Status:** Proposed
**Affected modules:** W-OBS, W-UI, W-RAG

## Context

The current monitoring setup uses basic CloudWatch Logs with 14-day retention. There is no metrics collection for latency percentiles, error rates, or request counts. CloudWatch Metrics would provide these capabilities but introduces additional AWS cost ($0.50 per metric per month). For a zero-cost portfolio project, this recurring cost is undesirable.

Constraints:
- Must not introduce CloudWatch Metrics cost (ADR-005: zero-cost philosophy)
- Must use existing infrastructure (ADR-002: SQLite on EFS)
- Must provide queryable time-series data (p50, p95, p99 latencies)
- Must integrate with existing logging (structured JSON logs)
- Must support RAG-specific metrics (retrieval latency, chunk counts)

## Decision

> We will implement SQLite-based metrics collection using the existing EFS-mounted SQLite database. A new `api_metrics` table stores request data with indexes for time-series queries. Structured logging with tracing provides real-time observability, while SQLite enables historical analysis without CloudWatch Metrics cost.

Specific rules:
1. **SQLite metrics table**: `api_metrics` with timestamp, endpoint, method, status_code, duration_ms, api_version
2. **Structured logging**: JSON-formatted logs via tracing crate with request IDs
3. **Async recording**: Metrics recorded fire-and-forget to avoid blocking request handlers
4. **Query endpoint**: `GET /api/v1/metrics` with filtering by endpoint and time range
5. **Percentile calculation**: p50, p95, p99 computed in-memory from query results
6. **RAG-specific metrics**: Separate table or columns for retrieval/generation latency
7. **Retention**: Metrics follow same backup/retention policy as main database

## Consequences

### Positive
- Zero additional AWS cost (uses existing SQLite on EFS)
- Queryable time-series data with standard SQL
- Structured JSON logs improve real-time debugging
- Metrics co-located with application data (single backup)
- Flexible queries beyond pre-defined metrics
- RAG-specific metrics track retrieval quality

### Negative / Trade-offs
- SQLite write overhead on every request (mitigated by async recording)
- Query performance degrades with large datasets (mitigated by indexes + retention)
- No CloudWatch Metrics integration (no AWS console dashboards)
- Percentile calculation requires loading data into memory (acceptable for portfolio scale)
- Metrics lost if Lambda fails before async write completes (acceptable for observability)

### Neutral
- Metrics retention tied to database backup policy (S3 lifecycle rules)
- Query endpoint requires auth (can use existing admin auth)
- Can be migrated to CloudWatch Metrics later if scale requires it
- Structured logs can be shipped to external observability platforms if needed

## Alternatives Considered

| Option | Rejected because |
|--------|-----------------|
| CloudWatch Metrics | Recurring cost ($0.50/metric/month), violates ADR-005 |
| DynamoDB metrics | Additional managed service cost, violates ADR-002 |
| In-memory metrics only | Lost on Lambda restart, no historical analysis |
| External SaaS (Datadog, New Relic) | Recurring cost, overkill for portfolio |
| CloudWatch Logs Insights | Query cost ($0.005 per GB scanned), less flexible than SQL |

## Cross-References

- → W-OBS (implementation module)
- → ADR-002 (SQLite on EFS — metrics co-located)
- → ADR-005 (Zero-cost philosophy — no CloudWatch Metrics)
- → ADR-010 (Upsert convention — metrics use INSERT, not upsert)
- → W-UI (middleware integration)
- → W-RAG (RAG-specific metrics)
- → W-RES (metrics-based circuit breaking)
- → W-MOD (module-specific metrics)
