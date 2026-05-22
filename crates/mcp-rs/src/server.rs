use crate::error::{create_error_response, validation, McpError};
use crate::protocol::*;
use crate::registry::ToolRegistry;
use crate::resource::ResourceRegistry;
use serde_json::{json, Value};
use std::io::{self, BufRead, Read, Write};
use std::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};

pub struct Server {
    tools: ToolRegistry,
    resources: ResourceRegistry,
    name: String,
    version: String,
}

impl Server {
    #[allow(dead_code)]
    pub fn new(tools: ToolRegistry, resources: ResourceRegistry) -> Self {
        Self::new_with_info(tools, resources, "mcp-rs".to_string(), "0.1.0".to_string())
    }

    pub fn new_with_info(
        tools: ToolRegistry,
        resources: ResourceRegistry,
        name: String,
        version: String,
    ) -> Self {
        Self {
            tools,
            resources,
            name,
            version,
        }
    }

    /// Process a JSON-RPC request and return the response
    pub fn handle_request(&self, request_id: &str, line: &str) -> Value {
        let start_time = std::time::Instant::now();

        // Parse the request line as JSON
        let request_value: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(e) => {
                warn!(
                    request_id = request_id,
                    error = %e,
                    "Failed to parse JSON request"
                );
                return create_error_response(
                    Value::Null,
                    &McpError::JsonError {
                        details: format!("Parse error: {}", e),
                    },
                );
            }
        };

        // Validate JSON-RPC structure
        if let Err(e) = validation::validate_jsonrpc_request(&request_value) {
            warn!(
                request_id = request_id,
                error = %e,
                "Invalid JSON-RPC request structure"
            );
            return create_error_response(
                request_value.get("id").cloned().unwrap_or(Value::Null),
                &e,
            );
        }

        // Parse as a proper request
        let req: Request = match serde_json::from_value(request_value.clone()) {
            Ok(req) => req,
            Err(e) => {
                warn!(
                    request_id = request_id,
                    error = %e,
                    "Failed to deserialize request"
                );
                return create_error_response(
                    request_value.get("id").cloned().unwrap_or(Value::Null),
                    &McpError::JsonRpcError {
                        details: format!("Invalid request structure: {}", e),
                    },
                );
            }
        };

        let id = req.id.clone();
        info!(
            request_id = request_id,
            method = req.method,
            id = ?id,
            "Processing JSON-RPC request"
        );

        let response = self.process_method(&req, request_id);

        let duration = start_time.elapsed();
        info!(
            request_id = request_id,
            method = req.method,
            duration_ms = duration.as_millis(),
            "Request completed"
        );

