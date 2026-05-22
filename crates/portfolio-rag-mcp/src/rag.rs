use anyhow::Result;
use rag_sqlite::RagStore;
use rusqlite::Connection;
use serde_json::Value;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::info;

pub struct PortfolioRAG {
    rag_store: Arc<RagStore>,
    db_conn: Arc<Mutex<Connection>>,
    corpora: Vec<String>,
}

impl PortfolioRAG {
    pub fn new() -> Result<Self> {
        let db_path =
            std::env::var("DATABASE_PATH").unwrap_or_else(|_| "deploy-baba.db".to_string());

        info!("Initializing Portfolio RAG with database: {}", db_path);

        let conn = Connection::open(&db_path)?;
        let rag_store = Arc::new(RagStore::new(conn)?);

        let stats_conn = Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        let corpora = vec![
            "openapi".to_string(),
            "portfolio".to_string(),
            "rust".to_string(),
            "hcl".to_string(),
            "plan".to_string(),
            "cache".to_string(),
            "challenge".to_string(),
        ];

        info!("Portfolio RAG initialized with {} corpora", corpora.len());

        Ok(Self {
            rag_store,
            db_conn: Arc::new(Mutex::new(stats_conn)),
            corpora,
        })
    }

    pub async fn query(
        &self,
        query: &str,
        corpus_filter: Option<&str>,
        top_k: usize,
        max_content_len: Option<usize>,
    ) -> Result<Vec<Value>> {
        info!(
            "Querying RAG: '{}' (corpus: {:?}, top_k: {}, max_content: {:?})",
            query, corpus_filter, top_k, max_content_len
        );

        let kinds: Option<Vec<&str>> = corpus_filter.map(|c| vec![c]);
        let chunks = self
            .rag_store
            .retrieve_filtered(query, top_k, kinds.as_deref())
            .map_err(|e| anyhow::anyhow!("RAG retrieval failed: {}", e))?;

        let results = chunks
            .into_iter()
            .map(|c| {
                let (content, truncated) = match max_content_len {
                    Some(max) if c.content.len() > max => {
                        let truncated_content: String =
                            c.content.chars().take(max).collect();
                        (truncated_content, true)
                    }
                    _ => (c.content, false),
                };
                serde_json::json!({
                    "id": c.chunk_id,
                    "corpus": c.source_kind,
                    "source_path": c.source_path,
                    "content": content,
                    "score": c.score,
                    "truncated": truncated,
                })
            })
            .collect::<Vec<_>>();

        info!("RAG query returned {} results", results.len());
        Ok(results)
    }

    pub fn get_corpora(&self) -> Vec<String> {
        self.corpora.clone()
    }

    pub fn get_corpus_stats(&self, corpus: &str) -> Result<Value> {
        let conn = self.db_conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT COUNT(rc.id), COALESCE(AVG(rc.token_count), 0),
                    COALESCE(MIN(rd.updated_at), ''), COALESCE(MAX(rd.updated_at), '')
             FROM rag_chunks rc
             JOIN rag_documents rd ON rc.document_id = rd.id
             WHERE rd.source_kind = ?1",
        )?;

        let (chunk_count, avg_tokens, oldest, newest): (i64, f64, String, String) =
            stmt.query_row([corpus], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?;

        let doc_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM rag_documents WHERE source_kind = ?1",
            [corpus],
            |row| row.get(0),
        )?;

