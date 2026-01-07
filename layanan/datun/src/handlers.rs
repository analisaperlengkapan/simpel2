use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::models::{CreateCaseRequest, DatunCase, UpdateCaseRequest};
use crate::state::AppState;

/// Get all cases
pub async fn get_cases(State(state): State<AppState>) -> Json<Vec<DatunCase>> {
    let cases = state.get_cases();
    Json(cases)
}

/// Get case by ID
pub async fn get_case_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DatunCase>, StatusCode> {
    match state.get_case(&id) {
        Some(case) => Ok(Json(case)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Create a new case
pub async fn create_case(
    State(state): State<AppState>,
    Json(payload): Json<CreateCaseRequest>,
) -> (StatusCode, Json<DatunCase>) {
    let case = DatunCase::new(
        payload.no_skk,
        payload.instansi_pemohon,
        payload.pihak_lawan,
        payload.judul_perkara,
        payload.jenis_layanan,
        payload.posisi_kasus,
    );

    state.add_case(case.clone());
    (StatusCode::CREATED, Json(case))
}

/// Update a case
pub async fn update_case(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateCaseRequest>,
) -> Result<Json<DatunCase>, StatusCode> {
    match state.update_case(&id, payload.status, payload.tim_jpn, payload.nilai_pemulihan) {
        Some(case) => Ok(Json(case)),
        None => Err(StatusCode::NOT_FOUND),
    }
}
