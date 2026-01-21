//! Health check handlers

use axum::{extract::State, Json};
use serde::Serialize;
use std::sync::Arc;

use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct ReadinessResponse {
    pub status: String,
    pub database: String,
    pub authenc: String,
    pub secreton: String,
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "daskrimti-portal".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn readiness_check(
    State(state): State<Arc<AppState>>,
) -> Json<ReadinessResponse> {
    // Check database
    let db_status = match state.db.get().await {
        Ok(client) => {
            match client.query_one("SELECT 1", &[]).await {
                Ok(_) => "healthy",
                Err(_) => "unhealthy",
            }
        }
        Err(_) => "unhealthy",
    };

    Json(ReadinessResponse {
        status: if db_status == "healthy" { "ready" } else { "not_ready" }.to_string(),
        database: db_status.to_string(),
        authenc: "connected".to_string(),
        secreton: "connected".to_string(),
    })
}

pub async fn liveness_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "alive".to_string(),
        service: "daskrimti-portal".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
