use crate::bantuan::error::AppError;
use crate::bantuan::ticket::TicketService;
use axum::{
    extract::{Json, Path, Query, State},
    response::IntoResponse,
};
use deadpool_postgres::Pool;
use serde_json::json;
use uuid::Uuid;

pub async fn list_tickets(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = params
        .get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let status = params.get("status").and_then(|v| v.as_str());
    let ticket = TicketService::new(pool);
    let tickets = ticket.list_tickets(user_id, status).await?;
    Ok(Json(tickets))
}
pub async fn create_ticket(
    State(pool): State<Pool>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = payload["user_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("user_id wajib".to_string()))?;
    let subject = payload["subject"]
        .as_str()
        .ok_or(AppError::Validation("subject wajib".to_string()))?;
    let description = payload.get("description").and_then(|v| v.as_str());
    let priority = payload.get("priority").and_then(|v| v.as_str());
    let category_id = payload
        .get("category_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let spam_score = payload
        .get("spam_score")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32;
    let ticket_service = TicketService::new(pool.clone());
    let ticket = ticket_service
        .create_ticket(
            user_id,
            subject,
            description,
            priority,
            category_id,
            spam_score,
        )
        .await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(ticket))
}
pub async fn get_ticket(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let ticket = TicketService::new(pool);
    let t = ticket.get_ticket(id).await?;
    Ok(Json(t))
}
pub async fn update_ticket(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let subject = payload["subject"]
        .as_str()
        .ok_or(AppError::Validation("subject wajib".to_string()))?;
    let description = payload.get("description").and_then(|v| v.as_str());
    let priority = payload.get("priority").and_then(|v| v.as_str());
    let category_id = payload
        .get("category_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let status = payload["status"].as_str().unwrap_or("open");
    let ticket_service = TicketService::new(pool.clone());
    let t = ticket_service
        .update_ticket(id, subject, description, priority, category_id, status)
        .await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(t))
}
pub async fn delete_ticket(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let ticket = TicketService::new(pool.clone());
    ticket.delete_ticket(id).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(json!({"status": "deleted", "id": id})))
}
pub async fn list_ticket_comments(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let ticket = TicketService::new(pool);
    let comments = ticket.list_comments(id).await?;
    Ok(Json(comments))
}
pub async fn add_ticket_comment(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = payload["user_id"]
        .as_str()
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(AppError::Validation("user_id wajib".to_string()))?;
    let content = payload["content"]
        .as_str()
        .ok_or(AppError::Validation("content wajib".to_string()))?;
    let ticket = TicketService::new(pool.clone());
    let comment = ticket.add_comment(id, user_id, content).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(comment))
}
pub async fn update_ticket_status(
    State(pool): State<Pool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let status = payload["status"]
        .as_str()
        .ok_or(AppError::Validation("status wajib".to_string()))?;
    let ticket = TicketService::new(pool.clone());
    let t = ticket.update_status(id, status).await?;
    // TODO(audit-log): emit AuditEvent via state.audit_sink once the bantuan router is mounted into the unified app + claims are plumbed through (deferred per plan A.5).
    Ok(Json(t))
}
pub async fn search_tickets(
    State(pool): State<Pool>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let query = params["q"].as_str().unwrap_or("");
    let max_results = params["limit"].as_u64().unwrap_or(20) as u32;
    let ticket = TicketService::new(pool);
    let tickets = ticket.search_tickets(query, max_results).await?;
    Ok(Json(tickets))
}
