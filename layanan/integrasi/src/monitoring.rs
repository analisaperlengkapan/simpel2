//! Monitoring module for integration service
//!
//! Provides REST API endpoints for monitoring sync status, viewing sync history,
//! and alerting on sync failures.
//!
//! # No request value is ever part of the SQL text
//!
//! These endpoints answer without authentication (the pod's port 8080; who can reach it is decided
//! by NetworkPolicy alone). `source`, `days` and `limit` used to be pasted into the statement with
//! `format!`: `?source=x' OR '1'='1` answered with every module, and a `UNION SELECT` with any
//! table the service role can read. Every statement in this file is now a constant, and a request
//! value reaches Postgres as a bind parameter or not at all. `tests/monitoring_sql_params.rs`
//! holds that against a real database.

use crate::db_handle::DbHandle;
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::{Error as DbError, Row};
use tracing::{error, info};

/// Bounds of `/api/monitoring/sync-history`. A value outside them is answered `400`, not clamped:
/// a caller that asked for 5000 rows and silently got 1000 has been told something untrue.
pub const MAX_HISTORY_DAYS: i32 = 3650;
pub const MAX_HISTORY_LIMIT: i64 = 1000;

/// Per-module totals. `$1` = module, or NULL for every module.
const SYNC_STATUS_SQL: &str = r#"
    SELECT
        module as source,
        MAX(started_at) as last_sync_at,
        COALESCE(SUM(record_count), 0) as total_records,
        COUNT(*) as total_syncs,
        SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful_syncs
    FROM integrasi.api_log
    WHERE ($1::text IS NULL OR module = $1)
    GROUP BY module
    ORDER BY module
"#;

/// The log itself, newest first. `$1` = module or NULL, `$2` = days to look back or NULL for no
/// bound, `$3` = row limit.
const SYNC_HISTORY_SQL: &str = r#"
    SELECT
        id, module as source, endpoint,
        COALESCE(record_count, 0) as records_fetched,
        success, error_message, started_at as sync_started_at,
        EXTRACT(EPOCH FROM (completed_at - started_at))::INTEGER as duration_seconds
    FROM integrasi.api_log
    WHERE ($1::text IS NULL OR module = $1)
      AND ($2::integer IS NULL OR started_at >= CURRENT_TIMESTAMP - make_interval(days => $2))
    ORDER BY started_at DESC, id DESC
    LIMIT $3::bigint
"#;

/// Modules with 3 or more failures in the last 24 hours, or 2 or more among their 5 latest calls.
///
/// `consecutive_failures` is cast to INTEGER on purpose. `COUNT(*)` is a `bigint`; it used to be
/// read into an `i32` behind `try_get(..).unwrap_or(0)`, which turned the type error into a zero:
/// every alert said `consecutive_failures: 0` and no alert could ever be raised to "high" or
/// "critical" by it.
const ALERTS_SQL: &str = r#"
    WITH recent_failures AS (
        SELECT
            module as source,
            COUNT(*) as failure_count,
            MAX(started_at) as last_failure_at,
            MAX(error_message) as last_error
        FROM integrasi.api_log
        WHERE success = false
            AND started_at >= CURRENT_TIMESTAMP - INTERVAL '24 hours'
        GROUP BY module
    ),
    consecutive_failures AS (
        SELECT
            module as source,
            COUNT(*) as consecutive_count
        FROM (
            SELECT
                module,
                success,
                ROW_NUMBER() OVER (PARTITION BY module ORDER BY started_at DESC) as rn
            FROM integrasi.api_log
            WHERE started_at >= CURRENT_TIMESTAMP - INTERVAL '24 hours'
        ) sub
        WHERE success = false AND rn <= 5
        GROUP BY module
    )
    SELECT
        rf.source,
        rf.failure_count,
        rf.last_failure_at,
        rf.last_error,
        COALESCE(cf.consecutive_count, 0)::INTEGER as consecutive_failures
    FROM recent_failures rf
    LEFT JOIN consecutive_failures cf ON rf.source = cf.source
    WHERE rf.failure_count >= 3 OR COALESCE(cf.consecutive_count, 0) >= 2
    ORDER BY rf.failure_count DESC, rf.last_failure_at DESC
"#;

