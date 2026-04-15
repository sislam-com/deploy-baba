//! Pure-Rust token-overlap keyword matcher for resume tailoring.
//!
//! Given a set of candidate resume bullets (job details, competencies,
//! tech stack entries) and a tokenised job description, scores each candidate
//! by the fraction of its unique normalised tokens that appear in the JD
//! token set. Returns candidates ranked by descending score.
//!
//! No LLM, no external I/O — fully deterministic and unit-testable.

use api_openapi::models::MatchedBullet;
use rusqlite::Connection;
use std::collections::HashSet;

// ── Stop words filtered out before scoring ────────────────────────────────

static STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "been", "by", "for", "from", "has", "have", "in",
    "is", "it", "its", "of", "on", "or", "that", "the", "their", "they", "this", "to", "was",
    "we", "were", "which", "will", "with", "you",
];

// ── Tokenisation ─────────────────────────────────────────────────────────

/// Normalise text → unique lowercase tokens with stop words removed.
///
/// Splits on whitespace and punctuation, lowercases, drops single-char tokens,
/// removes stop words, and de-duplicates the result.
pub fn tokenise(text: &str) -> HashSet<String> {
    let stop: HashSet<&str> = STOP_WORDS.iter().copied().collect();
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .filter(|t| t.len() > 1 && !stop.contains(t.as_str()))
        .collect()
}

// ── Candidate rows ────────────────────────────────────────────────────────

struct Candidate {
    job_slug: String,
    detail_text: String,
    category: Option<String>,
    tokens: HashSet<String>,
}

// ── Database loading ──────────────────────────────────────────────────────

/// Load all job details joined to their job slug.
fn load_candidates(conn: &Connection) -> rusqlite::Result<Vec<Candidate>> {
    let mut stmt = conn.prepare(
        "SELECT j.slug, jd.detail_text, jd.category
         FROM job_details jd
         JOIN jobs j ON j.id = jd.job_id
         ORDER BY j.sort_order, jd.sort_order",
    )?;

    stmt.query_map([], |row| {
        let detail_text: String = row.get(1)?;
        let tokens = tokenise(&detail_text);
        Ok(Candidate {
            job_slug: row.get(0)?,
            detail_text,
            category: row.get(2)?,
            tokens,
        })
    })
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
}

