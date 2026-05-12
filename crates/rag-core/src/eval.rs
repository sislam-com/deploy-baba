//! Deterministic groundedness scoring for RAG answers.
//!
//! Measures what fraction of answer sentences carry a `[source N]` citation,
//! per the ADR-016 grounding contract.

use regex::Regex;
use std::sync::LazyLock;

static SOURCE_REF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[source\s+\d+\]").expect("valid regex"));

/// Split text into sentences on `.`, `!`, `?` followed by whitespace or end.
fn split_sentences(text: &str) -> Vec<&str> {
    static SENTENCE_END: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"[.!?]\s+").expect("valid regex"));

    SENTENCE_END
        .split(text)
        .map(|s| s.trim())
        .filter(|s| s.len() > 10)
        .collect()
}

/// Fraction of answer sentences containing a `[source N]` citation (0.0–1.0).
///
/// Returns 1.0 for empty answers (nothing to ground).
pub fn score_groundedness(answer: &str) -> f32 {
    let sentences = split_sentences(answer);
    if sentences.is_empty() {
        return 1.0;
    }
    let cited = sentences.iter().filter(|s| SOURCE_REF.is_match(s)).count();
    cited as f32 / sentences.len() as f32
}

/// Check whether cited source numbers exist within a given chunk count.
///
/// Returns `(valid_count, invalid_refs)` where `invalid_refs` lists source
/// numbers that exceed `chunk_count` (e.g. `[source 99]` with only 5 chunks).
pub fn verify_citation_refs(answer: &str, chunk_count: usize) -> (usize, Vec<usize>) {
    static SOURCE_NUM: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\[source\s+(\d+)\]").expect("valid regex"));

    let mut valid = 0;
    let mut invalid = Vec::new();

    for cap in SOURCE_NUM.captures_iter(answer) {
        if let Ok(n) = cap[1].parse::<usize>() {
            if n >= 1 && n <= chunk_count {
                valid += 1;
            } else {
                invalid.push(n);
            }
        }
    }
    (valid, invalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fully_cited_answer() {
        let answer = "SQLite is used for storage [source 1]. \
                       ADR-002 forbids PostgreSQL [source 2]. \
                       The database lives on EFS [source 1].";
        let score = score_groundedness(answer);
        assert!((score - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn partially_cited_answer() {
        let answer = "SQLite is the database [source 1]. \
                       It runs on Lambda. \
                       EFS provides persistent storage [source 3].";
        let score = score_groundedness(answer);
        assert!(score > 0.5 && score < 1.0);
    }

    #[test]
    fn uncited_answer() {
        let answer = "The project uses Rust and AWS. \
                       It deploys to Lambda via a zip archive.";
        let score = score_groundedness(answer);
        assert!(score < f32::EPSILON);
    }

    #[test]
    fn empty_answer() {
        assert!((score_groundedness("") - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn verify_valid_refs() {
        let answer = "Uses SQLite [source 1] and Lambda [source 3].";
        let (valid, invalid) = verify_citation_refs(answer, 5);
        assert_eq!(valid, 2);
        assert!(invalid.is_empty());
    }

    #[test]
    fn verify_catches_invalid_refs() {
        let answer = "Real source [source 2]. Hallucinated [source 99].";
        let (valid, invalid) = verify_citation_refs(answer, 5);
        assert_eq!(valid, 1);
        assert_eq!(invalid, vec![99]);
    }
}
