//! `/bantuan/tiket*` handlers.
//!
//! Every handler builds its [`TicketActor`] from verified [`Claims`] and passes
//! it to the service — the request body carries *what* to do, never *who* is
//! doing it. Typed request DTOs (rather than raw `serde_json::Value`) mean an
//! unknown or spoofed `user_id` field cannot even be expressed by a client.

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::bantuan::models::{SupportTicket, TicketComment};
use crate::bantuan::ticket::{TicketActor, TicketService};
use crate::shared::error::AppResult;
use crate::shared::middleware::Claims;
use crate::state::AppState;
use lib_perlengkapan::response::ApiResponse;

/// Build the service with the notification + audit ports wired, so a ticket
/// event lands in the /notifikasi centre and `perlengkapan.audit_log` exactly
/// like a workflow event does.
fn service(state: &AppState) -> TicketService {
    TicketService::new(state.db_pool.clone())
        .with_notification_sender(state.notifier.clone())
        .with_audit_sink(state.audit_sink.clone())
}

#[derive(Debug, Deserialize)]
pub struct CreateTicketRequest {
    pub subject: String,
    pub description: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddCommentRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ListTicketsQuery {
    pub status: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

/// POST /bantuan/tiket — file a ticket as the authenticated caller.
pub async fn create_ticket(
    State(state): State<AppState>,
    claims: Claims,
    Json(req): Json<CreateTicketRequest>,
) -> AppResult<Json<ApiResponse<SupportTicket>>> {
    let actor = TicketActor::from_claims(&claims);
    let ticket = service(&state)
        .create_ticket(
            &actor,
            &req.subject,
            req.description.as_deref(),
            req.priority.as_deref(),
        )
        .await?;
    Ok(Json(ApiResponse::success(
        ticket,
        "Tiket bantuan berhasil dibuat".to_string(),
    )))
}

/// GET /bantuan/tiket — the caller's own tickets; every ticket for helpdesk staff.
pub async fn list_tickets(
    State(state): State<AppState>,
    claims: Claims,
    Query(q): Query<ListTicketsQuery>,
) -> AppResult<Json<ApiResponse<Vec<SupportTicket>>>> {
    let actor = TicketActor::from_claims(&claims);
    let tickets = service(&state)
        .list_tickets(&actor, q.status.as_deref(), q.limit, q.offset)
        .await?;
    Ok(Json(ApiResponse::success(
        tickets,
        "Daftar tiket bantuan".to_string(),
    )))
}

/// GET /bantuan/tiket/{id}
pub async fn get_ticket(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<SupportTicket>>> {
    let actor = TicketActor::from_claims(&claims);
    let ticket = service(&state).get_ticket(&actor, id).await?;
    Ok(Json(ApiResponse::success(
        ticket,
        "Detail tiket bantuan".to_string(),
    )))
}

/// PUT /bantuan/tiket/{id}/status — helpdesk staff only.
pub async fn update_ticket_status(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStatusRequest>,
) -> AppResult<Json<ApiResponse<SupportTicket>>> {
    let actor = TicketActor::from_claims(&claims);
    let ticket = service(&state)
        .update_status(&actor, id, &req.status)
        .await?;
    Ok(Json(ApiResponse::success(
        ticket,
        "Status tiket diperbarui".to_string(),
    )))
}

/// GET /bantuan/tiket/{id}/komentar
pub async fn list_ticket_comments(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<Vec<TicketComment>>>> {
    let actor = TicketActor::from_claims(&claims);
    let comments = service(&state).list_comments(&actor, id).await?;
    Ok(Json(ApiResponse::success(
        comments,
        "Daftar komentar tiket".to_string(),
    )))
}

/// POST /bantuan/tiket/{id}/komentar
pub async fn add_ticket_comment(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
    Json(req): Json<AddCommentRequest>,
) -> AppResult<Json<ApiResponse<TicketComment>>> {
    let actor = TicketActor::from_claims(&claims);
    let comment = service(&state)
        .add_comment(&actor, id, &req.content)
        .await?;
    Ok(Json(ApiResponse::success(
        comment,
        "Komentar ditambahkan".to_string(),
    )))
}
