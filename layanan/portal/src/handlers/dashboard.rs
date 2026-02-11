//! Dashboard handlers
//!
//! Implements portal dashboard endpoints with system-wide metrics aggregation.
//! Fetches data from multiple sources including Authenc and Integration services.

use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;
use uuid::Uuid;

use crate::error::{PortalError, Result};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct DashboardQuery {
    pub period: Option<String>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct OverviewResponse {
    pub total_users: i64,
    pub active_sessions: i64,
    pub total_documents: i64,
    pub pending_tasks: i64,
}

#[derive(Debug, Serialize)]
pub struct WidgetResponse {
    pub id: Uuid,
    pub name: String,
    pub widget_type: String,
    pub config: serde_json::Value,
}

/// Portal dashboard metrics response
#[derive(Debug, Serialize)]
pub struct PortalDashboardMetrics {
    /// System metrics
    pub system: SystemMetrics,
    /// Cross-domain metrics
    pub cross_domain: CrossDomainMetrics,
    /// Authentication metrics from Authenc
    pub auth: AuthMetrics,
    /// Integration health status
    pub integration_health: IntegrationHealth,
    /// Timestamp when metrics were collected
    pub collected_at: DateTime<Utc>,
}

/// System-level metrics
#[derive(Debug, Serialize)]
pub struct SystemMetrics {
    /// Total number of users in the system
    pub total_users: i64,
    /// Number of active sessions
    pub active_sessions: i64,
    /// System uptime in seconds
    pub uptime_seconds: i64,
    /// Server start time
    pub server_started_at: DateTime<Utc>,
}

/// Cross-domain metrics aggregated from multiple services
#[derive(Debug, Serialize)]
pub struct CrossDomainMetrics {
    /// Total documents across all domains
    pub total_documents: i64,
    /// Total notifications sent
    pub total_notifications: i64,
    /// Total API calls in the last 24 hours
    pub api_calls_24h: i64,
    /// Documents by status
    pub documents_by_status: Vec<StatusCount>,
    /// Notifications by channel
    pub notifications_by_channel: Vec<ChannelCount>,
}

/// Authentication metrics from Authenc service
#[derive(Debug, Serialize)]
pub struct AuthMetrics {
    /// Total login attempts in the last 24 hours
    pub login_attempts_24h: i64,
    /// Successful logins in the last 24 hours
    pub successful_logins_24h: i64,
    /// Failed logins in the last 24 hours
    pub failed_logins_24h: i64,
    /// Users with MFA enabled
    pub mfa_enabled_users: i64,
    /// Active sessions by role
    pub sessions_by_role: Vec<RoleCount>,
}

/// Integration service health status
#[derive(Debug, Serialize)]
pub struct IntegrationHealth {
    /// SIMAN integration status
    pub siman: ServiceStatus,
    /// MySIMKARI integration status
    pub mysimkari: ServiceStatus,
    /// MonSAKTI integration status (optional)
    pub monsakti: Option<ServiceStatus>,
    /// Overall integration health
    pub overall_status: String,
}

/// Individual service status
#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    /// Service name
    pub name: String,
    /// Health status: "healthy", "degraded", "down"
    pub status: String,
    /// Last successful sync time
    pub last_sync: Option<DateTime<Utc>>,
    /// Last sync duration in milliseconds
    pub last_sync_duration_ms: Option<i64>,
    /// Error message if unhealthy
    pub error_message: Option<String>,
}

/// Count by status
#[derive(Debug, Serialize)]
pub struct StatusCount {
    pub status: String,
    pub count: i64,
}

/// Count by channel
#[derive(Debug, Serialize)]
pub struct ChannelCount {
    pub channel: String,
    pub count: i64,
}

/// Count by role
#[derive(Debug, Serialize)]
pub struct RoleCount {
    pub role: String,
    pub count: i64,
}

// Static variable to track server start time
static SERVER_START_TIME: std::sync::OnceLock<SystemTime> = std::sync::OnceLock::new();

/// Initialize server start time (call this in main.rs)
pub fn init_server_start_time() {
    SERVER_START_TIME.get_or_init(SystemTime::now);
}

/// Get portal dashboard metrics
///
/// This endpoint aggregates metrics from multiple sources:
/// - System metrics (users, sessions, uptime)
/// - Cross-domain metrics (documents, notifications, API calls)
/// - Auth metrics from Authenc via gRPC
/// - Integration health from Integration Service via gRPC
pub async fn get_portal_dashboard_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<PortalDashboardMetrics>> {
    tracing::info!("Fetching portal dashboard metrics");

    // Fetch all metrics concurrently
    let (system, cross_domain, auth, integration_health) = tokio::try_join!(
        fetch_system_metrics(&state),
        fetch_cross_domain_metrics(&state),
        fetch_auth_metrics(&state),
        fetch_integration_health(&state),
    )?;

    let metrics = PortalDashboardMetrics {
        system,
        cross_domain,
        auth,
        integration_health,
        collected_at: Utc::now(),
    };

    Ok(Json(metrics))
}

