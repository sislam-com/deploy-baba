-- API metrics collection for zero-cost observability (ADR-025: Proposed)
-- Stores request-level metrics for latency analysis, error rates, and request counts
-- Co-located in existing SQLite database (ADR-002) to avoid CloudWatch Metrics cost
-- NOTE: Table created but no writer implemented yet - awaiting ADR-025 acceptance

CREATE TABLE IF NOT EXISTS api_metrics (
    id          INTEGER PRIMARY KEY,
    timestamp   TEXT    NOT NULL,  -- ISO 8601 timestamp (RFC 3339)
    endpoint    TEXT    NOT NULL,  -- Request path (e.g., /api/v1/jobs)
    method      TEXT    NOT NULL,  -- HTTP method (GET, POST, etc.)
    status_code INTEGER NOT NULL,  -- HTTP status code
    duration_ms INTEGER NOT NULL,  -- Request duration in milliseconds
    api_version TEXT    NOT NULL   -- API version (v1, v2, etc.)
);

-- Index for time-series queries by endpoint
CREATE INDEX IF NOT EXISTS idx_metrics_endpoint 
ON api_metrics(endpoint, timestamp DESC);

-- Index for time-series queries overall
CREATE INDEX IF NOT EXISTS idx_metrics_timestamp 
ON api_metrics(timestamp DESC);

-- Index for error rate analysis
CREATE INDEX IF NOT EXISTS idx_metrics_status 
ON api_metrics(status_code, timestamp DESC);

-- Index for API version analysis
CREATE INDEX IF NOT EXISTS idx_metrics_version 
ON api_metrics(api_version, timestamp DESC);
