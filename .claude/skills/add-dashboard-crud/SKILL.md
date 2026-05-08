---
name: add-dashboard-crud
description: Add admin dashboard CRUD (list/detail/new/edit/delete) for a new entity following the Jobs and About patterns. Covers migration, DB helpers, React components, and API endpoints.
argument-hint: "[entity-name]"
---

Add a new DB-backed entity with full admin dashboard management. Follow the established pattern from W-ABT (about sections) and W-SL (social links).

## The Pattern (5 steps)

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

Define the struct in `services/ui/src/db.rs` (or a models module):
```rust
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Entity {
    pub id: i64,
    pub title: String,
    // ...
}
```

### 3. Admin API endpoints (JSON)

File: `services/ui/src/routes/api/admin.rs`

```rust
pub async fn create_<entity>(
    State(db): State<Arc<Db>>,
    Json(input): Json<New<Entity>>,
) -> Result<impl IntoResponse, StatusCode> { ... }

pub async fn update_<entity>(
    State(db): State<Arc<Db>>,
    Path(id): Path<i64>,
    Json(input): Json<New<Entity>>,
) -> Result<impl IntoResponse, StatusCode> { ... }

pub async fn delete_<entity>(
    State(db): State<Arc<Db>>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, StatusCode> { ... }
```

Register in `services/ui/src/router.rs` under the `/api/admin` router with `require_auth` middleware:
```rust
.route("/api/admin/<entity>s", post(admin::create_<entity>))
.route("/api/admin/<entity>s/:id", put(admin::update_<entity>).delete(admin::delete_<entity>))
```

### 4. React dashboard components

Create in `web/src/routes/dashboard/`:

**List page** (`<Entity>s.tsx`):
```tsx
export default function Entitys() {
  const [items, setItems] = useState<Entity[]>([]);
  useEffect(() => { fetch('/api/<entity>s').then(...) }, []);
  return (
    <div>
      <h1>Entitys</h1>
      <Link to="/dashboard/<entity>s/new">New</Link>
      {items.map(item => <Link to={`/dashboard/<entity>s/${item.id}`}>{item.title}</Link>)}
    </div>
  );
}
```

**Detail page** (`<Entity>Detail.tsx`):
- Check if `id === 'new'` for create vs edit mode
- Form fields controlled by React state
- Create: `POST /api/admin/<entity>s`
- Update: `PUT /api/admin/<entity>s/:id`
- Delete: `DELETE /api/admin/<entity>s/:id` with confirmation
- On success: `navigate('/dashboard/<entity>s')`

Register both in `web/src/App.tsx` inside the `<DashboardLayout>` wrapper:
```tsx
<Route path="/dashboard/<entity>s" element={<Entitys />} />
<Route path="/dashboard/<entity>s/:id" element={<EntityDetail />} />
```

Add sidebar link in `web/src/routes/dashboard/Layout.tsx`.

### 5. Public API endpoint (if needed)

File: `services/ui/src/routes/api/<entity>s.rs`

```rust
pub async fn list(State(db): State<Arc<Db>>) -> Json<Vec<Entity>> { ... }
```

Register in router under `/api/<entity>s`.

## Reference Implementations

- **W-ABT** (about sections): migrations 008–009, `routes/api/about.rs`, React `web/src/routes/dashboard/About.tsx` + `AboutDetail.tsx`
- **W-SL** (social links): migrations 010–011, `db.rs` social_links helpers, React `web/src/routes/dashboard/SocialLinks.tsx` + `SocialLinkDetail.tsx`

## Key Files

- `services/ui/src/db.rs` — DB helpers and structs
- `services/ui/src/routes/api/admin.rs` — admin JSON API
- `services/ui/src/router.rs` — route registration
- `services/ui/migrations/` — SQL migration files
- `web/src/App.tsx` — React route definitions
- `web/src/routes/dashboard/` — dashboard React components
- `web/src/routes/dashboard/Layout.tsx` — sidebar navigation
