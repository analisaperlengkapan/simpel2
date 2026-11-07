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
//
// Reads the cross-module trail `GET /audit` (`perlengkapan.audit_log`), the
// table every module's `AuditSink` writes. This page used to call
// `/admin/audit`, which read two workflow tables nothing ever fills, so it was
// empty everywhere. The DTO below mirrors `crate::audit::models` field for
// field; `infra/scripts/check-fe-be-dto-drift.py` pairs them by name.

/// One row of `perlengkapan.audit_log`, as `GET /audit` sends it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditTrailEntry {
    pub id: String,
    pub occurred_at: String,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub actor_ip: Option<String>,
    /// Stable verb (`create`, `approve`, `custom`, ...).
    pub action: String,
    /// Free-form name for `custom` actions (e.g. `workflow.transition`).
    #[serde(default)]
    pub action_name: Option<String>,
    /// Kind of record acted on (`tiket`, `penghapusan_bmn`, ...).
    pub resource_type: String,
    #[serde(default)]
    pub resource_id: Option<String>,
    pub module: String,
    pub success: bool,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// The values that occur in the trail (`GET /audit/facets`) — the only options
/// the filter dropdowns offer. Mirrors `crate::audit::models::AuditFacets`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditFacets {
    pub modules: Vec<String>,
    pub entities: Vec<String>,
    pub actions: Vec<String>,
}

#[derive(Default, Debug, Clone)]
pub struct AuditFilter {
    pub page: i32,
    pub per_page: i32,
    pub actor: Option<String>,
    pub module: Option<String>,
    pub entity: Option<String>,
    pub action: Option<String>,
    /// `YYYY-MM-DD` from an `<input type="date">`, as the viewer sees the day.
    pub date_from: Option<String>,
    /// `YYYY-MM-DD`, inclusive of the whole day.
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

/// First (`end_of_day == false`) or last instant of a calendar day, as RFC 3339.
///
/// `GET /audit` filters on `occurred_at` with RFC 3339 instants and ignores
/// anything else — sending the bare `YYYY-MM-DD` an `<input type="date">` yields
/// would return every row while the page claimed a date range was applied. The
/// day is the viewer's own (`Local`), the same zone `format_iso_local` shows the
/// timestamps in, so "6 Nov" in the filter means the "6 Nov" printed in the table.
fn day_bound(date: &str, end_of_day: bool) -> Option<String> {
    use chrono::{Local, NaiveDate, TimeZone};
    let day = NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d").ok()?;
    let at = if end_of_day {
        day.and_hms_milli_opt(23, 59, 59, 999)?
    } else {
        day.and_hms_opt(0, 0, 0)?
    };
    Local
        .from_local_datetime(&at)
        .earliest()
        .map(|t| t.to_rfc3339())
}

/// The request URL for `filter`. Pure, so the exact query keys can be asserted.
fn audit_url(filter: &AuditFilter) -> String {
    let mut url = format!("{API_BASE}/audit");
    let page = if filter.page < 1 { 1 } else { filter.page };
    let per_page = if filter.per_page < 1 {
        25
    } else {
        filter.per_page
    };
    push_query(&mut url, "page", &page.to_string());
    push_query(&mut url, "per_page", &per_page.to_string());
    for (key, value) in [
        ("actor", &filter.actor),
        ("module", &filter.module),
        ("entity", &filter.entity),
        ("action", &filter.action),
        ("q", &filter.search),
    ] {
        if let Some(v) = value.as_deref().filter(|v| !v.trim().is_empty()) {
            push_query(&mut url, key, v.trim());
        }
    }
    if let Some(v) = filter
        .date_from
        .as_deref()
        .and_then(|d| day_bound(d, false))
    {
        push_query(&mut url, "from", &v);
    }
    if let Some(v) = filter.date_to.as_deref().and_then(|d| day_bound(d, true)) {
        push_query(&mut url, "to", &v);
    }
    url
}

pub async fn fetch_audit_logs(
    filter: &AuditFilter,
) -> AppResult<PaginatedResponse<AuditTrailEntry>> {
    api_get::<PaginatedResponse<AuditTrailEntry>>(&audit_url(filter)).await
}

/// The modules / entities / actions that occur in the trail, for the dropdowns.
pub async fn fetch_audit_facets() -> AppResult<AuditFacets> {
    let url = format!("{API_BASE}/audit/facets");
    let resp: Wrap<AuditFacets> = api_get(&url).await?;
    if !resp.success {
        return Err(AppError::server(resp.message));
    }
    Ok(resp.data)
}

#[cfg(test)]
mod audit_tests {
    use super::*;
    use chrono::{DateTime, Local};

    /// The bound must land on the viewer's own midnight / last millisecond of the
    /// chosen day whatever zone the machine runs in (asserted by converting back
    /// to `Local`, so the test passes in any `TZ`).
    #[test]
    fn day_bounds_are_the_viewers_whole_day() {
        let local = |s: &str| {
            DateTime::parse_from_rfc3339(s)
                .expect("RFC 3339")
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S%.3f")
                .to_string()
        };
        assert_eq!(
            local(&day_bound("2026-11-06", false).unwrap()),
            "2026-11-06 00:00:00.000"
        );
        assert_eq!(
            local(&day_bound("2026-11-06", true).unwrap()),
            "2026-11-06 23:59:59.999"
        );
    }

    /// Not a date → no bound, rather than a malformed one the server would drop.
    #[test]
    fn a_non_date_yields_no_bound() {
        assert_eq!(day_bound("", false), None);
        assert_eq!(day_bound("06/11/2026", true), None);
    }

    /// The keys must be the ones the backend's `AuditTrailQuery` reads. serde drops
    /// an unknown key silently, so a misspelt one (`entity_type`, the old name)
    /// would answer 200 with every row and the filter would look applied.
    #[test]
    fn the_query_uses_the_keys_the_trail_endpoint_reads() {
        let url = audit_url(&AuditFilter {
            page: 2,
            per_page: 25,
            module: Some("bantuan".into()),
            entity: Some("tiket".into()),
            action: Some("create".into()),
            actor: Some(" budi ".into()),
            search: Some("rusak".into()),
            date_from: Some("2026-11-06".into()),
            date_to: Some("2026-11-07".into()),
        });
        assert!(url.contains("/audit?page=2&per_page=25"), "{url}");
        for pair in [
            "module=bantuan",
            "entity=tiket",
            "action=create",
            "actor=budi",
            "q=rusak",
        ] {
            assert!(url.contains(pair), "{pair} missing from {url}");
        }
        assert!(!url.contains("entity_type"), "{url}");
        // Both date bounds travel as instants the server accepts, not bare dates.
        // (RFC 3339 keeps the viewer's local date, so this holds in any `TZ`.)
        assert!(url.contains("&from=2026-11-06T00%3A00%3A00"), "{url}");
        assert!(url.contains("&to=2026-11-07T23%3A59%3A59"), "{url}");
    }

    /// Nothing chosen → nothing sent beyond paging.
    #[test]
    fn an_empty_filter_sends_only_paging() {
        let url = audit_url(&AuditFilter {
            page: 0,
            per_page: 0,
            action: Some("  ".into()),
            ..Default::default()
        });
        assert!(url.ends_with("/audit?page=1&per_page=25"), "{url}");
    }
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
