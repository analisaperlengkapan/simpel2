use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use deadpool_postgres::Pool;
use uuid::Uuid;
use chrono::Utc;
use crate::models::{CreatePerkaraRequest, Perkara};
use garde::Validate;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
}

pub async fn list_perkara(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let client = state.pool.get().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    // Since we might not have the table yet, we'll try the query but fallback to empty list or mock if it fails in dev
    // But for "further development", we assume we should write the correct query.
    // If the table doesn't exist, this will error.
    // Ideally we would have migrations, but for this task I will write the query code.

    let rows = client
        .query("SELECT * FROM perkara ORDER BY created_at DESC", &[])
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let perkara_list: Vec<Perkara> = rows.into_iter().map(Perkara::from).collect();

    Ok(Json(perkara_list))
}

pub async fn create_perkara(
    State(state): State<AppState>,
    Json(payload): Json<CreatePerkaraRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Err(e) = payload.validate() {
        return Err((StatusCode::BAD_REQUEST, e.to_string()));
    }

    let client = state.pool.get().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    let row = client
        .query_one(
            "INSERT INTO perkara (id, nomor_perkara, judul, tanggal_kejadian, status, deskripsi, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING *",
            &[
                &id,
                &payload.nomor_perkara,
                &payload.judul,
                &payload.tanggal_kejadian,
                &payload.status,
                &payload.deskripsi,
                &now,
                &now,
            ],
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let perkara = Perkara::from(row);

    Ok((StatusCode::CREATED, Json(perkara)))
}

pub async fn get_perkara_detail(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let client = state.pool.get().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    let row = client
        .query_opt("SELECT * FROM perkara WHERE id = $1", &[&id])
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match row {
        Some(row) => Ok(Json(Perkara::from(row))),
        None => Err((StatusCode::NOT_FOUND, "Perkara not found".to_string())),
    }
}
