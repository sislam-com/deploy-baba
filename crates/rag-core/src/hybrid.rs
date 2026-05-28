use crate::chunk::portfolio::entity_to_prose;
use crate::portfolio::PortfolioDataProvider;
use crate::types::RankedChunk;
use crate::{RagError, Retriever};
use async_trait::async_trait;
use std::collections::VecDeque;

const PORTFOLIO_ENTITY_KEYWORDS: &[&str] = &[
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
    "career",
    "company",
    "position",
    "challenge",
    "challenges",
    "project",
    "projects",
];

const CODEBASE_KEYWORDS: &[&str] = &[
    "authentication",
    "auth",
    "cognito",
    "login",
    "architecture",
    "implement",
    "design",
    "infrastructure",
    "deploy",
    "lambda",
    "database",
    "sqlite",
    "endpoint",
    "api",
    "route",
    "handler",
    "middleware",
    "service",
];

pub struct HybridRetriever<R, P> {
    pub fts: R,
    pub portfolio: P,
}

#[derive(Clone, Copy)]
struct QueryIntent {
    skills: bool,
    challenges: bool,
    architecture: bool,
}

impl<R: Retriever, P: PortfolioDataProvider> HybridRetriever<R, P> {
    fn portfolio_budget_for(query: &str, top_k: usize) -> usize {
        let lower = query.to_lowercase();
        let has_entity = PORTFOLIO_ENTITY_KEYWORDS
            .iter()
            .any(|kw| lower.contains(kw));
        let has_codebase = CODEBASE_KEYWORDS.iter().any(|kw| lower.contains(kw));

        match (has_entity, has_codebase) {
            (true, false) => top_k.min(5),
            (false, true) => top_k.min(2),
            (true, true) => top_k.min(3),
            (false, false) => 0,
        }
    }

    fn classify_entity(val: &serde_json::Value) -> (String, String) {
        if let Some(entity_type) = val.get("entity_type").and_then(|v| v.as_str()) {
            let slug = val
                .get("slug")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            return (entity_type.to_string(), slug);
        }
        if val.get("company").is_some() && val.get("title").is_some() {
            return (
                "job".to_string(),
                val.get("slug")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string(),
            );
        }
        if val.get("name").is_some() && val.get("icon").is_some() {
            return (
                "competency".to_string(),
                val.get("slug")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string(),
            );
        }
        if val.get("heading").is_some() && val.get("body").is_some() {
            return (
                "about".to_string(),
                val.get("slug")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string(),
            );
        }
        ("unknown".to_string(), String::new())
    }

    fn value_to_chunk(val: &serde_json::Value, ord: usize) -> RankedChunk {
        let content = entity_to_prose(val);
        let (entity_type, slug) = Self::classify_entity(val);
        let source_path = if slug.is_empty() {
            format!("portfolio://{}", entity_type)
        } else {
            format!("portfolio://{}/{}", entity_type, slug.as_str())
        };

        RankedChunk {
            chunk_id: -(ord as i64 + 1),
            document_id: -1,
            source_kind: "portfolio".to_string(),
            source_path,
            git_sha: "live".to_string(),
            ord: ord as i64,
            content,
            score: 0.0,
        }
    }

    fn parse_intent(query: &str) -> QueryIntent {
        let lower = query.to_lowercase();
        QueryIntent {
            skills: ["skill", "skills", "competency", "competencies", "expertise"]
                .iter()
                .any(|kw| lower.contains(kw)),
            challenges: [
                "challenge",
                "challenges",
                "project",
                "projects",
                "tradeoff",
                "outcome",
            ]
            .iter()
            .any(|kw| lower.contains(kw)),
            architecture: [
                "architecture",
                "adr",
                "design",
                "infrastructure",
                "auth",
                "cognito",
            ]
            .iter()
            .any(|kw| lower.contains(kw)),
        }
    }

    fn live_mix_from_intent(intent: QueryIntent, portfolio_budget: usize) -> [usize; 4] {
        if portfolio_budget == 0 {
            return [0, 0, 0, 0];
        }
        let mut mix = if intent.skills {
            [1, 2, 1, 1]
        } else if intent.challenges {
            [1, 1, 1, 2]
        } else if intent.architecture {
            [1, 1, 2, 1]
        } else {
            [2, 1, 1, 1]
        };
        let mut total: usize = mix.iter().sum();
        while total > portfolio_budget {
            for item in &mut mix {
                if *item > 0 && total > portfolio_budget {
                    *item -= 1;
                    total -= 1;
                }
            }
        }
        while total < portfolio_budget {
            for item in &mut mix {
                if total < portfolio_budget {
                    *item += 1;
                    total += 1;
                }
            }
        }
        mix
    }

