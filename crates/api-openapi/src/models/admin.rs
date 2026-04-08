use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ApiModel;

/// Input for creating or updating a job (admin).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JobInput {
    pub slug: String,
    pub company: String,
    pub title: String,
    pub location: Option<String>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub summary: String,
    /// Comma-separated list, matches DB storage format.
    pub tech_stack: Option<String>,
    pub sort_order: i64,
}

impl ApiModel for JobInput {
    fn schema_name() -> &'static str {
        "JobInput"
    }
    fn example() -> Self {
        Self {
            slug: "example-corp".to_string(),
            company: "Example Corp".to_string(),
            title: "Senior Engineer".to_string(),
            location: Some("Remote".to_string()),
            start_date: "2022-01".to_string(),
            end_date: None,
            summary: "Led platform engineering.".to_string(),
            tech_stack: Some("Rust,AWS".to_string()),
            sort_order: 1,
        }
    }
}

/// Input for creating or updating a job detail bullet (admin).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JobDetailInput {
    pub detail_text: String,
    pub category: Option<String>,
    pub sort_order: i64,
}

impl ApiModel for JobDetailInput {
    fn schema_name() -> &'static str {
        "JobDetailInput"
    }
    fn example() -> Self {
        Self {
            detail_text: "Reduced deployment time by 60%.".to_string(),
            category: Some("impact".to_string()),
            sort_order: 1,
        }
    }
}

/// Input for creating or updating a competency (admin).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompetencyInput {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub sort_order: i64,
}

impl ApiModel for CompetencyInput {
    fn schema_name() -> &'static str {
        "CompetencyInput"
    }
    fn example() -> Self {
        Self {
            slug: "cloud-infrastructure".to_string(),
            name: "Cloud Infrastructure".to_string(),
            description: "AWS and zero-cost cloud deployments.".to_string(),
            icon: Some("cloud".to_string()),
            sort_order: 1,
        }
    }
}

/// Input for creating or updating a competency evidence link (admin).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvidenceInput {
    pub competency_id: i64,
    pub job_id: i64,
    pub detail_id: Option<i64>,
    pub highlight_text: Option<String>,
    pub sort_order: i64,
}

impl ApiModel for EvidenceInput {
    fn schema_name() -> &'static str {
        "EvidenceInput"
    }
    fn example() -> Self {
        Self {
            competency_id: 1,
            job_id: 1,
            detail_id: Some(1),
            highlight_text: Some("60% reduction".to_string()),
            sort_order: 1,
        }
    }
}

/// A persisted competency evidence link returned by the API (admin).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Evidence {
    pub id: i64,
    pub competency_id: i64,
    pub job_id: i64,
    pub detail_id: Option<i64>,
    pub highlight_text: Option<String>,
    pub sort_order: i64,
}

impl ApiModel for Evidence {
    fn schema_name() -> &'static str {
        "Evidence"
    }
    fn example() -> Self {
        Self {
            id: 1,
            competency_id: 1,
            job_id: 1,
            detail_id: Some(1),
            highlight_text: Some("60% reduction".to_string()),
            sort_order: 1,
        }
    }
}
