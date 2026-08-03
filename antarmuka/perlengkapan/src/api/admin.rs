//! Frontend API for admin-only endpoints: audit log viewer and master data hub.
//!
//! Backend endpoints are scheduled to land in commit 19 of the refactor plan.
//! The shapes here mirror what the backend is expected to return so the pages
//! can be implemented against a stable contract and simply switch from empty
//! states to real data once the endpoints are live.

use serde::{Deserialize, Serialize};

use crate::api::client::{API_BASE, api_delete_empty, api_get, api_post, api_put};
use crate::api::common::PaginatedResponse;
use crate::api::error::{AppError, AppResult};

// ═════════════════════════════════════════════════════════════════════════
// Audit log viewer
// ═════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditLogEntry {
    pub id: String,
    pub occurred_at: String,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub actor_role: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Default, Debug, Clone)]
pub struct AuditFilter {
    pub page: i32,
    pub per_page: i32,
    pub actor: Option<String>,
    pub entity_type: Option<String>,
    pub action: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub search: Option<String>,
}

fn push_query(buf: &mut String, key: &str, value: &str) {
    if buf.contains('?') {
        buf.push('&');
    } else {
        buf.push('?');
    }
    buf.push_str(key);
    buf.push('=');
    buf.push_str(&urlencoding::encode(value));
}

pub async fn fetch_audit_logs(filter: &AuditFilter) -> AppResult<PaginatedResponse<AuditLogEntry>> {
    let mut url = format!("{API_BASE}/admin/audit");
    let page = if filter.page < 1 { 1 } else { filter.page };
    let per_page = if filter.per_page < 1 {
        25
    } else {
        filter.per_page
    };
    push_query(&mut url, "page", &page.to_string());
    push_query(&mut url, "per_page", &per_page.to_string());
    if let Some(v) = &filter.actor {
        if !v.is_empty() {
            push_query(&mut url, "actor", v);
        }
    }
    if let Some(v) = &filter.entity_type {
        if !v.is_empty() {
            push_query(&mut url, "entity_type", v);
        }
    }
    if let Some(v) = &filter.action {
        if !v.is_empty() {
            push_query(&mut url, "action", v);
        }
    }
    if let Some(v) = &filter.date_from {
        if !v.is_empty() {
            push_query(&mut url, "from", v);
        }
    }
    if let Some(v) = &filter.date_to {
        if !v.is_empty() {
            push_query(&mut url, "to", v);
        }
    }
    if let Some(v) = &filter.search {
        if !v.is_empty() {
            push_query(&mut url, "q", v);
        }
    }
    api_get::<PaginatedResponse<AuditLogEntry>>(&url).await
}

// ═════════════════════════════════════════════════════════════════════════
// Master data hub
// ═════════════════════════════════════════════════════════════════════════

/// Catalog entry for one master-data source that the hub page lists. The
/// backend returns the full list so the frontend doesn't have to hardcode
/// which tables exist — new sources show up automatically.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MasterSource {
    pub key: String,
    pub label: String,
    pub description: String,
    pub icon: Option<String>,
    pub record_count: i64,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Generic master record: backend flattens each table into this shape so one
/// list/detail screen can serve every master-data source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MasterRecord {
    pub id: String,
    pub code: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub extra: serde_json::Value,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterUpsertRequest {
    pub code: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub active: bool,
    #[serde(default)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Wrap<T> {
    pub success: bool,
    pub data: T,
    #[serde(default)]
    pub message: String,
}

pub async fn fetch_master_sources() -> AppResult<Vec<MasterSource>> {
    let url = format!("{API_BASE}/admin/master");
    let resp: Wrap<Vec<MasterSource>> = api_get(&url).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

pub async fn fetch_master_records(
    source: &str,
    page: i32,
    per_page: i32,
    search: Option<&str>,
) -> AppResult<PaginatedResponse<MasterRecord>> {
    let mut url = format!("{API_BASE}/admin/master/{source}");
    let p = if page < 1 { 1 } else { page };
    let pp = if per_page < 1 { 25 } else { per_page };
    push_query(&mut url, "page", &p.to_string());
    push_query(&mut url, "per_page", &pp.to_string());
    if let Some(q) = search {
        if !q.is_empty() {
            push_query(&mut url, "q", q);
        }
    }
    api_get::<PaginatedResponse<MasterRecord>>(&url).await
}

pub async fn create_master_record(
    source: &str,
    req: &MasterUpsertRequest,
) -> AppResult<MasterRecord> {
    let url = format!("{API_BASE}/admin/master/{source}");
    let resp: Wrap<MasterRecord> = api_post(&url, req).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

pub async fn update_master_record(
    source: &str,
    id: &str,
    req: &MasterUpsertRequest,
) -> AppResult<MasterRecord> {
    let url = format!("{API_BASE}/admin/master/{source}/{id}");
    let resp: Wrap<MasterRecord> = api_put(&url, req).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

pub async fn delete_master_record(source: &str, id: &str) -> AppResult<()> {
    let url = format!("{API_BASE}/admin/master/{source}/{id}");
    api_delete_empty(&url).await
}

// NOTE: no user-catalog / role-assignment client here. The `/admin/users*`
// endpoints it called have been removed: they queried relations no migration
// creates, and user + role administration belongs to authenc (SSoT). See
// `layanan/perlengkapan/src/admin/mod.rs`.
