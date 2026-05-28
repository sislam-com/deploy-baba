//! RAG index pipeline — `rag ingest` and `rag query` subcommands.
//!
//! In P1 (FTS-only) no embedding provider is required. All retrieval is
//! done via SQLite FTS5 BM25. The commands are invoked through `just`:
//!
//! ```
//! just rag-index          # walk all 4 corpora and upsert into the RAG index
//! just rag-query "..."    # retrieve + print ranked chunks
//! ```

use anyhow::Context;
use clap::Subcommand;
use rag_core::chunk::chunk_file;
use rag_core::types::{RankedChunk, SourceKind};
use rag_core::Retriever;
use rag_sqlite::RagStore;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Subcommand)]
pub enum RagAction {
    /// Walk all corpora and upsert chunks into the RAG index.
    Ingest {
        /// Path to the SQLite database.
        #[arg(long, default_value = "deploy-baba.db")]
        db_path: PathBuf,
        /// Repo root directory to walk.
        #[arg(long, default_value = ".")]
        repo_root: PathBuf,
        /// Index `.claude/` cache corpus (local dev only — gitignored).
        #[arg(long)]
        include_cache: bool,
    },
    /// Retrieve and print ranked chunks for a query.
    Query {
        /// Path to the SQLite database.
        #[arg(long, default_value = "deploy-baba.db")]
        db_path: PathBuf,
        /// The natural-language query.
        query: String,
        /// Maximum number of chunks to return.
        #[arg(long, default_value = "10")]
        top_k: usize,
    },
    /// Retrieve + generate: retrieve chunks via FTS, assemble grounded prompt, call LLM.
    /// Requires ANTHROPIC_API_KEY or OPENAI_API_KEY env var depending on provider.
    Ask {
        /// Path to the SQLite database.
        #[arg(long, default_value = "deploy-baba.db")]
        db_path: PathBuf,
        /// The natural-language question.
        query: String,
        /// Number of source chunks to retrieve (default 10, max 20).
        #[arg(long, default_value = "10")]
        top_k: usize,
        /// Max tokens for the LLM response.
        #[arg(long, default_value = "1024")]
        max_tokens: u32,
        /// LLM provider to use (anthropic or openai).
        #[arg(long, default_value = "anthropic")]
        provider: String,
    },
    /// Run the RAG evaluation suite against seed cases in rag_eval_cases.
    Eval {
        /// Path to the SQLite database.
        #[arg(long, default_value = "deploy-baba.db")]
        db_path: PathBuf,
        /// LLM provider (anthropic or openai). Ignored in --retrieval-only mode.
        #[arg(long, default_value = "anthropic")]
        provider: String,
        /// Max tokens for LLM responses.
        #[arg(long, default_value = "1024")]
        max_tokens: u32,
        /// Number of chunks to retrieve per case.
        #[arg(long, default_value = "10")]
        top_k: usize,
        /// Only test retrieval (no LLM generation). No API key needed.
        #[arg(long)]
        retrieval_only: bool,
        /// Filter by category (portfolio, architecture, code, edge-case).
        #[arg(long)]
        category: Option<String>,
    },
}

pub async fn execute(action: RagAction) -> anyhow::Result<()> {
    match action {
        RagAction::Ingest {
            db_path,
            repo_root,
            include_cache,
        } => ingest(&db_path, &repo_root, include_cache).await,
        RagAction::Query {
            db_path,
            query,
            top_k,
        } => query_cmd(&db_path, &query, top_k).await,
        RagAction::Ask {
            db_path,
            query,
            top_k,
            max_tokens,
            provider,
        } => ask_cmd(&db_path, &query, top_k, max_tokens, &provider).await,
        RagAction::Eval {
            db_path,
            provider,
            max_tokens,
            top_k,
            retrieval_only,
            category,
        } => {
            eval_cmd(
                &db_path,
                &provider,
                max_tokens,
                top_k,
                retrieval_only,
                category.as_deref(),
            )
            .await
        }
    }
}

// ── Ingest ────────────────────────────────────────────────────────────────

async fn ingest(db_path: &Path, repo_root: &Path, include_cache: bool) -> anyhow::Result<()> {
    println!("Opening RAG store at {}", db_path.display());
    let conn = Connection::open(db_path)
        .with_context(|| format!("Failed to open database: {}", db_path.display()))?;
    let store = RagStore::new(conn).context("Failed to initialise RAG schema")?;

    // Current git HEAD SHA (best-effort; falls back to "unknown")
    let git_sha = git_head_sha(repo_root).unwrap_or_else(|_| "unknown".to_string());

    let corpora: Vec<(&str, SourceKind, Vec<&str>)> = vec![
        (
            "Rust source",
            SourceKind::Rust,
            vec!["crates", "services", "xtask"],
        ),
        ("HCL infra", SourceKind::Hcl, vec!["infra"]),
        ("Plans/ADRs", SourceKind::Plan, vec!["plans"]),
        ("TypeScript/React", SourceKind::TypeScript, vec!["web/src"]),
        (
            "Python/LangGraph",
            SourceKind::Python,
            vec!["services/agent/src"],
        ),
    ];

    let mut total_docs = 0u64;
    let mut total_chunks = 0u64;

    for (label, kind, dirs) in &corpora {
        let ext = match kind {
            SourceKind::Rust => "rs",
            SourceKind::Hcl => "tf",
            SourceKind::Plan => "md",
            SourceKind::Cache => "json",
            SourceKind::OpenApi => "json",
            SourceKind::Portfolio => "json",
            SourceKind::TypeScript => "ts,tsx",
            SourceKind::Python => "py",
        };
        println!("  Indexing {label}...");
        let (docs, chunks) = index_corpus(&store, repo_root, dirs, ext, kind, &git_sha)?;
        println!("    {docs} files, {chunks} chunks");
        total_docs += docs;
        total_chunks += chunks;
    }

    // ── OpenAPI spec corpus ──────────────────────────────────────────
    {
        println!("  Indexing OpenAPI spec...");
        let spec = api_openapi::apidoc::full_spec();
        let spec_json =
            serde_json::to_string_pretty(&spec).context("Failed to serialize OpenAPI spec")?;
        let spec_chunks = chunk_file(
            &SourceKind::OpenApi,
            Path::new("api/openapi.json"),
            &spec_json,
        );
        if !spec_chunks.is_empty() {
            store
                .upsert_document("openapi", "api/openapi.json", &git_sha, &spec_chunks)
                .map_err(|e| anyhow::anyhow!("upsert failed for openapi: {e}"))?;
            println!("    1 file, {} chunks", spec_chunks.len());
            total_docs += 1;
            total_chunks += spec_chunks.len() as u64;
        } else {
            println!("    0 chunks (empty spec)");
        }
    }

    // ── Portfolio data corpus ────────────────────────────────────────
    {
        println!("  Indexing portfolio data...");
        let portfolio_conn = Connection::open(db_path)
            .with_context(|| format!("Failed to open portfolio DB: {}", db_path.display()))?;

        let (pdocs, pchunks) = index_portfolio_data(&store, &portfolio_conn, &git_sha)?;
        println!("    {pdocs} tables, {pchunks} chunks");
        total_docs += pdocs;
        total_chunks += pchunks;
    }

    if include_cache {
        println!("  Indexing .claude/ cache...");
        let (docs, chunks) = index_corpus(
            &store,
            repo_root,
            &[".claude"],
            "*",
            &SourceKind::Cache,
            &git_sha,
        )?;
        println!("    {docs} files, {chunks} chunks");
        total_docs += docs;
        total_chunks += chunks;
    }

    println!("Done. Total: {total_docs} documents, {total_chunks} chunks indexed.");

    // ── Optional: embed chunks when OPENAI_API_KEY is set ──────────
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        println!("  Embedding chunks (OPENAI_API_KEY detected)...");
        let embedder = rag_sqlite::embed_bridge::LlmEmbedder::new(std::sync::Arc::new(
            llm_openai::OpenAIProvider::new(api_key),
        ));
        match store.upsert_embeddings(&embedder).await {
            Ok(stats) => {
                println!(
                    "    {} embedded, {} skipped, {} errors",
                    stats.embedded, stats.skipped, stats.errors
                );
            }
            Err(e) => {
                println!("    Embedding failed (FTS-only mode): {e}");
            }
        }
    } else {
        println!("  No OPENAI_API_KEY — skipping embedding (FTS-only mode).");
    }

    Ok(())
}

