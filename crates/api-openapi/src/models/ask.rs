use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ApiModel;

/// Request body for `POST /api/ask`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AskRequest {
    /// Natural-language question about the deploy-baba codebase.
    pub query: String,
    /// Maximum number of source chunks to retrieve (default 10, max 20).
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    10
}

impl ApiModel for AskRequest {
    fn schema_name() -> &'static str {
        "AskRequest"
    }
    fn example() -> Self {
        Self {
            query: "How does the Lambda cold-start load secrets from Secrets Manager?".to_string(),
            top_k: 10,
        }
    }
}

/// A source chunk cited in an `AskResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AskCitation {
    /// Source corpus kind: `rust`, `hcl`, `plan`, or `cache`.
    pub kind: String,
    /// Relative path within the repository.
    pub path: String,
    /// Git SHA at the time of indexing.
    pub sha: String,
    /// Ordinal position of this chunk within the source file.
    pub ord: i64,
    /// GitHub URL to the source file at the specific commit.
    pub url: String,
}

impl ApiModel for AskCitation {
    fn schema_name() -> &'static str {
        "AskCitation"
    }
    fn example() -> Self {
        Self {
            kind: "plan".to_string(),
            path: "plans/modules/secrets-manager.md".to_string(),
            sha: "abc1234".to_string(),
            ord: 3,
            url: "https://github.com/shantopagla/deploy-baba/blob/abc1234/plans/modules/secrets-manager.md".to_string(),
        }
    }
}

/// Response body for `POST /api/ask`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AskResponse {
    /// The model's grounded answer; every claim cites `[source N]`.
    pub answer: String,
    /// Source chunks used to construct the answer.
    pub citations: Vec<AskCitation>,
    /// Anthropic model ID used to generate the answer.
    pub model: String,
    /// Input token count.
    pub input_tokens: u32,
    /// Output token count.
    pub output_tokens: u32,
}

impl ApiModel for AskResponse {
    fn schema_name() -> &'static str {
        "AskResponse"
    }
    fn example() -> Self {
        Self {
            answer: "At cold start, the Lambda reads `ANTHROPIC_API_KEY_ARN` from its environment \
                and calls `secretsmanager:GetSecretValue` to load the key [source 1]. The result is \
                stored in a `OnceLock<Option<String>>` so subsequent invocations skip the SDK call \
                [source 2]."
                .to_string(),
            citations: vec![AskCitation::example()],
            model: "claude-haiku-4-5-20251001".to_string(),
            input_tokens: 512,
            output_tokens: 128,
        }
    }
}

// ── Internal Lambda-to-Lambda contract ───────────────────────────────────────
// Not part of the public OpenAPI spec. No `ApiModel` impl needed.

/// Payload sent from the UI Lambda to the LLM-proxy Lambda.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskProxyRequest {
    pub system_prompt: String,
    pub user_message: String,
    pub max_tokens: u32,
    pub temperature: f32,
    #[serde(default)]
    pub tools: Vec<serde_json::Value>,
    #[serde(default)]
    pub api_base_url: Option<String>,
    #[serde(default = "default_provider")]
    pub provider: String,
}

fn default_provider() -> String {
    "anthropic".to_string()
}

/// Response from the LLM-proxy Lambda.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskProxyResponse {
    pub content: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub tools_used: Vec<String>,
    #[serde(default)]
    pub turns: u32,
    pub provider: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ask_request_default_top_k() {
        // trigger default_top_k via serde deserialization with missing field
        let req: AskRequest = serde_json::from_str(r#"{"query": "how does auth work?"}"#).unwrap();
        assert_eq!(req.top_k, 10);
    }

    #[test]
    fn test_ask_proxy_roundtrip() {
        let req = AskProxyRequest {
            system_prompt: "You are a helpful assistant.".into(),
            user_message: "How does auth work?".into(),
            max_tokens: 1024,
            temperature: 0.2,
            tools: vec![],
            api_base_url: None,
            provider: "anthropic".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: AskProxyRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.max_tokens, 1024);
        assert!(back.tools.is_empty());
        assert_eq!(back.provider, "anthropic");

        let resp = AskProxyResponse {
            content: "Auth uses Cognito.".into(),
            model: "claude-haiku-4-5".into(),
            input_tokens: 10,
            output_tokens: 5,
            tools_used: vec![],
            turns: 1,
            provider: "anthropic".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: AskProxyResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.output_tokens, 5);
        assert_eq!(back.turns, 1);
        assert_eq!(back.provider, "anthropic");
    }
}
