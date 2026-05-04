use crate::portfolio::PortfolioDataProvider;
use crate::types::RankedChunk;
use crate::{RagError, Retriever};
use async_trait::async_trait;

const PORTFOLIO_KEYWORDS: &[&str] = &[
    "experience",
    "skills",
    "job",
    "jobs",
    "work",
    "competency",
    "competencies",
    "resume",
    "about",
    "contact",
    "social",
    "endpoint",
    "api",
    "career",
    "company",
    "position",
];

pub struct HybridRetriever<R, P> {
    pub fts: R,
    pub portfolio: P,
}

impl<R: Retriever, P: PortfolioDataProvider> HybridRetriever<R, P> {
    fn query_matches_portfolio(query: &str) -> bool {
        let lower = query.to_lowercase();
        PORTFOLIO_KEYWORDS.iter().any(|kw| lower.contains(kw))
    }

    fn value_to_chunk(val: &serde_json::Value, ord: usize) -> RankedChunk {
        let content = match serde_json::to_string_pretty(val) {
            Ok(s) => s,
            Err(_) => val.to_string(),
        };
        RankedChunk {
            chunk_id: -(ord as i64 + 1),
            document_id: -1,
            source_kind: "portfolio".to_string(),
            source_path: "live://portfolio".to_string(),
            git_sha: "live".to_string(),
            ord: ord as i64,
            content,
            score: 0.0,
        }
    }
}

#[async_trait]
impl<R: Retriever, P: PortfolioDataProvider> Retriever for HybridRetriever<R, P> {
    async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<RankedChunk>, RagError> {
        let fts_results = self.fts.retrieve(query, top_k).await?;

        let has_portfolio_chunks = fts_results
            .iter()
            .any(|c| c.source_kind == "portfolio" || c.source_kind == "openapi");

        if !has_portfolio_chunks && !Self::query_matches_portfolio(query) {
            return Ok(fts_results);
        }

        let mut live_chunks = Vec::new();
        let mut ord = 0usize;

        let jobs = self.portfolio.get_jobs_summary().await?;
        for val in &jobs {
            live_chunks.push(Self::value_to_chunk(val, ord));
            ord += 1;
        }

        let competencies = self.portfolio.get_competencies_summary().await?;
        for val in &competencies {
            live_chunks.push(Self::value_to_chunk(val, ord));
            ord += 1;
        }

        let about = self.portfolio.get_about_sections().await?;
        for val in &about {
            live_chunks.push(Self::value_to_chunk(val, ord));
            ord += 1;
        }

        let mut merged = fts_results;
        for chunk in live_chunks {
            if merged.len() >= top_k {
                break;
            }
            merged.push(chunk);
        }

        Ok(merged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubRetriever {
        chunks: Vec<RankedChunk>,
    }

    #[async_trait]
    impl Retriever for StubRetriever {
        async fn retrieve(
            &self,
            _query: &str,
            _top_k: usize,
        ) -> Result<Vec<RankedChunk>, RagError> {
            Ok(self.chunks.clone())
        }
    }

    struct StubPortfolio;

    #[async_trait]
    impl PortfolioDataProvider for StubPortfolio {
        async fn get_jobs_summary(&self) -> Result<Vec<serde_json::Value>, RagError> {
            Ok(vec![serde_json::json!({
                "company": "Acme",
                "title": "Rust Engineer",
                "slug": "acme",
            })])
        }
        async fn get_job_details(
            &self,
            _slug: &str,
        ) -> Result<Option<serde_json::Value>, RagError> {
            Ok(None)
        }
        async fn get_competencies_summary(&self) -> Result<Vec<serde_json::Value>, RagError> {
            Ok(vec![serde_json::json!({
                "name": "Cloud Infra",
                "description": "AWS, Lambda, EFS",
            })])
        }
        async fn get_about_sections(&self) -> Result<Vec<serde_json::Value>, RagError> {
            Ok(vec![serde_json::json!({
                "heading": "Bio",
                "body": "Senior engineer",
            })])
        }
    }

    fn make_fts_chunk(kind: &str, content: &str) -> RankedChunk {
        RankedChunk {
            chunk_id: 1,
            document_id: 1,
            source_kind: kind.to_string(),
            source_path: "test.rs".to_string(),
            git_sha: "abc".to_string(),
            ord: 0,
            content: content.to_string(),
            score: 5.0,
        }
    }

    #[tokio::test]
    async fn portfolio_query_injects_live_chunks() {
        let hybrid = HybridRetriever {
            fts: StubRetriever {
                chunks: vec![make_fts_chunk("rust", "fn main() {}")],
            },
            portfolio: StubPortfolio,
        };

        let results = hybrid
            .retrieve("what jobs does the owner have?", 20)
            .await
            .unwrap();
        assert!(results.len() > 1, "should have FTS + live chunks");
        assert!(
            results.iter().any(|c| c.git_sha == "live"),
            "should include live portfolio chunks"
        );
    }

    #[tokio::test]
    async fn code_query_skips_live_chunks() {
        let hybrid = HybridRetriever {
            fts: StubRetriever {
                chunks: vec![make_fts_chunk("rust", "fn main() {}")],
            },
            portfolio: StubPortfolio,
        };

        let results = hybrid
            .retrieve("how does the Lambda deploy process run?", 20)
            .await
            .unwrap();
        assert_eq!(
            results.len(),
            1,
            "pure code query should only have FTS results"
        );
        assert_eq!(results[0].source_kind, "rust");
    }

    #[tokio::test]
    async fn top_k_caps_merged_results() {
        let hybrid = HybridRetriever {
            fts: StubRetriever {
                chunks: vec![
                    make_fts_chunk("rust", "chunk 1"),
                    make_fts_chunk("rust", "chunk 2"),
                ],
            },
            portfolio: StubPortfolio,
        };

        let results = hybrid.retrieve("what experience?", 3).await.unwrap();
        assert!(results.len() <= 3, "should not exceed top_k");
    }
}