/// Today's totals, one row even when nothing ran today (hence the COALESCEs: `SUM` over no rows
/// is NULL).
const TODAY_STATS_SQL: &str = r#"
    SELECT
        COUNT(*) as total_syncs,
        COALESCE(SUM(CASE WHEN success THEN 1 ELSE 0 END), 0) as successful_syncs,
        COALESCE(SUM(CASE WHEN NOT success THEN 1 ELSE 0 END), 0) as failed_syncs
    FROM integrasi.api_log
    WHERE started_at >= CURRENT_DATE
"#;

/// Shared state for monitoring endpoints
#[derive(Clone)]
pub struct MonitoringState {
    pub db_client: Arc<DbHandle>,
}

/// Query parameters for sync status endpoint
#[derive(Debug, Deserialize)]
pub struct SyncStatusQuery {
    /// Data source filter (monsakti, mysimkari, siman)
    pub source: Option<String>,
}

/// Query parameters for sync history endpoint
#[derive(Debug, Deserialize)]
pub struct SyncHistoryQuery {
    /// Number of days to look back (default: 7)
    pub days: Option<i32>,
    /// Data source filter
    pub source: Option<String>,
    /// Limit number of results (default: 100)
    pub limit: Option<i64>,
}

/// Sync status response
#[derive(Debug, Serialize)]
pub struct SyncStatusResponse {
    pub source: String,
    pub last_sync_at: Option<String>,
    pub next_sync_at: Option<String>,
    pub state: String,
    pub records_synced: i64,
    pub success_rate: f64,
    pub error_message: Option<String>,
}

/// Sync history item
#[derive(Debug, Serialize)]
pub struct SyncHistoryItem {
    pub id: i64,
    pub source: String,
    pub table_name: String,
    pub module: String,
    pub endpoint: String,
    pub records_fetched: i32,
    pub records_inserted: i32,
    pub records_updated: i32,
    pub records_failed: i32,
    pub success: bool,
    pub error_message: Option<String>,
    pub sync_started_at: String,
    pub duration_seconds: Option<i32>,
}

/// Sync alert item
#[derive(Debug, Serialize)]
pub struct SyncAlert {
    pub id: i64,
    pub source: String,
    pub severity: String,
    pub message: String,
    pub error_details: Option<String>,
    pub occurred_at: String,
    pub consecutive_failures: i32,
}

/// Dashboard metrics response
#[derive(Debug, Serialize)]
pub struct DashboardMetrics {
    pub total_syncs_today: i64,
    pub successful_syncs_today: i64,
    pub failed_syncs_today: i64,
    pub success_rate_today: f64,
    pub active_alerts: Vec<SyncAlert>,
    pub sync_status_by_source: HashMap<String, SyncStatusResponse>,
    pub recent_sync_history: Vec<SyncHistoryItem>,
}

/// Create monitoring router
pub fn create_monitoring_router(state: MonitoringState) -> Router {
    Router::new()
        .route("/api/monitoring/sync-status", get(get_sync_status))
        .route("/api/monitoring/sync-history", get(get_sync_history))
        .route("/api/monitoring/alerts", get(get_alerts))
        .route("/api/monitoring/dashboard", get(get_dashboard_metrics))
        .route("/api/monitoring/health", get(health_check))
        .with_state(state)
}

// ---------------------------------------------------------------------------------------------
// Readers. One per statement, shared by the endpoint and by the dashboard, so a number has one
// definition. A column of the wrong type is an ERROR here (`?`), never a default value.
// ---------------------------------------------------------------------------------------------

fn status_from_row(row: &Row) -> Result<SyncStatusResponse, DbError> {
    let last_sync_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("last_sync_at")?;
    let total_syncs: i64 = row.try_get("total_syncs")?;
    let successful_syncs: i64 = row.try_get("successful_syncs")?;

    let success_rate = if total_syncs > 0 {
        (successful_syncs as f64 / total_syncs as f64) * 100.0
    } else {
        0.0
    };

    Ok(SyncStatusResponse {
        source: row.try_get("source")?,
        last_sync_at: last_sync_at.map(|dt| dt.to_rfc3339()),
        next_sync_at: None, // Will be populated from K8s CronJob schedule
        state: if success_rate > 90.0 {
            "healthy".to_string()
        } else if success_rate > 50.0 {
            "degraded".to_string()
        } else {
            "unhealthy".to_string()
        },
        records_synced: row.try_get("total_records")?,
        success_rate,
        error_message: None,
    })
}