/// Load tech stack entries from all jobs as additional candidates.
fn load_tech_candidates(conn: &Connection) -> rusqlite::Result<Vec<Candidate>> {
    let mut stmt = conn.prepare(
        "SELECT j.slug, j.tech_stack FROM jobs j WHERE j.tech_stack IS NOT NULL AND j.tech_stack != ''",
    )?;

    let candidates = stmt
        .query_map([], |row| {
            let slug: String = row.get(0)?;
            let tech_raw: String = row.get(1)?;
            Ok((slug, tech_raw))
        })?
        .filter_map(|r| r.ok())
        .flat_map(|(slug, tech_raw)| {
            tech_raw
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .map(move |tech| {
                    let tokens = tokenise(&tech);
                    Candidate {
                        job_slug: slug.clone(),
                        detail_text: tech,
                        category: Some("tech".to_string()),
                        tokens,
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();

    Ok(candidates)
}

// ── Scoring ───────────────────────────────────────────────────────────────

/// Score a candidate against the JD token set.
///
/// Returns the Jaccard-style overlap: `|candidate ∩ jd| / |candidate|`.
/// A score of 1.0 means every token in the candidate appears in the JD.
/// Returns 0.0 if the candidate has no tokens.
fn overlap_score(candidate_tokens: &HashSet<String>, jd_tokens: &HashSet<String>) -> f32 {
    if candidate_tokens.is_empty() {
        return 0.0;
    }
    let intersection = candidate_tokens.intersection(jd_tokens).count();
    intersection as f32 / candidate_tokens.len() as f32
}

// ── Public API ────────────────────────────────────────────────────────────

/// Rank all resume bullets by token overlap with `jd_text`.
///
/// Queries `job_details` + `jobs.tech_stack` from `conn`, scores each row,
/// and returns up to `top_n` results sorted by descending score. Bullets with
/// zero overlap are excluded.
///
/// # Errors
///
/// Propagates any SQLite errors from the database queries.
pub fn rank_bullets(
    conn: &Connection,
    jd_text: &str,
    top_n: usize,
) -> rusqlite::Result<Vec<MatchedBullet>> {
    let jd_tokens = tokenise(jd_text);

    let mut candidates = load_candidates(conn)?;
    candidates.extend(load_tech_candidates(conn)?);

    let mut scored: Vec<(f32, Candidate)> = candidates
        .into_iter()
        .map(|c| {
            let score = overlap_score(&c.tokens, &jd_tokens);
            (score, c)
        })
        .filter(|(score, _)| *score > 0.0)
        .collect();

    // Descending sort by score, stable so equal scores stay in DB order.
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_n);

    Ok(scored
        .into_iter()
        .map(|(score, c)| MatchedBullet {
            job_slug: c.job_slug,
            detail_text: c.detail_text.clone(),
            rewritten_text: c.detail_text, // generator.rs will fill this in later
            score,
            category: c.category,
        })
        .collect())
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenise_lowercases_and_strips_stop_words() {
        let tokens = tokenise("A Rust engineer with AWS experience");
        assert!(tokens.contains("rust"));
        assert!(tokens.contains("aws"));
        assert!(tokens.contains("engineer"));
        assert!(tokens.contains("experience"));
        // stop words removed
        assert!(!tokens.contains("a"));
        assert!(!tokens.contains("with"));
    }

    #[test]
    fn tokenise_deduplicates() {
        let tokens = tokenise("rust rust Rust");
        assert_eq!(tokens.len(), 1);
    }

    #[test]
    fn tokenise_strips_punctuation() {
        let tokens = tokenise("async/await, tokio.");
        assert!(tokens.contains("async"));
        assert!(tokens.contains("await"));
        assert!(tokens.contains("tokio"));
    }

    #[test]
    fn overlap_score_full_match() {
        let candidate: HashSet<String> = ["rust", "aws"].iter().map(|s| s.to_string()).collect();
        let jd: HashSet<String> = ["rust", "aws", "lambda"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let score = overlap_score(&candidate, &jd);
        assert!((score - 1.0).abs() < f32::EPSILON, "expected 1.0, got {score}");
    }

    #[test]
    fn overlap_score_partial_match() {
        let candidate: HashSet<String> = ["rust", "python"].iter().map(|s| s.to_string()).collect();
        let jd: HashSet<String> = ["rust", "aws"].iter().map(|s| s.to_string()).collect();
        let score = overlap_score(&candidate, &jd);
        assert!((score - 0.5).abs() < f32::EPSILON, "expected 0.5, got {score}");
    }

    #[test]
    fn overlap_score_no_match() {
        let candidate: HashSet<String> = ["python"].iter().map(|s| s.to_string()).collect();
        let jd: HashSet<String> = ["rust"].iter().map(|s| s.to_string()).collect();
        assert_eq!(overlap_score(&candidate, &jd), 0.0);
    }

    #[test]
    fn overlap_score_empty_candidate() {
        let candidate: HashSet<String> = HashSet::new();
        let jd: HashSet<String> = ["rust"].iter().map(|s| s.to_string()).collect();
        assert_eq!(overlap_score(&candidate, &jd), 0.0);
    }

    #[test]
    fn rank_bullets_with_in_memory_db() {
        use rusqlite::Connection;

        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE jobs (id INTEGER PRIMARY KEY, slug TEXT, sort_order INTEGER, tech_stack TEXT);
             CREATE TABLE job_details (id INTEGER PRIMARY KEY, job_id INTEGER, detail_text TEXT, category TEXT, sort_order INTEGER);
             INSERT INTO jobs VALUES (1, 'rust-corp', 1, 'Rust,AWS');
             INSERT INTO jobs VALUES (2, 'python-corp', 2, 'Python,Django');
             INSERT INTO job_details VALUES (1, 1, 'Built async Rust microservices on AWS Lambda', 'achievement', 1);
             INSERT INTO job_details VALUES (2, 1, 'Managed CI/CD pipelines with GitHub Actions', 'responsibility', 2);
             INSERT INTO job_details VALUES (3, 2, 'Developed Django REST APIs', 'responsibility', 1);",
        )
        .unwrap();

        let jd = "We need a senior Rust engineer experienced in async systems and AWS Lambda deployments.";
        let bullets = rank_bullets(&conn, jd, 10).unwrap();

        // The Rust bullet should rank first
        assert!(!bullets.is_empty());
        assert_eq!(bullets[0].job_slug, "rust-corp");
        assert!(bullets[0].score > 0.0);

        // All returned bullets have positive score
        for b in &bullets {
            assert!(b.score > 0.0, "zero-score bullet should be filtered");
        }
    }
}
