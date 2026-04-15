# Cross-Cutting: LLM Policy

**Applies to:** W-LLM, W-RST, W-RAG (and any future LLM-touching feature)

## Rule 1 — All LLM access goes through `llm-core`

No feature crate or xtask module may make direct HTTP calls to Anthropic, OpenAI, or any other
LLM provider. All access must go through the `llm-core` trait surface (`Completer`, `Embedder`,
`PromptAssembler`). The concrete implementation lives in `llm-anthropic`.

**Why:** centralises retry logic, token counting, error types, and secrets management in one place.
Changing provider or model requires only a new impl crate, not changes in consumers.

## Rule 2 — Secrets via AWS Secrets Manager only

The Anthropic API key is stored at `deploy-baba/prod/anthropic-api-key` in AWS Secrets Manager
(W-SEC). It is read once at Lambda cold start via an `init_api_key()` function mirroring
`init_pow_secret()`. It must never appear in:
- Lambda environment variables (visible in AWS console)
- Source code or hardcoded fallbacks (except `dev-*` local-only defaults for offline dev)
- Committed files of any kind

Use `just secret-put anthropic-api-key $KEY prod` to write it.

## Rule 3 — Universal grounding contract (all generation calls)

Every `PromptBundle` produced by a `PromptAssembler` must include:

### System prompt fragment (append to every system prompt)

```
You are a precise assistant. Answer only from the provided <source> blocks.
Cite every claim with [source N] referring to the Nth source block.
If the answer cannot be found in the sources, say so explicitly.
```

### Retrieved context format

Each retrieved chunk is wrapped as:

```xml
<source kind="{rust|hcl|plan|cache}" path="{repo-relative path}" sha="{git sha of file}">
{chunk content}
</source>
```

Blocks are numbered sequentially from 1 and referenced as `[source 1]`, `[source 2]`, etc. in the
model's response.

### Response format

The `Completer` response is parsed for `[source N]` markers. The caller is responsible for
mapping marker indices back to `RankedChunk` metadata for citation rendering.

**Why this contract exists:** without explicit citations, LLM responses about code are hard to
verify and may hallucinate file paths or function names. Grounding with `<source>` blocks and
mandatory citation markers makes outputs auditable.

## Rule 4 — Embedding caching by content hash

Before calling `Embedder::embed`, check whether the chunk's content hash (stored in
`meta_json.content_hash`) matches the stored embedding. If it matches, skip the API call. This
applies to both `rag-sqlite` (W-RAG) and any future embedding pipeline in W-RST.

Invalidation: whenever a chunk's content changes (detected by hash mismatch), the stored embedding
is overwritten.

## Rule 5 — `.claude/` cache corpus is local-CLI only

The `.claude/` agent-cache, memory files, and conversation history are gitignored and
machine-local. They may be indexed by `just rag-index` for the local developer CLI (W-RAG P1).
They must **not** be bundled into the Lambda zip or served via `/api/ask` (W-RAG P3).

## Cross-References

- → ADR-015 (LLM Provider Abstraction — trait design and provider selection)
- → ADR-016 (RAG Architecture — grounding contract application in retrieval pipeline)
- → W-SEC (secrets management — `anthropic-api-key` in AWS Secrets Manager)
- → W-LLM (implementing crates: `llm-core`, `llm-anthropic`)
- → W-RAG (primary consumer)
- → W-RST (secondary consumer)
