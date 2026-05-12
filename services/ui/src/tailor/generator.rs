//! Grounded bullet generator using LLM for resume tailoring.
//!
//! Takes matched bullets from the matcher and rewrites them in a grounded manner
//! using the LLM provider. The grounding contract ensures that the generator
//! may only rephrase or reorder the whitelisted bullets, never invent skills or
//! add roles not present in the source material.

use llm_core::{
    assemble_grounded_prompt, ChatMessage, GenerationConfig, GroundingContract, LlmError,
    LlmProvider, MessageRole, RefusalPolicy,
};
use serde::{Deserialize, Serialize};

/// Request for grounded bullet generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct GenerationRequest {
    /// Job description being tailored for
    pub job_description: String,
    /// Matched bullets with their original text
    pub matched_bullets: Vec<BulletInput>,
}

/// Input bullet for generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BulletInput {
    pub job_slug: String,
    pub detail_text: String,
    pub category: Option<String>,
}

/// Generated response with rewritten bullets.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct GenerationResponse {
    /// Summary of the tailored resume section
    pub summary: String,
    /// Rewritten bullets in the same order as input
    pub rewritten_bullets: Vec<String>,
}

/// Generate grounded rewrites of matched bullets for a job description.
///
/// This function uses the LLM provider with a grounding contract that:
/// 1. Allows rephrasing of existing bullet text
/// 2. Allows reordering of bullets
/// 3. Forbids inventing new skills or roles not present in source
/// 4. Forbids adding achievements not present in source
///
/// The grounding contract is enforced at the prompt-assembly layer in llm-core.
pub async fn generate_grounded_rewrites(
    provider: &dyn LlmProvider,
    request: GenerationRequest,
) -> Result<GenerationResponse, LlmError> {
    // Build allowed source text for grounding contract
    let allowed_source_text: Vec<String> = request
        .matched_bullets
        .iter()
        .map(|b| b.detail_text.clone())
        .collect();

    let grounding = GroundingContract {
        allowed_source_text,
        refusal_policy: RefusalPolicy::WarnAndLog,
    };

    // Format the bullets for the prompt
    let bullets_text = request
        .matched_bullets
        .iter()
        .enumerate()
        .map(|(i, b)| {
            format!(
                "{}. {} ({})",
                i + 1,
                b.detail_text,
                b.category.as_deref().unwrap_or("general")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let user_prompt = format!(
        r#"Job Description:
{}

Matched Resume Bullets:
{}

Rewrite these bullets to be more compelling for this specific job while keeping them truthful:
- Maintain the core achievement/responsibility
- Emphasize aspects most relevant to the JD
- Use strong action verbs
- Keep each bullet concise (1-2 sentences)
- Maintain the same order
- Do NOT invent new skills, technologies, or achievements
- Do NOT add details not present in the original bullets

Return JSON in this exact format:
{{
  "summary": "2-3 sentence summary of the tailored section",
  "rewritten_bullets": ["rewritten bullet 1", "rewrite bullet 2", ...]
}}"#,
        request.job_description, bullets_text
    );

    let req = llm_core::LlmRequest {
        model: provider.default_model().to_string(),
        messages: vec![ChatMessage::text(MessageRole::User, user_prompt)],
        system: Some("You are a professional resume writer specializing in tailoring resumes for specific job descriptions. Always respond with valid JSON.".to_string()),
        tools: vec![],
        grounding: Some(grounding),
        config: GenerationConfig {
            max_tokens: 1000,
            temperature: 0.7,
            prompt_version: "tailor-generator-v1",
        },
    };

    let req = assemble_grounded_prompt(req)?;

    let response = provider.generate(req).await?;

    // Try to parse the response as JSON
    let parsed: GenerationResponse = serde_json::from_str(&response.content)
        .map_err(|e| LlmError::Other(format!("Failed to parse LLM response as JSON: {}", e)))?;

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_request_serialization() {
        let request = GenerationRequest {
            job_description: "Rust engineer".to_string(),
            matched_bullets: vec![BulletInput {
                job_slug: "job1".to_string(),
                detail_text: "Built API with Rust".to_string(),
                category: Some("achievement".to_string()),
            }],
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Rust engineer"));
    }

    #[test]
    fn test_generation_response_serialization() {
        let response = GenerationResponse {
            summary: "Backend engineer with Rust experience".to_string(),
            rewritten_bullets: vec!["Built scalable API".to_string()],
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Backend engineer"));
    }
}
