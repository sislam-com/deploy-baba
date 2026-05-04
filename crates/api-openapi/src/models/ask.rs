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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ask_request_default_top_k() {
        // trigger default_top_k via serde deserialization with missing field
        let req: AskRequest = serde_json::from_str(r#"{"query": "how does auth work?"}"#).unwrap();
        assert_eq!(req.top_k, 10);
    }
}
