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
            "typescript".to_string(),
            "python".to_string(),
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
                        let truncated_content: String = c.content.chars().take(max).collect();
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

        let (chunk_count, avg_tokens, oldest, newest): (i64, f64, String, String) = stmt
            .query_row([corpus], |row| {
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
        let workspace_root = std::env::var("RAG_CORPORA_PATH").unwrap_or_else(|_| ".".to_string());
        let root = Path::new(&workspace_root);

        let plan_coverage = self.compute_plan_coverage(root);
        let drift_items = self.count_open_drift(root);
        let cache_age = self.compute_cache_age(root);
        let total_chunks = {
            let conn = self.db_conn.lock().unwrap();
            conn.query_row("SELECT COUNT(*) FROM rag_chunks", [], |row| {
                row.get::<_, i64>(0)
            })
            .unwrap_or(0)
        };
        let total_docs = {
            let conn = self.db_conn.lock().unwrap();
            conn.query_row("SELECT COUNT(*) FROM rag_documents", [], |row| {
                row.get::<_, i64>(0)
            })
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

    pub fn eval_report(&self) -> Result<Value> {
        let conn = self.db_conn.lock().unwrap();

        let run = conn.query_row(
            "SELECT total_cases, pass_count, avg_groundedness, avg_correctness, run_at
             FROM rag_eval_runs ORDER BY id DESC LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<f64>>(2)?,
                    row.get::<_, Option<f64>>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        );

        let (total, passed, groundedness, correctness, run_at) = match run {
            Ok(r) => r,
            Err(_) => return Ok(serde_json::json!({"status": "no eval runs yet"})),
        };

        let pass_rate = if total > 0 {
            (passed as f64 / total as f64 * 100.0).round()
        } else {
            0.0
        };

        let mut category_stmt = conn.prepare(
            "SELECT category, COUNT(*) as total,
                    SUM(CASE WHEN passed = 1 THEN 1 ELSE 0 END) as pass_count
             FROM rag_eval_results
             WHERE run_id = (SELECT MAX(id) FROM rag_eval_runs)
             GROUP BY category",
        )?;

        let categories: Vec<Value> = category_stmt
            .query_map([], |row| {
                let cat: String = row.get(0)?;
                let cat_total: i64 = row.get(1)?;
                let cat_passed: i64 = row.get(2)?;
                let cat_rate = if cat_total > 0 {
                    (cat_passed as f64 / cat_total as f64 * 100.0).round()
                } else {
                    0.0
                };
                Ok(serde_json::json!({
                    "category": cat,
                    "total": cat_total,
                    "passed": cat_passed,
                    "pass_rate_pct": cat_rate,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(serde_json::json!({
            "last_run": run_at,
            "total_cases": total,
            "pass_count": passed,
            "pass_rate_pct": pass_rate,
            "avg_groundedness": groundedness,
            "avg_correctness": correctness,
            "categories": categories,
        }))
    }

    pub fn eval_failures(&self) -> Result<Value> {
        let conn = self.db_conn.lock().unwrap();

        let latest_run_id: Option<i64> = conn
            .query_row("SELECT MAX(id) FROM rag_eval_runs", [], |row| row.get(0))
            .unwrap_or(None);

        let run_id = match latest_run_id {
            Some(id) => id,
            None => return Ok(serde_json::json!({"status": "no eval runs yet", "failures": []})),
        };

        let mut stmt = conn.prepare(
            "SELECT er.category, ec.question, er.answer, er.groundedness_score,
                    er.correctness_score, ec.expected_hit, ec.source_path
             FROM rag_eval_results er
             JOIN rag_eval_cases ec ON er.case_id = ec.id
             WHERE er.run_id = ?1 AND er.passed = 0",
        )?;

        let failures: Vec<Value> = stmt
            .query_map([run_id], |row| {
                Ok(serde_json::json!({
                    "category": row.get::<_, String>(0)?,
                    "question": row.get::<_, String>(1)?,
                    "answer": row.get::<_, Option<String>>(2)?,
                    "groundedness_score": row.get::<_, Option<f64>>(3)?,
                    "correctness_score": row.get::<_, Option<f64>>(4)?,
                    "expected_hit": row.get::<_, Option<String>>(5)?,
                    "source_path": row.get::<_, Option<String>>(6)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(serde_json::json!({
            "run_id": run_id,
            "failure_count": failures.len(),
            "failures": failures,
        }))
    }

    pub fn corpus_gaps(&self) -> Result<Value> {
        let workspace_root = std::env::var("RAG_CORPORA_PATH").unwrap_or_else(|_| ".".to_string());
        let root = Path::new(&workspace_root);

        let indexed_docs = {
            let conn = self.db_conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT source_kind, COUNT(*), GROUP_CONCAT(DISTINCT source_path)
                 FROM rag_documents GROUP BY source_kind",
            )?;
            let results: Vec<_> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();
            results
        };

        let corpus_dirs: &[(&str, &[&str], &[&str])] = &[
            (
                "rust",
                &["crates", "services/ui/src", "services/email/src"],
                &["rs"],
            ),
            ("hcl", &["infra"], &["tf"]),
            ("plan", &["plans"], &["md"]),
            ("typescript", &["web/src"], &["ts", "tsx"]),
            ("openapi", &["crates/api-openapi"], &["rs"]),
            ("python", &["services/agent/src"], &["py"]),
        ];

        let mut gaps = Vec::new();
        for (corpus, dirs, extensions) in corpus_dirs {
            let mut fs_count = 0u64;
            for dir in *dirs {
                let full = root.join(dir);
                if full.exists() {
                    fs_count += count_files_recursive(&full, extensions);
                }
            }

            let indexed_count = indexed_docs
                .iter()
                .find(|(k, _, _)| k == corpus)
                .map(|(_, c, _)| *c as u64)
                .unwrap_or(0);

            if fs_count > indexed_count {
                gaps.push(serde_json::json!({
                    "corpus": corpus,
                    "filesystem_files": fs_count,
                    "indexed_documents": indexed_count,
                    "gap": fs_count - indexed_count,
                    "directories": dirs,
                }));
            }
        }

        let indexed_summary: Vec<Value> = indexed_docs
            .iter()
            .map(|(kind, count, _)| {
                serde_json::json!({
                    "corpus": kind,
                    "indexed_documents": count,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "indexed_corpora": indexed_summary,
            "gaps": gaps,
            "gap_count": gaps.len(),
        }))
    }

    pub fn reindex_status(&self) -> Result<Value> {
        let conn = self.db_conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT source_kind,
                    COUNT(*) as doc_count,
                    (SELECT COUNT(*) FROM rag_chunks rc
                     JOIN rag_documents rd2 ON rc.document_id = rd2.id
                     WHERE rd2.source_kind = rd.source_kind) as chunk_count,
                    MIN(updated_at) as oldest,
                    MAX(updated_at) as newest
             FROM rag_documents rd
             GROUP BY source_kind",
        )?;

        let corpora: Vec<Value> = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "corpus": row.get::<_, String>(0)?,
                    "document_count": row.get::<_, i64>(1)?,
                    "chunk_count": row.get::<_, i64>(2)?,
                    "oldest_update": row.get::<_, String>(3)?,
                    "newest_update": row.get::<_, String>(4)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(serde_json::json!({
            "corpora": corpora,
            "corpus_count": corpora.len(),
        }))
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

fn count_files_recursive(dir: &Path, extensions: &[&str]) -> u64 {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "node_modules" && name != "target" {
                    count += count_files_recursive(&path, extensions);
                }
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext) {
                    count += 1;
                }
            }
        }
    }
    count
}
