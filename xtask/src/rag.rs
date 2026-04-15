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
        };
        println!("  Indexing {label}...");
        let (docs, chunks) = index_corpus(&store, repo_root, dirs, ext, kind, &git_sha)?;
        println!("    {docs} files, {chunks} chunks");
        total_docs += docs;
        total_chunks += chunks;
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
fn walk_dir(
    dir: &Path,
    ext: &str,
    mut f: impl FnMut(&Path) -> anyhow::Result<()>,
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
            walk_dir(&path, ext, &mut f)?;
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
