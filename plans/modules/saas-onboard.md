# W-SAAS: SaaS AI-DLC Onboarding
**Path:** `xtask/src/onboard.rs`, `crates/portfolio-rag-mcp/`, `services/ui/src/routes/api/eval.rs` | **Status:** WIP
**Coverage floor:** 70% | **Depends on:** W-RAG, W-MCP, W-AIL, W-LLM | **Depended on by:** —

## W-SAAS.1 Purpose

Package the AI-DLC pattern as a replicable product. The first concrete feature is `project-onboard`:
given any git repo URL, auto-generate the plan system artifacts, agent cache, MCP configuration, and
RAG index that the AI-DLC requires. This transforms the portfolio from a showcase into a product
prototype.

Secondary features: eval-driven accuracy dashboard and a `project_health` MCP tool that combines
plan-doctor, drift-detector, and cache-status into a single queryable metric.

## W-SAAS.2 Public Surface

### CLI (via `just`)

```
just onboard <repo-url>              # clone, analyze, generate artifacts
just onboard <repo-url> --output <dir>  # write artifacts to a specific directory
```

### MCP tools (portfolio-rag-mcp)

```
onboard_project  — accepts { repo_url, description? }, returns generated artifact summary
project_health   — returns { plan_coverage, drift_items, cache_age_hours, eval_score }
```

### HTTP (services/ui)

```
GET /api/v1/eval/dashboard  — retrieval accuracy metrics over time (Cognito-gated)
```

## W-SAAS.3 Implementation Notes

### Onboard Flow

The `xtask onboard` subcommand:

1. Clones the repo into a sandboxed tmpdir (`git clone --depth=1`)
2. Runs language detection: scan file extensions, look for `Cargo.toml`, `package.json`,
   `pyproject.toml`, `go.mod`, `Makefile`, `Dockerfile`
3. Discovers project structure: count source files per language, identify entry points, find
   test directories
4. Generates artifacts:
   - `plans/INDEX.md` — module table with discovered components
   - `.agent-cache/index.json` — project snapshot with detected tech stack
   - `plans/adr/ADR-001-<detected-pattern>.md` — initial ADRs for detected patterns
   - `.mcp-rs.toml` — MCP config scoped to the repo's file structure
5. Optionally indexes the repo into a RAG store using existing chunkers (Rust, Markdown;
   new chunkers needed for other languages)
6. Cleans up the tmpdir

Security: no `cargo build`, `npm install`, `pip install`, or any code execution from the
target repo. Analysis is purely structural (file system + AST where applicable).

### Eval Dashboard

Extend `rag-core/eval.rs` (already implements `score_groundedness` and `verify_citation_refs`):

- Add a `rag_eval_results` migration table:
  ```sql
  CREATE TABLE IF NOT EXISTS rag_eval_results (
      id INTEGER PRIMARY KEY,
      eval_category TEXT NOT NULL,
      query TEXT NOT NULL,
      retrieval_score REAL,
      groundedness_score REAL,
      expected_sources TEXT,
      actual_sources TEXT,
      created_at TEXT NOT NULL DEFAULT (datetime('now')),
      UNIQUE(eval_category, query, created_at)
  );
  ```
- `just rag-eval` writes results to this table (currently only prints to terminal)
- `GET /api/v1/eval/dashboard` returns aggregated scores by category and time period

### Project Health MCP Tool

Combines three existing capabilities into one queryable tool:
- Plan coverage: count modules with Status=DONE vs total modules
- Drift items: count open DRL entries (grep `plans/drift/` for non-RESOLVED)
- Cache age: compare `.agent-cache/index.json` `last_updated` to now
- Eval score: average groundedness from most recent eval run

## W-SAAS.4 Work Items

| ID | Task | Status | Notes |
|---|---|---|---|
| W-SAAS.4.1 | ADR-030 (SaaS AI-DLC Pattern) | DONE | `plans/adr/ADR-030-saas-ai-dlc-pattern.md` |
| W-SAAS.4.2 | Module plan (this file) | DONE | `plans/modules/saas-onboard.md` |
| W-SAAS.4.3 | `project_health` MCP tool in portfolio-rag-mcp | DONE | Combines plan coverage + drift + cache age + eval score + RAG index stats |
| W-SAAS.4.4 | `rag_eval_results` migration | TODO | New migration; ADR-010 upsert convention |
| W-SAAS.4.5 | Extend `xtask rag eval` to persist results to DB | TODO | Currently prints to terminal only |
| W-SAAS.4.6 | `GET /api/v1/eval/dashboard` endpoint | TODO | Cognito-gated; returns aggregated eval scores |
| W-SAAS.4.7 | `xtask onboard` — language detection + structure analysis | TODO | Read-only; sandboxed tmpdir |
| W-SAAS.4.8 | `xtask onboard` — artifact generation (plans, cache, MCP config) | TODO | Template-based generation |
| W-SAAS.4.9 | `xtask onboard` — RAG indexing of external repos | TODO | Reuse existing chunkers where applicable |
| W-SAAS.4.10 | `onboard_project` MCP tool | TODO | Wraps xtask onboard for MCP-based invocation |
| W-SAAS.4.11 | Justfile recipes: `onboard`, `eval-dashboard` | TODO | |
| W-SAAS.4.12 | TypeScript/Python/Go chunkers for external repo support | DEFERRED | Rust + Markdown chunkers cover initial use cases |

## W-SAAS.5 Test Strategy

- **Unit:** Language detection against fixture directory structures (Rust project, Node project, Python project)
- **Unit:** Artifact generation — given detected structure, assert correct INDEX.md format and cache JSON schema
- **Integration:** Full onboard flow against a small fixture repo (committed as `tests/fixtures/sample-repo/`)
- **Smoke:** `just onboard https://github.com/user/small-public-repo` runs without error
- **Eval:** `just rag-eval` results appear in `rag_eval_results` table; dashboard endpoint returns them

## W-SAAS.6 Cross-References

- → ADR-030 (SaaS AI-DLC Pattern)
- → ADR-016 (RAG Architecture — chunkers reused)
- → ADR-017 (AI-DLC — session lifecycle)
- → ADR-018 (Anti-rot Agents — plan-doctor feeds project_health)
- → W-RAG (retrieval infrastructure)
- → W-MCP (MCP tool surface)
- → W-AIL (anti-rot agents)
- → `plans/cross-cutting/ai-dlc.md` (session lifecycle)