fn index_corpus(
    store: &RagStore,
    repo_root: &Path,
    dirs: &[&str],
    ext: &str,
    kind: &SourceKind,
    git_sha: &str,
) -> anyhow::Result<(u64, u64)> {
    let mut docs = 0u64;
    let mut chunks_total = 0u64;

    for dir in dirs {
        let base = repo_root.join(dir);
        if !base.exists() {
            continue;
        }
        walk_dir(&base, ext, |file_path| {
            let content = std::fs::read_to_string(file_path)
                .with_context(|| format!("Failed to read {}", file_path.display()))?;

            let rel_path = file_path
                .strip_prefix(repo_root)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            let chunks = chunk_file(kind, file_path, &content);
            if chunks.is_empty() {
                return Ok(());
            }
            let n = chunks.len() as u64;
            store
                .upsert_document(kind.as_str(), &rel_path, git_sha, &chunks)
                .map_err(|e| anyhow::anyhow!("upsert failed for {rel_path}: {e}"))?;
            docs += 1;
            chunks_total += n;
            Ok(())
        })?;
    }

    Ok((docs, chunks_total))
}

/// Recursively walk `dir`, calling `f` for each file whose extension matches `ext`.
/// Use `"*"` for `ext` to match all files.
///
/// Uses `&mut dyn FnMut` internally to avoid monomorphization recursion depth.
fn walk_dir(
    dir: &Path,
    ext: &str,
    f: impl FnMut(&Path) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    walk_dir_dyn(dir, ext, &mut { f })
}

fn walk_dir_dyn(
    dir: &Path,
    ext: &str,
    f: &mut dyn FnMut(&Path) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    for entry in
        std::fs::read_dir(dir).with_context(|| format!("Failed to read dir: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden dirs and target/
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') || name == "target" {
                continue;
            }
            walk_dir_dyn(&path, ext, f)?;
        } else if ext == "*"
            || path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| ext.split(',').any(|allowed| allowed == e))
                .unwrap_or(false)
        {
            f(&path)?;
        }
    }
    Ok(())
}

// ── Portfolio data ────────────────────────────────────────────────────────

fn index_portfolio_data(
    store: &RagStore,
    conn: &Connection,
    git_sha: &str,
) -> anyhow::Result<(u64, u64)> {
    let mut docs = 0u64;
    let mut chunks_total = 0u64;

    type QueryFn = Box<dyn Fn(&Connection) -> anyhow::Result<Vec<serde_json::Value>>>;
    let tables: Vec<(&str, &str, QueryFn)> = vec![
        ("jobs", "portfolio/jobs.json", Box::new(query_jobs)),
        (
            "competencies",
            "portfolio/competencies.json",
            Box::new(query_competencies),
        ),
        (
            "about_sections",
            "portfolio/about.json",
            Box::new(query_about_sections),
        ),
        (
            "social_links",
            "portfolio/social.json",
            Box::new(query_social_links),
        ),
        (
            "challenges",
            "portfolio/challenges.json",
            Box::new(query_challenges),
        ),
    ];

    for (label, virtual_path, query_fn) in &tables {
        match query_fn(conn) {
            Ok(rows) if !rows.is_empty() => {
                let json = serde_json::to_string_pretty(&rows)?;
                let file_chunks =
                    chunk_file(&SourceKind::Portfolio, Path::new(virtual_path), &json);
                if !file_chunks.is_empty() {
                    store
                        .upsert_document("portfolio", virtual_path, git_sha, &file_chunks)
                        .map_err(|e| anyhow::anyhow!("upsert failed for {label}: {e}"))?;
                    docs += 1;
                    chunks_total += file_chunks.len() as u64;
                }
            }
            Ok(_) => {} // empty table, skip
            Err(e) => {
                // Table may not exist yet — skip gracefully
                println!("    skipping {label}: {e}");
            }
        }
    }

    Ok((docs, chunks_total))
}

