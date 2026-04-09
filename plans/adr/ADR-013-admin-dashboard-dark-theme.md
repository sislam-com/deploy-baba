# ADR-013: Admin Dashboard Dark Theme Convention

**Date:** 2026-04-09
**Status:** Accepted
**Affected modules:** W-AUTH, W-ABT, W-SL, W-RSM, W-UI

---

## Context

The site's global chrome is dark: `bg-gray-900 text-white` body, `bg-gray-800/50` nav and
footer (`services/ui/templates/base.html:14,16,47`). The non-admin content pages
(`about_me.html`, `about_repo.html`) established a consistent card aesthetic:
`bg-gray-800/50 rounded-lg p-6 border border-gray-700`, `text-cyan-400` accent headings,
`text-gray-300` body text.

Admin dashboard templates were scaffolded from a Tailwind light-theme admin UI template.
Some have been rewritten to match the dark site chrome (e.g. `dashboard_competencies_list.html`,
`dashboard_home.html`); others have not (e.g. `dashboard_about_list.html`). The drift
produces jarring light-on-dark card rendering and inconsistent visual language across `/dashboard/**`.

No written rule existed preventing new dashboard views from repeating the light theme. The
first symptom was discovered on 2026-04-09 when `dashboard_about_list.html` displayed a
`bg-white` / `bg-gray-50` card against the dark body.

To identify currently non-compliant files:

```sh
grep -lE 'bg-white|bg-gray-50|bg-gray-100|text-gray-900|border-gray-200|border-gray-100|hover:bg-gray-50|hover:bg-gray-100|divide-gray-200' services/ui/templates/dashboard_*.html
```

---

## Decision

> All `services/ui/templates/dashboard_*.html` templates shall use the site's dark palette.
> The light-theme tokens listed in the compliance grep above are **banned** inside any
> dashboard template.

### Canonical class table

| Element | Dark-theme classes |
|---|---|
| Page container | `max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-8` |
| Page heading `<h1>` | `text-2xl font-bold text-white` |
| Breadcrumb `<nav>` | `text-sm text-gray-400`, links `hover:text-white` |
| Section label `<h2>` above a list | `text-sm font-semibold text-gray-400 uppercase tracking-wide mb-3` |
| List wrapper | `space-y-3` (no outer card wrapper) |
| List item card `<a>` / `<div>` | `bg-gray-800/50 hover:bg-gray-800 border border-gray-700 hover:border-cyan-600/50 rounded-lg px-6 py-5 transition group` |
| Item title `<h3>` | `text-lg font-semibold text-white group-hover:text-cyan-400 transition` |
| Item subtitle / slug `<p>` | `text-gray-500 font-mono text-sm` |
| Chevron / action icon `<svg>` | `text-gray-500 group-hover:text-cyan-400` |
| Detail / form panel | `bg-gray-800/50 border border-gray-700 rounded-lg p-6` |
| Form input | `bg-gray-900 border border-gray-700 text-white rounded-lg focus:border-cyan-500 focus:ring-cyan-500` |
| Form label | `text-sm font-medium text-gray-300` |
| Primary action button | `bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg` |
| Secondary / logout button | `bg-gray-700 hover:bg-gray-600 text-gray-300 hover:text-white rounded-lg` |
| Destructive button | `bg-red-900/50 hover:bg-red-900 border border-red-700 text-red-300 hover:text-white rounded-lg` |
| Badge — neutral | `bg-gray-700/50 text-gray-300 border border-gray-600 px-2 py-1 text-xs font-medium rounded-full` |
| Badge — accent A (e.g. "About Me") | `bg-cyan-900/50 text-cyan-300 border border-cyan-700/50 px-2 py-1 text-xs font-medium rounded-full` |
| Badge — accent B (e.g. "About Project") | `bg-purple-900/50 text-purple-300 border border-purple-700/50 px-2 py-1 text-xs font-medium rounded-full` |
| Empty state wrapper | `bg-gray-800/50 border border-gray-700 rounded-lg px-6 py-16 text-center` |
| Empty state icon `<svg>` | `text-gray-600` |
| Empty state copy `<p>` | `text-gray-400` |
| Filter tab bar | `bg-gray-800 rounded-xl p-1 inline-flex` |
| Active tab | `bg-cyan-600 text-white rounded-lg text-sm font-medium` |
| Inactive tab | `text-gray-400 hover:text-white rounded-lg text-sm font-medium` |

