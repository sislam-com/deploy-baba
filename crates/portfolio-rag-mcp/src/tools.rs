use anyhow::Result;
use serde_json::Value;
use tracing::{error, info};

use crate::rag::PortfolioRAG;

#[derive(Clone, serde::Serialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

pub fn list_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "query_rag".to_string(),
            description: Some(
                "Query the portfolio RAG system for relevant information".to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    },
                    "corpus": {
                        "type": "string",
                        "description": "Optional source_kind filter (openapi, portfolio, rust, hcl, plan, cache)"
                    }
                },
                "required": ["query"]
            }),
        },
        Tool {
            name: "list_corpora".to_string(),
            description: Some("List all available RAG corpora".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        Tool {
            name: "get_corpus_stats".to_string(),
            description: Some("Get statistics for a specific corpus".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "corpus": {
                        "type": "string",
                        "description": "The corpus name"
                    }
                },
                "required": ["corpus"]
            }),
        },
        Tool {
            name: "search_portfolio".to_string(),
            description: Some(
                "Search across all portfolio corpora with semantic ranking".to_string(),
            ),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results (default: 10)",
                        "default": 10
                    }
                },
                "required": ["query"]
            }),
        },
    ]
}

pub async fn query_rag(rag: &PortfolioRAG, args: Value) -> Result<Value> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'query' parameter"))?;

    let corpus = args.get("corpus").and_then(|v| v.as_str());

    info!("RAG query request: '{}' (corpus: {:?})", query, corpus);

    match rag.query(query, corpus).await {
        Ok(results) => {
            let response = serde_json::json!({
                "success": true,
                "query": query,
                "corpus_filter": corpus,
                "results": results,
                "result_count": results.len()
            });
            Ok(response)
        }
        Err(e) => {
            error!("RAG query failed: {}", e);
            Err(anyhow::anyhow!("RAG query failed: {}", e))
        }
    }
}

pub fn list_corpora(rag: &PortfolioRAG) -> Result<Value> {
    let corpora = rag.get_corpora();

    Ok(serde_json::json!({
        "success": true,
        "corpora": corpora,
        "corpus_count": corpora.len()
    }))
}

pub fn get_corpus_stats(rag: &PortfolioRAG, args: Value) -> Result<Value> {
    let corpus = args
        .get("corpus")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'corpus' parameter"))?;

    info!("Corpus stats request for: {}", corpus);

    match rag.get_corpus_stats(corpus) {
        Ok(stats) => Ok(serde_json::json!({
            "success": true,
            "stats": stats
        })),
        Err(e) => {
            error!("Corpus stats failed: {}", e);
            Err(anyhow::anyhow!("Corpus stats failed: {}", e))
        }
    }
}

pub async fn search_portfolio(rag: &PortfolioRAG, args: Value) -> Result<Value> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'query' parameter"))?;

    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10);

    info!("Portfolio search request: '{}' (limit: {})", query, limit);

    // For now, use the basic RAG query
    // In a full implementation, this would include semantic search with embeddings
    match rag.query(query, None).await {
        Ok(mut results) => {
            // Limit results
            results.truncate(limit as usize);

            let response = serde_json::json!({
                "success": true,
                "query": query,
                "limit": limit,
                "results": results,
                "result_count": results.len()
            });
            Ok(response)
        }
        Err(e) => {
            error!("Portfolio search failed: {}", e);
            Err(anyhow::anyhow!("Portfolio search failed: {}", e))
        }
    }
}