fn query_jobs(conn: &Connection) -> anyhow::Result<Vec<serde_json::Value>> {
    let mut stmt = conn.prepare(
        "SELECT slug, company, title, location, start_date, end_date, summary, tech_stack
         FROM jobs ORDER BY sort_order ASC",
    )?;
    let jobs: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "slug": row.get::<_, String>(0)?,
                "company": row.get::<_, String>(1)?,
                "title": row.get::<_, String>(2)?,
                "location": row.get::<_, Option<String>>(3)?,
                "start_date": row.get::<_, Option<String>>(4)?,
                "end_date": row.get::<_, Option<String>>(5)?,
                "summary": row.get::<_, Option<String>>(6)?,
                "tech_stack": row.get::<_, Option<String>>(7)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Attach details per job
    let mut result = Vec::new();
    for mut job in jobs {
        let slug = job["slug"].as_str().unwrap_or("").to_string();
        let mut detail_stmt = conn.prepare(
            "SELECT jd.detail_text FROM job_details jd
             JOIN jobs j ON jd.job_id = j.id
             WHERE j.slug = ?1
             ORDER BY jd.sort_order ASC",
        )?;
        let details: Vec<serde_json::Value> = detail_stmt
            .query_map([&slug], |row| {
                Ok(serde_json::json!({ "text": row.get::<_, String>(0)? }))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        job["details"] = serde_json::json!(details);
        result.push(job);
    }

    Ok(result)
}

fn query_competencies(conn: &Connection) -> anyhow::Result<Vec<serde_json::Value>> {
    let mut stmt = conn.prepare(
        "SELECT slug, name, description, icon FROM competencies ORDER BY sort_order ASC",
    )?;
    let comps: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "slug": row.get::<_, String>(0)?,
                "name": row.get::<_, String>(1)?,
                "description": row.get::<_, Option<String>>(2)?,
                "icon": row.get::<_, Option<String>>(3)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut result = Vec::new();
    for mut comp in comps {
        let slug = comp["slug"].as_str().unwrap_or("").to_string();
        let mut hl_stmt = conn.prepare(
            "SELECT jd.detail_text, j.company FROM competency_evidence ce
             LEFT JOIN jobs j ON ce.job_id = j.id
             LEFT JOIN job_details jd ON ce.detail_id = jd.id
             WHERE ce.competency_id = (SELECT id FROM competencies WHERE slug = ?1)
             ORDER BY ce.sort_order ASC",
        )?;
        let highlights: Vec<serde_json::Value> = hl_stmt
            .query_map([&slug], |row| {
                Ok(serde_json::json!({
                    "text": row.get::<_, Option<String>>(0)?,
                    "company": row.get::<_, Option<String>>(1)?,
                }))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        comp["evidence"] = serde_json::json!(highlights);
        result.push(comp);
    }

    Ok(result)
}

fn query_about_sections(conn: &Connection) -> anyhow::Result<Vec<serde_json::Value>> {
    let mut stmt =
        conn.prepare("SELECT slug, heading, body FROM about_sections ORDER BY sort_order ASC")?;
    let rows: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "slug": row.get::<_, String>(0)?,
                "heading": row.get::<_, String>(1)?,
                "body": row.get::<_, Option<String>>(2)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn query_social_links(conn: &Connection) -> anyhow::Result<Vec<serde_json::Value>> {
    let mut stmt = conn.prepare(
        "SELECT platform, url, label FROM social_links WHERE visible = 1 ORDER BY sort_order ASC",
    )?;
    let rows: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "platform": row.get::<_, String>(0)?,
                "url": row.get::<_, String>(1)?,
                "label": row.get::<_, Option<String>>(2)?,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn query_challenges(conn: &Connection) -> anyhow::Result<Vec<serde_json::Value>> {
    let structured = has_column(conn, "challenges", "problem");
    let sql = if structured {
        "SELECT slug, title, description, short_description, tech_stack, category, url,
                problem, constraints, decisions, implementation, outcomes, metrics,
                related_job_slug, related_plan_module, related_adr, featured
         FROM challenges ORDER BY sort_order ASC"
    } else {
        "SELECT slug, title, description, short_description, tech_stack, category, url,
                NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, featured
         FROM challenges ORDER BY sort_order ASC"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            let featured: i64 = row.get(16)?;
            Ok(serde_json::json!({
                "entity_type": "challenge",
                "slug": row.get::<_, String>(0)?,
                "title": row.get::<_, String>(1)?,
                "description": row.get::<_, String>(2)?,
                "short_description": row.get::<_, Option<String>>(3)?,
                "tech_stack": row.get::<_, Option<String>>(4)?,
                "category": row.get::<_, Option<String>>(5)?,
                "url": row.get::<_, Option<String>>(6)?,
                "problem": row.get::<_, Option<String>>(7)?,
                "constraints": row.get::<_, Option<String>>(8)?,
                "decisions": row.get::<_, Option<String>>(9)?,
                "implementation": row.get::<_, Option<String>>(10)?,
                "outcomes": row.get::<_, Option<String>>(11)?,
                "metrics": row.get::<_, Option<String>>(12)?,
                "related_job_slug": row.get::<_, Option<String>>(13)?,
                "related_plan_module": row.get::<_, Option<String>>(14)?,
                "related_adr": row.get::<_, Option<String>>(15)?,
                "featured": featured != 0,
            }))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn has_column(conn: &Connection, table: &str, column: &str) -> bool {
    let pragma = format!("PRAGMA table_info({table})");
    let Ok(mut stmt) = conn.prepare(&pragma) else {
        return false;
    };
    let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(1)) else {
        return false;
    };
    let cols: Vec<String> = rows.filter_map(|r| r.ok()).collect();
    cols.iter().any(|c| c == column)
}

// ── Ask ───────────────────────────────────────────────────────────────────

async fn ask_cmd(
    db_path: &Path,
    query: &str,
    top_k: usize,
    max_tokens: u32,
    provider_id: &str,
) -> anyhow::Result<()> {
    use llm_anthropic::AnthropicProvider;
    use llm_core::{ChatMessage, GenerationConfig, LlmProvider, LlmRequest, MessageRole};
    use llm_openai::OpenAIProvider;
    use rag_core::{DefaultPromptAssembler, PromptAssembler, Retriever};

    let api_key = match provider_id {
        "anthropic" => std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
            anyhow::anyhow!(
                "ANTHROPIC_API_KEY env var required for `rag ask` with provider=anthropic"
            )
        })?,
        "openai" => std::env::var("OPENAI_API_KEY").map_err(|_| {
            anyhow::anyhow!("OPENAI_API_KEY env var required for `rag ask` with provider=openai")
        })?,
        _ => return Err(anyhow::anyhow!("Unknown provider: {provider_id}")),
    };

    let top_k = top_k.clamp(1, 20);

    println!("Retrieving chunks for: {query:?}");
    let conn = Connection::open(db_path)
        .with_context(|| format!("Failed to open {}", db_path.display()))?;
    let store = RagStore::new(conn).context("Failed to initialise RAG schema")?;

    let chunks = store
        .retrieve(query, top_k)
        .await
        .map_err(|e| anyhow::anyhow!("retrieval failed: {e}"))?;

    if chunks.is_empty() {
        println!("No matching chunks found — try `just rag-index` first.");
        return Ok(());
    }

    println!(
        "Found {} chunk(s). Calling {}...\n",
        chunks.len(),
        provider_id
    );

    let assembler = DefaultPromptAssembler;
    let bundle = assembler.assemble(query, &chunks);

    let provider: Box<dyn LlmProvider> = match provider_id {
        "anthropic" => Box::new(AnthropicProvider::new(api_key)),
        "openai" => Box::new(OpenAIProvider::new(api_key)),
        _ => return Err(anyhow::anyhow!("Unknown provider: {provider_id}")),
    };

    let req = LlmRequest {
        model: provider.default_model().to_owned(),
        messages: vec![ChatMessage::text(MessageRole::User, bundle.user_message)],
        system: Some(bundle.system_prompt),
        tools: vec![],
        grounding: None,
        config: GenerationConfig {
            max_tokens,
            temperature: 0.2,
            prompt_version: "ask-v1",
        },
    };

    let resp = provider
        .generate(req)
        .await
        .map_err(|e| anyhow::anyhow!("LLM generate failed: {e}"))?;

    println!("─── Answer ───────────────────────────────────────────────────");
    println!("{}", resp.content);
    println!();
    println!(
        "─── Sources ({} cited) ────────────────────────────────────────",
        bundle.citations.len()
    );
    for (i, c) in bundle.citations.iter().enumerate() {
        println!(
            "  [{n}] {path} (sha={sha})",
            n = i + 1,
            path = c.path,
            sha = &c.sha[..7.min(c.sha.len())]
        );
    }
    println!();
    println!(
        "Tokens: {} in / {} out | Model: {}",
        resp.input_tokens, resp.output_tokens, resp.model
    );
    Ok(())
}

// ── Eval ─────────────────────────────────────────────────────────────────

struct EvalCase {
    id: i64,
    question: String,
    expected_hit: String,
    expected_hit_aliases: Vec<String>,
    source_path: Option<String>,
    expected_source_kind: Option<String>,
    expected_entity_type: Option<String>,
    category: String,
    difficulty: String,
}

fn check_retrieval_hit(case: &EvalCase, chunks: &[RankedChunk]) -> bool {
    let path_hit = match case.source_path.as_deref() {
        None => true,
        Some(prefix) => chunks.iter().any(|c| c.source_path.starts_with(prefix)),
    };
    let kind_hit = match case.expected_source_kind.as_deref() {
        None => true,
        Some(kind) => chunks.iter().any(|c| c.source_kind == kind),
    };
    let entity_hit = match case.expected_entity_type.as_deref() {
        None => true,
        Some(entity_type) => chunks.iter().any(|c| {
            c.source_path
                .starts_with(&format!("portfolio://{entity_type}"))
        }),
    };
    path_hit && kind_hit && entity_hit
}

fn ensure_eval_v2_schema(conn: &Connection) -> anyhow::Result<()> {
    if !has_column(conn, "rag_eval_cases", "expected_source_kind") {
        conn.execute(
            "ALTER TABLE rag_eval_cases ADD COLUMN expected_source_kind TEXT",
            [],
        )?;
    }
    if !has_column(conn, "rag_eval_cases", "expected_entity_type") {
        conn.execute(
            "ALTER TABLE rag_eval_cases ADD COLUMN expected_entity_type TEXT",
            [],
        )?;
    }
    if !has_column(conn, "rag_eval_cases", "expected_hit_aliases") {
        conn.execute(
            "ALTER TABLE rag_eval_cases ADD COLUMN expected_hit_aliases TEXT",
            [],
        )?;
    }
    if !has_column(conn, "rag_eval_cases", "corpus_filter") {
        conn.execute(
            "ALTER TABLE rag_eval_cases ADD COLUMN corpus_filter TEXT",
            [],
        )?;
    }
    if !has_column(conn, "rag_eval_results", "top_k_hit") {
        conn.execute(
            "ALTER TABLE rag_eval_results ADD COLUMN top_k_hit INTEGER",
            [],
        )?;
    }

    conn.execute_batch(
        "UPDATE rag_eval_results
         SET top_k_hit = retrieval_hit
         WHERE top_k_hit IS NULL;",
    )?;

    conn.execute_batch(
        "UPDATE rag_eval_cases
         SET expected_source_kind = COALESCE(expected_source_kind, 'portfolio'),
             expected_entity_type = COALESCE(expected_entity_type, 'competency')
         WHERE question IN (
           'What are your primary skills and technical expertise?',
           'Tell me about your experience with AI/LLM systems and RAG pipelines',
           'What is your experience with cloud infrastructure and AWS?',
           'How many competencies does the portfolio list?'
         );

         UPDATE rag_eval_cases
         SET expected_source_kind = COALESCE(expected_source_kind, 'portfolio'),
             expected_entity_type = COALESCE(expected_entity_type, 'job')
         WHERE question IN (
           'Describe your technical leadership and team management experience',
           'What platforms and products have you built end-to-end?',
           'Compare the jobs at Scala Computing and the personal projects'
         );

         UPDATE rag_eval_cases
         SET expected_source_kind = COALESCE(expected_source_kind, 'portfolio'),
             expected_entity_type = COALESCE(expected_entity_type, 'challenge')
         WHERE question IN (
           'How does the RAG pipeline in this portfolio project work?',
           'Tell me about the 27-step deployment challenge'
         );

         UPDATE rag_eval_cases
         SET expected_source_kind = COALESCE(expected_source_kind, 'portfolio'),
             expected_entity_type = COALESCE(expected_entity_type, 'about')
         WHERE question IN (
           'What are the key architecture decisions in this portfolio?'
         );

         INSERT INTO rag_eval_cases (
           question, expected_hit, source_path, category, difficulty, expected_source_kind, expected_entity_type
         )
         VALUES
           (
             'Which challenge explains key constraints and tradeoffs?',
             'constraint',
             'portfolio://challenge',
             'challenge',
             'medium',
             'portfolio',
             'challenge'
           ),
           (
             'Which challenge documents measurable outcomes and metrics?',
             'metric',
             'portfolio://challenge',
             'challenge',
             'medium',
             'portfolio',
             'challenge'
           ),
           (
             'Which challenge references ADR or module alignment?',
             'ADR',
             'portfolio://challenge',
             'challenge',
             'hard',
             'portfolio',
             'challenge'
           )
         ON CONFLICT(question) DO UPDATE SET
           expected_hit = EXCLUDED.expected_hit,
           source_path = EXCLUDED.source_path,
           category = EXCLUDED.category,
           difficulty = EXCLUDED.difficulty,
           expected_source_kind = EXCLUDED.expected_source_kind,
           expected_entity_type = EXCLUDED.expected_entity_type;",
    )?;

    conn.execute_batch(
        "UPDATE rag_eval_cases SET expected_hit_aliases = '[source,cite,ground'
         WHERE question LIKE '%grounding%citation%';

         UPDATE rag_eval_cases SET expected_hit_aliases = 'retrieval,FTS,chunk'
         WHERE question LIKE '%RAG pipeline%';

         UPDATE rag_eval_cases SET expected_hit_aliases = 'portfolio,FTS,live'
         WHERE question LIKE '%hybrid retriever%' OR question LIKE '%Hybrid%';

         UPDATE rag_eval_cases SET expected_hit_aliases = 'status,error,Result'
         WHERE expected_hit = 'StatusCode';

         UPDATE rag_eval_cases SET expected_hit_aliases = 'sha256,hash,proof,challenge'
         WHERE expected_hit = 'SHA';",
    )?;

    conn.execute_batch(
        "INSERT INTO rag_eval_cases (
           question, expected_hit, source_path, category, difficulty, expected_source_kind, corpus_filter
         )
         VALUES
           (
             'How does the SPA login form work?',
             'signin',
             'web/src/routes/auth',
             'code',
             'medium',
             'typescript',
             'typescript'
           ),
           (
             'What happens when useAuth detects an unauthenticated user?',
             'navigate',
             'web/src/hooks',
             'code',
             'easy',
             'typescript',
             'typescript'
           ),
           (
             'What are the auth routes in the React SPA?',
             'Login',
             'web/src/routes/auth',
             'architecture',
             'easy',
             'typescript',
             'typescript'
           )
         ON CONFLICT(question) DO UPDATE SET
           expected_hit = EXCLUDED.expected_hit,
           source_path = EXCLUDED.source_path,
           category = EXCLUDED.category,
           difficulty = EXCLUDED.difficulty,
           expected_source_kind = EXCLUDED.expected_source_kind,
           corpus_filter = EXCLUDED.corpus_filter;",
    )?;

    Ok(())
}

struct SqlitePortfolioProvider {
    conn: std::sync::Mutex<Connection>,
}

#[async_trait::async_trait]
impl rag_core::portfolio::PortfolioDataProvider for SqlitePortfolioProvider {
    async fn get_jobs_summary(&self) -> Result<Vec<serde_json::Value>, rag_core::RagError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT j.slug, j.company, j.title, j.location, j.start_date, j.end_date, j.summary, j.tech_stack
                 FROM jobs j ORDER BY j.sort_order ASC",
            )
            .map_err(|e| rag_core::RagError::Database(e.to_string()))?;
        let jobs: Vec<(String, serde_json::Value)> = stmt
            .query_map([], |row| {
                let slug = row.get::<_, String>(0)?;
                Ok((
                    slug.clone(),
                    serde_json::json!({
                        "slug": slug,
                        "company": row.get::<_, String>(1)?,
                        "title": row.get::<_, String>(2)?,
                        "location": row.get::<_, Option<String>>(3)?,
                        "start_date": row.get::<_, Option<String>>(4)?,
                        "end_date": row.get::<_, Option<String>>(5)?,
                        "summary": row.get::<_, Option<String>>(6)?,
                        "tech_stack": row.get::<_, Option<String>>(7)?,
                    }),
                ))
            })
            .map_err(|e| rag_core::RagError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        let mut result = Vec::new();
        for (slug, mut job_val) in jobs {
            let job_id: Option<i64> = conn
                .query_row(
                    "SELECT id FROM jobs WHERE slug = ?1",
                    rusqlite::params![&slug],
                    |row| row.get(0),
                )
                .ok();
            if let Some(jid) = job_id {
                let mut ds = conn
                    .prepare("SELECT detail_text, category FROM job_details WHERE job_id = ?1 ORDER BY sort_order ASC")
                    .map_err(|e| rag_core::RagError::Database(e.to_string()))?;
                let details: Vec<serde_json::Value> = ds
                    .query_map(rusqlite::params![jid], |row| {
                        Ok(serde_json::json!({
                            "text": row.get::<_, String>(0)?,
                            "category": row.get::<_, Option<String>>(1)?,
                        }))
                    })
                    .map_err(|e| rag_core::RagError::Database(e.to_string()))?
                    .filter_map(|r| r.ok())
                    .collect();
                job_val["details"] = serde_json::Value::Array(details);
            }
            result.push(job_val);
        }
        Ok(result)
    }

    async fn get_job_details(
        &self,
        slug: &str,
    ) -> Result<Option<serde_json::Value>, rag_core::RagError> {
        let conn = self.conn.lock().unwrap();
        let job = conn
            .query_row(
                "SELECT id, slug, company, title, summary, tech_stack FROM jobs WHERE slug = ?1",
                rusqlite::params![slug],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        serde_json::json!({
                            "slug": row.get::<_, String>(1)?,
                            "company": row.get::<_, String>(2)?,
                            "title": row.get::<_, String>(3)?,
                            "summary": row.get::<_, Option<String>>(4)?,
                            "tech_stack": row.get::<_, Option<String>>(5)?,
                        }),
                    ))
                },
            )
            .ok();
        let Some((job_id, mut val)) = job else {
            return Ok(None);
        };
        let mut ds = conn
            .prepare(
                "SELECT detail_text FROM job_details WHERE job_id = ?1 ORDER BY sort_order ASC",
            )
            .map_err(|e| rag_core::RagError::Database(e.to_string()))?;
        let details: Vec<serde_json::Value> = ds
            .query_map(rusqlite::params![job_id], |row| {
                Ok(serde_json::json!({ "text": row.get::<_, String>(0)? }))
            })
            .map_err(|e| rag_core::RagError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        val["details"] = serde_json::Value::Array(details);
        Ok(Some(val))
    }

    async fn get_competencies_summary(&self) -> Result<Vec<serde_json::Value>, rag_core::RagError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT slug, name, description, icon FROM competencies ORDER BY sort_order ASC",
            )
            .map_err(|e| rag_core::RagError::Database(e.to_string()))?;
        let comps: Vec<(String, serde_json::Value)> = stmt
            .query_map([], |row| {
                let slug = row.get::<_, String>(0)?;
                Ok((
                    slug.clone(),
                    serde_json::json!({
                        "slug": slug,
                        "name": row.get::<_, String>(1)?,
                        "description": row.get::<_, Option<String>>(2)?,
                        "icon": row.get::<_, Option<String>>(3)?,
                    }),
                ))
            })
            .map_err(|e| rag_core::RagError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        let mut result = Vec::new();
        for (slug, mut val) in comps {
            let cid: Option<i64> = conn
                .query_row(
                    "SELECT id FROM competencies WHERE slug = ?1",
                    rusqlite::params![&slug],
                    |row| row.get(0),
                )
                .ok();
            if let Some(cid) = cid {
                let mut es = conn
                    .prepare("SELECT jd.detail_text, j.company FROM competency_evidence ce LEFT JOIN jobs j ON ce.job_id = j.id LEFT JOIN job_details jd ON ce.detail_id = jd.id WHERE ce.competency_id = ?1 ORDER BY ce.sort_order ASC")
                    .map_err(|e| rag_core::RagError::Database(e.to_string()))?;
                let evidence: Vec<serde_json::Value> = es
                    .query_map(rusqlite::params![cid], |row| {
                        Ok(serde_json::json!({
                            "text": row.get::<_, Option<String>>(0)?,
                            "company": row.get::<_, Option<String>>(1)?,
                        }))
                    })
                    .map_err(|e| rag_core::RagError::Database(e.to_string()))?
                    .filter_map(|r| r.ok())
                    .collect();
                val["evidence"] = serde_json::Value::Array(evidence);
            }
            result.push(val);
        }
        Ok(result)
    }

    async fn get_about_sections(&self) -> Result<Vec<serde_json::Value>, rag_core::RagError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT slug, heading, body FROM about_sections ORDER BY sort_order ASC")
            .map_err(|e| rag_core::RagError::Database(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "slug": row.get::<_, String>(0)?,
                    "heading": row.get::<_, String>(1)?,
                    "body": row.get::<_, Option<String>>(2)?,
                }))
            })
            .map_err(|e| rag_core::RagError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    async fn get_challenges_summary(&self) -> Result<Vec<serde_json::Value>, rag_core::RagError> {
        let conn = self.conn.lock().unwrap();
        let structured = has_column(&conn, "challenges", "problem");
        let sql = if structured {
            "SELECT slug, title, description, short_description, tech_stack, category, url,
                    problem, constraints, decisions, implementation, outcomes, metrics,
                    related_job_slug, related_plan_module, related_adr, featured
             FROM challenges ORDER BY sort_order ASC"
        } else {
            "SELECT slug, title, description, short_description, tech_stack, category, url,
                    NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, featured
             FROM challenges ORDER BY sort_order ASC"
        };
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| rag_core::RagError::Database(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                let featured: i64 = row.get(16)?;
                Ok(serde_json::json!({
                    "entity_type": "challenge",
                    "slug": row.get::<_, String>(0)?,
                    "title": row.get::<_, String>(1)?,
                    "description": row.get::<_, String>(2)?,
                    "short_description": row.get::<_, Option<String>>(3)?,
                    "tech_stack": row.get::<_, Option<String>>(4)?,
                    "category": row.get::<_, Option<String>>(5)?,
                    "url": row.get::<_, Option<String>>(6)?,
                    "problem": row.get::<_, Option<String>>(7)?,
                    "constraints": row.get::<_, Option<String>>(8)?,
                    "decisions": row.get::<_, Option<String>>(9)?,
                    "implementation": row.get::<_, Option<String>>(10)?,
                    "outcomes": row.get::<_, Option<String>>(11)?,
                    "metrics": row.get::<_, Option<String>>(12)?,
                    "related_job_slug": row.get::<_, Option<String>>(13)?,
                    "related_plan_module": row.get::<_, Option<String>>(14)?,
                    "related_adr": row.get::<_, Option<String>>(15)?,
                    "featured": featured != 0,
                }))
            })
            .map_err(|e| rag_core::RagError::Database(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }
}

async fn eval_cmd(
    db_path: &Path,
    provider_id: &str,
    max_tokens: u32,
    top_k: usize,
    retrieval_only: bool,
    category_filter: Option<&str>,
) -> anyhow::Result<()> {
    use llm_anthropic::AnthropicProvider;
    use llm_core::{ChatMessage, GenerationConfig, LlmProvider, LlmRequest, MessageRole};
    use llm_openai::OpenAIProvider;
    use rag_core::{DefaultPromptAssembler, PromptAssembler};

    let top_k = top_k.clamp(1, 20);
    let mode_label = if retrieval_only {
        "retrieval-only"
    } else {
        "full"
    };

    // Connection 1: consumed by RagStore for retrieval
    let conn1 = Connection::open(db_path)
        .with_context(|| format!("Failed to open {}", db_path.display()))?;
    let store =
        std::sync::Arc::new(RagStore::new(conn1).context("Failed to initialise RAG schema")?);

    // Connection 2: eval table reads/writes
    let conn2 = Connection::open(db_path)
        .with_context(|| format!("Failed to open {}", db_path.display()))?;
    conn2.execute_batch(include_str!(
        "../../services/ui/migrations/023_rag_eval.sql"
    ))?;
    ensure_eval_v2_schema(&conn2)?;

    // Connection 3: portfolio data provider for HybridRetriever
    let conn3 = Connection::open(db_path)
        .with_context(|| format!("Failed to open {}", db_path.display()))?;
    let portfolio = SqlitePortfolioProvider {
        conn: std::sync::Mutex::new(conn3),
    };
    let hybrid = rag_core::HybridRetriever {
        fts: std::sync::Arc::clone(&store),
        portfolio,
    };

    // Read eval cases
    let sql = if category_filter.is_some() {
        "SELECT id, question, expected_hit, source_path, expected_source_kind, expected_entity_type, category, difficulty, expected_hit_aliases \
         FROM rag_eval_cases WHERE category = ?1 ORDER BY id"
    } else {
        "SELECT id, question, expected_hit, source_path, expected_source_kind, expected_entity_type, category, difficulty, expected_hit_aliases \
         FROM rag_eval_cases ORDER BY id"
    };
    let parse_aliases = |raw: Option<String>| -> Vec<String> {
        raw.map(|s| {
            s.split(',')
                .map(|a| a.trim().to_string())
                .filter(|a| !a.is_empty())
                .collect()
        })
        .unwrap_or_default()
    };
    let mut stmt = conn2.prepare(sql)?;
    let cases: Vec<EvalCase> = if let Some(cat) = category_filter {
        stmt.query_map([cat], |row| {
            Ok(EvalCase {
                id: row.get(0)?,
                question: row.get(1)?,
                expected_hit: row.get(2)?,
                expected_hit_aliases: parse_aliases(row.get(8)?),
                source_path: row.get(3)?,
                expected_source_kind: row.get(4)?,
                expected_entity_type: row.get(5)?,
                category: row.get(6)?,
                difficulty: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    } else {
        stmt.query_map([], |row| {
            Ok(EvalCase {
                id: row.get(0)?,
                question: row.get(1)?,
                expected_hit: row.get(2)?,
                expected_hit_aliases: parse_aliases(row.get(8)?),
                source_path: row.get(3)?,
                expected_source_kind: row.get(4)?,
                expected_entity_type: row.get(5)?,
                category: row.get(6)?,
                difficulty: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?
    };

    if cases.is_empty() {
        println!("No eval cases found. Run migrations first (`just rag-index`).");
        return Ok(());
    }

    let git_sha = git_head_sha(Path::new(".")).unwrap_or_else(|_| "unknown".to_string());

    // Create eval run placeholder
    conn2.execute(
        "INSERT INTO rag_eval_runs (git_sha, prompt_version, total_cases, pass_count) \
         VALUES (?1, ?2, ?3, 0)",
        rusqlite::params![git_sha, "eval-v1", cases.len()],
    )?;
    let run_id = conn2.last_insert_rowid();

    // Build LLM provider if full mode
    let provider: Option<Box<dyn LlmProvider>> = if retrieval_only {
        None
    } else {
        let api_key = match provider_id {
            "anthropic" => std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
                anyhow::anyhow!(
                    "ANTHROPIC_API_KEY required for full eval (use --retrieval-only to skip LLM)"
                )
            })?,
            "openai" => std::env::var("OPENAI_API_KEY").map_err(|_| {
                anyhow::anyhow!(
                    "OPENAI_API_KEY required for full eval (use --retrieval-only to skip LLM)"
                )
            })?,
            _ => return Err(anyhow::anyhow!("Unknown provider: {provider_id}")),
        };
        Some(match provider_id {
            "anthropic" => Box::new(AnthropicProvider::new(api_key)),
            "openai" => Box::new(OpenAIProvider::new(api_key)),
            _ => unreachable!(),
        })
    };

    println!("══ RAG Eval ═══════════════════════════════════════════════════════");
    println!("  Run #{run_id} | sha: {git_sha} | prompt: eval-v1 | mode: {mode_label}");
    println!("  {} cases", cases.len());
    println!("───────────────────────────────────────────────────────────────────");

    if retrieval_only {
        println!(
            "  {:>3} │ {:12} │ {:10} │ {:3} │ Pass",
            "#", "Category", "Difficulty", "Hit"
        );
    } else {
        println!(
            "  {:>3} │ {:12} │ {:10} │ {:3} │ {:>5} │ {:>4} │ {:>4} │ {:>6} │ Pass",
            "#", "Category", "Difficulty", "Hit", "Grnd", "Corr", "Cite", "ms"
        );
    }

    let mut pass_count = 0u32;
    let mut category_totals: std::collections::BTreeMap<String, (u32, u32)> =
        std::collections::BTreeMap::new();
    let mut sum_groundedness = 0.0f64;
    let mut sum_correctness = 0.0f64;
    let mut scored_count = 0u32;

    for (i, case) in cases.iter().enumerate() {
        let start = Instant::now();

        let chunks = match hybrid.retrieve(&case.question, top_k).await {
            Ok(c) => c,
            Err(e) => {
                let failure = format!("retrieval_error: {e}");
                insert_eval_result(
                    &conn2,
                    run_id,
                    case.id,
                    "",
                    None,
                    None,
                    false,
                    None,
                    None,
                    None,
                    0,
                    Some(&failure),
                )?;
                print_eval_row(
                    i + 1,
                    case,
                    false,
                    None,
                    None,
                    None,
                    0,
                    false,
                    retrieval_only,
                );
                continue;
            }
        };

        let hit = check_retrieval_hit(case, &chunks);
        let latency_ms;
        let mut groundedness: Option<f64> = None;
        let mut correctness: Option<f64> = None;
        let mut citation_accuracy: Option<f64> = None;
        let mut answer = String::new();
        let mut failure_type: Option<String> = None;

        if retrieval_only || chunks.is_empty() {
            latency_ms = start.elapsed().as_millis() as i64;
            if chunks.is_empty() {
                failure_type = Some("no_chunks_retrieved".to_string());
            }
        } else if let Some(ref prov) = provider {
            let assembler = DefaultPromptAssembler;
            let bundle = assembler.assemble(&case.question, &chunks);

            let req = LlmRequest {
                model: prov.default_model().to_owned(),
                messages: vec![ChatMessage::text(MessageRole::User, &bundle.user_message)],
                system: Some(bundle.system_prompt),
                tools: vec![],
                grounding: None,
                config: GenerationConfig {
                    max_tokens,
                    temperature: 0.0,
                    prompt_version: "eval-v1",
                },
            };

            match prov.generate(req).await {
                Ok(resp) => {
                    answer = resp.content;
                    let g = rag_core::eval::score_groundedness(&answer) as f64;
                    let answer_lower = answer.to_lowercase();
                    let c = if answer_lower.contains(&case.expected_hit.to_lowercase())
                        || case
                            .expected_hit_aliases
                            .iter()
                            .any(|alias| answer_lower.contains(&alias.to_lowercase()))
                    {
                        1.0
                    } else {
                        0.0
                    };
                    let (valid, invalid) =
                        rag_core::eval::verify_citation_refs(&answer, chunks.len());
                    let total_refs = valid + invalid.len();
                    let ca = if total_refs == 0 {
                        1.0
                    } else {
                        valid as f64 / total_refs as f64
                    };

                    groundedness = Some(g);
                    correctness = Some(c);
                    citation_accuracy = Some(ca);
                    sum_groundedness += g;
                    sum_correctness += c;
                    scored_count += 1;
                }
                Err(e) => {
                    failure_type = Some(format!("llm_error: {e}"));
                }
            }

            latency_ms = start.elapsed().as_millis() as i64;
        } else {
            latency_ms = start.elapsed().as_millis() as i64;
        }

        let passed = hit
            && failure_type.is_none()
            && (retrieval_only || correctness.is_some_and(|c| c >= 1.0));

        if passed {
            pass_count += 1;
        }
        let entry = category_totals
            .entry(case.category.clone())
            .or_insert((0, 0));
        entry.0 += 1;
        if passed {
            entry.1 += 1;
        }

        insert_eval_result(
            &conn2,
            run_id,
            case.id,
            &answer,
            None,
            None,
            hit,
            groundedness,
            correctness,
            citation_accuracy,
            latency_ms,
            failure_type.as_deref(),
        )?;

        print_eval_row(
            i + 1,
            case,
            hit,
            groundedness,
            correctness,
            citation_accuracy,
            latency_ms,
            passed,
            retrieval_only,
        );
    }

    // Update run summary
    let avg_g = if scored_count > 0 {
        Some(sum_groundedness / scored_count as f64)
    } else {
        None
    };
    let avg_c = if scored_count > 0 {
        Some(sum_correctness / scored_count as f64)
    } else {
        None
    };

    conn2.execute(
        "UPDATE rag_eval_runs SET pass_count = ?1, avg_groundedness = ?2, avg_correctness = ?3 \
         WHERE id = ?4",
        rusqlite::params![pass_count, avg_g, avg_c, run_id],
    )?;

    let total = cases.len() as f64;
    let pct = if total > 0.0 {
        (pass_count as f64 / total) * 100.0
    } else {
        0.0
    };

    println!("───────────────────────────────────────────────────────────────────");
    if let (Some(g), Some(c)) = (avg_g, avg_c) {
        println!(
            "  {pass_count}/{} passed ({pct:.1}%) │ avg groundedness: {g:.2} │ avg correctness: {c:.2}",
            cases.len()
        );
    } else {
        println!("  {pass_count}/{} passed ({pct:.1}%)", cases.len());
    }
    println!("  Category scorecard:");
    for (category, (total_cases, passed_cases)) in category_totals {
        let cat_pct = if total_cases > 0 {
            (passed_cases as f64 / total_cases as f64) * 100.0
        } else {
            0.0
        };
        println!("    - {category}: {passed_cases}/{total_cases} ({cat_pct:.1}%)");
    }
    println!("═══════════════════════════════════════════════════════════════════");

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn insert_eval_result(
    conn: &Connection,
    run_id: i64,
    case_id: i64,
    answer: &str,
    citations_json: Option<&str>,
    chunks_json: Option<&str>,
    retrieval_hit: bool,
    groundedness: Option<f64>,
    correctness: Option<f64>,
    citation_accuracy: Option<f64>,
    latency_ms: i64,
    failure_type: Option<&str>,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO rag_eval_results \
         (run_id, case_id, answer, citations_json, chunks_json, retrieval_hit, \
          top_k_hit, groundedness, correctness, citation_accuracy, latency_ms, failure_type) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            run_id,
            case_id,
            answer,
            citations_json,
            chunks_json,
            retrieval_hit as i32,
            retrieval_hit as i32,
            groundedness,
            correctness,
            citation_accuracy,
            latency_ms,
            failure_type,
        ],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn print_eval_row(
    num: usize,
    case: &EvalCase,
    hit: bool,
    groundedness: Option<f64>,
    correctness: Option<f64>,
    citation_accuracy: Option<f64>,
    latency_ms: i64,
    passed: bool,
    retrieval_only: bool,
) {
    let hit_s = if hit { " Y " } else { " N " };
    let pass_s = if passed { " OK " } else { "FAIL" };

    if retrieval_only {
        println!(
            "  {:>3} │ {:12} │ {:10} │ {:3} │ {pass_s}",
            num, case.category, case.difficulty, hit_s
        );
    } else {
        let g_s = groundedness.map_or(" -  ".to_string(), |v| format!("{v:.2}"));
        let c_s = correctness.map_or(" -  ".to_string(), |v| format!("{v:.2}"));
        let ca_s = citation_accuracy.map_or(" -  ".to_string(), |v| format!("{v:.2}"));
        println!(
            "  {:>3} │ {:12} │ {:10} │ {:3} │ {:>5} │ {:>4} │ {:>4} │ {:>6} │ {pass_s}",
            num, case.category, case.difficulty, hit_s, g_s, c_s, ca_s, latency_ms
        );
    }
}

/// Diagnose a deployment failure using RAG to find relevant documentation
///
/// This function queries the RAG system with context about a deployment failure
/// to provide automated diagnosis suggestions.
pub async fn diagnose_failure(error_context: &str) -> anyhow::Result<Vec<String>> {
    let db_path = std::path::PathBuf::from("deploy-baba.db");
    let query = format!("deployment failure error: {}", error_context);

    let conn = Connection::open(&db_path)
        .with_context(|| format!("Failed to open database: {}", db_path.display()))?;
    let store = RagStore::new(conn).context("Failed to initialise RAG schema")?;

    let results = store
        .retrieve(&query, 5) // Get top 5 most relevant chunks
        .await
        .map_err(|e| anyhow::anyhow!("retrieval failed: {e}"))?;

    if results.is_empty() {
        return Ok(vec![
            "No relevant documentation found for this error.".to_string()
        ]);
    }

    let suggestions: Vec<String> = results
        .iter()
        .map(|chunk| {
            let preview: String = chunk.content.chars().take(200).collect();
            format!("{}: {}", chunk.source_path, preview)
        })
        .collect();

    Ok(suggestions)
}

fn git_head_sha(repo_root: &Path) -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(repo_root)
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// ── Query ─────────────────────────────────────────────────────────────────

async fn query_cmd(db_path: &Path, query: &str, top_k: usize) -> anyhow::Result<()> {
    use rag_core::Retriever;

    println!("Querying: {query:?}");
    let conn = Connection::open(db_path)
        .with_context(|| format!("Failed to open database: {}", db_path.display()))?;
    let store = RagStore::new(conn).context("Failed to initialise RAG schema")?;

    // Use hybrid (FTS + ANN) retrieval when OPENAI_API_KEY is set and embeddings exist
    let results = if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        use rag_core::Embedder;
        let embedder = rag_sqlite::embed_bridge::LlmEmbedder::new(std::sync::Arc::new(
            llm_openai::OpenAIProvider::new(api_key),
        ));
        let query_vecs: Vec<Vec<f32>> = embedder
            .embed(&[query])
            .await
            .map_err(|e| anyhow::anyhow!("query embedding failed: {e}"))?;
        let qe = query_vecs.first().map(|v: &Vec<f32>| v.as_slice());
        println!("  Using hybrid retrieval (FTS + ANN)");
        store
            .retrieve_hybrid(query, qe, top_k)
            .await
            .map_err(|e| anyhow::anyhow!("hybrid retrieval failed: {e}"))?
    } else {
        store
            .retrieve(query, top_k)
            .await
            .map_err(|e| anyhow::anyhow!("retrieval failed: {e}"))?
    };

    if results.is_empty() {
        println!("No results found.");
        return Ok(());
    }

    println!("Found {} result(s):\n", results.len());
    for (i, chunk) in results.iter().enumerate() {
        println!(
            "── [{n}] {path} (ord={ord}, score={score:.4}) ──",
            n = i + 1,
            path = chunk.source_path,
            ord = chunk.ord,
            score = chunk.score,
        );
        // Print first 300 chars of content
        let preview: String = chunk.content.chars().take(300).collect();
        println!("{preview}");
        if chunk.content.len() > 300 {
            println!("… [{} more chars]", chunk.content.len() - 300);
        }
        println!();
    }

    Ok(())
}
