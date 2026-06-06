# DRL-2026-05-28 — RAG Quality Phase 6

**Date:** 2026-05-28
**Modules:** W-RAG, W-AGT, W-UI
**Status:** RESOLVED

## Entries

### DRL-P6-1: TypeScript 8th corpus addition
- **Claim:** RAG indexed 7 corpora
- **Reality:** 8th corpus added: TypeScript/TSX from `web/src/` (~75 files)
- **Fix:** `SourceKind::TypeScript` + brace-balance chunker (`chunk/typescript.rs`, 6 tests)
- **Status:** RESOLVED

### DRL-P6-2: Python 9th corpus addition
- **Claim:** RAG indexed 8 corpora (after TypeScript)
- **Reality:** 9th corpus added: Python from `services/agent/src/` (LangGraph agent code)
- **Fix:** `SourceKind::Python` + indent-based chunker (`chunk/python.rs`, 6 tests)
- **Status:** RESOLVED

### DRL-P6-3: FTS query builder improvements
- **Claim:** FTS queries passed raw user terms to FTS5
- **Reality:** Stop words polluted BM25 ranking; multi-term queries lacked phrase proximity
- **Fix:** 30+ stop-word filter + phrase boost (`"auth login" OR auth OR login` format); 3 tests
- **Status:** RESOLVED

### DRL-P6-4: Corpora aliasing split
- **Claim:** Single `PORTFOLIO_KEYWORDS` list (31 terms) with binary match → 5 portfolio slots
- **Reality:** Architecture/auth queries got too many portfolio slots, crowding FTS results
- **Fix:** Split into `PORTFOLIO_ENTITY_KEYWORDS` vs `CODEBASE_KEYWORDS`; entity-only=5, codebase-only=2, mixed=3 portfolio budget; 2 new tests
- **Status:** RESOLVED

### DRL-P6-5: MCP eval tools
- **Claim:** No programmatic access to eval results outside sqlite3
- **Reality:** Needed token-efficient introspection for agents and CI
- **Fix:** 4 new tools in portfolio-rag-mcp (`eval_report`, `eval_failures`, `corpus_gaps`, `reindex_status`); `project://rag-eval` resource
- **Status:** RESOLVED

### DRL-P6-6: Eval scoring false negatives
- **Claim:** Correctness check only matched `expected_hit` string
- **Reality:** Valid answers using synonyms/aliases scored as failures
- **Fix:** `expected_hit_aliases` column + seed aliases for 5 cases; correctness accepts primary OR any alias
- **Status:** RESOLVED

### DRL-P6-7: UI service RAG endpoints
- **Claim:** No HTTP API for RAG operations
- **Reality:** LangGraph agent needs HTTP endpoints to query eval/health/corpus state
- **Fix:** 5 routes under `/api/v1/rag/*` in `services/ui/src/routes/api/rag.rs`
- **Status:** RESOLVED

### DRL-P6-8: LangGraph RAG sync agent
- **Claim:** No automated quality analysis pipeline
- **Reality:** Manual `just rag-eval` + sqlite3 queries for diagnosis
- **Fix:** `rag_sync.py` ReAct graph + `rag_eval.py` (5 tools) in `services/agent/`; `just rag-sync-agent` recipe
- **Status:** RESOLVED

### DRL-P6-9: Plan system drift (INDEX.md, ADR-016, agent.md)
- **Claim:** INDEX.md said "7 corpora", agent "TODO", ADR-016 listed 4 chunker types
- **Reality:** 9 corpora, agent WIP with scaffold + sync graph done, 6 chunker types
- **Fix:** Updated INDEX.md (module status, work items, dep graph), ADR-016 (status + chunker list), agent.md (W-AGT.4.1 DONE, W-AGT.4.17 added), rag.md (9 corpora, Python chunker row, W-RAG.13.10)
- **Status:** RESOLVED
