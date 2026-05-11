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
            featured: true,
            sort_order: 0,
        }
    }
}
