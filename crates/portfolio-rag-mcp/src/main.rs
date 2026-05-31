use anyhow::Result;
use serde_json::Value;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, info, warn};

mod rag;
mod tools;

use rag::PortfolioRAG;

// Constants for security and resource management
const MAX_REQUEST_SIZE: usize = 1_048_576; // 1MB
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RESPONSE_SIZE: usize = 10_485_760; // 10MB

// JSON-RPC error codes
const PARSE_ERROR: i32 = -32700;
const INVALID_REQUEST: i32 = -32600;
const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;
const INTERNAL_ERROR: i32 = -32603;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting Portfolio RAG MCP Server");

    // Initialize RAG system
    let rag = Arc::new(PortfolioRAG::new()?);

    info!("Portfolio RAG MCP Server ready");

    // Proper MCP server loop with buffering and timeouts
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = BufReader::new(stdin);

    loop {
        // Read JSON-RPC request with size limit
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                if line.len() > MAX_REQUEST_SIZE {
                    error!("Request too large: {} bytes", line.len());
                    send_error_response(
                        &mut stdout,
                        None,
                        INVALID_REQUEST,
                        "Request too large".to_string(),
                    )?;
                    continue;
                }

                // Handle the request with timeout
                let rag_clone = Arc::clone(&rag);
                let request_id = extract_request_id(&line);

                match timeout(REQUEST_TIMEOUT, handle_request(&rag_clone, &line)).await {
                    Ok(Ok(response)) => {
                        if response.len() > MAX_RESPONSE_SIZE {
                            warn!("Response too large: {} bytes", response.len());
                            send_error_response(
                                &mut stdout,
                                request_id,
                                INTERNAL_ERROR,
                                "Response too large".to_string(),
                            )?;
                        } else {
                            writeln!(stdout, "{}", response)?;
                            stdout.flush()?;
                        }
                    }
                    Ok(Err(e)) => {
                        error!("Error handling request: {}", e);
                        send_error_response(
                            &mut stdout,
                            request_id,
                            INTERNAL_ERROR,
                            e.to_string(),
                        )?;
                    }
                    Err(_) => {
                        error!("Request processing timeout");
                        send_error_response(
                            &mut stdout,
                            request_id,
                            INTERNAL_ERROR,
                            "Processing timeout".to_string(),
                        )?;
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

async fn handle_request(rag: &PortfolioRAG, request: &str) -> Result<String> {
    // Parse and validate basic JSON-RPC structure
    let request: Value =
        serde_json::from_str(request).map_err(|e| anyhow::anyhow!("{}: {}", PARSE_ERROR, e))?;

    // Validate JSON-RPC version
    let jsonrpc_version = request
        .get("jsonrpc")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing jsonrpc version"))?;

    if jsonrpc_version != "2.0" {
        return Err(anyhow::anyhow!(
            "{}: Unsupported JSON-RPC version: {}",
            INVALID_REQUEST,
            jsonrpc_version
        ));
    }

    // Validate method
    let method = request
        .get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("{}: Missing method", INVALID_REQUEST))?;

    // Validate method name (prevent injection)
    if !method
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '/')
    {
        return Err(anyhow::anyhow!(
            "{}: Invalid method name: {}",
            METHOD_NOT_FOUND,
            method
        ));
    }

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
                .ok_or_else(|| anyhow::anyhow!("{}: Missing params", INVALID_PARAMS))?;

            // Validate tool name
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("{}: Missing tool name", INVALID_PARAMS))?;

            // Validate tool name characters
            if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return Err(anyhow::anyhow!(
                    "{}: Invalid tool name: {}",
                    METHOD_NOT_FOUND,
                    name
                ));
            }

            // Validate arguments if present
            let arguments = params.get("arguments").cloned();
            if let Some(ref args) = arguments {
                // Check arguments size
                let args_str = serde_json::to_string(args)?;
                if args_str.len() > 100_000 {
                    // 100KB limit for arguments
                    return Err(anyhow::anyhow!(
                        "{}: Arguments too large: {} bytes",
                        INVALID_PARAMS,
                        args_str.len()
                    ));
                }
            }

            let result = match name {
                "context_brief" => {
                    let args = arguments
                        .ok_or_else(|| anyhow::anyhow!("{}: Missing arguments", INVALID_PARAMS))?;
                    tools::context_brief(rag, args).await?
                }
                "query_rag" => {
                    let args = arguments
                        .ok_or_else(|| anyhow::anyhow!("{}: Missing arguments", INVALID_PARAMS))?;
                    tools::query_rag(rag, args).await?
                }
                "list_corpora" => tools::list_corpora(rag)?,
                "project_health" => tools::project_health(rag)?,
                "get_corpus_stats" => {
                    let args = arguments
                        .ok_or_else(|| anyhow::anyhow!("{}: Missing arguments", INVALID_PARAMS))?;
                    tools::get_corpus_stats(rag, args)?
                }
                "eval_report" => tools::eval_report(rag)?,
                "eval_failures" => tools::eval_failures(rag)?,
                "corpus_gaps" => tools::corpus_gaps(rag)?,
                "reindex_status" => tools::reindex_status(rag)?,
                _ => {
                    return Err(anyhow::anyhow!(
                        "{}: Unknown tool: {}",
                        METHOD_NOT_FOUND,
                        name
                    ));
                }
            };

            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": result,
                "id": id
            });
            Ok(serde_json::to_string(&response)?)
        }
        _ => Err(anyhow::anyhow!(
            "{}: Unknown method: {}",
            METHOD_NOT_FOUND,
            method
        )),
    }
}

/// Send a standardized error response
fn send_error_response(
    stdout: &mut io::Stdout,
    id: Option<Value>,
    code: i32,
    message: String,
) -> Result<()> {
    let error_response = serde_json::json!({
        "jsonrpc": "2.0",
        "error": {
            "code": code,
            "message": message
        },
        "id": id
    });
    writeln!(stdout, "{}", error_response)?;
    stdout.flush()?;
    Ok(())
}

/// Extract request ID from JSON-RPC request string
fn extract_request_id(request: &str) -> Option<Value> {
    if let Ok(parsed) = serde_json::from_str::<Value>(request) {
        parsed.get("id").cloned()
    } else {
        None
    }
}
