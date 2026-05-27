//! Deterministic groundedness scoring and response validation for RAG answers.
//!
//! Measures what fraction of answer sentences carry a `[source N]` citation,
//! per the ADR-016 grounding contract. The [`ResponseValidator`] combines
//! groundedness scoring with citation validity checks to produce an actionable
//! [`ValidationVerdict`].

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

/// What the validator recommends after inspecting a response.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationVerdict {
    /// Response meets quality thresholds — return it to the user.
    Accept,
    /// Response is below thresholds — retry with a more capable model and
    /// stricter prompting.
    RetryWithUpgrade {
        groundedness: f32,
        invalid_refs: Vec<usize>,
    },
    /// Response failed validation even after retry — return a structured
    /// "insufficient context" fallback instead.
    Reject {
        groundedness: f32,
        invalid_refs: Vec<usize>,
    },
}

/// Tunable thresholds for response validation.
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Minimum groundedness score (0.0–1.0) to accept a response.
    pub min_groundedness: f32,
    /// Maximum number of invalid citation refs before rejecting.
    pub max_invalid_refs: usize,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            min_groundedness: 0.5,
            max_invalid_refs: 0,
        }
    }
}

/// Validates LLM responses for groundedness and citation accuracy.
///
/// Combines `score_groundedness` + `verify_citation_refs` into an actionable
/// verdict that the caller can use to decide whether to accept, retry, or
/// reject the response.
pub struct ResponseValidator {
    pub config: ValidatorConfig,
}

impl ResponseValidator {
    pub fn new(config: ValidatorConfig) -> Self {
        Self { config }
    }

    /// Validate `answer` against `chunk_count` source chunks.
    ///
    /// `is_retry` indicates whether this is already a retry attempt — if so,
    /// a failing score produces `Reject` instead of `RetryWithUpgrade`.
    pub fn validate(&self, answer: &str, chunk_count: usize, is_retry: bool) -> ValidationVerdict {
        let groundedness = score_groundedness(answer);
        let (_, invalid_refs) = verify_citation_refs(answer, chunk_count);

        let passes_groundedness = groundedness >= self.config.min_groundedness;
        let passes_citations = invalid_refs.len() <= self.config.max_invalid_refs;

        if passes_groundedness && passes_citations {
            return ValidationVerdict::Accept;
        }

        if is_retry {
            ValidationVerdict::Reject {
                groundedness,
                invalid_refs,
            }
        } else {
            ValidationVerdict::RetryWithUpgrade {
                groundedness,
                invalid_refs,
            }
        }
    }
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

    // ── ResponseValidator tests ─────────────────────────────────────────

    #[test]
    fn validator_accepts_well_grounded_answer() {
        let v = ResponseValidator::new(ValidatorConfig::default());
        let answer = "SQLite is used [source 1]. ADR-002 forbids PostgreSQL [source 2].";
        assert_eq!(v.validate(answer, 3, false), ValidationVerdict::Accept);
    }

    #[test]
    fn validator_retries_poorly_grounded_first_attempt() {
        let v = ResponseValidator::new(ValidatorConfig::default());
        let answer = "The project uses Rust and AWS. It deploys to Lambda.";
        match v.validate(answer, 3, false) {
            ValidationVerdict::RetryWithUpgrade { groundedness, .. } => {
                assert!(groundedness < 0.5);
            }
            other => panic!("expected RetryWithUpgrade, got {:?}", other),
        }
    }

    #[test]
    fn validator_rejects_on_retry_failure() {
        let v = ResponseValidator::new(ValidatorConfig::default());
        let answer = "The project uses Rust and AWS. It deploys to Lambda.";
        match v.validate(answer, 3, true) {
            ValidationVerdict::Reject { groundedness, .. } => {
                assert!(groundedness < 0.5);
            }
            other => panic!("expected Reject, got {:?}", other),
        }
    }

    #[test]
    fn validator_rejects_invalid_citation_refs() {
        let v = ResponseValidator::new(ValidatorConfig::default());
        let answer = "SQLite is used [source 1]. Also [source 99].";
        match v.validate(answer, 3, false) {
            ValidationVerdict::RetryWithUpgrade { invalid_refs, .. } => {
                assert!(invalid_refs.contains(&99));
            }
            other => panic!("expected RetryWithUpgrade, got {:?}", other),
        }
    }

    #[test]
    fn validator_custom_thresholds() {
        let v = ResponseValidator::new(ValidatorConfig {
            min_groundedness: 0.8,
            max_invalid_refs: 1,
        });
        // 2/3 sentences cited = 0.67, below 0.8 threshold
        let answer = "SQLite is used [source 1]. Lambda runs it. EFS stores data [source 2].";
        assert!(matches!(
            v.validate(answer, 3, false),
            ValidationVerdict::RetryWithUpgrade { .. }
        ));
    }
}
