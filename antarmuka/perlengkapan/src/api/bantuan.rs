//! Frontend client for `/api/v1/perlengkapan/bantuan/*`.
//!
//! Backs `pages::bantuan::HelpdeskPage`: filing a support ticket and tracking
//! it afterwards. The reporter's identity is *not* sent — the backend takes it
//! from the JWT claims, so there is deliberately no `user_id` field on
//! [`CreateTicketRequest`].
//!
//! `/bantuan/faq` and `/bantuan/panduan` are static pages and have no client.

use serde::{Deserialize, Serialize};

use crate::api::client::{api_get, api_post, api_put};
use crate::api::error::{AppError, AppResult};

#[derive(Debug, Clone, Deserialize)]
struct ApiResponseWrap<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

/// A helpdesk ticket as returned by the backend.
#[derive(Debug, Clone, Deserialize)]
pub struct SupportTicket {
    pub id: String,
    pub user_id: String,
    pub satker_code: Option<String>,
    pub subject: String,
    pub description: Option<String>,
    pub priority: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
}

/// A comment on a ticket. The payload also carries `ticket_id`, but a thread
/// is always fetched per-ticket so the FE doesn't need it (serde ignores it).
#[derive(Debug, Clone, Deserialize)]
pub struct TicketComment {
    pub id: String,
    pub user_id: String,
    pub content: String,
    pub created_at: String,
}

/// Body for `POST /bantuan/tiket`. Note the absence of any identity field:
/// the reporter is whoever the bearer token says they are.
#[derive(Debug, Clone, Serialize)]
pub struct CreateTicketRequest {
    pub subject: String,
    pub description: String,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize)]
struct AddCommentRequest {
    content: String,
}

#[derive(Debug, Clone, Serialize)]
struct UpdateStatusRequest {
    status: String,
}

/// Human label for a ticket status, matching the backend's status vocabulary.
pub fn status_label(status: &str) -> &'static str {
    match status {
        "open" => "Terbuka",
        "in_progress" => "Sedang Ditangani",
        "resolved" => "Selesai",
        "closed" => "Ditutup",
        _ => "Tidak Diketahui",
    }
}

/// `POST /bantuan/tiket`
pub async fn create_ticket(req: &CreateTicketRequest) -> AppResult<SupportTicket> {
    let resp: ApiResponseWrap<SupportTicket> = api_post("/bantuan/tiket", req).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

/// `GET /bantuan/tiket?status=&limit=&offset=`
///
/// Returns the caller's own tickets; helpdesk staff get every ticket. The
/// scoping is enforced server-side — this client cannot widen it.
pub async fn list_tickets(
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<SupportTicket>> {
    let mut url = format!("/bantuan/tiket?limit={limit}&offset={offset}");
    if let Some(s) = status {
        url.push_str(&format!("&status={s}"));
    }
    let resp: ApiResponseWrap<Vec<SupportTicket>> = api_get(&url).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

/// `GET /bantuan/tiket/{id}`
pub async fn get_ticket(id: &str) -> AppResult<SupportTicket> {
    let resp: ApiResponseWrap<SupportTicket> = api_get(&format!("/bantuan/tiket/{id}")).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

/// `GET /bantuan/tiket/{id}/komentar`
pub async fn list_comments(id: &str) -> AppResult<Vec<TicketComment>> {
    let resp: ApiResponseWrap<Vec<TicketComment>> =
        api_get(&format!("/bantuan/tiket/{id}/komentar")).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

/// `POST /bantuan/tiket/{id}/komentar`
pub async fn add_comment(id: &str, content: &str) -> AppResult<TicketComment> {
    let body = AddCommentRequest {
        content: content.to_string(),
    };
    let resp: ApiResponseWrap<TicketComment> =
        api_post(&format!("/bantuan/tiket/{id}/komentar"), &body).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

/// `PUT /bantuan/tiket/{id}/status` — helpdesk staff only (403 otherwise).
pub async fn update_status(id: &str, status: &str) -> AppResult<SupportTicket> {
    let body = UpdateStatusRequest {
        status: status.to_string(),
    };
    let resp: ApiResponseWrap<SupportTicket> =
        api_put(&format!("/bantuan/tiket/{id}/status"), &body).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}
