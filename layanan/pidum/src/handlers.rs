use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use deadpool_postgres::Pool;
use uuid::Uuid;
use chrono::Utc;
use crate::models::{CreatePerkaraRequest, Perkara, CreateCommentRequest, PerkaraComment, PerkaraTimeline};
use garde::Validate;
use serde::Deserialize;
use serde_json::json;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
}

#[derive(Deserialize)]
pub struct Pagination {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

pub async fn list_perkara(
    State(state): State<AppState>,
    Query(params): Query<Pagination>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let client = state.pool.get().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    let limit = params.limit.unwrap_or(10);
    let offset = (params.page.unwrap_or(1) - 1) * limit;

    let rows = client
        .query(
            "SELECT * FROM perkara ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            &[&(limit as i64), &(offset as i64)]
        )
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

    let mut client = state.pool.get().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    let tx = client.transaction().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Transaction error: {}", e))
    })?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    let row = tx
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

    // Auto-create timeline event
    let timeline_id = Uuid::new_v4();
    tx.execute(
            "INSERT INTO perkara_timeline (id, perkara_id, action_type, description, created_at)
             VALUES ($1, $2, $3, $4, $5)",
            &[
                &timeline_id,
                &perkara.id,
                &"CREATED".to_string(),
                &format!("Perkara created with status {}", perkara.status.to_string()),
                &now
            ],
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create timeline: {}", e)))?;

    tx.commit().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to commit transaction: {}", e))
    })?;

    Ok((StatusCode::CREATED, Json(perkara)))
}

pub async fn add_comment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Err(e) = payload.validate() {
        return Err((StatusCode::BAD_REQUEST, e.to_string()));
    }

    let mut client = state.pool.get().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    let tx = client.transaction().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Transaction error: {}", e))
    })?;

    // Verify perkara exists (could be skipped if we trust FK, but good for returning 404)
    // In transaction, if we insert into comments with FK, it will fail if not exists.
    // But checking first gives better error message.
    let _ = tx
        .query_one("SELECT id FROM perkara WHERE id = $1", &[&id])
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Perkara not found".to_string()))?;

    let comment_id = Uuid::new_v4();
    let now = Utc::now();
    let user_info = json!({ "name": payload.user_name.unwrap_or("Anonymous".to_string()) });

    let row = tx
        .query_one(
            "INSERT INTO perkara_comments (id, perkara_id, content, user_info, created_at)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING *",
            &[&comment_id, &id, &payload.content, &user_info, &now],
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let comment = PerkaraComment::from(row);

    // Add to timeline
    let timeline_id = Uuid::new_v4();
    tx.execute(
            "INSERT INTO perkara_timeline (id, perkara_id, action_type, description, user_info, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)",
            &[
                &timeline_id,
                &id,
                &"COMMENT".to_string(),
                &"New comment added".to_string(),
                &user_info,
                &now
            ],
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update timeline: {}", e)))?;

    tx.commit().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to commit transaction: {}", e))
    })?;

    Ok((StatusCode::CREATED, Json(comment)))
}

pub async fn get_timeline(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let client = state.pool.get().await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    // Fetch timeline events
    let rows = client
        .query(
            "SELECT * FROM perkara_timeline WHERE perkara_id = $1 ORDER BY created_at DESC",
            &[&id],
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let events: Vec<PerkaraTimeline> = rows.into_iter().map(PerkaraTimeline::from).collect();

    // Fetch comments (optional, could be merged in frontend or here)
    // For now we just return timeline events. The requirement was 'Activity Stream'.
    // If we want mixed stream, we can return both or rely on 'COMMENT' events in timeline table which we just added.

    Ok(Json(events))
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
