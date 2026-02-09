use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use deadpool_postgres::Pool;
use uuid::Uuid;

use crate::models::{CreateRencanaRequest, RencanaPengadaan, UpdateRencanaRequest};

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
}

// Handler Error type
pub enum ApiError {
    Internal(anyhow::Error),
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::Internal(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": format!("Internal Server Error: {}", err)
                })),
            )
                .into_response(),
            ApiError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "message": msg
                })),
            )
                .into_response(),
        }
    }
}

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Internal(err.into())
    }
}

pub async fn list_rencana(
    State(state): State<AppState>,
) -> Result<Json<Vec<RencanaPengadaan>>, ApiError> {
    let client = state.pool.get().await?;
    let rows = client
        .query(
            "SELECT * FROM perencanaan_pengadaan ORDER BY created_at DESC",
            &[],
        )
        .await?;

    let plans: Vec<RencanaPengadaan> = rows.iter().map(RencanaPengadaan::from_row).collect();

    Ok(Json(plans))
}

pub async fn create_rencana(
    State(state): State<AppState>,
    Json(payload): Json<CreateRencanaRequest>,
) -> Result<Json<RencanaPengadaan>, ApiError> {
    let client = state.pool.get().await?;
    let row = client
        .query_one(
            "INSERT INTO perencanaan_pengadaan (nama_kegiatan, kode_rekening, pagu_anggaran, tanggal_mulai, tanggal_selesai)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING *",
            &[
                &payload.nama_kegiatan,
                &payload.kode_rekening,
                &payload.pagu_anggaran,
                &payload.tanggal_mulai,
                &payload.tanggal_selesai,
            ],
        )
        .await?;

    Ok(Json(RencanaPengadaan::from_row(&row)))
}

pub async fn update_rencana(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRencanaRequest>,
) -> Result<Json<RencanaPengadaan>, ApiError> {
    let client = state.pool.get().await?;

    // Build dynamic update query
    let mut query = "UPDATE perencanaan_pengadaan SET ".to_string();
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
    let mut idx = 1;

    if let Some(nama) = &payload.nama_kegiatan {
        query.push_str(&format!("nama_kegiatan = ${}, ", idx));
        params.push(Box::new(nama.clone()));
        idx += 1;
    }
    if let Some(kode) = &payload.kode_rekening {
        query.push_str(&format!("kode_rekening = ${}, ", idx));
        params.push(Box::new(kode.clone()));
        idx += 1;
    }
    if let Some(pagu) = payload.pagu_anggaran {
        query.push_str(&format!("pagu_anggaran = ${}, ", idx));
        params.push(Box::new(pagu));
        idx += 1;
    }
    if let Some(mulai) = payload.tanggal_mulai {
        query.push_str(&format!("tanggal_mulai = ${}, ", idx));
        params.push(Box::new(mulai));
        idx += 1;
    }
    if let Some(selesai) = payload.tanggal_selesai {
        query.push_str(&format!("tanggal_selesai = ${}, ", idx));
        params.push(Box::new(selesai));
        idx += 1;
    }
    if let Some(status) = &payload.status {
        query.push_str(&format!("status = ${}, ", idx));
        params.push(Box::new(status.clone()));
        idx += 1;
    }

    // Remove trailing comma and space
    if query.ends_with(", ") {
        query.truncate(query.len() - 2);
    } else {
        // No fields to update
        return Err(ApiError::BadRequest("No fields to update".to_string()));
    }

    query.push_str(&format!(" WHERE id = ${} RETURNING *", idx));
    params.push(Box::new(id));

    // Convert params to slice for query method
    let params_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
        .iter()
        .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect();

    let row = client.query_one(&query, &params_refs).await?;

    Ok(Json(RencanaPengadaan::from_row(&row)))
}

pub async fn delete_rencana(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let client = state.pool.get().await?;
    let result = client
        .execute("DELETE FROM perencanaan_pengadaan WHERE id = $1", &[&id])
        .await?;

    if result == 0 {
        return Ok(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
