# Claude Code Skills — Developer Guide

Skills are project-specific instructions that Claude Code loads when you type `/skill-name`.
They encode the project's conventions, file paths, and multi-step workflows so Claude
doesn't need to re-discover them from scratch each session.

**Location:** `.claude/skills/<name>/SKILL.md`

---

## Available Skills

### `/deploy` — Deploy Lambda to AWS

Builds and uploads Lambda function(s) to AWS. **Manual-only** (will not auto-trigger).

```
/deploy personal             # standard: quality gate → build → upload
/deploy personal --fast      # skip quality gate (hotfixes only)
/deploy personal --email     # deploy the email Lambda instead
/deploy personal --infra     # run infra-apply first, then deploy
```

What it does: runs `just quality` → `just lambda-build` → `just lambda-deploy <profile>`,
with variants for the email Lambda, infrastructure changes, and secrets rotation.

See: `.claude/skills/deploy/SKILL.md`

---

### `/add-migration` — Add a SQLite Migration

Creates the next numbered migration file and wires it into `db.rs`.

```
/add-migration add_blog_posts
```

What it does:
1. Detects the next migration number from `services/ui/migrations/`
2. Creates `NNN_<description>.sql` with idiomatic SQLite
3. Appends `include_str!` entry to the `MIGRATIONS` array in `db.rs`
4. Runs `just dev` to verify

See: `.claude/skills/add-migration/SKILL.md`

---

### `/add-route` — Add an Axum Route + Template

Scaffolds a new page or API endpoint in `services/ui`.

```
/add-route /blog blog_list
```

What it does:
1. Creates a handler in `services/ui/src/routes/<name>.rs`
2. Registers the module in `routes/mod.rs`
3. Adds the route to `router.rs`
4. Creates a React component in `web/src/routes/` (since ADR-019)

See: `.claude/skills/add-route/SKILL.md`

---

### `/add-dashboard-crud` — Admin Dashboard CRUD

Adds full admin management for a new DB-backed entity (list/detail/new/edit/delete).

```
/add-dashboard-crud blog_post
```

What it does: follows the W-ABT / W-SL / W-CHL reference pattern across 6 steps:
1. Migration(s) for the new table
2. DB query helpers in `db.rs`
3. Admin JSON API endpoints in `routes/api/admin.rs`
4. React list + detail components in `web/src/routes/dashboard/`
5. Route registration in `web/src/App.tsx`
6. Router registration with `require_auth()` middleware

See: `.claude/skills/add-dashboard-crud/SKILL.md`

---

### `/add-secret` — Add a Managed Secret

Adds a new AWS Secrets Manager secret following the W-SEC policy.

```
/add-secret smtp-password
```

What it does:
1. Registers the secret name in `xtask/src/secret.rs`
2. Adds the SM resource and `lifecycle { ignore_changes }` block to `infra/secrets.tf`
3. Adds IAM `secretsmanager:GetSecretValue` policy in `infra/iam.tf`
4. Wires the ARN as a Lambda env var in `infra/lambda.tf`
5. Shows the commands to apply infra and write the secret value

**Never** store secrets in Lambda env vars directly — they are visible in the AWS console.

See: `.claude/skills/add-secret/SKILL.md`

---

### `/add-plan-module` — Create a Plan Module

Creates a new module in the `plans/` system when adding a new component.

```
/add-plan-module BLG blog
```

What it does:
1. Registers the domain code in `plans/CONVENTIONS.md`
2. Creates `plans/modules/<component>.md` from the CONVENTIONS template
3. Adds the module row to `plans/INDEX.md`
4. Updates `.agent-cache/index.json`

See: `.claude/skills/add-plan-module/SKILL.md`

---

### `/add-drift-log` — Document an Incident

Creates a DRL (drift log) to record a post-mortem, gap, or course correction.

```
/add-drift-log email-lambda-timeout
```

What it does:
1. Creates `plans/drift/DRL-<date>-<topic>.md` from the template
2. Registers it in the INDEX.md drift log table
3. Cross-references the affected module plans

See: `.claude/skills/add-drift-log/SKILL.md`

---

### `/sync-dashboard-data` — Sync Dashboard Edits to Migrations

Walks the four-phase workflow (Pull → Diff → Author → Verify+Deploy) to capture
dashboard edits as upsert-style source migrations, using ADR-010 and W-SYNC.

