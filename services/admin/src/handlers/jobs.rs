use service_protocol::ServiceResponse;

pub async fn list_jobs() -> ServiceResponse {
    // TODO: Implement admin jobs CRUD
    ServiceResponse::ok(serde_json::json!({"message": "admin list jobs stub"}))
}

pub async fn create_job(_body: Option<String>) -> ServiceResponse {
    ServiceResponse::ok(serde_json::json!({"message": "admin create job stub"}))
}

pub async fn update_job(_slug: &str, _body: Option<String>) -> ServiceResponse {
    ServiceResponse::ok(serde_json::json!({"message": "admin update job stub"}))
}

pub async fn delete_job(_slug: &str) -> ServiceResponse {
    ServiceResponse::ok(serde_json::json!({"message": "admin delete job stub"}))
}
