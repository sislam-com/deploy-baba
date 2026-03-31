use askama::Template;
use askama_axum::IntoResponse;
use axum::Extension;

use crate::auth::Claims;

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub username: String,
}

pub async fn dashboard_handler(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    DashboardTemplate {
        username: claims.username,
    }
}
