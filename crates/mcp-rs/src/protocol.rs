use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct Request {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct Initialized {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub method: String,
    #[allow(dead_code)]
    #[serde(default)]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct Response<T> {
    pub jsonrpc: &'static str,
    pub id: Value,
    pub result: T,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct InitializeResult {
    pub protocolVersion: &'static str,
    pub capabilities: InitializeCapabilities,
    pub serverInfo: ServerInfo,
}

#[derive(Debug, Serialize)]
pub struct InitializeCapabilities {
    pub tools: Value,
    pub resources: Value,
}

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct ToolCallResult {
    pub content: Vec<ContentItem>,
}

#[derive(Debug, Serialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub type_: &'static str,
    pub text: String,
}
