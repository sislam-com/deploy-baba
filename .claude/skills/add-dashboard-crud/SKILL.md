---
name: add-dashboard-crud
description: Add admin dashboard CRUD (list/detail/new/edit/delete) for a new entity following the about-section and social-links patterns. Covers migration, DB helpers, templates, routes, and API endpoints.
argument-hint: "[entity-name]"
---

Add a new DB-backed entity with full admin dashboard management. Follow the established pattern from W-ABT (about sections) and W-SL (social links).

## The Pattern (6 steps)

### 1. Migration(s)

Create `NNN_create_<entity>s.sql` and optionally `NNN+1_seed_<entity>s.sql`.

Run `/add-migration create_<entity>s` for step-by-step migration guidance.

Example schema pattern:
```sql
CREATE TABLE IF NOT EXISTS <entity>s (
    id       INTEGER PRIMARY KEY,
    title    TEXT NOT NULL,
    body     TEXT,
    sort_key INTEGER NOT NULL DEFAULT 0,
    created  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 2. DB query helpers

File: `services/ui/src/db.rs`

Add these functions following existing patterns:
```rust
pub fn get_<entity>s(pool: &DbPool) -> Result<Vec<Entity>> { ... }
pub fn get_<entity>(pool: &DbPool, id: i64) -> Result<Entity> { ... }
pub fn create_<entity>(pool: &DbPool, input: &NewEntity) -> Result<i64> { ... }
pub fn update_<entity>(pool: &DbPool, id: i64, input: &NewEntity) -> Result<()> { ... }
pub fn delete_<entity>(pool: &DbPool, id: i64) -> Result<()> { ... }
```

Define the struct in `services/ui/src/models.rs` (or inline in `db.rs` if small):
```rust
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Entity {
    pub id: i64,
    pub title: String,
    // ...
}
```

### 3. Dashboard route handlers

File: `services/ui/src/routes/dashboard.rs`

Add handlers for the list and detail/edit views:
```rust
pub async fn <entity>s_list(State(pool): State<DbPool>) -> Html<String> { ... }
pub async fn <entity>_detail(State(pool): State<DbPool>, Path(id): Path<i64>) -> Html<String> { ... }
pub async fn <entity>_new_form(State(pool): State<DbPool>) -> Html<String> { ... }
```

Each handler struct **must** include `social_links: Vec<SocialLink>` for nav rendering.

### 4. Templates

Create in `services/ui/templates/dashboard/`:
- `<entity>s.html` — list view (table of all records + "Add new" button)
- `<entity>_detail.html` — edit form (populated fields + DELETE button)
- `<entity>_new.html` — blank create form

Extend `dashboard_base.html` (not `base.html`).

### 5. Admin API endpoints (JSON)

File: `services/ui/src/routes/api/admin.rs`

```rust
pub async fn create_<entity>(
    State(pool): State<DbPool>,
    Json(input): Json<New<Entity>>,
) -> Result<impl IntoResponse, StatusCode> { ... }

pub async fn update_<entity>(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Json(input): Json<New<Entity>>,
) -> Result<impl IntoResponse, StatusCode> { ... }

pub async fn delete_<entity>(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, StatusCode> { ... }
```

### 6. Router registration

File: `services/ui/src/routes/router.rs`

```rust
// Dashboard pages (HTML, auth-gated)
.route("/dashboard/<entity>s", get(dashboard::<entity>s_list))
.route("/dashboard/<entity>s/new", get(dashboard::<entity>_new_form))
.route("/dashboard/<entity>s/:id", get(dashboard::<entity>_detail))
// Admin API (JSON, auth-gated)
.route("/api/admin/<entity>s", post(admin::create_<entity>))
.route("/api/admin/<entity>s/:id", put(admin::update_<entity>).delete(admin::delete_<entity>))
```

All dashboard + admin routes must be inside the `require_auth()` layer.

## Reference Implementations

- **W-ABT** (about sections): migrations 008–009, `routes/about.rs`, `templates/about_*.html`, `routes/api/admin.rs`
- **W-SL** (social links): migrations 010–011, `db.rs` social_links helpers, `routes/dashboard.rs`, `routes/api/admin.rs`

## Key Files

- `services/ui/src/db.rs` — DB helpers and structs
- `services/ui/src/routes/dashboard.rs` — dashboard page handlers
- `services/ui/src/routes/api/admin.rs` — admin JSON API
- `services/ui/src/routes/router.rs` — route registration
- `services/ui/templates/dashboard/` — HTML templates
- `services/ui/migrations/` — SQL migration files
