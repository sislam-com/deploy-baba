---
name: add-route
description: Add a new Axum JSON API endpoint and/or React page route following the services/ui + web/ SPA patterns. Handles handler creation, router registration, and React component wiring.
argument-hint: "[route-path] [handler-name]"
---

Add a new page or API endpoint to the portfolio. Backend routes use Axum (JSON API). Frontend pages are React components in `web/` (ADR-019).

## Decision: Page Route vs API Endpoint?

- **Page route** (visible in browser) → Steps 1–4
- **API endpoint** (JSON, public or auth-gated) → Step 5
- **Both** (new page that needs new data) → Steps 1–5

## Steps for a Page Route

### 1. Create the React component

Path: `web/src/routes/<Name>.tsx` (public) or `web/src/routes/dashboard/<Name>.tsx` (auth-gated)

```tsx
export default function Name() {
  return (
    <section className="max-w-4xl mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold text-white mb-6">Page Title</h1>
    </section>
  );
}
```

For pages that fetch data, use `useEffect` + `useState` to call the corresponding `/api/*` endpoint.

### 2. Register in App.tsx

File: `web/src/App.tsx`

For public routes, add inside the `<Layout>` wrapper:
```tsx
<Route path="/path" element={<Name />} />
```

For dashboard routes, add inside the `<DashboardLayout>` wrapper:
```tsx
<Route path="/dashboard/path" element={<Name />} />
```

### 3. Add navigation link (if needed)

- **Public nav:** Edit `web/src/components/Layout.tsx`
- **Dashboard sidebar:** Edit `web/src/routes/dashboard/Layout.tsx`

### 4. Verify

```
cd web && pnpm typecheck   # TypeScript strict mode
cd web && pnpm dev         # confirm the route loads in a browser
```

## Step 5: API Endpoint (JSON)

### 5a. Create the handler

Path: `services/ui/src/routes/api/<name>.rs`

```rust
use axum::{extract::State, Json};
use std::sync::Arc;
use crate::db::Db;

pub async fn handler(
    State(db): State<Arc<Db>>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}
```

### 5b. Register in router.rs

File: `services/ui/src/router.rs`

```rust
.route("/api/<path>", get(api::<name>::handler))
```

For auth-protected endpoints, nest under the `/api/admin` router with `require_auth` middleware.

### 5c. Add OpenAPI annotation (if public)

Add `#[utoipa::path(...)]` attribute to the handler. Register in `openapi.rs`.

### 5d. Verify

```
just dev       # fmt + lint + test
just ui        # confirm the endpoint responds
```

## Key Files

- `web/src/App.tsx` — React route definitions
- `web/src/routes/` — page components (public)
- `web/src/routes/dashboard/` — dashboard components (auth-gated)
- `services/ui/src/routes/` — Axum API handlers
- `services/ui/src/router.rs` — router assembly
- `services/ui/src/db.rs` — DB query helpers
