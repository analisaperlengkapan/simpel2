use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use garde::Validate;
use crate::models::{CreateCaseRequest, DatunCase, UpdateCaseRequest};
use crate::state::AppState;
use tracing::{info, instrument, error};

/// Get all cases
#[instrument(skip(state))]
pub async fn get_cases(State(state): State<AppState>) -> Json<Vec<DatunCase>> {
    info!("Fetching all cases");
    let cases = state.get_cases();
    Json(cases)
}

/// Get case by ID
#[instrument(skip(state))]
pub async fn get_case_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DatunCase>, StatusCode> {
    match state.get_case(&id) {
        Some(case) => {
            info!("Case found: {}", id);
            Ok(Json(case))
        },
        None => {
            error!("Case not found: {}", id);
            Err(StatusCode::NOT_FOUND)
        },
    }
}

/// Create a new case
#[instrument(skip(state))]
pub async fn create_case(
    State(state): State<AppState>,
    Json(payload): Json<CreateCaseRequest>,
) -> Result<(StatusCode, Json<DatunCase>), (StatusCode, String)> {
    // Validate payload
    if let Err(e) = payload.validate() {
        error!("Validation error: {:?}", e);
        return Err((StatusCode::BAD_REQUEST, e.to_string()));
    }

    let case = DatunCase::new(
        payload.no_skk,
        payload.instansi_pemohon,
        payload.pihak_lawan,
        payload.judul_perkara,
        payload.jenis_layanan,
        payload.posisi_kasus,
    );

    state.add_case(case.clone());
    info!("Created new case with ID: {}", case.id);
    Ok((StatusCode::CREATED, Json(case)))
}

/// Update a case
#[instrument(skip(state))]
pub async fn update_case(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateCaseRequest>,
) -> Result<Json<DatunCase>, StatusCode> {
    if let Err(e) = payload.validate() {
        error!("Validation error: {:?}", e);
        // Axum doesn't easily support returning different types (Json vs String) in simple Result,
        // usually we'd use IntoResponse but here keeping it simple: just fail 400 if validation fails,
        // but since UpdateCaseRequest validation is minimal (optional fields), it might not fail often.
        // Actually I didn't add validation rules to UpdateCaseRequest fields yet, just derived Validate.
        return Err(StatusCode::BAD_REQUEST);
    }

    match state.update_case(&id, payload.status, payload.tim_jpn, payload.nilai_pemulihan) {
        Some(case) => {
            info!("Updated case: {}", id);
            Ok(Json(case))
        },
        None => {
            error!("Case not found for update: {}", id);
            Err(StatusCode::NOT_FOUND)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ServiceType;
    use crate::state::AppState;

    #[tokio::test]
    async fn test_create_and_get_case() {
        let state = AppState::new();

        let payload = CreateCaseRequest {
            no_skk: "SKK-TEST/01".to_string(),
            instansi_pemohon: "Tester".to_string(),
            pihak_lawan: "Opponent".to_string(),
            judul_perkara: "Test Case Title".to_string(),
            jenis_layanan: ServiceType::LegalOpinion,
            posisi_kasus: "Long enough description for validation".to_string(),
        };

        // Create
        let (status, Json(case)) = create_case(State(state.clone()), Json(payload)).await.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(case.judul_perkara, "Test Case Title");

        // Get
        let Json(cases) = get_cases(State(state.clone())).await;
        // There are mock cases in AppState::new(), so len > 1
        assert!(cases.len() >= 1);
        assert!(cases.iter().any(|c| c.id == case.id));
    }
}
