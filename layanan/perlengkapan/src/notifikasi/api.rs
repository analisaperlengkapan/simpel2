//! Unified `/notifikasi/*` REST handlers.
//!
//! Surfaces the rows that `NotifikasiService` writes into
//! `notifikasi.in_app_notifications` so the frontend notification center
//! and the toolbar unread-badge can fetch them. Every endpoint scopes its
//! query by `claims.user_id` — admins look at their own inbox, not the
//! whole tenant.

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::ApiResponse;
use crate::notifikasi::in_app::InAppNotificationChannel;
use crate::shared::error::{AppError, AppResult};
use crate::shared::middleware::Claims;
use crate::state::AppState;

/// Serializable DTO mirroring the `notifikasi.in_app_notifications` columns
/// the UI actually needs. We don't expose `metadata` raw — the frontend
/// only cares about title/message/category/action_url to render the row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifikasiItem {
    pub id: String,
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub priority: String,
    pub category: String,
    pub action_url: Option<String>,
    pub read: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListNotifikasiQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    #[serde(default)]
    pub unread_only: bool,
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Serialize)]
pub struct UnreadCountResponse {
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct MarkAllReadResponse {
    pub updated: u64,
}

fn channel(state: &AppState) -> InAppNotificationChannel {
    InAppNotificationChannel::new(state.db_pool.clone())
}

fn lift(e: crate::notifikasi::error::AppError) -> AppError {
    AppError::Internal(format!("notifikasi: {e}"))
}

/// GET /notifikasi/unread-count
pub async fn unread_count(
    State(state): State<AppState>,
    claims: Claims,
) -> AppResult<Json<ApiResponse<UnreadCountResponse>>> {
    let count = channel(&state)
        .get_unread_count(claims.user_id)
        .await
        .map_err(lift)?;
    Ok(Json(ApiResponse::success(
        UnreadCountResponse { count },
        "Unread count retrieved".to_string(),
    )))
}

/// GET /notifikasi?limit=&offset=&unread_only=
pub async fn list_notifikasi(
    State(state): State<AppState>,
    claims: Claims,
    Query(q): Query<ListNotifikasiQuery>,
) -> AppResult<Json<ApiResponse<Vec<NotifikasiItem>>>> {
    let limit = q.limit.clamp(1, 200);
    let offset = q.offset.max(0);
    let rows = channel(&state)
        .get_notifications(claims.user_id, limit, offset, q.unread_only)
        .await
        .map_err(lift)?;
    let items = rows
        .into_iter()
        .map(|n| NotifikasiItem {
            id: n.id.to_string(),
            notification_type: n.notification_type,
            title: n.title,
            message: n.message,
            priority: n.priority,
            category: n.category,
            action_url: n.action_url,
            read: n.read,
            created_at: n.created_at.to_rfc3339(),
        })
        .collect();
    Ok(Json(ApiResponse::success(
        items,
        "Notifications retrieved".to_string(),
    )))
}

/// PATCH /notifikasi/{id}/read
pub async fn mark_read(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    claims: Claims,
) -> AppResult<Json<ApiResponse<NotifikasiItem>>> {
    let ch = channel(&state);
    ch.mark_as_read(id, claims.user_id).await.map_err(lift)?;
    // Refetch the row so the caller has the updated state without a second
    // round-trip; if it isn't in the visible scope anymore (shouldn't happen
    // — same user_id), fall back to a minimal stub.
    let rows = ch
        .get_notifications(claims.user_id, 1, 0, false)
        .await
        .map_err(lift)?;
    let updated = rows
        .into_iter()
        .find(|n| n.id == id)
        .map(|n| NotifikasiItem {
            id: n.id.to_string(),
            notification_type: n.notification_type,
            title: n.title,
            message: n.message,
            priority: n.priority,
            category: n.category,
            action_url: n.action_url,
            read: n.read,
            created_at: n.created_at.to_rfc3339(),
        })
        .unwrap_or(NotifikasiItem {
            id: id.to_string(),
            notification_type: String::new(),
            title: String::new(),
            message: String::new(),
            priority: "normal".into(),
            category: "info".into(),
            action_url: None,
            read: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        });
    Ok(Json(ApiResponse::success(
        updated,
        "Notification marked as read".to_string(),
    )))
}

/// POST /notifikasi/read-all
pub async fn mark_all_read(
    State(state): State<AppState>,
    claims: Claims,
) -> AppResult<Json<ApiResponse<MarkAllReadResponse>>> {
    let updated = channel(&state)
        .mark_all_as_read(claims.user_id)
        .await
        .map_err(lift)?;
    Ok(Json(ApiResponse::success(
        MarkAllReadResponse { updated },
        "All notifications marked as read".to_string(),
    )))
}
