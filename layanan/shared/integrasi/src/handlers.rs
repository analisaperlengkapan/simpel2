use crate::error::Result;
use axum::{response::Json, routing::get, Router};
use serde_json::{json, Value};

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "integrasi",
        "timestamp": chrono::Utc::now()
    }))
}

pub async fn get_services() -> Json<Value> {
    Json(json!({
        "services": [],
        "message": "Service discovery not implemented yet"
    }))
}

pub async fn register_service(Json(payload): Json<Value>) -> Result<Json<Value>> {
    Ok(Json(json!({
        "message": "Service registration not implemented yet",
        "received": payload
    })))
}

pub fn create_routes() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/services", get(get_services))
}