    fn pop_take(queue: &mut VecDeque<RankedChunk>, count: usize, out: &mut Vec<RankedChunk>) {
        for _ in 0..count {
            if let Some(chunk) = queue.pop_front() {
                out.push(chunk);
            } else {
                return;
            }
        }
    }

    fn gather_live_chunks(
        jobs: &[serde_json::Value],
        competencies: &[serde_json::Value],
        about: &[serde_json::Value],
        challenges: &[serde_json::Value],
    ) -> Vec<RankedChunk> {
        let mut all = Vec::new();
        let mut ord = 0usize;
        for entity in jobs {
            all.push(Self::value_to_chunk(entity, ord));
            ord += 1;
        }
        for entity in competencies {
            all.push(Self::value_to_chunk(entity, ord));
            ord += 1;
        }
        for entity in about {
            all.push(Self::value_to_chunk(entity, ord));
            ord += 1;
        }
        for entity in challenges {
            all.push(Self::value_to_chunk(entity, ord));
            ord += 1;
        }
        all
    }
}

#[async_trait]
impl<R: Retriever, P: PortfolioDataProvider> Retriever for HybridRetriever<R, P> {
    async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<RankedChunk>, RagError> {
        let portfolio_budget = Self::portfolio_budget_for(query, top_k);

        if portfolio_budget == 0 {
            return self.fts.retrieve(query, top_k).await;
        }

        let fts_budget = top_k.saturating_sub(portfolio_budget);
        let intent = Self::parse_intent(query);

        let fts_results = if fts_budget > 0 {
            self.fts.retrieve(query, fts_budget).await?
        } else {
            Vec::new()
        };

        let jobs = self.portfolio.get_jobs_summary().await?;
        let competencies = self.portfolio.get_competencies_summary().await?;
        let about = self.portfolio.get_about_sections().await?;
        let challenges = self.portfolio.get_challenges_summary().await?;

        let mut job_q: VecDeque<_> = jobs
            .iter()
            .enumerate()
            .map(|(i, v)| Self::value_to_chunk(v, i))
            .collect();
        let mut comp_q: VecDeque<_> = competencies
            .iter()
            .enumerate()
            .map(|(i, v)| Self::value_to_chunk(v, i + 100))
            .collect();
        let mut about_q: VecDeque<_> = about
            .iter()
            .enumerate()
            .map(|(i, v)| Self::value_to_chunk(v, i + 200))
            .collect();
        let mut challenge_q: VecDeque<_> = challenges
            .iter()
            .enumerate()
            .map(|(i, v)| Self::value_to_chunk(v, i + 300))
            .collect();

        let mix = Self::live_mix_from_intent(intent, portfolio_budget);
        let mut live_chunks = Vec::new();
        Self::pop_take(&mut job_q, mix[0], &mut live_chunks);
        Self::pop_take(&mut comp_q, mix[1], &mut live_chunks);
        Self::pop_take(&mut about_q, mix[2], &mut live_chunks);
        Self::pop_take(&mut challenge_q, mix[3], &mut live_chunks);

        if live_chunks.len() < portfolio_budget {
            let mut overflow = VecDeque::from(Self::gather_live_chunks(
                &jobs,
                &competencies,
                &about,
                &challenges,
            ));
            while live_chunks.len() < portfolio_budget {
                if let Some(next) = overflow.pop_front() {
                    if !live_chunks
                        .iter()
                        .any(|c| c.source_path == next.source_path)
                    {
                        live_chunks.push(next);
                    }
                } else {
                    break;
                }
            }
        }

        let mut merged = live_chunks;
        merged.extend(fts_results);
        merged.truncate(top_k);

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
                "icon": "cloud",
            })])
        }
        async fn get_about_sections(&self) -> Result<Vec<serde_json::Value>, RagError> {
            Ok(vec![serde_json::json!({
                "heading": "Bio",
                "body": "Senior engineer",
            })])
        }
        async fn get_challenges_summary(&self) -> Result<Vec<serde_json::Value>, RagError> {
            Ok(vec![serde_json::json!({
                "entity_type": "challenge",
                "title": "Test Project",
                "slug": "test-project",
                "description": "A test challenge project",
                "tech_stack": "Rust,React",
                "category": "fullstack",
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
    async fn portfolio_query_prioritizes_live_chunks() {
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
        assert!(results.len() >= 2, "should have live chunks");
        assert!(
            results.iter().any(|c| c.git_sha == "live"),
            "should include live portfolio chunks"
        );
        // Live chunks should come first
        if results.len() >= 2 {
            assert_eq!(
                results[0].git_sha, "live",
                "first result should be live data"
            );
        }
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
            .retrieve("how does the error handling macro expand?", 20)
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

    #[tokio::test]
    async fn portfolio_budget_capped_fts_always_included() {
        let fts_chunks: Vec<_> = (0..8)
            .map(|i| make_fts_chunk("rust", &format!("fts chunk {}", i)))
            .collect();
        let hybrid = HybridRetriever {
            fts: StubRetriever { chunks: fts_chunks },
            portfolio: StubPortfolio,
        };

        let results = hybrid.retrieve("what skills?", 10).await.unwrap();
        let live_count = results.iter().filter(|c| c.git_sha == "live").count();
        let fts_count = results.iter().filter(|c| c.git_sha != "live").count();
        assert!(live_count <= 5, "portfolio chunks must be capped at 5");
        assert!(fts_count > 0, "FTS chunks must always be included");
    }

    #[tokio::test]
    async fn auth_query_triggers_portfolio_injection() {
        let hybrid = HybridRetriever {
            fts: StubRetriever {
                chunks: vec![make_fts_chunk("rust", "cognito auth handler")],
            },
            portfolio: StubPortfolio,
        };

        let results = hybrid
            .retrieve("how is authentication implemented?", 10)
            .await
            .unwrap();
        assert!(
            results.iter().any(|c| c.git_sha == "live"),
            "auth query should inject portfolio context"
        );
        assert!(
            results.iter().any(|c| c.git_sha != "live"),
            "auth query should also include FTS code chunks"
        );
    }

    #[tokio::test]
    async fn challenge_entity_has_stable_source_path() {
        let chunk = HybridRetriever::<StubRetriever, StubPortfolio>::value_to_chunk(
            &serde_json::json!({
                "entity_type": "challenge",
                "slug": "rag-grounding-citation",
                "title": "RAG Grounding",
                "description": "Grounded retrieval and citations"
            }),
            0,
        );
        assert_eq!(
            chunk.source_path,
            "portfolio://challenge/rag-grounding-citation"
        );
    }

    #[tokio::test]
    async fn codebase_query_reduces_portfolio_budget() {
        let fts_chunks: Vec<_> = (0..8)
            .map(|i| make_fts_chunk("rust", &format!("auth handler chunk {}", i)))
            .collect();
        let hybrid = HybridRetriever {
            fts: StubRetriever { chunks: fts_chunks },
            portfolio: StubPortfolio,
        };

        let results = hybrid
            .retrieve("how is authentication implemented?", 10)
            .await
            .unwrap();
        let live_count = results.iter().filter(|c| c.git_sha == "live").count();
        let fts_count = results.iter().filter(|c| c.git_sha != "live").count();
        assert!(
            live_count <= 2,
            "codebase-only query should cap portfolio at 2, got {}",
            live_count
        );
        assert!(
            fts_count >= 8,
            "codebase query should get most slots for FTS code results, got {}",
            fts_count
        );
    }

    #[tokio::test]
    async fn mixed_query_gets_middle_budget() {
        let fts_chunks: Vec<_> = (0..8)
            .map(|i| make_fts_chunk("rust", &format!("chunk {}", i)))
            .collect();
        let hybrid = HybridRetriever {
            fts: StubRetriever { chunks: fts_chunks },
            portfolio: StubPortfolio,
        };

        let results = hybrid
            .retrieve("what skills relate to authentication?", 10)
            .await
            .unwrap();
        let live_count = results.iter().filter(|c| c.git_sha == "live").count();
        assert!(
            live_count <= 3,
            "mixed entity+codebase query should cap portfolio at 3, got {}",
            live_count
        );
    }

    #[tokio::test]
    async fn challenge_query_reserves_challenge_slots() {
        let hybrid = HybridRetriever {
            fts: StubRetriever {
                chunks: vec![make_fts_chunk("plan", "adr and architecture chunk")],
            },
            portfolio: StubPortfolio,
        };
        let results = hybrid
            .retrieve("tell me about your challenge project outcomes", 6)
            .await
            .unwrap();
        assert!(
            results
                .iter()
                .any(|c| c.source_path.starts_with("portfolio://challenge/")),
            "challenge intent should include challenge live chunks"
        );
    }
}
