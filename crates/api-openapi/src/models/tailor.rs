use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ApiModel;

/// Request body for `POST /api/admin/tailor`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TailorRequest {
    /// Raw text of the target job description.
    pub job_description: String,
}

impl ApiModel for TailorRequest {
    fn schema_name() -> &'static str {
        "TailorRequest"
    }
    fn example() -> Self {
        Self {
            job_description: "We are looking for a Senior Rust Engineer with experience in \
                async systems, AWS Lambda, and distributed data pipelines."
                .to_string(),
        }
    }
}

/// A single resume bullet matched and rewritten for the target JD.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MatchedBullet {
    /// Slug of the job this bullet came from.
    pub job_slug: String,
    /// Original bullet text from the database (grounding source).
    pub detail_text: String,
    /// LLM-rewritten version, constrained by the grounding contract.
    pub rewritten_text: String,
    /// Keyword-overlap score in [0.0, 1.0].
    pub score: f32,
    /// Bullet category: `achievement`, `responsibility`, or `sub-engagement`.
    pub category: Option<String>,
}

impl ApiModel for MatchedBullet {
    fn schema_name() -> &'static str {
        "MatchedBullet"
    }
    fn example() -> Self {
        Self {
            job_slug: "example-corp".to_string(),
            detail_text: "Reduced deployment time by 60% via automated CI/CD pipeline.".to_string(),
            rewritten_text: "Accelerated release cadence 60% through a fully automated \
                CI/CD pipeline tailored to async Rust Lambda workloads."
                .to_string(),
            score: 0.87,
            category: Some("achievement".to_string()),
        }
    }
}

/// Response body for `POST /api/admin/tailor`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TailorResponse {
    /// Polished professional summary grounded in the existing bio.
    pub summary: String,
    /// Ranked, rewritten bullets most relevant to the JD.
    pub ordered_bullets: Vec<MatchedBullet>,
    /// Competency slugs that matched the JD keywords.
    pub matched_competencies: Vec<String>,
    /// S3 presigned URL for the generated DOCX file.
    pub docx_url: String,
    /// S3 presigned URL for the generated PDF file.
    pub pdf_url: String,
    /// True if result was served from `tailor_cache` without LLM calls.
    pub cache_hit: bool,
}

impl ApiModel for TailorResponse {
    fn schema_name() -> &'static str {
        "TailorResponse"
    }
    fn example() -> Self {
        Self {
            summary: "Experienced Rust engineer specialising in serverless data pipelines \
                and zero-cost AWS deployments."
                .to_string(),
            ordered_bullets: vec![MatchedBullet::example()],
            matched_competencies: vec!["async-rust".to_string(), "aws-lambda".to_string()],
            docx_url: "https://s3.amazonaws.com/bucket/resume-tailored.docx".to_string(),
            pdf_url: "https://s3.amazonaws.com/bucket/resume-tailored.pdf".to_string(),
            cache_hit: false,
        }
    }
}
