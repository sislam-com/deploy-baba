# Conventions — deploy-baba Plan System

## Notation System

### WBS Identifiers
Work items use the format: `W-<DOMAIN>.<section>.<sub>`

Examples:
- `W-CFG.1` — config-core, section 1 (Purpose)
- `W-XT.4.3` — xtask, section 4 (Work Items), item 3
- `W-TF.4` — terraform, section 4 (Work Items)

### ADR Identifiers
Architecture decision records: `ADR-<NNN>`

Format: `ADR-001-short-title.md`

### DRL Identifiers
Drift / lessons learned: `DRL-<YYYY-MM-DD>-<topic>.md`

Example: `DRL-2026-03-18-terraform.md`

---

## Domain Codes

| Code | Component | Path |
|------|-----------|------|
| `CFG` | config-core | `crates/config-core/` |
| `CFGT` | config-toml | `crates/config-toml/` |
| `CFGY` | config-yaml | `crates/config-yaml/` |
| `CFGJ` | config-json | `crates/config-json/` |
| `API` | api-core | `crates/api-core/` |
| `APIO` | api-openapi | `crates/api-openapi/` |
| `APIG` | api-graphql | `crates/api-graphql/` |
| `APIR` | api-grpc | `crates/api-grpc/` |
| `APIM` | api-merger | `crates/api-merger/` |
| `INFR` | infra-types | `crates/infra-types/` |
| `UI` | ui-service | `services/ui/` |
| `XT` | xtask | `xtask/` |
| `TF` | terraform | `infra/` (SUPERSEDED by OTF) |
| `OTF` | opentofu | `infra/` + `xtask/src/infra/` |
| `DX` | justfile + docs + examples | `justfile`, `docs/`, `examples/` |
| `PUB` | Publishing | crates.io release |
| `AUTH` | auth | `services/ui/src/auth.rs`, `routes/auth.rs`, `routes/api/admin.rs`, `routes/dashboard.rs` |

---

## Status Codes

| Code | Meaning |
|------|---------|
| `DONE` | Fully implemented and tested |
| `WIP` | In progress — partially implemented |
| `TODO` | Not started |
| `BLOCKED` | Waiting on dependency or external action |
| `DROPPED` | Decided not to implement |

---

## Module File Structure

Each module file under `plans/modules/` follows this template:

```markdown
# W-<CODE>: <crate-name>
**Crate:** `path/to/crate` | **Status:** DONE/WIP/TODO
**Coverage floor:** N% | **Depends on:** W-XXX | **Depended on by:** W-YYY

## W-<CODE>.1 Purpose
## W-<CODE>.2 Public API Surface
## W-<CODE>.3 Implementation Notes
## W-<CODE>.4 Work Items
| ID | Task | Status | Notes |
## W-<CODE>.5 Test Strategy
## W-<CODE>.6 Cross-References
```

---

## Cross-Reference Syntax

- Module → ADR: `→ ADR-001`
- Module → Drift: `→ DRL-2026-03-18-terraform`
- Module → Module: `→ W-CFG` (depends on), `← W-UI` (depended on by)
- Module → Cross-cutting: `→ aws-architecture`, `→ quality-gates`

---

## File Naming Rules

- Module plans: `plans/modules/<crate-name>.md` (kebab-case, matches crate directory)
- ADRs: `plans/adr/ADR-NNN-short-title.md` (zero-padded 3 digits)
- Drift logs: `plans/drift/DRL-YYYY-MM-DD-topic.md`
- Cross-cutting: `plans/cross-cutting/<topic>.md` (kebab-case)

---

## AI Agent Usage Notes

- To work on a single crate: read `INDEX.md` + `modules/<crate>.md` + relevant cross-cutting file
- Total context per crate: ≤600 lines
- To find all xtask references: `grep -r 'W-XT\.' plans/`
- STATUS in INDEX.md is the authoritative source; module files have detail
