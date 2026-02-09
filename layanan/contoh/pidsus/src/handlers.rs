use crate::models::CreateCaseRequest;
use crate::repository::Repository;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use garde::Validate;
use serde_json::json;
use uuid::Uuid;

// AppState wrapper to share repository
#[derive(Clone)]
pub struct AppState {
    pub repository: Repository,
}

pub async fn list_cases(State(state): State<AppState>) -> impl IntoResponse {
    match state.repository.get_cases().await {
        Ok(cases) => (StatusCode::OK, Json(cases)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_case(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match state.repository.get_case(id).await {
        Ok(Some(case)) => (StatusCode::OK, Json(case)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Case not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_case(
    State(state): State<AppState>,
    Json(payload): Json<CreateCaseRequest>,
) -> impl IntoResponse {
    // Validate request
    if let Err(e) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Validation error: {}", e)})),
        )
            .into_response();
    }

    match state.repository.create_case(payload).await {
        Ok(case) => (StatusCode::CREATED, Json(case)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_dashboard_stats(State(state): State<AppState>) -> impl IntoResponse {
    match state.repository.get_stats().await {
        Ok(stats) => (StatusCode::OK, Json(stats)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "pidsus Service OK")
}
