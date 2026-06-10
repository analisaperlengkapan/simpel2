//! Frontend client for `/api/v1/perlengkapan/notifikasi/*`.
//!
//! Mirrors the slim `NotifikasiItem` payload the backend emits — title,
//! message, category etc. are pre-formatted server-side so the page just
//! renders rows. Used by:
//! - `pages::notifikasi::NotifikasiInboxPage` for the inbox.
//! - `app_chrome` for the toolbar unread-count badge (poll every 30s).

use serde::{Deserialize, Serialize};

use crate::api::client::{api_get, api_patch_empty, api_post_empty};
use crate::api::error::{AppError, AppResult};

#[derive(Debug, Clone, Deserialize)]
struct ApiResponseWrap<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct UnreadCount {
    pub count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarkAllRead {
    pub updated: u64,
}

/// `GET /notifikasi/unread-count`
pub async fn fetch_unread_count() -> AppResult<i64> {
    let resp: ApiResponseWrap<UnreadCount> = api_get("/notifikasi/unread-count").await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data.count)
}

/// `GET /notifikasi?limit=&offset=&unread_only=`
pub async fn list_notifikasi(
    limit: i64,
    offset: i64,
    unread_only: bool,
) -> AppResult<Vec<NotifikasiItem>> {
    let url = format!("/notifikasi?limit={limit}&offset={offset}&unread_only={unread_only}");
    let resp: ApiResponseWrap<Vec<NotifikasiItem>> = api_get(&url).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

/// `PATCH /notifikasi/{id}/read`
pub async fn mark_read(id: &str) -> AppResult<()> {
    let url = format!("/notifikasi/{id}/read");
    let _: serde_json::Value = api_patch_empty(&url).await?;
    Ok(())
}

/// `POST /notifikasi/read-all`
pub async fn mark_all_read() -> AppResult<u64> {
    let resp: ApiResponseWrap<MarkAllRead> = api_post_empty("/notifikasi/read-all").await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data.updated)
}
