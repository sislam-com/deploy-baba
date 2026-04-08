---
name: add-route
description: Add a new Axum route handler with an Askama template following the services/ui patterns. Handles handler creation, router registration, and template wiring including social_links for nav.
argument-hint: "[route-path] [handler-name]"
---

Add a new page or API endpoint to `services/ui`. All routes use Axum + Askama (compile-time Jinja2 templates).

## Decision: Page Route vs API Endpoint?

- **Page route** (HTML response, public) → follow Steps 1–5
- **Admin API endpoint** (JSON, auth-gated) → see `add-dashboard-crud` skill instead

## Steps for a Page Route

### 1. Create the handler file

Path: `services/ui/src/routes/<name>.rs`

```rust
use askama::Template;
use axum::response::Html;
use crate::db::DbPool;
use crate::routes::SocialLink;

#[derive(Template)]
#[template(path = "<name>.html")]
struct <Name>Template {
    social_links: Vec<SocialLink>,
    // add page-specific fields here
}

pub async fn handler(
    axum::extract::State(pool): axum::extract::State<DbPool>,
) -> Html<String> {
    let social_links = crate::db::get_social_links(&pool).unwrap_or_default();
    let tmpl = <Name>Template { social_links };
    Html(tmpl.render().unwrap())
}
```

**Critical:** Always include `social_links: Vec<SocialLink>` — the base template renders the nav from this field.

### 2. Register in routes/mod.rs

File: `services/ui/src/routes/mod.rs`

```rust
pub mod <name>;
```

### 3. Register in router.rs

File: `services/ui/src/routes/router.rs` (or wherever `Router::new()` is built)

```rust
.route("/<path>", get(<name>::handler))
```

For auth-protected routes, chain `.route_layer(require_auth())`.

### 4. Create the Askama template

Path: `services/ui/templates/<name>.html`

```html
{% extends "base.html" %}

{% block title %}<Page Title>{% endblock %}

{% block content %}
<section class="...">
  <h1>...</h1>
</section>
{% endblock %}
```

The `base.html` template iterates `social_links` for the nav — no extra wiring needed beyond including the field in the template struct.

### 5. Verify

```
just dev       # fmt + lint + test
just ui-run    # confirm the route loads in a browser
```

## API Endpoint (JSON, no template)

```rust
pub async fn handler(
    axum::extract::State(pool): axum::extract::State<DbPool>,
) -> axum::Json<serde_json::Value> {
    // ...
    axum::Json(serde_json::json!({ "status": "ok" }))
}
```

Register with the appropriate HTTP verb: `get()`, `post()`, `put()`, `delete()`.

## Key Files

- `services/ui/src/routes/` — all route handlers
- `services/ui/src/routes/router.rs` — router assembly
- `services/ui/src/routes/mod.rs` — module declarations
- `services/ui/templates/` — Askama templates
- `services/ui/templates/base.html` — base layout with nav
- `services/ui/src/db.rs` — `get_social_links()` and other DB helpers