async fn fetch_sync_status(
    db: &DbHandle,
    source: Option<&str>,
) -> Result<Vec<SyncStatusResponse>, DbError> {
    let rows = db.query(SYNC_STATUS_SQL, &[&source]).await?;
    rows.iter().map(status_from_row).collect()
}

fn history_from_row(row: &Row) -> Result<SyncHistoryItem, DbError> {
    let source: String = row.try_get("source")?;
    // `endpoint`, `success` and `error_message` are nullable columns; `completed_at` is NULL for a
    // call that never finished, and so is the duration computed from it.
    let endpoint: Option<String> = row.try_get("endpoint")?;
    let success: Option<bool> = row.try_get("success")?;
    let started_at: chrono::DateTime<chrono::Utc> = row.try_get("sync_started_at")?;

    Ok(SyncHistoryItem {
        id: row.try_get("id")?,
        module: source.clone(),
        source,
        table_name: String::new(),
        endpoint: endpoint.unwrap_or_default(),
        records_fetched: row.try_get("records_fetched")?,
        records_inserted: 0,
        records_updated: 0,
        records_failed: 0,
        success: success.unwrap_or(false),
        error_message: row.try_get("error_message")?,
        sync_started_at: started_at.to_rfc3339(),
        duration_seconds: row.try_get("duration_seconds")?,
    })
}

async fn fetch_sync_history(
    db: &DbHandle,
    source: Option<&str>,
    days: Option<i32>,
    limit: i64,
) -> Result<Vec<SyncHistoryItem>, DbError> {
    let rows = db
        .query(SYNC_HISTORY_SQL, &[&source, &days, &limit])
        .await?;
    rows.iter().map(history_from_row).collect()
}

fn alert_from_row(idx: usize, row: &Row) -> Result<SyncAlert, DbError> {
    let source: String = row.try_get("source")?;
    let failure_count: i64 = row.try_get("failure_count")?;
    let consecutive_failures: i32 = row.try_get("consecutive_failures")?;
    let last_failure_at: chrono::DateTime<chrono::Utc> = row.try_get("last_failure_at")?;

    let severity = if consecutive_failures >= 5 || failure_count >= 10 {
        "critical"
    } else if consecutive_failures >= 3 || failure_count >= 5 {
        "high"
    } else {
        "medium"
    };

    let message = if consecutive_failures >= 2 {
        format!(
            "{} consecutive sync failures for {}",
            consecutive_failures, source
        )
    } else {
        format!("{} sync failures in last 24h for {}", failure_count, source)
    };

    Ok(SyncAlert {
        id: idx as i64 + 1,
        source,
        severity: severity.to_string(),
        message,
        error_details: row.try_get("last_error")?,
        occurred_at: last_failure_at.to_rfc3339(),
        consecutive_failures,
    })
}

async fn fetch_alerts(db: &DbHandle) -> Result<Vec<SyncAlert>, DbError> {
    let rows = db.query(ALERTS_SQL, &[]).await?;
    rows.iter()
        .enumerate()
        .map(|(idx, row)| alert_from_row(idx, row))
        .collect()
}

async fn fetch_today_stats(db: &DbHandle) -> Result<(i64, i64, i64), DbError> {
    let row = db.query_one(TODAY_STATS_SQL, &[]).await?;
    Ok((
        row.try_get("total_syncs")?,
        row.try_get("successful_syncs")?,
        row.try_get("failed_syncs")?,
    ))
}

/// A failed query, as the caller sees it. The database's own message goes to the log and stays
/// there: nobody has proven who they are to these endpoints, and the text of a Postgres error
/// names tables, columns and the statement that failed.
fn query_failed(what: &'static str, e: &DbError) -> Response {
    error!("{}: {}", what, e);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": what })),
    )
        .into_response()
}

fn out_of_range(name: &str, max: i64) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({
            "error": format!("{} must be between 1 and {}", name, max)
        })),
    )
        .into_response()
}

// ---------------------------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------------------------