```
/sync-dashboard-data
```

What it does:
1. Pulls the live EFS DB snapshot via `GET /api/admin/db-dump`
2. Diffs it table-by-table against a freshly seeded local DB
3. Scaffolds the sync migration via `/add-migration` with the ADR-010 upsert template
4. Guides verification (INSERT path + UPDATE path) and `just lambda-deploy`

See: `.claude/skills/sync-dashboard-data/SKILL.md`

---

### `/add-adr` — Write an Architecture Decision Record

Documents a significant architectural decision.

```
/add-adr sqlite-wal-mode
```

What it does:
1. Determines the next ADR number (currently ADR-027 is the highest)
2. Creates `plans/adr/ADR-<NNN>-<title>.md` from the template
3. Registers it in INDEX.md
4. Cross-references affected module plans

See: `.claude/skills/add-adr/SKILL.md`

---

### `/cache-refresh` — Refresh Agent Cache

Re-derives `.agent-cache/index.json` from the current repo state.

```
/cache-refresh
```

What it does:
1. Invokes `just cache-refresh` to re-scan the codebase
2. Updates all component entries, git SHA, and timestamps
3. Verifies idempotency (running twice produces the same result)

Use when the cache SHA diverges from HEAD or after a session that touched multiple components.

See: `.claude/skills/cache-refresh/SKILL.md`

---

### `/memory-curate` — Curate Agent Memory

Walks `~/.claude/projects/-Users-shantopagla-portfolio/memory/`, verifies project memories against current repo and infra state, and proposes prunes with per-file confirmation.

```
/memory-curate
```

What it does:
1. Reads all memory files and MEMORY.md index
2. Checks each memory against current codebase state (file existence, git history)
3. Proposes removals for stale or outdated memories
4. Asks for confirmation before deleting each file

See: `.claude/skills/memory-curate/SKILL.md`

---

### `/plan-sync` — Sync Plan System

Audits and repairs the plan system. Safe writes only; destructive changes require confirmation.

```
/plan-sync              # full audit + auto-fix
/plan-sync ADR-015      # scope drift check to one ADR
```

What it does:
1. Runs `plan-doctor` and `drift-detector` subagents in parallel
2. Auto-fixes: syncs INDEX.md status table to match module files, inserts missing ADR back-references
3. Gates: shows proposed DRL files for approval before creating them
4. Reports unfixed findings that need manual review

See: `.claude/skills/plan-sync/SKILL.md`

---

### `/resume-generate` — Regenerate Resume Outputs

Regenerates resume outputs (Markdown, DOCX, PDF) from the SQLite database.

```
/resume-generate
/resume-generate --update-summary
```

What it does:
1. Reads resume data from the database (jobs, competencies, about sections)
2. Optionally updates the Professional Summary from the `me-bio` about section ([ADR-014](../plans/adr/ADR-014-resume-summary-from-db.md))
3. Generates Markdown → DOCX → PDF pipeline via xtask
4. Outputs to `target/resume/`

See: `.claude/skills/resume-generate/SKILL.md`

---

## Global Commands (available in all projects)

These live in `~/.claude/commands/` and are not project-specific:

| Command | What it does |
|---------|-------------|
| `/cleanup` | `cargo fmt` + `cargo clippy -- -D warnings` |
| `/quick-test` | Detects project type and runs tests (`just dev` for this project) |
| `/review` | Reviews `git diff` changes for quality, bugs, and security |
| `/project-status` | Gives an overview of the project state |
| `/linear` | Manages Linear issues via MCP |

---

## Adding a New Skill

To add a project-specific skill:

```
mkdir .claude/skills/<name>
touch .claude/skills/<name>/SKILL.md
```

Minimal `SKILL.md`:
```markdown
---
name: <name>
description: What this skill does (used by Claude to decide when to auto-invoke)
argument-hint: "[optional-arg]"
---

Instructions for Claude go here...
```

Fields:
- `name` — the `/command` name (lowercase, hyphens)
- `description` — Claude uses this for auto-invocation decisions
- `argument-hint` — shown in autocomplete
- `disable-model-invocation: true` — prevents Claude from triggering it automatically (use for risky actions like `/deploy`)

See CLAUDE.md for project conventions to encode.
