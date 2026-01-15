use crate::{
    models::{CreateCaseRequest, CreateSuspectRequest},
    services::PidmilService,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use garde::Validate;
use serde_json::json;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub service: PidmilService,
}

pub async fn get_cases(State(state): State<AppState>) -> impl IntoResponse {
    match state.service.get_all_cases().await {
        Ok(cases) => Json(cases).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_case(
    State(state): State<AppState>,
    Json(req): Json<CreateCaseRequest>,
) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
            .into_response();
    }

    match state.service.create_case(req).await {
        Ok(case) => (StatusCode::CREATED, Json(case)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_case_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.service.get_case_by_id(id).await {
        Ok(Some(case)) => Json(case).into_response(),
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

pub async fn get_suspects(
    State(state): State<AppState>,
    Path(case_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.service.get_suspects_by_case(case_id).await {
        Ok(suspects) => Json(suspects).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn create_suspect(
    State(state): State<AppState>,
    Path(case_id): Path<Uuid>,
    Json(req): Json<CreateSuspectRequest>,
) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        )
            .into_response();
    }

    match state.service.create_suspect(case_id, req).await {
        Ok(suspect) => (StatusCode::CREATED, Json(suspect)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
