use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ApiModel;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LinkedInPosition {
    pub id: i64,
    pub linkedin_id: Option<String>,
    pub company: String,
    pub title: String,
    pub location: Option<String>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub description: Option<String>,
    pub mapped_job_id: Option<i64>,
    pub sync_status: String,
    pub imported_at: String,
    pub reviewed_at: Option<String>,
}

impl ApiModel for LinkedInPosition {
    fn schema_name() -> &'static str {
        "LinkedInPosition"
    }
    fn example() -> Self {
        Self {
            id: 1,
            linkedin_id: None,
            company: "Acme Corp".to_string(),
            title: "Senior Software Engineer".to_string(),
            location: Some("San Francisco, CA".to_string()),
            start_date: "2022-01".to_string(),
            end_date: None,
            description: Some("Led backend architecture migration.".to_string()),
            mapped_job_id: Some(1),
            sync_status: "synced".to_string(),
            imported_at: "2026-05-25T12:00:00".to_string(),
            reviewed_at: Some("2026-05-25T12:30:00".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LinkedInProject {
    pub id: i64,
    pub linkedin_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub associated_position: Option<String>,
    pub mapped_challenge_id: Option<i64>,
    pub sync_status: String,
    pub imported_at: String,
    pub reviewed_at: Option<String>,
}

impl ApiModel for LinkedInProject {
    fn schema_name() -> &'static str {
        "LinkedInProject"
    }
    fn example() -> Self {
        Self {
            id: 1,
            linkedin_id: None,
            title: "deploy-baba Portfolio".to_string(),
            description: Some("Zero-cost Rust portfolio platform.".to_string()),
            url: Some("https://github.com/shantopagla/deploy-baba".to_string()),
            start_date: Some("2026-01".to_string()),
            end_date: None,
            associated_position: Some("Acme Corp".to_string()),
            mapped_challenge_id: Some(1),
            sync_status: "diverged".to_string(),
            imported_at: "2026-05-25T12:00:00".to_string(),
            reviewed_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LinkedInPositionInput {
    pub company: String,
    pub title: String,
    pub location: Option<String>,
    pub start_date: String,
    pub end_date: Option<String>,
    pub description: Option<String>,
}

impl ApiModel for LinkedInPositionInput {
    fn schema_name() -> &'static str {
        "LinkedInPositionInput"
    }
    fn example() -> Self {
        Self {
            company: "Acme Corp".to_string(),
            title: "Senior Software Engineer".to_string(),
            location: Some("San Francisco, CA".to_string()),
            start_date: "2022-01".to_string(),
            end_date: None,
            description: Some("Led backend architecture migration.".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LinkedInProjectInput {
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub associated_position: Option<String>,
}

impl ApiModel for LinkedInProjectInput {
    fn schema_name() -> &'static str {
        "LinkedInProjectInput"
    }
    fn example() -> Self {
        Self {
            title: "deploy-baba Portfolio".to_string(),
            description: Some("Zero-cost Rust portfolio platform.".to_string()),
            url: Some("https://github.com/shantopagla/deploy-baba".to_string()),
            start_date: Some("2026-01".to_string()),
            end_date: None,
            associated_position: Some("Acme Corp".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LinkedInImportPayload {
    pub positions: Vec<LinkedInPositionInput>,
    pub projects: Vec<LinkedInProjectInput>,
}

impl ApiModel for LinkedInImportPayload {
    fn schema_name() -> &'static str {
        "LinkedInImportPayload"
    }
    fn example() -> Self {
        Self {
            positions: vec![LinkedInPositionInput::example()],
            projects: vec![LinkedInProjectInput::example()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LinkedInImportResult {
    pub positions_imported: i64,
    pub projects_imported: i64,
    pub positions_matched: i64,
    pub projects_matched: i64,
}

impl ApiModel for LinkedInImportResult {
    fn schema_name() -> &'static str {
        "LinkedInImportResult"
    }
    fn example() -> Self {
        Self {
            positions_imported: 5,
            projects_imported: 3,
            positions_matched: 4,
            projects_matched: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LinkedInSyncLogEntry {
    pub id: i64,
    pub source: String,
    pub positions_count: i64,
    pub projects_count: i64,
    pub imported_at: String,
}

impl ApiModel for LinkedInSyncLogEntry {
    fn schema_name() -> &'static str {
        "LinkedInSyncLogEntry"
    }
    fn example() -> Self {
        Self {
            id: 1,
            source: "upload".to_string(),
            positions_count: 5,
            projects_count: 3,
            imported_at: "2026-05-25T12:00:00".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SyncFieldComparison {
    pub field: String,
    pub linkedin_value: Option<String>,
    pub db_value: Option<String>,
    pub differs: bool,
}

impl ApiModel for SyncFieldComparison {
    fn schema_name() -> &'static str {
        "SyncFieldComparison"
    }
    fn example() -> Self {
        Self {
            field: "title".to_string(),
            linkedin_value: Some("Senior Software Engineer".to_string()),
            db_value: Some("Sr. Software Engineer".to_string()),
            differs: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PositionDiff {
    pub position: LinkedInPosition,
    pub job_title: Option<String>,
    pub job_company: Option<String>,
    pub fields: Vec<SyncFieldComparison>,
}

impl ApiModel for PositionDiff {
    fn schema_name() -> &'static str {
        "PositionDiff"
    }
    fn example() -> Self {
        Self {
            position: LinkedInPosition::example(),
            job_title: Some("Sr. Software Engineer".to_string()),
            job_company: Some("Acme Corp".to_string()),
            fields: vec![SyncFieldComparison::example()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProjectDiff {
    pub project: LinkedInProject,
    pub challenge_title: Option<String>,
    pub fields: Vec<SyncFieldComparison>,
}

impl ApiModel for ProjectDiff {
    fn schema_name() -> &'static str {
        "ProjectDiff"
    }
    fn example() -> Self {
        Self {
            project: LinkedInProject::example(),
            challenge_title: Some("deploy-baba Portfolio Platform".to_string()),
            fields: vec![SyncFieldComparison::example()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MapRequest {
    pub target_id: Option<i64>,
}

impl ApiModel for MapRequest {
    fn schema_name() -> &'static str {
        "MapRequest"
    }
    fn example() -> Self {
        Self { target_id: Some(1) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StatusUpdateRequest {
    pub status: String,
}

impl ApiModel for StatusUpdateRequest {
    fn schema_name() -> &'static str {
        "StatusUpdateRequest"
    }
    fn example() -> Self {
        Self {
            status: "synced".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LinkedInOAuthToken {
    pub access_token: String,
    pub expires_at: i64,
    pub name: Option<String>,
    pub email: Option<String>,
    pub picture_url: Option<String>,
}

impl ApiModel for LinkedInOAuthToken {
    fn schema_name() -> &'static str {
        "LinkedInOAuthToken"
    }
    fn example() -> Self {
        Self {
            access_token: "li-token-abc123".to_string(),
            expires_at: 1748900000,
            name: Some("Shanto".to_string()),
            email: Some("test@example.com".to_string()),
            picture_url: Some("https://example.com/photo.jpg".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LinkedInOAuthStatus {
    pub connected: bool,
    pub name: Option<String>,
    pub email: Option<String>,
    pub picture_url: Option<String>,
    pub token_expires_at: Option<String>,
}

impl ApiModel for LinkedInOAuthStatus {
    fn schema_name() -> &'static str {
        "LinkedInOAuthStatus"
    }
    fn example() -> Self {
        Self {
            connected: true,
            name: Some("Shanto".to_string()),
            email: Some("test@example.com".to_string()),
            picture_url: Some("https://example.com/photo.jpg".to_string()),
            token_expires_at: Some("1748900000".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BulkStatusRequest {
    pub ids: Vec<i64>,
    pub status: String,
}

impl ApiModel for BulkStatusRequest {
    fn schema_name() -> &'static str {
        "BulkStatusRequest"
    }
    fn example() -> Self {
        Self {
            ids: vec![1, 2, 3],
            status: "synced".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BulkStatusResult {
    pub updated: i64,
}

impl ApiModel for BulkStatusResult {
    fn schema_name() -> &'static str {
        "BulkStatusResult"
    }
    fn example() -> Self {
        Self { updated: 3 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AutoMatchResult {
    pub positions_matched: i64,
    pub projects_matched: i64,
}

impl ApiModel for AutoMatchResult {
    fn schema_name() -> &'static str {
        "AutoMatchResult"
    }
    fn example() -> Self {
        Self {
            positions_matched: 2,
            projects_matched: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApplyFieldsRequest {
    pub fields: Vec<String>,
}

impl ApiModel for ApplyFieldsRequest {
    fn schema_name() -> &'static str {
        "ApplyFieldsRequest"
    }
    fn example() -> Self {
        Self {
            fields: vec!["title".to_string(), "description".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApplyResult {
    pub fields_applied: Vec<String>,
}

impl ApiModel for ApplyResult {
    fn schema_name() -> &'static str {
        "ApplyResult"
    }
    fn example() -> Self {
        Self {
            fields_applied: vec!["title".to_string(), "description".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReconciliationItem {
    pub id: i64,
    pub entity_type: String,
    pub title: String,
    pub sync_status: String,
    pub has_mapping: bool,
    pub differing_fields: Vec<String>,
}

impl ApiModel for ReconciliationItem {
    fn schema_name() -> &'static str {
        "ReconciliationItem"
    }
    fn example() -> Self {
        Self {
            id: 1,
            entity_type: "position".to_string(),
            title: "Senior Software Engineer".to_string(),
            sync_status: "diverged".to_string(),
            has_mapping: true,
            differing_fields: vec!["title".to_string(), "location".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SeedFromDbResult {
    pub positions_seeded: i64,
    pub projects_seeded: i64,
}

impl ApiModel for SeedFromDbResult {
    fn schema_name() -> &'static str {
        "SeedFromDbResult"
    }
    fn example() -> Self {
        Self {
            positions_seeded: 5,
            projects_seeded: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReconciliationSummary {
    pub needs_linkedin_update: Vec<ReconciliationItem>,
    pub needs_db_import: Vec<ReconciliationItem>,
    pub in_sync: Vec<ReconciliationItem>,
}

impl ApiModel for ReconciliationSummary {
    fn schema_name() -> &'static str {
        "ReconciliationSummary"
    }
    fn example() -> Self {
        Self {
            needs_linkedin_update: vec![ReconciliationItem::example()],
            needs_db_import: vec![],
            in_sync: vec![],
        }
    }
}
