//! Monitoring module for integration service
//!
//! Provides REST API endpoints for monitoring sync status, viewing sync history,
//! and alerting on sync failures.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::Client;
use tracing::{error, info};

// Note: These functions are available but not currently used in the monitoring endpoints
// use crate::audit::{get_api_stats_by_module, get_recent_failed_calls, get_token_health};

/// Shared state for monitoring endpoints
#[derive(Clone)]
pub struct MonitoringState {
    pub db_client: Arc<Client>,
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

/// Get sync status for all sources or specific source
async fn get_sync_status(
    State(state): State<MonitoringState>,
    Query(params): Query<SyncStatusQuery>,
) -> impl IntoResponse {
    info!("GET /api/monitoring/sync-status - source: {:?}", params.source);

    let query = if let Some(source) = params.source {
        format!(
            r#"
            SELECT
                module as source,
                MAX(started_at) as last_sync_at,
                COALESCE(SUM(record_count), 0) as total_records,
                COUNT(*) as total_syncs,
                SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful_syncs
            FROM integrasi.api_log
            WHERE module = '{}'
            GROUP BY module
        "#,
            source
        )
    } else {
        r#"
            SELECT
                module as source,
                MAX(started_at) as last_sync_at,
                COALESCE(SUM(record_count), 0) as total_records,
                COUNT(*) as total_syncs,
                SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful_syncs
            FROM integrasi.api_log
            GROUP BY module
        "#
        .to_string()
    };

    match state.db_client.query(&query, &[]).await {
        Ok(rows) => {
            let mut statuses = Vec::new();

            for row in rows {
                let source: String = row.try_get("source").unwrap_or_default();
                let last_sync_at: Option<chrono::DateTime<chrono::Utc>> =
                    row.try_get("last_sync_at").ok();
                let total_records: i64 = row.try_get("total_records").unwrap_or(0);
                let total_syncs: i64 = row.try_get("total_syncs").unwrap_or(0);
                let successful_syncs: i64 = row.try_get("successful_syncs").unwrap_or(0);

                let success_rate = if total_syncs > 0 {
                    (successful_syncs as f64 / total_syncs as f64) * 100.0
                } else {
                    0.0
                };

                statuses.push(SyncStatusResponse {
                    source,
                    last_sync_at: last_sync_at.map(|dt| dt.to_rfc3339()),
                    next_sync_at: None, // Will be populated from K8s CronJob schedule
                    state: if success_rate > 90.0 {
                        "healthy".to_string()
                    } else if success_rate > 50.0 {
                        "degraded".to_string()
                    } else {
                        "unhealthy".to_string()
                    },
                    records_synced: total_records,
                    success_rate,
                    error_message: None,
                });
            }

            (StatusCode::OK, Json(statuses)).into_response()
        }
        Err(e) => {
            error!("Failed to query sync status: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to query sync status",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

/// Get sync history with optional filters
async fn get_sync_history(
    State(state): State<MonitoringState>,
    Query(params): Query<SyncHistoryQuery>,
) -> impl IntoResponse {
    let days = params.days.unwrap_or(7);
    let limit = params.limit.unwrap_or(100);

    info!(
        "GET /api/monitoring/sync-history - days: {}, source: {:?}, limit: {}",
        days, params.source, limit
    );

    let query = if let Some(source) = params.source {
        format!(
            r#"
            SELECT
                id, module as source, endpoint,
                COALESCE(record_count, 0) as records_fetched,
                success, error_message, started_at as sync_started_at,
                EXTRACT(EPOCH FROM (completed_at - started_at))::INTEGER as duration_seconds
            FROM integrasi.api_log
            WHERE module = '{}' AND started_at >= CURRENT_TIMESTAMP - INTERVAL '{} days'
            ORDER BY started_at DESC
            LIMIT {}
        "#,
            source, days, limit
        )
    } else {
        format!(
            r#"
            SELECT
                id, module as source, endpoint,
                COALESCE(record_count, 0) as records_fetched,
                success, error_message, started_at as sync_started_at,
                EXTRACT(EPOCH FROM (completed_at - started_at))::INTEGER as duration_seconds
            FROM integrasi.api_log
            WHERE started_at >= CURRENT_TIMESTAMP - INTERVAL '{} days'
            ORDER BY started_at DESC
            LIMIT {}
        "#,
            days, limit
        )
    };

    match state.db_client.query(&query, &[]).await {
        Ok(rows) => {
            let history: Vec<SyncHistoryItem> = rows
                .iter()
                .map(|row| SyncHistoryItem {
                    id: row.try_get("id").unwrap_or(0),
                    source: row.try_get("source").unwrap_or_default(),
                    table_name: String::new(),
                    module: row.try_get("source").unwrap_or_default(),
                    endpoint: row.try_get("endpoint").unwrap_or_default(),
                    records_fetched: row.try_get("records_fetched").unwrap_or(0),
                    records_inserted: 0,
                    records_updated: 0,
                    records_failed: 0,
                    success: row.try_get("success").unwrap_or(false),
                    error_message: row.try_get("error_message").ok(),
                    sync_started_at: row
                        .try_get::<_, chrono::DateTime<chrono::Utc>>("sync_started_at")
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_default(),
                    duration_seconds: row.try_get("duration_seconds").ok(),
                })
                .collect();

            (StatusCode::OK, Json(history)).into_response()
        }
        Err(e) => {
            error!("Failed to query sync history: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to query sync history",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

/// Get active alerts for sync failures
async fn get_alerts(State(state): State<MonitoringState>) -> impl IntoResponse {
    info!("GET /api/monitoring/alerts");

    // Query for recent failures (last 24 hours)
    let query = r#"
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
            COALESCE(cf.consecutive_count, 0) as consecutive_failures
        FROM recent_failures rf
        LEFT JOIN consecutive_failures cf ON rf.source = cf.source
        WHERE rf.failure_count >= 3 OR COALESCE(cf.consecutive_count, 0) >= 2
        ORDER BY rf.failure_count DESC, rf.last_failure_at DESC
    "#;

    match state.db_client.query(query, &[]).await {
        Ok(rows) => {
            let alerts: Vec<SyncAlert> = rows
                .iter()
                .enumerate()
                .map(|(idx, row)| {
                    let failure_count: i64 = row.try_get("failure_count").unwrap_or(0);
                    let consecutive_failures: i32 =
                        row.try_get("consecutive_failures").unwrap_or(0);
                    let source: String = row.try_get("source").unwrap_or_default();

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

                    SyncAlert {
                        id: idx as i64 + 1,
                        source,
                        severity: severity.to_string(),
                        message,
                        error_details: row.try_get("last_error").ok(),
                        occurred_at: row
                            .try_get::<_, chrono::DateTime<chrono::Utc>>("last_failure_at")
                            .map(|dt| dt.to_rfc3339())
                            .unwrap_or_default(),
                        consecutive_failures,
                    }
                })
                .collect();

            (StatusCode::OK, Json(alerts)).into_response()
        }
        Err(e) => {
            error!("Failed to query alerts: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to query alerts",
                    "details": e.to_string()
                })),
            )
                .into_response()
        }
    }
}

/// Get comprehensive dashboard metrics
async fn get_dashboard_metrics(State(state): State<MonitoringState>) -> impl IntoResponse {
    info!("GET /api/monitoring/dashboard");

    // Get today's sync statistics
    let stats_query = r#"
        SELECT
            COUNT(*) as total_syncs,
            SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful_syncs,
            SUM(CASE WHEN NOT success THEN 1 ELSE 0 END) as failed_syncs
        FROM integrasi.api_log
        WHERE started_at >= CURRENT_DATE
    "#;

    let (total_syncs, successful_syncs, failed_syncs) =
        match state.db_client.query_one(stats_query, &[]).await {
            Ok(row) => {
                let total: i64 = row.try_get("total_syncs").unwrap_or(0);
                let successful: i64 = row.try_get("successful_syncs").unwrap_or(0);
                let failed: i64 = row.try_get("failed_syncs").unwrap_or(0);
                (total, successful, failed)
            }
            Err(e) => {
                error!("Failed to query sync statistics: {}", e);
                (0, 0, 0)
            }
        };

    let success_rate = if total_syncs > 0 {
        (successful_syncs as f64 / total_syncs as f64) * 100.0
    } else {
        0.0
    };

    // Get sync status by source
    let status_query = r#"
        SELECT
            module as source,
            MAX(started_at) as last_sync_at,
            COALESCE(SUM(record_count), 0) as total_records,
            COUNT(*) as total_syncs,
            SUM(CASE WHEN success THEN 1 ELSE 0 END) as successful_syncs
        FROM integrasi.api_log
        GROUP BY module
    "#;

    let mut sync_status_by_source = HashMap::new();
    if let Ok(rows) = state.db_client.query(status_query, &[]).await {
        for row in rows {
            let source: String = row.try_get("source").unwrap_or_default();
            let last_sync_at: Option<chrono::DateTime<chrono::Utc>> =
                row.try_get("last_sync_at").ok();
            let total_records: i64 = row.try_get("total_records").unwrap_or(0);
            let total_syncs: i64 = row.try_get("total_syncs").unwrap_or(0);
            let successful_syncs: i64 = row.try_get("successful_syncs").unwrap_or(0);

            let success_rate = if total_syncs > 0 {
                (successful_syncs as f64 / total_syncs as f64) * 100.0
            } else {
                0.0
            };

            sync_status_by_source.insert(
                source.clone(),
                SyncStatusResponse {
                    source,
                    last_sync_at: last_sync_at.map(|dt| dt.to_rfc3339()),
                    next_sync_at: None,
                    state: if success_rate > 90.0 {
                        "healthy".to_string()
                    } else if success_rate > 50.0 {
                        "degraded".to_string()
                    } else {
                        "unhealthy".to_string()
                    },
                    records_synced: total_records,
                    success_rate,
                    error_message: None,
                },
            );
        }
    }

    // Get recent sync history (last 10)
    let history_query = r#"
        SELECT
            id, module as source, endpoint,
            COALESCE(record_count, 0) as records_fetched,
            success, error_message, started_at as sync_started_at,
            EXTRACT(EPOCH FROM (completed_at - started_at))::INTEGER as duration_seconds
        FROM integrasi.api_log
        ORDER BY started_at DESC
        LIMIT 10
    "#;

    let recent_sync_history = match state.db_client.query(history_query, &[]).await {
        Ok(rows) => rows
            .iter()
            .map(|row| SyncHistoryItem {
                id: row.try_get("id").unwrap_or(0),
                source: row.try_get("source").unwrap_or_default(),
                table_name: String::new(),
                module: row.try_get("source").unwrap_or_default(),
                endpoint: row.try_get("endpoint").unwrap_or_default(),
                records_fetched: row.try_get("records_fetched").unwrap_or(0),
                records_inserted: 0,
                records_updated: 0,
                records_failed: 0,
                success: row.try_get("success").unwrap_or(false),
                error_message: row.try_get("error_message").ok(),
                sync_started_at: row
                    .try_get::<_, chrono::DateTime<chrono::Utc>>("sync_started_at")
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default(),
                duration_seconds: row.try_get("duration_seconds").ok(),
            })
            .collect(),
        Err(e) => {
            error!("Failed to query recent sync history: {}", e);
            Vec::new()
        }
    };

    // Get active alerts
    let alerts_query = r#"
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
            COALESCE(cf.consecutive_count, 0) as consecutive_failures
        FROM recent_failures rf
        LEFT JOIN consecutive_failures cf ON rf.source = cf.source
        WHERE rf.failure_count >= 3 OR COALESCE(cf.consecutive_count, 0) >= 2
        ORDER BY rf.failure_count DESC, rf.last_failure_at DESC
    "#;

    let active_alerts = match state.db_client.query(alerts_query, &[]).await {
        Ok(rows) => rows
            .iter()
            .enumerate()
            .map(|(idx, row)| {
                let failure_count: i64 = row.try_get("failure_count").unwrap_or(0);
                let consecutive_failures: i32 = row.try_get("consecutive_failures").unwrap_or(0);
                let source: String = row.try_get("source").unwrap_or_default();

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

                SyncAlert {
                    id: idx as i64 + 1,
                    source,
                    severity: severity.to_string(),
                    message,
                    error_details: row.try_get("last_error").ok(),
                    occurred_at: row
                        .try_get::<_, chrono::DateTime<chrono::Utc>>("last_failure_at")
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_default(),
                    consecutive_failures,
                }
            })
            .collect(),
        Err(e) => {
            error!("Failed to query alerts: {}", e);
            Vec::new()
        }
    };

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
