# Web SPA

Last updated: 2026-05-19

React single-page application serving the portfolio frontend. Replaced server-side Askama templates ([ADR-019](../plans/adr/ADR-019-spa-replaces-askama.md)).

## Stack

| Technology | Version | Purpose |
|-----------|---------|---------|
| React | 18 | UI framework |
| TypeScript | strict mode | Type safety |
| Vite | 6 | Build tool + dev server |
| Tailwind CSS | v4 | Utility-first styling (PostCSS plugin) |
| react-router | v6 | Client-side routing |
| openapi-fetch | — | Type-safe API client from OpenAPI spec |
| react-markdown | — | Markdown rendering (About pages, RAG responses) |
| react-helmet-async | — | Document head management (SEO) |
| vitest | — | Unit testing |

## Route Map

### Marketing Routes

Shared `Layout` component with nav bar (social links from API) and footer.

| Path | Component | Description |
|------|-----------|-------------|
| `/` | `Home.tsx` | Resume timeline + career overview |
| `/about/me` | `AboutMe.tsx` | Personal bio from `about_sections` |
| `/about/repo` | `AboutRepo.tsx` | Repository / project description |
| `/contact` | `Contact.tsx` | Contact form with PoW challenge |
| `/ask` | `Ask.tsx` | RAG-powered Q&A about the portfolio |
| `*` | `NotFound.tsx` | 404 page |

### Dashboard Routes

Nested under `/dashboard` with a sidebar layout. All routes require Cognito authentication.

| Path | Component | Description |
|------|-----------|-------------|
| `/dashboard` | `index.tsx` | Dashboard home / overview |
| `/dashboard/jobs` | `Jobs.tsx` | Job experience list |
| `/dashboard/jobs/:id` | `JobDetail.tsx` | Job edit / detail |
| `/dashboard/competencies` | `Competencies.tsx` | Competencies list |
| `/dashboard/competencies/:id` | `CompetencyDetail.tsx` | Competency edit |
| `/dashboard/about` | `About.tsx` | About sections list |
| `/dashboard/about/:id` | `AboutDetail.tsx` | About section edit |
| `/dashboard/social-links` | `SocialLinks.tsx` | Social links list |
| `/dashboard/social-links/:id` | `SocialLinkDetail.tsx` | Social link edit |
| `/dashboard/challenges` | `Challenges.tsx` | Challenges list |
| `/dashboard/challenges/:id` | `ChallengeDetail.tsx` | Challenge edit |

Dashboard follows the dark theme convention ([ADR-013](../plans/adr/ADR-013-admin-dashboard-dark-theme.md)).

## API Client

The SPA communicates with the backend through a type-safe client generated from the OpenAPI spec.

**Generation pipeline:**
```bash
cargo run -p api-openapi --bin emit-spec > web/openapi.json
pnpm --dir web exec openapi-typescript openapi.json -o src/api/types.gen.ts
```

The generated `types.gen.ts` provides full TypeScript types for all request/response bodies. The `openapi-fetch` library uses these types to provide compile-time safety on API calls — incorrect paths, missing parameters, or wrong body shapes are caught by `tsc`.

**Client setup:** `web/src/api/client.ts`

## Authentication

- `web/src/hooks/useAuth.ts` — custom hook managing auth state
- Dashboard routes check auth via the hook; unauthenticated users are redirected to the Cognito Hosted UI
- After login, Cognito redirects back to `/auth/callback` with a JWT
- The JWT is stored as an HttpOnly cookie by the backend (not accessible to JS)
- `GET /api/auth/me` returns the current user's identity (or 401)

See [ADR-008](../plans/adr/ADR-008-cognito-authentication.md) for the full authentication architecture.

## Build and Development

```bash
just web              # Vite dev server on :3000
just dev-stack        # Vite on :3000 + Rust API on :3001 (hot reload for both)
just web-build        # Production build to web/dist/
just web-types-offline  # Regenerate TypeScript types from OpenAPI spec
just web-test         # Run vitest unit tests
```

During development, Vite proxies `/api/*` requests to the Rust backend on `:3001`.

## Deployment

1. `just web-build` produces optimized static assets in `web/dist/`
2. CI uploads `web/dist/` to the S3 SPA bucket (`s3-spa.tf`)
3. CloudFront serves the SPA via S3 Origin Access Control (OAC)
4. API requests (`/api/*`) pass through to the Lambda Function URL origin
5. SPA client-side routing: CloudFront returns `index.html` for 403/404 responses on non-API paths

## Directory Structure

```
web/
├── src/
│   ├── api/           # openapi-fetch client + generated types
│   ├── components/    # Shared React components
│   ├── hooks/         # Custom hooks (useAuth, etc.)
│   ├── routes/        # Page components
│   │   ├── dashboard/ # Admin dashboard pages (12 files)
│   │   ├── Home.tsx
│   │   ├── AboutMe.tsx
│   │   ├── Contact.tsx
│   │   └── ...
│   ├── App.tsx        # Router setup
│   └── main.tsx       # Entry point
├── openapi.json       # Generated spec (gitignored, built by CI)
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## Cross-References

- [ADR-019](../plans/adr/ADR-019-spa-replaces-askama.md) — React SPA replaces Askama templates
- [ADR-013](../plans/adr/ADR-013-admin-dashboard-dark-theme.md) — Dashboard dark theme convention
- [ADR-008](../plans/adr/ADR-008-cognito-authentication.md) — Cognito authentication
- [plans/modules/web.md](../plans/modules/web.md) — Module plan and work items
