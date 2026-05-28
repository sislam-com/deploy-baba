use axum::{extract::State, routing::get, Json, Router};
use serde_json::Value;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/eval/report", get(eval_report))
        .route("/eval/failures", get(eval_failures))
        .route("/corpus/gaps", get(corpus_gaps))
        .route("/reindex/status", get(reindex_status))
}

async fn health(State(state): State<AppState>) -> Json<Value> {
    let conn = state.db.conn.lock().unwrap();
    let total_docs: i64 = conn
        .query_row("SELECT COUNT(*) FROM rag_documents", [], |row| row.get(0))
        .unwrap_or(0);
    let total_chunks: i64 = conn
        .query_row("SELECT COUNT(*) FROM rag_chunks", [], |row| row.get(0))
        .unwrap_or(0);
    let corpus_count: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT source_kind) FROM rag_documents",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let eval = latest_eval(&conn);

    Json(serde_json::json!({
        "rag_index": {
            "total_documents": total_docs,
            "total_chunks": total_chunks,
            "corpora_count": corpus_count,
        },
        "eval": eval,
    }))
}

async fn eval_report(State(state): State<AppState>) -> Json<Value> {
    let conn = state.db.conn.lock().unwrap();

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
        Err(_) => return Json(serde_json::json!({"status": "no eval runs yet"})),
    };

    let pass_rate = if total > 0 {
        (passed as f64 / total as f64 * 100.0).round()
    } else {
        0.0
    };

    let categories = category_breakdown(&conn);

    Json(serde_json::json!({
        "last_run": run_at,
        "total_cases": total,
        "pass_count": passed,
        "pass_rate_pct": pass_rate,
        "avg_groundedness": groundedness,
        "avg_correctness": correctness,
        "categories": categories,
    }))
}

async fn eval_failures(State(state): State<AppState>) -> Json<Value> {
    let conn = state.db.conn.lock().unwrap();

    let latest_run_id: Option<i64> = conn
        .query_row("SELECT MAX(id) FROM rag_eval_runs", [], |row| row.get(0))
        .unwrap_or(None);

    let run_id = match latest_run_id {
        Some(id) => id,
        None => return Json(serde_json::json!({"status": "no eval runs yet", "failures": []})),
    };

    let mut stmt = match conn.prepare(
        "SELECT er.category, ec.question, er.answer, er.groundedness_score,
                er.correctness_score, ec.expected_hit, ec.source_path
         FROM rag_eval_results er
         JOIN rag_eval_cases ec ON er.case_id = ec.id
         WHERE er.run_id = ?1 AND er.passed = 0",
    ) {
        Ok(s) => s,
        Err(e) => return Json(serde_json::json!({"error": e.to_string()})),
    };

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
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    Json(serde_json::json!({
        "run_id": run_id,
        "failure_count": failures.len(),
        "failures": failures,
    }))
}

async fn corpus_gaps(State(state): State<AppState>) -> Json<Value> {
    let conn = state.db.conn.lock().unwrap();

    let mut stmt = match conn
        .prepare("SELECT source_kind, COUNT(*) FROM rag_documents GROUP BY source_kind")
    {
        Ok(s) => s,
        Err(e) => return Json(serde_json::json!({"error": e.to_string()})),
    };

    let indexed: Vec<Value> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "corpus": row.get::<_, String>(0)?,
                "indexed_documents": row.get::<_, i64>(1)?,
            }))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    Json(serde_json::json!({
        "indexed_corpora": indexed,
    }))
}

async fn reindex_status(State(state): State<AppState>) -> Json<Value> {
    let conn = state.db.conn.lock().unwrap();

    let mut stmt = match conn.prepare(
        "SELECT source_kind,
                COUNT(*) as doc_count,
                MIN(updated_at) as oldest,
                MAX(updated_at) as newest
         FROM rag_documents
         GROUP BY source_kind",
    ) {
        Ok(s) => s,
        Err(e) => return Json(serde_json::json!({"error": e.to_string()})),
    };

    let corpora: Vec<Value> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "corpus": row.get::<_, String>(0)?,
                "document_count": row.get::<_, i64>(1)?,
                "oldest_update": row.get::<_, String>(2)?,
                "newest_update": row.get::<_, String>(3)?,
            }))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    Json(serde_json::json!({
        "corpora": corpora,
        "corpus_count": corpora.len(),
    }))
}

fn latest_eval(conn: &rusqlite::Connection) -> Value {
    conn.query_row(
        "SELECT total_cases, pass_count, avg_groundedness, avg_correctness, run_at
         FROM rag_eval_runs ORDER BY id DESC LIMIT 1",
        [],
        |row| {
            let total: i64 = row.get(0)?;
            let passed: i64 = row.get(1)?;
            let pass_rate = if total > 0 {
                (passed as f64 / total as f64 * 100.0).round()
            } else {
                0.0
            };
            Ok(serde_json::json!({
                "last_run": row.get::<_, String>(4)?,
                "total_cases": total,
                "pass_count": passed,
                "pass_rate_pct": pass_rate,
                "avg_groundedness": row.get::<_, Option<f64>>(2)?,
                "avg_correctness": row.get::<_, Option<f64>>(3)?,
            }))
        },
    )
    .unwrap_or(serde_json::json!({"status": "no eval runs yet"}))
}

fn category_breakdown(conn: &rusqlite::Connection) -> Vec<Value> {
    let mut stmt = match conn.prepare(
        "SELECT category, COUNT(*) as total,
                SUM(CASE WHEN passed = 1 THEN 1 ELSE 0 END) as pass_count
         FROM rag_eval_results
         WHERE run_id = (SELECT MAX(id) FROM rag_eval_runs)
         GROUP BY category",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    stmt.query_map([], |row| {
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
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}