/// Fetch system metrics from database
async fn fetch_system_metrics(state: &AppState) -> Result<SystemMetrics> {
    let client = state.db.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        PortalError::Database(e.to_string())
    })?;

    // Get total users count
    let total_users_row = client
        .query_one("SELECT COUNT(*) as count FROM authenc.users WHERE enabled = true", &[])
        .await
        .map_err(|e| {
            tracing::error!("Failed to query total users: {}", e);
            PortalError::Database(e.to_string())
        })?;
    let total_users: i64 = total_users_row.get("count");

    // Get active sessions count (sessions active in last 15 minutes)
    let active_sessions_row = client
        .query_one(
            "SELECT COUNT(*) as count FROM authenc.sessions
             WHERE expires_at > NOW() AND last_activity > NOW() - INTERVAL '15 minutes'",
            &[],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to query active sessions: {}", e);
            PortalError::Database(e.to_string())
        })?;
    let active_sessions: i64 = active_sessions_row.get("count");

    // Calculate uptime
    let start_time = SERVER_START_TIME.get().copied().unwrap_or_else(SystemTime::now);
    let uptime_seconds = SystemTime::now()
        .duration_since(start_time)
        .unwrap_or_default()
        .as_secs() as i64;

    let server_started_at: DateTime<Utc> = start_time.into();

    Ok(SystemMetrics {
        total_users,
        active_sessions,
        uptime_seconds,
        server_started_at,
    })
}

/// Fetch cross-domain metrics from database
async fn fetch_cross_domain_metrics(state: &AppState) -> Result<CrossDomainMetrics> {
    let client = state.db.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        PortalError::Database(e.to_string())
    })?;

    // Get total documents count
    let total_documents: i64 = match client
        .query_one("SELECT COUNT(*) as count FROM dokumen.documents", &[])
        .await
    {
        Ok(row) => row.get("count"),
        Err(_) => {
            // If dokumen schema doesn't exist yet, return 0
            tracing::warn!("dokumen.documents table not found, returning 0");
            0
        }
    };

    // Get total notifications count
    let total_notifications: i64 = match client
        .query_one("SELECT COUNT(*) as count FROM notifikasi.notifications", &[])
        .await
    {
        Ok(row) => row.get("count"),
        Err(_) => {
            tracing::warn!("notifikasi.notifications table not found, returning 0");
            0
        }
    };

    // Get API calls in last 24 hours
    let api_calls_row = client
        .query_one(
            "SELECT COUNT(*) as count FROM integrasi.api_call_log
             WHERE called_at > NOW() - INTERVAL '24 hours'",
            &[],
        )
        .await;
    let api_calls_24h: i64 = match api_calls_row {
        Ok(row) => row.get("count"),
        Err(_) => {
            tracing::warn!("integrasi.api_call_log table not found, returning 0");
            0
        }
    };

    // Get documents by status
    let documents_by_status_rows = client
        .query(
            "SELECT status, COUNT(*) as count FROM dokumen.documents
             GROUP BY status ORDER BY count DESC",
            &[],
        )
        .await
        .unwrap_or_default();

    let documents_by_status = documents_by_status_rows
        .iter()
        .map(|row| StatusCount {
            status: row.get("status"),
            count: row.get("count"),
        })
        .collect();

    // Get notifications by channel
    let notifications_by_channel_rows = client
        .query(
            "SELECT channel, COUNT(*) as count FROM notifikasi.notifications
             GROUP BY channel ORDER BY count DESC",
            &[],
        )
        .await
        .unwrap_or_default();

    let notifications_by_channel = notifications_by_channel_rows
        .iter()
        .map(|row| ChannelCount {
            channel: row.get("channel"),
            count: row.get("count"),
        })
        .collect();

    Ok(CrossDomainMetrics {
        total_documents,
        total_notifications,
        api_calls_24h,
        documents_by_status,
        notifications_by_channel,
    })
}

