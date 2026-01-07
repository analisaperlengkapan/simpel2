use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use chrono::Utc;
use crate::models::{SupervisionSchedule, CreateScheduleRequest};
use crate::errors::{AppResult, AppError};
use deadpool_postgres::Pool;
use garde::Validate;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool,
}

impl AppState {
    pub fn new(pool: Pool) -> Self {
        Self { db: pool }
    }
}

pub async fn list_schedules(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<SupervisionSchedule>>> {
    let client = state.db.get().await.map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let rows = client
        .query("SELECT id, title, scheduled_at, location, created_at FROM schedules ORDER BY scheduled_at DESC", &[])
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let schedules: Vec<SupervisionSchedule> = rows.iter().map(|row| SupervisionSchedule {
        id: row.get("id"),
        title: row.get("title"),
        scheduled_at: row.get("scheduled_at"),
        location: row.get("location"),
        created_at: row.get("created_at"),
    }).collect();

    Ok(Json(schedules))
}

pub async fn create_schedule(
    State(state): State<AppState>,
    Json(payload): Json<CreateScheduleRequest>,
) -> AppResult<Json<SupervisionSchedule>> {
    if let Err(e) = payload.validate() {
        return Err(AppError::BadRequest(e.to_string()));
    }

    let client = state.db.get().await.map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let schedule = SupervisionSchedule {
        id: Uuid::new_v4(),
        title: payload.title,
        scheduled_at: payload.scheduled_at,
        location: payload.location,
        created_at: Utc::now(),
    };

    client.execute(
        "INSERT INTO schedules (id, title, scheduled_at, location, created_at) VALUES ($1, $2, $3, $4, $5)",
        &[&schedule.id, &schedule.title, &schedule.scheduled_at, &schedule.location, &schedule.created_at],
    ).await.map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Json(schedule))
}

pub async fn get_schedule(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<SupervisionSchedule>> {
    let client = state.db.get().await.map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let row = client
        .query_opt("SELECT id, title, scheduled_at, location, created_at FROM schedules WHERE id = $1", &[&id])
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    match row {
        Some(row) => {
            Ok(Json(SupervisionSchedule {
                id: row.get("id"),
                title: row.get("title"),
                scheduled_at: row.get("scheduled_at"),
                location: row.get("location"),
                created_at: row.get("created_at"),
            }))
        },
        None => Err(AppError::NotFound("Schedule not found".into())),
    }
}
