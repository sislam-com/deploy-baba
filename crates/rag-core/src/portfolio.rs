use crate::RagError;
use async_trait::async_trait;

#[async_trait]
pub trait PortfolioDataProvider: Send + Sync {
    async fn get_jobs_summary(&self) -> Result<Vec<serde_json::Value>, RagError>;
    async fn get_job_details(&self, slug: &str) -> Result<Option<serde_json::Value>, RagError>;
    async fn get_competencies_summary(&self) -> Result<Vec<serde_json::Value>, RagError>;
    async fn get_about_sections(&self) -> Result<Vec<serde_json::Value>, RagError>;
}