### Rationale

- **`bg-gray-800/50`** over solid `bg-gray-800` — subtle translucency echoes the nav and
  footer chrome (`bg-gray-800/50`), grounding dashboard cards visually in the same layer.
- **`border-gray-700`** — matches the nav border (`base.html:16`) and footer divider.
- **`text-cyan-400`** — the site's single primary accent (logo, footer headings, nav brand).
  Accent B (`purple`) is reserved for secondary category labels only.
- **`text-gray-300` / `text-gray-500`** — the AA-contrast ladder already validated across
  all non-admin pages; no per-view re-testing required.

### Template rebuild note

Askama templates are compiled into the `services/ui` binary at build time via
`#[derive(Template)]`. Editing `.html` files requires a rebuild to take effect. There is no
Tailwind CSS stylesheet to purge — styles are injected at runtime by `cdn.tailwindcss.com`
JIT. After rebuilding, a browser hard-reload (`Cmd+Shift+R`) is sufficient.

```sh
just ui            # rebuild + run locally
# or
just lambda-deploy PROFILE  # rebuild + deploy to Lambda
```

---

## Consequences

### Positive

- Every dashboard view is visually coherent with the rest of the site.
- The compliance grep becomes a repeatable lint step (add to `just quality` or CI when
  dashboard surface grows).
- New dashboard templates can be scaffolded by copying the class table above.
- Contrast ratios proven on public-facing pages carry over — no per-view accessibility
  re-testing.

### Negative / Trade-offs

- A one-time sweep of existing non-compliant `dashboard_*.html` files is required (see
  Compliance scope below). Small effort; surgical find-and-replace per file.
- Banning `text-gray-900` means any future intentional dark-text-on-light-surface (e.g. a
  print-preview card) must be explicit in scope via an in-line comment citing this ADR.

### Neutral

- Public non-admin templates (`about_*.html`, `resume.html`, `contact.html`) are **not**
  in scope — they already conform. A separate ADR would govern public UI if drift occurs.
- The `bg-gray-900` base is set on `<body>` in `base.html:14`, so the dark background is
  guaranteed for all routes without per-template override.

---

## Compliance scope

Run the grep above to enumerate non-compliant files at adoption time. Expected initial
findings (as of 2026-04-09):

- `dashboard_about_list.html` — confirmed non-compliant (driver for this ADR).
  Fix: replace lines 37-80 (Section Master List block) using canonical classes above.

Record sweep results in a drift log: `plans/drift/DRL-2026-04-09-dashboard-light-theme.md`.

---

## Alternatives considered

| Option | Rejected because |
|---|---|
| Keep dashboards light-themed | Jarring against dark site body; already producing bug reports |
| Separate admin subdomain with own base template | Violates ADR-003 (single Lambda Function URL); doubles template maintenance surface |
| CSS custom properties + theme switcher | No user-facing need; adds runtime JS cost; complicates the Askama compile-time model |
| Third-party Tailwind admin preset (Catalyst, Tremor) | External dependency; visual language diverges from public site; re-introduces same drift in a different package |
| Convert to a single `dashboard_base.html` with dark overrides | Askama 0.12 supports block inheritance but not multiple inheritance; would require restructuring all existing templates; benefit does not justify the cost at current template count (~12) |

---

## Cross-references

- → ADR-003 (Lambda Function URL — single origin, single template surface)
- → ADR-008 (Cognito dashboard auth — defines the `/dashboard/**` route set in scope)
- → W-AUTH.4.22-4.30 (dashboard master/detail templates — first consumers of this convention)
- → W-ABT (dashboard about-section templates — initial non-compliant surface)
- → `plans/modules/dashboard.md` (should gain a "Theme" section linking to this ADR if the module exists)
- → `plans/drift/DRL-2026-04-09-dashboard-light-theme.md` (compliance sweep results)
