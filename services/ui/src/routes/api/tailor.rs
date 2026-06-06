use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use std::sync::Arc;

use api_openapi::models::{MatchedBullet, TailorRequest};

use crate::db::Db;
use crate::state::AppState;
use crate::tailor::matcher;

type ApiError = (StatusCode, String);

fn db_err(e: impl std::fmt::Display) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

#[utoipa::path(
    post,
    path = "/api/v1/tailor/match",
    tag = "tailor",
    request_body = TailorRequest,
    responses(
        (status = 200, description = "Ranked resume bullets matching the JD", body = Vec<MatchedBullet>),
        (status = 400, description = "Missing or empty job_description"),
        (status = 500, description = "Database error")
    )
)]
pub async fn match_bullets(
    State(db): State<Arc<Db>>,
    Json(body): Json<TailorRequest>,
) -> Result<Json<Vec<MatchedBullet>>, ApiError> {
    if body.job_description.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "job_description is required".into(),
        ));
    }
    let conn = db.conn.lock().unwrap();
    let bullets = matcher::rank_bullets(&conn, &body.job_description, 20).map_err(db_err)?;
    Ok(Json(bullets))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/match", post(match_bullets))
}
