# W-GDR: Google Drive AI Workflow Integration
**Path:** `.claude/settings.json`, `justfile`, `infra/` (CI/CD) | **Status:** TODO
**Coverage floor:** N/A (tooling) | **Depends on:** W-DX, W-XT | **Depended on by:** —

---

## W-GDR.1 Purpose

Add Google Drive as an **intake and export layer** for the `plans/` system — not as a replacement source of truth. `plans/INDEX.md` remains the single source of truth (per CLAUDE.md). Drive is a convenience surface for drafting new module plans and sharing status with non-CLI stakeholders.

Additionally wire Claude Code hooks so that `just quality` runs before any session end, closing a gap in the verification loop.

This module captures the actionable subset of a Gemini-generated proposal evaluated on 2026-04-15. See W-GDR.3 for what was rejected and why.

---

## W-GDR.2 Interface Surface

No new Rust code. All changes are tooling and CI:

- `justfile` — two new recipes: `plan-export` and `plan-import`
- `.claude/settings.json` — `Stop` hook entry
- `.github/workflows/` — optional CI step for Drive status update

---

## W-GDR.3 Implementation Notes

### What to build (validated items)

**1. Google Drive MCP — plan intake / export**

The Drive API v3 natively exports Google Docs as Markdown (`exportFormat=markdown`). This eliminates any transformation layer.

- `just plan-export` — exports `plans/INDEX.md` + all active `plans/modules/*.md` to a configured Drive folder as `.md` files.
- `just plan-import GDOC_ID` — fetches a Google Doc by ID, applies the module plan template from `plans/CONVENTIONS.md`, writes to `plans/modules/<derived-name>.md`, and registers it in `INDEX.md`.

Implementation runtime: **Gemini CLI** (`gemini` terminal tool) has Drive MCP baked in and authenticated Drive access out of the box. `just plan-import` and `just plan-export` shell out to `gemini` for Drive reads/writes rather than requiring a custom OAuth + Drive API script. This is the **only** role Gemini plays in this project — it is a Drive scripting tool, not the primary AI agent.

Drive authentication: OAuth 2.0 desktop flow via Gemini CLI. OAuth tokens stored locally — never committed.

**2. `Stop` hook — `just quality` verification gate**

Currently Claude can declare a task complete without the quality gate running. A `Stop` hook in `.claude/settings.json` that invokes `just quality` and returns exit code 2 on failure will block the session from ending until tests and clippy pass.

```json
"stop": [
  {
    "hooks": [{ "type": "command", "command": "just quality || exit 2" }]
  }
]
```

**3. CI/CD → Drive status update (optional, P3)**

On PR merge, a GitHub Actions step uses a GCP Service Account to update a Drive spreadsheet row status field. Low priority; valuable only once Drive is established as a stakeholder-facing surface.

### Gemini as LLM provider (separate concern from Drive integration)

The W-LLM plan (ADR-015) defines the canonical path for adding any LLM vendor: implement `crates/llm-gemini` following the `LlmProvider` trait. This is architecturally viable and tracked under W-LLM.4.5 (DEFERRED). It is **distinct** from the Drive/IDE integration discussed in this module — do not conflate them.

If Gemini Flash or Pro is ever evaluated as a cost alternative to Claude Haiku for W-RST, the entry point is W-LLM, not W-GDR.

### What was rejected from the Gemini proposals

| Rejected item | Reason |
|---|---|
| Drive as primary planning layer, Git as secondary | Inverts the SSOT principle in CLAUDE.md. `plans/` is authoritative. |
| Gemini Code Assist (VS Code IDE-first workflow) | Conflicts with ADR-001 (justfile-only interface). IDE-first diff-approval breaks autonomous agent operation and the hook system. |
| `gcloud scc iac-validation-reports` for IaC | Validates GCP org policies only. This project runs on AWS. Entirely inapplicable. |
| `GEMINI.md` alongside `CLAUDE.md` | Two competing instruction files require active synchronization — creates plan drift by design. |
| `adr-index.toml` for branch linking | Project uses `plans/adr/ADR-NNN-*.md` + `INDEX.md`. No TOML index. |
| `github-branch-creator` plugin | Does not exist. Fabricated. |
| `gas-fakes` CI library | Not applicable to Rust/Lambda stack. |
| "Ruflo" multi-agent framework | Does not exist. Fabricated product name. |
| Vector-indexed "Memory Bank" MCP | `.agent-cache/index.json` handles this at current scale. Unnecessary complexity. |
| "hudson-star" project / LocalStack → MinIO | Not this project. Hallucinated context in Gemini doc. |
| API Gateway as default transport | Conflicts with ADR-003. Lambda Function URL is default; API Gateway only for `POST /api/contact` (ADR-009). |

---

## W-GDR.4 Work Items

| ID | Task | Status | Notes |
|----|------|--------|-------|
| W-GDR.4.1 | Configure Google Drive MCP server in local Claude Code session | TODO | OAuth desktop flow; store tokens locally |
| W-GDR.4.2 | Add `plan-export` justfile recipe | TODO | Shells out to `gemini` CLI (Drive MCP baked in); exports `plans/INDEX.md` + active modules to Drive folder; folder ID in `stack.toml` |
| W-GDR.4.3 | Add `plan-import GDOC_ID` justfile recipe | TODO | Shells out to `gemini` CLI to fetch Doc as Markdown; applies CONVENTIONS.md module template; writes to `plans/modules/`; registers in `INDEX.md` |
| W-GDR.4.4 | Add `Stop` hook to `.claude/settings.json` | TODO | `just quality \|\| exit 2`; blocks session end on test/clippy failure |
| W-GDR.4.5 | Document Drive folder structure in `docs/` | TODO | Folder IDs, naming convention, access policy |
| W-GDR.4.6 | CI step: update Drive status on PR merge | TODO | P3 — GCP Service Account; GitHub Actions; service account email must have Editor access on Drive folder |

---

## W-GDR.5 Test Strategy

- W-GDR.4.1–4.3: Manual smoke test — export `plans/INDEX.md` to Drive; verify Markdown renders correctly; import a test Doc and confirm `INDEX.md` registration.
- W-GDR.4.4: Intentionally introduce a clippy warning; verify `Stop` hook blocks session.
- W-GDR.4.6: Trigger PR merge in CI; verify Drive row status field updates to "Merged".

---

## W-GDR.6 Cross-References

- → W-DX (justfile recipes)
- → W-XT (hooks configuration)
- → W-LLM (if Gemini is ever evaluated as an LLM provider, the entry point is `crates/llm-gemini` under W-LLM.4.5 — not this module)
- → ADR-001 (justfile-only interface — all Drive commands go through `just`)
- → ADR-015 (LLM provider abstraction — governs any future Gemini LLM adapter)
- → `plans/cross-cutting/quality-gates.md` (Stop hook ties in here)
- → `plans/cross-cutting/llm-policy.md` (if Gemini LLM adapter is activated, cost caps and retry policy apply)
- Source proposals: Gemini-generated "Agentic Synthesis" framework (2026-04-15) + Gemini VS Code integration strategy (2026-04-15); both evaluated and trimmed to viable subset