        response
    }

    /// Process a specific JSON-RPC method
    fn process_method(&self, req: &Request, request_id: &str) -> Value {
        let id = req.id.clone();

        match req.method.as_str() {
            "initialize" => self.handle_initialize(id),
            "tools/list" => self.handle_tools_list(id),
            "tools/call" => self.handle_tools_call(id, &req.params, request_id),
            "resources/list" => self.handle_resources_list(id),
            "resources/read" => self.handle_resources_read(id, &req.params),
            _ => {
                warn!(
                    request_id = request_id,
                    method = req.method,
                    "Unknown method"
                );
                create_error_response(
                    id,
                    &McpError::JsonRpcError {
                        details: format!("Method not found: {}", req.method),
                    },
                )
            }
        }
    }

    /// Handle initialize method
    fn handle_initialize(&self, id: Value) -> Value {
        debug!("Handling initialize request");

        let result = InitializeResult {
            protocolVersion: "2024-11-05",
            capabilities: InitializeCapabilities {
                tools: json!({}),
                resources: json!({}),
            },
            serverInfo: ServerInfo {
                name: self.name.clone(),
                version: self.version.clone(),
            },
        };

        match serde_json::to_value(Response {
            jsonrpc: "2.0",
            id: id.clone(),
            result,
        }) {
            Ok(response) => response,
            Err(e) => {
                error!(error = %e, "Failed to serialize initialize response");
                create_error_response(
                    id,
                    &McpError::InternalError {
                        details: "Failed to serialize response".to_string(),
                    },
                )
            }
        }
    }

    /// Handle tools/list method
    fn handle_tools_list(&self, id: Value) -> Value {
        debug!("Handling tools/list request");

        let result = self.tools.list();

        match serde_json::to_value(Response {
            jsonrpc: "2.0",
            id: id.clone(),
            result,
        }) {
            Ok(response) => response,
            Err(e) => {
                error!(error = %e, "Failed to serialize tools/list response");
                create_error_response(
                    id,
                    &McpError::InternalError {
                        details: "Failed to serialize response".to_string(),
                    },
                )
            }
        }
    }

    /// Handle tools/call method
    fn handle_tools_call(&self, id: Value, params: &Option<Value>, request_id: &str) -> Value {
        let default_params = Value::Object(serde_json::Map::new());
        let params = params.as_ref().unwrap_or(&default_params);

        // Validate and extract tool call parameters
        let (tool_name, arguments) = match validation::validate_tool_call_params(params) {
            Ok((name, args)) => (name, args.clone()),
            Err(e) => {
                warn!(
                    request_id = request_id,
                    error = %e,
                    "Invalid tool call parameters"
                );
                return create_error_response(id, &e);
            }
        };

        debug!(
            request_id = request_id,
            tool_name = tool_name,
            "Executing tool"
        );

        // Execute the tool with audit logging
        match self
            .tools
            .call_with_audit(tool_name, arguments, request_id.to_string(), id.clone())
        {
            Ok(tool_output) => {
                // Serialize the tool output
                match serde_json::to_string(&tool_output) {
                    Ok(text) => {
                        let result = ToolCallResult {
                            content: vec![ContentItem {
                                type_: "text",
                                text,
                            }],
                        };

                        match serde_json::to_value(Response {
                            jsonrpc: "2.0",
                            id: id.clone(),
                            result,
                        }) {
                            Ok(response) => {
                                debug!(
                                    request_id = request_id,
                                    tool_name = tool_name,
                                    "Tool execution successful"
                                );
                                response
                            }
                            Err(e) => {
                                error!(
                                    request_id = request_id,
                                    tool_name = tool_name,
                                    error = %e,
                                    "Failed to serialize tool response"
                                );
                                create_error_response(
                                    id,
                                    &McpError::InternalError {
                                        details: "Failed to serialize response".to_string(),
                                    },
                                )
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            request_id = request_id,
                            tool_name = tool_name,
                            error = %e,
                            "Failed to serialize tool output"
                        );
                        create_error_response(
                            id,
                            &McpError::tool_execution_failed(
                                tool_name,
                                format!("Failed to serialize output: {}", e),
                            ),
                        )
                    }
                }
            }
            Err(tool_error) => {
                warn!(
                    request_id = request_id,
                    tool_name = tool_name,
                    error = %tool_error,
                    "Tool execution failed"
                );
                create_error_response(id, &tool_error)
            }
        }
    }

    fn handle_resources_list(&self, id: Value) -> Value {
        debug!("Handling resources/list request");

        let result = self.resources.list();

        match serde_json::to_value(Response {
            jsonrpc: "2.0",
            id: id.clone(),
            result,
        }) {
            Ok(response) => response,
            Err(e) => {
                error!(error = %e, "Failed to serialize resources/list response");
                create_error_response(
                    id,
                    &McpError::InternalError {
                        details: "Failed to serialize response".to_string(),
                    },
                )
            }
        }
    }

    fn handle_resources_read(&self, id: Value, params: &Option<Value>) -> Value {
        let default_params = Value::Object(serde_json::Map::new());
        let params = params.as_ref().unwrap_or(&default_params);

        let uri = match params.get("uri").and_then(|v| v.as_str()) {
            Some(uri) => uri,
            None => {
                return create_error_response(
                    id,
                    &McpError::InvalidArguments {
                        tool_name: "resources/read".to_string(),
                        details: "Missing 'uri' parameter".to_string(),
                    },
                );
            }
        };

        debug!(uri = uri, "Reading resource");

        match self.resources.read(uri) {
            Ok(result) => match serde_json::to_value(Response {
                jsonrpc: "2.0",
                id: id.clone(),
                result,
            }) {
                Ok(response) => response,
                Err(e) => {
                    error!(error = %e, "Failed to serialize resources/read response");
                    create_error_response(
                        id,
                        &McpError::InternalError {
                            details: "Failed to serialize response".to_string(),
                        },
                    )
                }
            },
            Err(e) => create_error_response(id, &e),
        }
    }

    pub fn run_stdio(self) {
        info!("MCP-RS server starting");

        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut request_counter = 0u64;

        for line in stdin.lock().lines() {
            let line = match line {
                Ok(line) => line,
                Err(e) => {
                    error!(error = %e, "Failed to read line from stdin");
                    continue;
                }
            };

            request_counter += 1;
            let request_id = format!("req_{}", request_counter);

            debug!(request_id = request_id, line = %line, "Received line");

            // Handle initialized notification (no response needed)
            if let Ok(notif) = serde_json::from_str::<Initialized>(&line) {
                if notif.method == "initialized" {
                    info!(request_id = request_id, "Received initialized notification");
                    continue;
                }
            }

            // Handle requests (with id field)
            let response = self.handle_request(&request_id, &line);

            // Write response
            match serde_json::to_string(&response) {
                Ok(response_str) => {
                    if let Err(e) = writeln!(stdout, "{}", response_str) {
                        error!(request_id = request_id, error = %e, "Failed to write response");
                    }
                    if let Err(e) = stdout.flush() {
                        error!(request_id = request_id, error = %e, "Failed to flush stdout");
                    }
                }
                Err(e) => {
                    error!(
                        request_id = request_id,
                        error = %e,
                        "Failed to serialize response to JSON"
                    );
                }
            }
        }

        info!("MCP-RS server shutting down");
    }

    #[allow(dead_code)]
    pub fn run(self) {
        self.run_stdio();
    }

    pub fn run_http(self, addr: &str) -> io::Result<()> {
        let listener = TcpListener::bind(addr)?;
        info!(addr = addr, "MCP-RS HTTP server starting");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    if let Err(e) = self.handle_http_stream(&mut stream) {
                        warn!(error = %e, "Failed to handle HTTP request");
                    }
                }
                Err(e) => warn!(error = %e, "Failed to accept HTTP connection"),
            }
        }

        Ok(())
    }

    fn handle_http_stream(&self, stream: &mut TcpStream) -> io::Result<()> {
        let mut buffer = Vec::new();
        let mut temp = [0_u8; 4096];
        let mut header_end = None;

        while header_end.is_none() {
            let n = stream.read(&mut temp)?;
            if n == 0 {
                break;
            }
            buffer.extend_from_slice(&temp[..n]);
            header_end = find_header_end(&buffer);
            if buffer.len() > 1024 * 1024 {
                write_http_response(stream, 413, "text/plain", b"request too large")?;
                return Ok(());
            }
        }

        let Some(header_end) = header_end else {
            write_http_response(stream, 400, "text/plain", b"invalid request")?;
            return Ok(());
        };

        let header_text = String::from_utf8_lossy(&buffer[..header_end]);
        let mut lines = header_text.lines();
        let request_line = lines.next().unwrap_or_default();
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or_default();
        let path = parts.next().unwrap_or_default();

        if method == "GET" && path == "/health" {
            let body = json!({
                "status": "ok",
                "name": self.name,
                "version": self.version,
            })
            .to_string();
            write_http_response(stream, 200, "application/json", body.as_bytes())?;
            return Ok(());
        }

        if method != "POST" || path != "/mcp" {
            write_http_response(stream, 404, "text/plain", b"not found")?;
            return Ok(());
        }

        let content_length = header_text
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                if name.eq_ignore_ascii_case("content-length") {
                    value.trim().parse::<usize>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let body_start = header_end + 4;
        while buffer.len().saturating_sub(body_start) < content_length {
            let n = stream.read(&mut temp)?;
            if n == 0 {
                break;
            }
            buffer.extend_from_slice(&temp[..n]);
        }

        let body_end = body_start.saturating_add(content_length).min(buffer.len());
        let request_body = String::from_utf8_lossy(&buffer[body_start..body_end]);
        let response = self.handle_request("http_req", &request_body);
        let response_body = response.to_string();
        write_http_response(stream, 200, "application/json", response_body.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditLogger;
    use crate::policy::Policy;
    use crate::resource::ResourceRegistry;
    use crate::tools::health::Health;
    use crate::tools::hello::Hello;

    fn test_server(policy: Policy) -> Server {
        let mut registry = ToolRegistry::new_with_audit(policy, AuditLogger::disabled());
        registry.register(Hello);
        registry.register(Health);
        Server::new_with_info(
            registry,
            ResourceRegistry::new(vec![]),
            "test-mcp".to_string(),
            "2.0.0".to_string(),
        )
    }

    #[test]
    fn test_initialize_uses_configured_server_info() {
        let server = test_server(Policy::default());
        let response = server.handle_request(
            "test",
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        );

        assert_eq!(response["result"]["serverInfo"]["name"], "test-mcp");
        assert_eq!(response["result"]["serverInfo"]["version"], "2.0.0");
        assert!(response["result"]["capabilities"]["resources"].is_object());
    }

    #[test]
    fn test_tools_list_hides_disabled_tools() {
        let mut policy = Policy::default();
        policy.enabled_tools = vec!["health".to_string()];
        let server = test_server(policy);

        let response = server.handle_request(
            "test",
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#,
        );
        let tools = response["result"]["tools"].as_array().unwrap();

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "health");
    }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn write_http_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        413 => "Payload Too Large",
        _ => "Internal Server Error",
    };
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        reason,
        content_type,
        body.len()
    )?;
    stream.write_all(body)
}
