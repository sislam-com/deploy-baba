use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ApiModel;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Challenge {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub job_id: Option<i64>,
    pub description: String,
    pub short_description: Option<String>,
    pub tech_stack: Option<Vec<String>>,
    pub category: Option<String>,
    pub url: Option<String>,
    pub image_url: Option<String>,
    pub problem: Option<String>,
    pub constraints: Option<String>,
    pub decisions: Option<String>,
    pub implementation: Option<String>,
    pub outcomes: Option<String>,
    pub metrics: Option<String>,
    pub related_job_slug: Option<String>,
    pub related_plan_module: Option<String>,
    pub related_adr: Option<String>,
    pub featured: bool,
    pub sort_order: i64,
}

impl ApiModel for Challenge {
    fn schema_name() -> &'static str {
        "Challenge"
    }
    fn example() -> Self {
        Self {
            id: 1,
            slug: "deploy-baba-portfolio".to_string(),
            title: "deploy-baba Portfolio Platform".to_string(),
            job_id: Some(1),
            description: "Full-stack portfolio platform on AWS Lambda.".to_string(),
            short_description: Some("Zero-cost Rust portfolio platform".to_string()),
            tech_stack: Some(vec![
                "Rust".to_string(),
                "Axum".to_string(),
                "React".to_string(),
            ]),
            category: Some("fullstack".to_string()),
            url: Some("https://github.com/shantopagla/deploy-baba".to_string()),
            image_url: None,
            problem: Some(
                "Portfolio and resume delivery lacked operational automation.".to_string(),
            ),
            constraints: Some("Zero recurring cost, low-latency global delivery.".to_string()),
            decisions: Some("Use Lambda + SQLite on EFS with OpenTofu-managed infra.".to_string()),
            implementation: Some("Rust/Axum backend plus React/Vite SPA and CI/CD.".to_string()),
            outcomes: Some(
                "Production portfolio with admin workflows and grounded AI Q&A.".to_string(),
            ),
            metrics: Some(
                "Deployment flow reduced to minutes with repeatable automation.".to_string(),
            ),
            related_job_slug: Some("personal-projects".to_string()),
            related_plan_module: Some("W-RAG".to_string()),
            related_adr: Some("ADR-016".to_string()),
            featured: true,
            sort_order: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChallengeInput {
    pub slug: String,
    pub title: String,
    pub job_id: Option<i64>,
    pub description: String,
    pub short_description: Option<String>,
    pub tech_stack: Option<String>,
    pub category: Option<String>,
    pub url: Option<String>,
    pub image_url: Option<String>,
    pub problem: Option<String>,
    pub constraints: Option<String>,
    pub decisions: Option<String>,
    pub implementation: Option<String>,
    pub outcomes: Option<String>,
    pub metrics: Option<String>,
    pub related_job_slug: Option<String>,
    pub related_plan_module: Option<String>,
    pub related_adr: Option<String>,
    pub featured: bool,
    pub sort_order: i64,
}

impl ApiModel for ChallengeInput {
    fn schema_name() -> &'static str {
        "ChallengeInput"
    }
    fn example() -> Self {
        Self {
            slug: "deploy-baba-portfolio".to_string(),
            title: "deploy-baba Portfolio Platform".to_string(),
            job_id: Some(1),
            description: "Full-stack portfolio platform on AWS Lambda.".to_string(),
            short_description: Some("Zero-cost Rust portfolio platform".to_string()),
            tech_stack: Some("Rust,Axum,React".to_string()),
            category: Some("fullstack".to_string()),
            url: Some("https://github.com/shantopagla/deploy-baba".to_string()),
            image_url: None,
            problem: Some(
                "Portfolio and resume delivery lacked operational automation.".to_string(),
            ),
            constraints: Some("Zero recurring cost, low-latency global delivery.".to_string()),
            decisions: Some("Use Lambda + SQLite on EFS with OpenTofu-managed infra.".to_string()),
            implementation: Some("Rust/Axum backend plus React/Vite SPA and CI/CD.".to_string()),
            outcomes: Some(
                "Production portfolio with admin workflows and grounded AI Q&A.".to_string(),
            ),
            metrics: Some(
                "Deployment flow reduced to minutes with repeatable automation.".to_string(),
            ),
            related_job_slug: Some("personal-projects".to_string()),
            related_plan_module: Some("W-RAG".to_string()),
            related_adr: Some("ADR-016".to_string()),
            featured: true,
            sort_order: 0,
        }
    }
}
