//! Job description parser using LLM for keyword extraction.
//!
//! Extracts relevant keywords and implied skill categories from a job description
//! using the LLM provider trait. This enables intelligent matching beyond simple
//! token overlap by understanding the semantic meaning of the JD.

use llm_core::{ChatMessage, GenerationConfig, LlmError, LlmProvider, MessageRole};
use serde::{Deserialize, Serialize};

/// Extracted keywords and skill categories from a job description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedKeywords {
    /// Primary keywords from the JD (e.g., "rust", "aws", "lambda", "kubernetes")
    pub keywords: Vec<String>,
    /// Implied skill categories (e.g., "backend", "devops", "cloud", "database")
    pub categories: Vec<String>,
}

/// Parse a job description to extract keywords and categories using the LLM.
///
/// This function uses the LLM provider to analyze the JD and extract:
/// 1. Technical keywords mentioned in the description
/// 2. Implied skill categories based on the keywords and context
///
/// The prompt is designed to get structured JSON output that can be parsed
/// into the `ParsedKeywords` struct.
pub async fn parse_jd_keywords(
    provider: &dyn LlmProvider,
    jd_text: &str,
) -> Result<ParsedKeywords, LlmError> {
    let system_prompt = r#"You are a technical recruiter assistant. Analyze the job description and extract:
1. Technical keywords (programming languages, frameworks, tools, platforms)
2. Implied skill categories (e.g., backend, frontend, devops, cloud, database, ml, mobile)

Return JSON in this exact format:
{
  "keywords": ["keyword1", "keyword2", ...],
  "categories": ["category1", "category2", ...]
}

Be concise. Extract 5-10 keywords and 3-5 categories maximum."#;

    let req = llm_core::LlmRequest {
        model: provider.default_model().to_string(),
        messages: vec![
            ChatMessage::text(MessageRole::System, system_prompt.to_string()),
            ChatMessage::text(MessageRole::User, jd_text.to_string()),
        ],
        system: None,
        tools: vec![],
        grounding: None,
        config: GenerationConfig {
            max_tokens: 200,
            temperature: 0.3,
            prompt_version: "tailor-parser-v1",
        },
    };

    let response = provider.generate(req).await?;

    // Try to parse the response as JSON
    let parsed: ParsedKeywords = serde_json::from_str(&response.content)
        .map_err(|e| LlmError::Other(format!("Failed to parse LLM response as JSON: {}", e)))?;

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_keywords_struct() {
        let parsed = ParsedKeywords {
            keywords: vec!["rust".to_string(), "aws".to_string()],
            categories: vec!["backend".to_string(), "cloud".to_string()],
        };
        let json = serde_json::to_string(&parsed).unwrap();
        assert!(json.contains("rust"));
        assert!(json.contains("backend"));
    }
}
