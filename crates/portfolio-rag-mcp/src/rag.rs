use anyhow::Result;
use rusqlite::Connection;
use serde_json::Value;
use tracing::info;

pub struct PortfolioRAG {
    db: Connection,
    corpora: Vec<String>,
}

impl PortfolioRAG {
    pub fn new() -> Result<Self> {
        // Get database path from environment
        let db_path =
            std::env::var("DATABASE_PATH").unwrap_or_else(|_| "deploy-baba.db".to_string());

        let corpora_path = std::env::var("RAG_CORPORA_PATH").unwrap_or_else(|_| ".".to_string());

        info!("Initializing Portfolio RAG with database: {}", db_path);

        // Connect to SQLite database
        let db = Connection::open(&db_path)?;

        // Initialize RAG system
        let mut rag = Self {
            db,
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
        // Define the 6 corpora from ADR-016
        self.corpora = vec![
            "openapi_spec".to_string(),
            "portfolio_data".to_string(),
            "source_code".to_string(),
            "documentation".to_string(),
            "architecture_decisions".to_string(),
            "plans".to_string(),
        ];

        // Initialize RAG tables if they don't exist
        self.initialize_rag_tables()?;

        Ok(())
    }

    fn initialize_rag_tables(&self) -> Result<()> {
        // Create RAG-related tables if they don't exist
        let queries = vec![
            "CREATE TABLE IF NOT EXISTS rag_chunks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                corpus TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT,
                embedding BLOB,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            "CREATE TABLE IF NOT EXISTS rag_search_index (
                chunk_id INTEGER,
                corpus TEXT,
                content_fts TEXT,
                FOREIGN KEY (chunk_id) REFERENCES rag_chunks (id)
            )",
            "CREATE VIRTUAL TABLE IF NOT EXISTS rag_fts USING fts5(
                corpus, 
                content, 
                content=rag_search_index, 
                content_rowid=rowid
            )",
        ];

        for query in queries {
            self.db.execute(query, [])?;
        }

        Ok(())
    }

    pub fn query(&self, query: &str, corpus_filter: Option<&str>) -> Result<Vec<Value>> {
        info!(
            "Querying RAG: '{}' (corpus filter: {:?})",
            query, corpus_filter
        );

        // For now, return mock results since the RAG system isn't fully populated
        let mock_results = vec![
            serde_json::json!({
                "id": 1,
                "corpus": "architecture_decisions",
                "content": "ADR-015: LLM Provider Abstraction + Grounding Contract",
                "metadata": "{\"adr\": \"ADR-015\", \"title\": \"LLM Provider Abstraction\"}",
                "rank": 0.95
            }),
            serde_json::json!({
                "id": 2,
                "corpus": "documentation",
                "content": "The deploy-baba project uses a zero-cost philosophy with boring infrastructure",
                "metadata": "{\"type\": \"project_overview\"}",
                "rank": 0.87
            }),
        ];

        info!("RAG query returned {} results", mock_results.len());
        Ok(mock_results)
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
