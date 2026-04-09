use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::ApiModel;

/// A single job position.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Job {
    pub id: i64,
    pub slug: String,
    pub company: String,
    pub title: String,
    pub location: Option<String>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub summary: String,
    /// Technology names, split from comma-separated DB storage.
    pub tech_stack: Option<Vec<String>>,
    pub sort_order: i64,
}

impl ApiModel for Job {
    fn schema_name() -> &'static str {
        "Job"
    }
    fn example() -> Self {
        Self {
            id: 1,
            slug: "example-corp".to_string(),
            company: "Example Corp".to_string(),
            title: "Senior Engineer".to_string(),
            location: Some("Remote".to_string()),
            start_date: "2022-01".to_string(),
            end_date: None,
            summary: "Led platform engineering initiatives.".to_string(),
            tech_stack: Some(vec!["Rust".to_string(), "AWS".to_string()]),
            sort_order: 1,
        }
    }
}

/// A bullet-point accomplishment tied to a job.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JobDetail {
    pub id: i64,
    pub detail_text: String,
    pub category: Option<String>,
    pub sort_order: i64,
}

impl ApiModel for JobDetail {
    fn schema_name() -> &'static str {
        "JobDetail"
    }
    fn example() -> Self {
        Self {
            id: 1,
            detail_text: "Reduced deployment time by 60%.".to_string(),
            category: Some("impact".to_string()),
            sort_order: 1,
        }
    }
}

/// A job with its associated bullet-point details.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JobWithDetails {
    #[serde(flatten)]
    pub job: Job,
    pub details: Vec<JobDetail>,
}

impl ApiModel for JobWithDetails {
    fn schema_name() -> &'static str {
        "JobWithDetails"
    }
    fn example() -> Self {
        Self {
            job: Job::example(),
            details: vec![JobDetail::example()],
        }
    }
}

/// Query parameters for `GET /api/jobs`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct JobsQuery {
    /// View mode: `"chronological"` (default).
    pub view: Option<String>,
}

impl ApiModel for JobsQuery {
    fn schema_name() -> &'static str {
        "JobsQuery"
    }
    fn example() -> Self {
        Self { view: None }
    }
}

/// A competency / skill category.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Competency {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub sort_order: i64,
}

impl ApiModel for Competency {
    fn schema_name() -> &'static str {
        "Competency"
    }
    fn example() -> Self {
        Self {
            id: 1,
            slug: "cloud-infrastructure".to_string(),
            name: "Cloud Infrastructure".to_string(),
            description: "AWS and zero-cost cloud deployments.".to_string(),
            icon: Some("cloud".to_string()),
            sort_order: 1,
        }
    }
}

/// A cross-referenced evidence item linking a competency to a job detail.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EvidenceItem {
    pub id: i64,
    pub job_id: i64,
    pub job_slug: String,
    pub company: String,
    pub detail_id: Option<i64>,
    pub highlight_text: Option<String>,
    pub detail_text: Option<String>,
    pub sort_order: i64,
}

impl ApiModel for EvidenceItem {
    fn schema_name() -> &'static str {
        "EvidenceItem"
    }
    fn example() -> Self {
        Self {
            id: 1,
            job_id: 1,
            job_slug: "example-corp".to_string(),
            company: "Example Corp".to_string(),
            detail_id: Some(1),
            highlight_text: Some("60% reduction".to_string()),
            detail_text: Some("Reduced deployment time by 60%.".to_string()),
            sort_order: 1,
        }
    }
}

/// A competency with its supporting evidence items.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompetencyWithEvidence {
    #[serde(flatten)]
    pub competency: Competency,
    pub evidence: Vec<EvidenceItem>,
}

impl ApiModel for CompetencyWithEvidence {
    fn schema_name() -> &'static str {
        "CompetencyWithEvidence"
    }
    fn example() -> Self {
        Self {
            competency: Competency::example(),
            evidence: vec![EvidenceItem::example()],
        }
    }
}
