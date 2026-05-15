# W-MOD: module-decomposition
**Path:** `services/ui/src/modules/` | **Status:** TODO
**Coverage floor:** 70% | **Depends on:** W-UI, W-VER, W-OBS, W-RES | **Depended on by:** (future Lambda extraction)

---

## W-MOD.1 Purpose

Logical module separation within the single Lambda service (portfolio, RAG, admin, auth modules). Enables independent testing, clear boundaries, and provides a future extraction path to separate Lambdas if scaling needs require it. Maintains zero-cost principle by using code-level organization instead of infrastructure changes.

→ ADR-027

---

## W-MOD.2 Public API Surface

### Module Structure

```rust
// services/ui/src/modules/mod.rs
pub mod portfolio;    // Jobs, competencies, about, social-links
pub mod rag;          // RAG retrieval, /api/ask
pub mod admin;        // Dashboard CRUD operations
pub mod auth;         // Cognito token validation

// Each module has:
// - router() function
// - error types
// - state management
// - metrics collection
```

### Module Router Interface

```rust
pub trait ModuleRouter {
    fn router() -> Router<AppState>;
    fn module_name() -> &'static str;
    fn metrics_prefix() -> &'static str;
}
```

### Module-Specific State

```rust
// Each module can have its own state extensions
pub struct PortfolioModuleState {
    db: SqlitePool,
    cache: Arc<Mutex<LruCache<String, Job>>>,
}

pub struct RagModuleState {
    retriever: Arc<HybridRetriever>,
    embedder: Option<Arc<dyn Embedder>>,
}
```

---

## W-MOD.3 Implementation Notes

### Module File Structure

```
services/ui/src/modules/
├── mod.rs              # Module trait definitions
├── portfolio/
│   ├── mod.rs          # Portfolio module router
│   ├── jobs.rs         # Jobs handlers
│   ├── competencies.rs # Competencies handlers
│   ├── about.rs        # About handlers
│   └── social_links.rs # Social links handlers
├── rag/
│   ├── mod.rs          # RAG module router
│   ├── ask.rs          # /api/ask handler
│   └── metrics.rs      # RAG-specific metrics
├── admin/
│   ├── mod.rs          # Admin module router
│   ├── jobs.rs         # Admin jobs CRUD
│   ├── competencies.rs # Admin competencies CRUD
│   └── challenges.rs    # Admin challenges CRUD
└── auth/
    ├── mod.rs          # Auth module router
    └── middleware.rs   # Auth validation middleware
```

### Module Router Composition

```rust
// services/ui/src/router.rs
use crate::modules::{portfolio, rag, admin, auth};

pub fn build(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1/portfolio", portfolio::router())
        .nest("/api/v1/rag", rag::router())
        .nest("/api/v1/admin", admin::router())
        .nest("/api/v1/auth", auth::router())
        .with_state(state)
}
```

### Module-Specific Metrics

```rust
// Each module records metrics with its own prefix
portfolio::record_metric(&db, "/api/v1/portfolio/jobs", "GET", 200, 45, "v1").await;
rag::record_metric(&db, "/api/v1/rag/ask", "POST", 200, 1200, "v1").await;
admin::record_metric(&db, "/api/v1/admin/jobs", "GET", 200, 35, "v1").await;
```

### Module-Specific Rate Limits

```rust
// Configure different rate limits per module
let portfolio_limiter = RateLimiter::new(100, Duration::from_secs(60));
let rag_limiter = RateLimiter::new(10, Duration::from_secs(60));  // Stricter for LLM cost
let admin_limiter = RateLimiter::new(50, Duration::from_secs(60));
```

### Module Testing

```rust
// Each module has independent tests
#[cfg(test)]
mod portfolio_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_list_jobs() {
        // Test portfolio module in isolation
    }
}

#[cfg(test)]
mod rag_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rag_ask() {
        // Test RAG module in isolation
    }
}
```

### Future Lambda Extraction Path

When scaling needs require separate Lambdas:

1. **Phase 1**: Move module to separate binary crate
   ```toml
   # services/portfolio-lambda/Cargo.toml
   [dependencies]
   portfolio-module = { path = "../ui/src/modules/portfolio" }
   ```

2. **Phase 2**: Deploy separate Lambda with shared EFS
   ```hcl
   # infra/portfolio-lambda.tf
   resource "aws_lambda_function" "portfolio" {
     function_name = "deploy-baba-portfolio"
     # ... same EFS mount as main Lambda
   }
   ```

3. **Phase 3**: Route via CloudFront (no API Gateway)
   ```hcl
   # CloudFront origin per Lambda
   # Path pattern: /api/v1/portfolio/*
   ```

---

## W-MOD.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-MOD.4.1 | Create module trait definitions | TODO | ModuleRouter trait, state interfaces |
| W-MOD.4.2 | Extract portfolio module | TODO | Move jobs/competencies/about/social-links handlers |
| W-MOD.4.3 | Extract RAG module | TODO | Move /api/ask handler, RAG-specific metrics |
| W-MOD.4.4 | Extract admin module | TODO | Move dashboard CRUD handlers |
| W-MOD.4.5 | Extract auth module | TODO | Move auth validation, middleware |
| W-MOD.4.6 | Add module-specific metrics | TODO | Per-module metric prefixes |
| W-MOD.4.7 | Add module-specific rate limits | TODO | Different limits per module |
| W-MOD.4.8 | Add independent module tests | TODO | Isolated test suites per module |

---

## W-MOD.5 Test Strategy

- Independent test suites per module
- Module router integration tests
- Module-specific metrics validation
- Module-specific rate limit tests
- Test coverage floor: 70% (lower due to shared code)

---

## W-MOD.6 Cross-References

- → ADR-027 (Module-Based Service Decomposition)
- → ADR-005 (Zero-cost philosophy — code-level organization)
- → ADR-003 (Lambda Function URL — future multi-Lambda routing)
- → W-UI (router composition)
- → W-VER (version-aware module routing)
- → W-OBS (module-specific metrics)
- → W-RES (module-specific rate limits)
- → W-RAG (RAG module extraction)
- → W-AUTH (auth module extraction)