/// Get sync status for all sources or specific source
async fn get_sync_status(
    State(state): State<MonitoringState>,
    Query(params): Query<SyncStatusQuery>,
) -> Response {
    info!(
        "GET /api/monitoring/sync-status - source: {:?}",
        params.source
    );

    match fetch_sync_status(&state.db_client, params.source.as_deref()).await {
        Ok(statuses) => (StatusCode::OK, Json(statuses)).into_response(),
        Err(e) => query_failed("Failed to query sync status", &e),
    }
}

/// Get sync history with optional filters
async fn get_sync_history(
    State(state): State<MonitoringState>,
    Query(params): Query<SyncHistoryQuery>,
) -> Response {
    let days = params.days.unwrap_or(7);
    let limit = params.limit.unwrap_or(100);

    info!(
        "GET /api/monitoring/sync-history - days: {}, source: {:?}, limit: {}",
        days, params.source, limit
    );

    if !(1..=MAX_HISTORY_DAYS).contains(&days) {
        return out_of_range("days", i64::from(MAX_HISTORY_DAYS));
    }
    if !(1..=MAX_HISTORY_LIMIT).contains(&limit) {
        return out_of_range("limit", MAX_HISTORY_LIMIT);
    }

    match fetch_sync_history(
        &state.db_client,
        params.source.as_deref(),
        Some(days),
        limit,
    )
    .await
    {
        Ok(history) => (StatusCode::OK, Json(history)).into_response(),
        Err(e) => query_failed("Failed to query sync history", &e),
    }
}

/// Get active alerts for sync failures
async fn get_alerts(State(state): State<MonitoringState>) -> Response {
    info!("GET /api/monitoring/alerts");

    match fetch_alerts(&state.db_client).await {
        Ok(alerts) => (StatusCode::OK, Json(alerts)).into_response(),
        Err(e) => query_failed("Failed to query alerts", &e),
    }
}

/// Get comprehensive dashboard metrics
///
/// Each part degrades on its own, as before: a part whose query failed is logged and comes back
/// empty, and the others are still answered.
async fn get_dashboard_metrics(State(state): State<MonitoringState>) -> Response {
    info!("GET /api/monitoring/dashboard");
    let db = &state.db_client;

    let (total_syncs, successful_syncs, failed_syncs) =
        fetch_today_stats(db).await.unwrap_or_else(|e| {
            error!("Failed to query sync statistics: {}", e);
            (0, 0, 0)
        });

    let success_rate = if total_syncs > 0 {
        (successful_syncs as f64 / total_syncs as f64) * 100.0
    } else {
        0.0
    };

    let sync_status_by_source: HashMap<String, SyncStatusResponse> =
        match fetch_sync_status(db, None).await {
            Ok(statuses) => statuses
                .into_iter()
                .map(|status| (status.source.clone(), status))
                .collect(),
            Err(e) => {
                error!("Failed to query sync status: {}", e);
                HashMap::new()
            }
        };

    // The 10 latest calls, whatever their age.
    let recent_sync_history = fetch_sync_history(db, None, None, 10)
        .await
        .unwrap_or_else(|e| {
            error!("Failed to query recent sync history: {}", e);
            Vec::new()
        });

    let active_alerts = fetch_alerts(db).await.unwrap_or_else(|e| {
        error!("Failed to query alerts: {}", e);
        Vec::new()
    });

    let metrics = DashboardMetrics {
        total_syncs_today: total_syncs,
        successful_syncs_today: successful_syncs,
        failed_syncs_today: failed_syncs,
        success_rate_today: success_rate,
        active_alerts,
        sync_status_by_source,
        recent_sync_history,
    };

    (StatusCode::OK, Json(metrics)).into_response()
}

/// Health check endpoint
async fn health_check(State(state): State<MonitoringState>) -> impl IntoResponse {
    match state.db_client.simple_query("SELECT 1").await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "healthy",
                "database": "connected"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "unhealthy",
                "database": "disconnected",
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_query_params() {
        let query = SyncStatusQuery {
            source: Some("monsakti".to_string()),
        };
        assert_eq!(query.source, Some("monsakti".to_string()));
    }

    #[test]
    fn test_sync_history_query_params() {
        let query = SyncHistoryQuery {
            days: Some(7),
            source: Some("siman".to_string()),
            limit: Some(50),
        };
        assert_eq!(query.days, Some(7));
        assert_eq!(query.limit, Some(50));
    }
}