        Ok(serde_json::json!({
            "corpus": corpus,
            "document_count": doc_count,
            "chunk_count": chunk_count,
            "avg_token_count": (avg_tokens * 10.0).round() / 10.0,
            "oldest_update": oldest,
            "newest_update": newest,
        }))
    }

    pub fn project_health(&self) -> Result<Value> {
        let workspace_root =
            std::env::var("RAG_CORPORA_PATH").unwrap_or_else(|_| ".".to_string());
        let root = Path::new(&workspace_root);

        let plan_coverage = self.compute_plan_coverage(root);
        let drift_items = self.count_open_drift(root);
        let cache_age = self.compute_cache_age(root);
        let total_chunks = {
            let conn = self.db_conn.lock().unwrap();
            conn.query_row("SELECT COUNT(*) FROM rag_chunks", [], |row| row.get::<_, i64>(0))
                .unwrap_or(0)
        };
        let total_docs = {
            let conn = self.db_conn.lock().unwrap();
            conn.query_row("SELECT COUNT(*) FROM rag_documents", [], |row| row.get::<_, i64>(0))
                .unwrap_or(0)
        };
        let eval_score = self.latest_eval_score();

        Ok(serde_json::json!({
            "plan_coverage": plan_coverage,
            "open_drift_items": drift_items,
            "cache_age_description": cache_age,
            "rag_index": {
                "total_documents": total_docs,
                "total_chunks": total_chunks,
                "corpora_count": self.corpora.len(),
            },
            "eval": eval_score,
        }))
    }

    fn compute_plan_coverage(&self, root: &Path) -> Value {
        let index_path = root.join("plans/INDEX.md");
        let content = match std::fs::read_to_string(&index_path) {
            Ok(c) => c,
            Err(_) => return serde_json::json!({"error": "plans/INDEX.md not found"}),
        };

        let mut total = 0u32;
        let mut done = 0u32;
        for line in content.lines() {
            if line.starts_with('|') && !line.contains("---") && !line.contains("Module") {
                total += 1;
                let upper = line.to_uppercase();
                if upper.contains("| DONE") {
                    done += 1;
                }
            }
        }

        let pct = if total > 0 {
            (done as f64 / total as f64 * 100.0).round()
        } else {
            0.0
        };

        serde_json::json!({
            "total_modules": total,
            "done_modules": done,
            "percentage": pct,
        })
    }

    fn count_open_drift(&self, root: &Path) -> Value {
        let drift_dir = root.join("plans/drift");
        let mut total = 0u32;
        let mut resolved = 0u32;

        if let Ok(entries) = std::fs::read_dir(&drift_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "md") {
                    total += 1;
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let upper = content.to_uppercase();
                        if upper.contains("RESOLVED") && !upper.contains("PARTIALLY RESOLVED") {
                            resolved += 1;
                        }
                    }
                }
            }
        }

        serde_json::json!({
            "total_drift_logs": total,
            "resolved": resolved,
            "open": total - resolved,
        })
    }

    fn latest_eval_score(&self) -> Value {
        let conn = self.db_conn.lock().unwrap();

        let result = conn.query_row(
            "SELECT total_cases, pass_count, avg_groundedness, avg_correctness, run_at
             FROM rag_eval_runs ORDER BY id DESC LIMIT 1",
            [],
            |row| {
                let total: i64 = row.get(0)?;
                let passed: i64 = row.get(1)?;
                let groundedness: Option<f64> = row.get(2)?;
                let correctness: Option<f64> = row.get(3)?;
                let run_at: String = row.get(4)?;
                let pass_rate = if total > 0 {
                    (passed as f64 / total as f64 * 100.0).round()
                } else {
                    0.0
                };
                Ok(serde_json::json!({
                    "last_run": run_at,
                    "total_cases": total,
                    "pass_count": passed,
                    "pass_rate_pct": pass_rate,
                    "avg_groundedness": groundedness,
                    "avg_correctness": correctness,
                }))
            },
        );

        match result {
            Ok(val) => val,
            Err(_) => serde_json::json!({"status": "no eval runs yet"}),
        }
    }

    fn compute_cache_age(&self, root: &Path) -> String {
        let cache_path = root.join(".agent-cache/index.json");
        match std::fs::metadata(&cache_path) {
            Ok(meta) => match meta.modified() {
                Ok(modified) => {
                    let age = std::time::SystemTime::now()
                        .duration_since(modified)
                        .unwrap_or_default();
                    let hours = age.as_secs() / 3600;
                    if hours < 1 {
                        format!("{} minutes ago", age.as_secs() / 60)
                    } else if hours < 24 {
                        format!("{} hours ago", hours)
                    } else {
                        format!("{} days ago", hours / 24)
                    }
                }
                Err(_) => "unknown".to_string(),
            },
            Err(_) => "cache not found".to_string(),
        }
    }
}
