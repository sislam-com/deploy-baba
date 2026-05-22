use anyhow::Result;
use rag_sqlite::RagStore;
use rusqlite::Connection;
use serde_json::Value;
use std::sync::Arc;
use tracing::info;

pub struct PortfolioRAG {
    rag_store: Arc<RagStore>,
    corpora: Vec<String>,
}

impl PortfolioRAG {
    pub fn new() -> Result<Self> {
        // Get database path from environment
        let db_path =
            std::env::var("DATABASE_PATH").unwrap_or_else(|_| "deploy-baba.db".to_string());

        let corpora_path = std::env::var("RAG_CORPORA_PATH").unwrap_or_else(|_| ".".to_string());

        info!("Initializing Portfolio RAG with database: {}", db_path);

        // Connect to SQLite database and create RAG store
        let conn = Connection::open(&db_path)?;
        let rag_store = Arc::new(RagStore::new(conn)?);

        // Initialize RAG system
        let mut rag = Self {
            rag_store,
            corpora: Vec::new(),
        };

        // Load corpora information
        rag.load_corpora(&corpora_path)?;

        info!(
            "Portfolio RAG initialized with {} corpora",
            rag.corpora.len()
        );
        Ok(rag)
    }

    fn load_corpora(&mut self, _base_path: &str) -> Result<()> {
        // Match the source_kind values stored by rag-sqlite.
        self.corpora = vec![
            "openapi".to_string(),
            "portfolio".to_string(),
            "rust".to_string(),
            "hcl".to_string(),
            "plan".to_string(),
            "cache".to_string(),
        ];

        // RagStore handles schema migration automatically
        info!("RAG schema initialized by RagStore");

        Ok(())
    }

    pub async fn query(&self, query: &str, corpus_filter: Option<&str>) -> Result<Vec<Value>> {
        info!(
            "Querying RAG: '{}' (corpus filter: {:?})",
            query, corpus_filter
        );

        let kinds: Option<Vec<&str>> = corpus_filter.map(|c| vec![c]);
        let chunks = self
            .rag_store
            .retrieve_filtered(query, 10, kinds.as_deref())
            .map_err(|e| anyhow::anyhow!("RAG retrieval failed: {}", e))?;

        let results = chunks
            .into_iter()
            .map(|c| {
                serde_json::json!({
                    "id": c.chunk_id,
                    "corpus": c.source_kind,
                    "source_path": c.source_path,
                    "content": c.content,
                    "score": c.score,
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
        // Return mock stats for now
        let result = serde_json::json!({
            "corpus": corpus,
            "chunk_count": 42,
            "avg_chunk_size": 512.5,
            "oldest_chunk": "2026-01-01T00:00:00Z",
            "newest_chunk": "2026-05-08T00:00:00Z",
        });

        Ok(result)
    }
}
