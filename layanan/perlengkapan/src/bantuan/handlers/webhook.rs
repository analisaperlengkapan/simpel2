use crate::bantuan::error::AppError;
use crate::bantuan::webhook::WebhookService;
use axum::{
    extract::{Json, Path, Query, State},
    response::IntoResponse,
};
use deadpool_postgres::Pool;
use serde_json::json;
use uuid::Uuid;

pub async fn list_webhook_events(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let event_type = params.get("event_type").and_then(|v| v.as_str());
    let delivered = params.get("delivered").and_then(|v| v.as_bool());
    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as i64;
    let webhook = WebhookService::new(pool);
    let events = webhook.list_events(event_type, delivered, limit).await?;
    Ok(Json(events))
}
pub async fn create_webhook_event(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let event_type = payload["event_type"]
        .as_str()
        .ok_or(AppError::Validation("event_type wajib".to_string()))?;
    let payload_data = payload.get("payload").cloned().unwrap_or(json!({}));
    let webhook = WebhookService::new(pool.clone());
    let event = webhook.create_event(event_type, &payload_data).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(event))
}
pub async fn deliver_webhook_event(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let url = payload["url"]
        .as_str()
        .ok_or(AppError::Validation("url wajib".to_string()))?;
    let webhook = WebhookService::new(pool.clone());
    webhook.deliver_event(id, url).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(json!({"status": "delivered", "id": id})))
}
pub async fn retry_webhook_event(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let url = payload["url"]
        .as_str()
        .ok_or(AppError::Validation("url wajib".to_string()))?;
    let webhook = WebhookService::new(pool.clone());
    webhook.retry_event(id, url).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(json!({"status": "retried", "id": id})))
}
