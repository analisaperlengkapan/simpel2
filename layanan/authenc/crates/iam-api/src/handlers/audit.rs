//! Audit log HTTP handlers
//!
//! Reads the real `authenc.audit_logs` table (written by the API's security
//! monitoring middleware). The response contract matches the portal's
//! `AuditLogEntry` / `PaginatedResponse` shapes — the previous handler was a
//! silent-empty stub, so the portal defined the contract first.

use axum::{
    Json,
    extract::{Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_postgres::types::ToSql;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::IamApiState;
use authenc_types::AuthencError;

/// Hard cap on export size — the export endpoint streams a single response,
/// not a cursor, so it must be bounded.
const EXPORT_ROW_LIMIT: i64 = 10_000;

/// Query parameters (field names are the portal's `AuditLogQuery`)
#[derive(Debug, Deserialize)]
pub struct ListAuditLogsQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub per_page: u32,
    pub event_type: Option<String>,
    pub user_id: Option<Uuid>,
    /// ISO 8601 lower bound (inclusive)
    pub from_date: Option<chrono::DateTime<chrono::Utc>>,
    /// ISO 8601 upper bound (inclusive)
    pub to_date: Option<chrono::DateTime<chrono::Utc>>,
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    50
}

/// Audit log entry (portal `AuditLogEntry` shape)
#[derive(Debug, Serialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub event_type: String,
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub details: Option<String>,
    pub realm: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub success: bool,
}

/// Paginated audit logs (portal `PaginatedResponse` shape)
#[derive(Debug, Serialize)]
pub struct PaginatedAuditLogs {
    pub data: Vec<AuditLogEntry>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

/// Export format query parameter
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    /// Export format: "csv" or "json"
    #[serde(default = "default_format")]
    pub format: String,

    #[serde(flatten)]
    pub filters: ListAuditLogsQuery,
}

fn default_format() -> String {
    "json".to_string()
}

/// Build the WHERE clause + owned params for the shared filter set.
fn build_filters(q: &ListAuditLogsQuery) -> (String, Vec<Box<dyn ToSql + Sync + Send>>) {
    let mut clauses: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn ToSql + Sync + Send>> = Vec::new();

    if let Some(ref et) = q.event_type {
        params.push(Box::new(et.clone()));
        clauses.push(format!("a.event_type = ${}", params.len()));
    }
    if let Some(uid) = q.user_id {
        params.push(Box::new(uid));
        clauses.push(format!("a.user_id = ${}", params.len()));
    }
    if let Some(from) = q.from_date {
        params.push(Box::new(from));
        clauses.push(format!("a.\"timestamp\" >= ${}", params.len()));
    }
    if let Some(to) = q.to_date {
        params.push(Box::new(to));
        clauses.push(format!("a.\"timestamp\" <= ${}", params.len()));
    }

    let where_sql = if clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", clauses.join(" AND "))
    };
    (where_sql, params)
}

fn row_to_entry(row: &tokio_postgres::Row) -> AuditLogEntry {
    let details: Option<serde_json::Value> = row.get("details");
    let ip: Option<std::net::IpAddr> = row.get("ip_address");
    let status: String = row.get("status");
    AuditLogEntry {
        id: row.get("id"),
        event_type: row.get("event_type"),
        user_id: row.get("user_id"),
        username: row.get("username"),
        ip_address: ip.map(|i| i.to_string()),
        details: details.map(|d| d.to_string()),
        realm: None,
        timestamp: row.get("timestamp"),
        success: status == "success",
    }
}

const ENTRY_SELECT: &str = r#"
    SELECT a.id, a.event_type, a.user_id, u.username, a.ip_address,
           a.details, a."timestamp", a.status
    FROM audit_logs a
    LEFT JOIN users u ON u.id = a.user_id
"#;

async fn query_entries(
    state: &IamApiState,
    filters: &ListAuditLogsQuery,
    limit: i64,
    offset: i64,
) -> Result<Vec<AuditLogEntry>, AuthencError> {
    let (where_sql, params) = build_filters(filters);
    let sql = format!(
        "{ENTRY_SELECT} {where_sql} ORDER BY a.\"timestamp\" DESC LIMIT ${} OFFSET ${}",
        params.len() + 1,
        params.len() + 2,
    );
    let mut refs: Vec<&(dyn ToSql + Sync)> = params
        .iter()
        .map(|p| p.as_ref() as &(dyn ToSql + Sync))
        .collect();
    refs.push(&limit);
    refs.push(&offset);

    let rows = state.database.query(&sql, &refs).await?;
    Ok(rows.iter().map(row_to_entry).collect())
}

/// GET /api/v1/iam/audit-logs - List audit logs with filters
pub async fn list_audit_logs(
    State(state): State<Arc<IamApiState>>,
    Query(params): Query<ListAuditLogsQuery>,
) -> ApiResult<Json<PaginatedAuditLogs>> {
    let per_page = params.per_page.clamp(1, 200);
    let page = params.page.max(1);

    let (where_sql, count_params) = build_filters(&params);
    let count_sql = format!("SELECT COUNT(*) FROM audit_logs a {where_sql}");
    let count_refs: Vec<&(dyn ToSql + Sync)> = count_params
        .iter()
        .map(|p| p.as_ref() as &(dyn ToSql + Sync))
        .collect();
    let total: i64 = state
        .database
        .query_one(&count_sql, &count_refs)
        .await
        .map_err(crate::error::ApiError)?
        .get(0);

    let data = query_entries(
        &state,
        &params,
        per_page as i64,
        ((page - 1) * per_page) as i64,
    )
    .await
    .map_err(crate::error::ApiError)?;

    Ok(Json(PaginatedAuditLogs {
        data,
        total: total as u64,
        page,
        per_page,
        total_pages: (total as u32).div_ceil(per_page),
    }))
}

/// GET /api/v1/iam/audit-logs/export - Export audit logs (CSV/JSON)
pub async fn export_audit_logs(
    State(state): State<Arc<IamApiState>>,
    Query(params): Query<ExportQuery>,
) -> ApiResult<Response> {
    let entries = query_entries(&state, &params.filters, EXPORT_ROW_LIMIT, 0)
        .await
        .map_err(crate::error::ApiError)?;

    match params.format.as_str() {
        "csv" => {
            let mut csv =
                String::from("timestamp,event_type,user_id,username,ip_address,success,details\n");
            for e in &entries {
                // Quote free-text fields; details JSON may contain commas.
                csv.push_str(&format!(
                    "{},{},{},{},{},{},\"{}\"\n",
                    e.timestamp.to_rfc3339(),
                    e.event_type,
                    e.user_id.map(|u| u.to_string()).unwrap_or_default(),
                    e.username.clone().unwrap_or_default(),
                    e.ip_address.clone().unwrap_or_default(),
                    e.success,
                    e.details.clone().unwrap_or_default().replace('"', "\"\""),
                ));
            }
            Ok((StatusCode::OK, [(header::CONTENT_TYPE, "text/csv")], csv).into_response())
        }
        "json" => {
            let body = serde_json::json!({
                "logs": entries,
                "exported_at": chrono::Utc::now(),
            });
            Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                serde_json::to_string(&body).unwrap_or_else(|_| "{}".to_string()),
            )
                .into_response())
        }
        _ => Err(crate::error::ApiError(AuthencError::validation(
            "Invalid format. Use 'csv' or 'json'",
        ))),
    }
}
