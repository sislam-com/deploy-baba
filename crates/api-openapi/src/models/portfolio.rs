use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{ApiModel, Challenge, Competency, Job, SocialLink};

/// Combined resume payload returned by `GET /api/resume`.
///
/// Bundles name, title, bio, jobs, competencies, and social links so the SPA home page
/// can render the full resume with a single fetch.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResumeData {
    pub name: String,
    pub title: String,
    pub bio: String,
    pub summary: String,
    pub jobs: Vec<Job>,
    pub competencies: Vec<Competency>,
    pub social_links: Vec<SocialLink>,
    pub challenges: Vec<Challenge>,
}

impl ApiModel for ResumeData {
    fn schema_name() -> &'static str {
        "ResumeData"
    }
    fn example() -> Self {
        Self {
            name: "Sharful Islam".to_string(),
            title: "AI Systems Engineer".to_string(),
            bio: "Rust engineer focused on zero-cost AWS deployments.".to_string(),
            summary: "Senior Rust engineer with 8 years of cloud-native experience.".to_string(),
            jobs: vec![Job::example()],
            competencies: vec![Competency::example()],
            social_links: vec![SocialLink::example()],
            challenges: vec![Challenge::example()],
        }
    }
}
