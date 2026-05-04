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
use rag_core::types::SourceKind;
use rag_sqlite::RagStore;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

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
    /// Retrieve + generate: retrieve chunks via FTS, assemble grounded prompt, call Claude.
    /// Requires ANTHROPIC_API_KEY env var.
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
    },
}

pub async fn execute(action: RagAction) -> anyhow::Result<()> {
    match action {
        RagAction::Ingest {
            db_path,
            repo_root,
            include_cache,
        } => ingest(&db_path, &repo_root, include_cache),
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
        } => ask_cmd(&db_path, &query, top_k, max_tokens).await,
    }
}

// ── Ingest ────────────────────────────────────────────────────────────────

fn ingest(db_path: &Path, repo_root: &Path, include_cache: bool) -> anyhow::Result<()> {
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
                .map(|e| e == ext)
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
            "SELECT ch.highlight_text, j.company FROM competency_highlights ch
             LEFT JOIN jobs j ON ch.job_id = j.id
             WHERE ch.competency_id = (SELECT id FROM competencies WHERE slug = ?1)
             ORDER BY ch.sort_order ASC",
        )?;
        let highlights: Vec<serde_json::Value> = hl_stmt
            .query_map([&slug], |row| {
                Ok(serde_json::json!({
                    "highlight_text": row.get::<_, String>(0)?,
                    "company": row.get::<_, Option<String>>(1)?,
                }))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        comp["highlights"] = serde_json::json!(highlights);
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

// ── Ask ───────────────────────────────────────────────────────────────────

async fn ask_cmd(db_path: &Path, query: &str, top_k: usize, max_tokens: u32) -> anyhow::Result<()> {
    use llm_anthropic::AnthropicProvider;
    use llm_core::{ChatMessage, GenerationConfig, LlmProvider, LlmRequest, MessageRole};
    use rag_core::{DefaultPromptAssembler, PromptAssembler, Retriever};

    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY env var required for `rag ask`"))?;

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

    println!("Found {} chunk(s). Calling Claude...\n", chunks.len());

    let assembler = DefaultPromptAssembler;
    let bundle = assembler.assemble(query, &chunks);

    let provider = AnthropicProvider::new(api_key);
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

    let results = store
        .retrieve(query, top_k)
        .await
        .map_err(|e| anyhow::anyhow!("retrieval failed: {e}"))?;

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
