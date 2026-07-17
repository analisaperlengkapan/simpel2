//! Frontend client for the `/admin/templates` endpoints.
//!
//! Mirrors the backend `DocumentTemplate` shape closely enough for the list
//! page; the editor will widen the type when it lands. Preview returns raw
//! bytes so the UI can drop them into a blob URL + `<iframe>`.

use serde::{Deserialize, Serialize};

use crate::api::client::{api_get, api_post_binary};
use crate::api::error::{AppError, AppResult};

#[derive(Debug, Clone, Deserialize)]
struct ApiResponseWrap<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

/// Subset of the backend's template payload that the admin page renders;
/// serde ignores the fields we don't need.
#[derive(Debug, Clone, Deserialize)]
pub struct DocumentTemplate {
    pub id: String,
    pub name: String,
    pub template_type: String,
    pub version: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListTemplatesResponse {
    pub templates: Vec<DocumentTemplate>,
}

/// `GET /admin/templates?template_type=...&is_active=...&page=...&per_page=...`
pub async fn list_templates(
    template_type: Option<&str>,
    is_active: Option<bool>,
    page: i64,
    per_page: i64,
) -> AppResult<ListTemplatesResponse> {
    let mut url = format!("/admin/templates?page={page}&per_page={per_page}");
    if let Some(tt) = template_type {
        if !tt.is_empty() {
            url.push_str(&format!("&template_type={}", url_encode(tt)));
        }
    }
    if let Some(ia) = is_active {
        url.push_str(&format!("&is_active={ia}"));
    }
    let resp: ApiResponseWrap<ListTemplatesResponse> = api_get(&url).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct PreviewBody {
    /// Free-form JSON the template engine renders against. When `None` the
    /// backend falls back to the template's stored `sample_data`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Render a live preview of `template_id` in `format` (defaults to PDF on
/// the server when omitted). Returns the raw bytes the caller drops into a
/// blob URL + `<iframe>`.
pub async fn preview_template(
    template_id: &str,
    format: &str,
    body: &PreviewBody,
) -> AppResult<Vec<u8>> {
    let url = format!("/admin/templates/{template_id}/preview?format={format}");
    api_post_binary(&url, body).await
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
            out.push(ch);
        } else {
            for byte in ch.to_string().as_bytes() {
                out.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    out
}
