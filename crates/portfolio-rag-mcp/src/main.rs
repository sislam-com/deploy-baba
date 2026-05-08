use anyhow::Result;
use serde_json::Value;
use std::io::{self, Write};
use tracing::{error, info};

mod rag;
mod tools;

use rag::PortfolioRAG;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting Portfolio RAG MCP Server");

    // Initialize RAG system
    let rag = PortfolioRAG::new()?;

    info!("Portfolio RAG MCP Server ready");

    // Simple MCP server loop
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        // Read JSON-RPC request from stdin
        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                // Parse and handle request
                match handle_request(&rag, &line) {
                    Ok(response) => {
                        writeln!(stdout, "{}", response)?;
                        stdout.flush()?;
                    }
                    Err(e) => {
                        error!("Error handling request: {}", e);
                        let error_response = serde_json::json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": -32603,
                                "message": e.to_string()
                            },
                            "id": null
                        });
                        writeln!(stdout, "{}", error_response)?;
                        stdout.flush()?;
                    }
                }
            }
            Err(e) => {
                error!("Error reading from stdin: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn handle_request(rag: &PortfolioRAG, request: &str) -> Result<String> {
    let request: Value = serde_json::from_str(request)?;

    let method = request
        .get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing method"))?;

    let id = request.get("id").cloned();

    match method {
        "initialize" => {
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "Portfolio RAG MCP Server",
                        "version": "0.1.0"
                    }
                },
                "id": id
            });
            Ok(serde_json::to_string(&response)?)
        }
        "tools/list" => {
            let tools = tools::list_tools();
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": {
                    "tools": tools
                },
                "id": id
            });
            Ok(serde_json::to_string(&response)?)
        }
        "tools/call" => {
            let params = request
                .get("params")
                .ok_or_else(|| anyhow::anyhow!("Missing params"))?;

            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;

            let arguments = params.get("arguments").cloned();

            let result = match name {
                "query_rag" => {
                    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                    tools::query_rag(rag, args)?
                }
                "list_corpora" => tools::list_corpora(rag)?,
                "get_corpus_stats" => {
                    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                    tools::get_corpus_stats(rag, args)?
                }
                "search_portfolio" => {
                    let args = arguments.ok_or_else(|| anyhow::anyhow!("Missing arguments"))?;
                    tools::search_portfolio(rag, args)?
                }
                _ => {
                    return Err(anyhow::anyhow!("Unknown tool: {}", name));
                }
            };

            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": result,
                "id": id
            });
            Ok(serde_json::to_string(&response)?)
        }
        _ => Err(anyhow::anyhow!("Unknown method: {}", method)),
    }
}