/// Fetch authentication metrics from Authenc via gRPC
async fn fetch_auth_metrics(state: &AppState) -> Result<AuthMetrics> {
    // For now, fetch from database directly
    // TODO: Implement gRPC endpoint in Authenc for metrics
    let client = state.db.get().await.map_err(|e| {
        tracing::error!("Failed to get database connection: {}", e);
        PortalError::Database(e.to_string())
    })?;

    // Get login attempts in last 24 hours from audit log
    let login_attempts_row = client
        .query_one(
            "SELECT COUNT(*) as count FROM authenc.audit_log
             WHERE event_type = 'login_attempt' AND created_at > NOW() - INTERVAL '24 hours'",
            &[],
        )
        .await;
    let login_attempts_24h: i64 = match login_attempts_row {
        Ok(row) => row.get("count"),
        Err(_) => {
            tracing::warn!("authenc.audit_log table not found, returning 0");
            0
        }
    };

    // Get successful logins
    let successful_logins_row = client
        .query_one(
            "SELECT COUNT(*) as count FROM authenc.audit_log
             WHERE event_type = 'login_success' AND created_at > NOW() - INTERVAL '24 hours'",
            &[],
        )
        .await;
    let successful_logins_24h: i64 = match successful_logins_row {
        Ok(row) => row.get("count"),
        Err(_) => 0,
    };

    // Calculate failed logins
    let failed_logins_24h = login_attempts_24h.saturating_sub(successful_logins_24h);

    // Get users with MFA enabled
    let mfa_enabled_row = client
        .query_one(
            "SELECT COUNT(*) as count FROM authenc.users WHERE mfa_enabled = true",
            &[],
        )
        .await;
    let mfa_enabled_users: i64 = match mfa_enabled_row {
        Ok(row) => row.get("count"),
        Err(_) => 0,
    };

    // Get active sessions by role
    let sessions_by_role_rows = client
        .query(
            "SELECT r.name as role, COUNT(DISTINCT s.id) as count
             FROM authenc.sessions s
             JOIN authenc.user_roles ur ON s.user_id = ur.user_id
             JOIN authenc.roles r ON ur.role_id = r.id
             WHERE s.expires_at > NOW() AND s.last_activity > NOW() - INTERVAL '15 minutes'
             GROUP BY r.name
             ORDER BY count DESC",
            &[],
        )
        .await
        .unwrap_or_default();

    let sessions_by_role = sessions_by_role_rows
        .iter()
        .map(|row| RoleCount {
            role: row.get("role"),
            count: row.get("count"),
        })
        .collect();

    Ok(AuthMetrics {
        login_attempts_24h,
        successful_logins_24h,
        failed_logins_24h,
        mfa_enabled_users,
        sessions_by_role,
    })
}

/// Fetch integration health status
async fn fetch_integration_health(_state: &AppState) -> Result<IntegrationHealth> {
    // TODO: Implement gRPC call to Integration Service
    // For now, return mock data
    tracing::warn!("Integration health check not yet implemented, returning mock data");

    Ok(IntegrationHealth {
        siman: ServiceStatus {
            name: "SIMAN".to_string(),
            status: "healthy".to_string(),
            last_sync: Some(Utc::now() - chrono::Duration::hours(2)),
            last_sync_duration_ms: Some(1500),
            error_message: None,
        },
        mysimkari: ServiceStatus {
            name: "MySIMKARI".to_string(),
            status: "healthy".to_string(),
            last_sync: Some(Utc::now() - chrono::Duration::hours(3)),
            last_sync_duration_ms: Some(2000),
            error_message: None,
        },
        monsakti: Some(ServiceStatus {
            name: "MonSAKTI".to_string(),
            status: "degraded".to_string(),
            last_sync: Some(Utc::now() - chrono::Duration::hours(12)),
            last_sync_duration_ms: Some(5000),
            error_message: Some("Slow response time".to_string()),
        }),
        overall_status: "healthy".to_string(),
    })
}

// Legacy endpoints (kept for backward compatibility)

pub async fn get_overview(
    State(_state): State<Arc<AppState>>,
    Query(_query): Query<DashboardQuery>,
) -> Result<Json<OverviewResponse>> {
    // TODO: Implement actual data aggregation
    Ok(Json(OverviewResponse {
        total_users: 1000,
        active_sessions: 150,
        total_documents: 5000,
        pending_tasks: 25,
    }))
}

pub async fn get_widgets(
    State(_state): State<Arc<AppState>>,
    Query(_query): Query<DashboardQuery>,
) -> Result<Json<Vec<WidgetResponse>>> {
    // TODO: Fetch from database
    Ok(Json(vec![]))
}

pub async fn get_widget(
    State(_state): State<Arc<AppState>>,
    Path(widget_id): Path<Uuid>,
) -> Result<Json<WidgetResponse>> {
    Ok(Json(WidgetResponse {
        id: widget_id,
        name: "Sample Widget".to_string(),
        widget_type: "chart".to_string(),
        config: serde_json::json!({}),
    }))
}
