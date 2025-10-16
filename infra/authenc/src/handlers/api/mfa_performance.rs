//! MFA Performance Monitoring API Handlers
//!
//! This module provides HTTP API endpoints for accessing MFA performance
//! metrics, dashboard data, and performance alerts.

use crate::app::AppState;
use crate::error::Result;
use crate::services::mfa_performance_monitor::{
    MfaDashboardData, MfaMetrics, MfaPerformanceMonitor, PerformanceAlert,
};
use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Query parameters for performance metrics
#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    /// Time range for metrics (in minutes)
    #[serde(default = "default_time_range")]
    pub time_range: u32,
    /// Include detailed breakdown
    #[serde(default)]
    pub detailed: bool,
}

fn default_time_range() -> u32 {
    60 // 1 hour default
}

/// Response for performance metrics endpoint
#[derive(Debug, Serialize)]
pub struct MetricsResponse {
    /// Current performance metrics
    pub metrics: MfaMetrics,
    /// Performance summary
    pub summary: PerformanceSummary,
    /// System status
    pub status: SystemStatus,
}

/// Performance summary for quick overview
#[derive(Debug, Serialize)]
pub struct PerformanceSummary {
    /// Overall system health score (0.0 to 1.0)
    pub health_score: f64,
    /// Total MFA operations in the last hour
    pub total_operations: u64,
    /// Overall success rate
    pub success_rate: f64,
    /// Average response time across all operations
    pub avg_response_time_ms: f64,
    /// Cache effectiveness
    pub cache_hit_ratio: f64,
}

/// System status information
#[derive(Debug, Serialize)]
pub struct SystemStatus {
    /// Overall system status
    pub status: String,
    /// Active alerts count
    pub active_alerts: u32,
    /// Performance trend (improving, stable, degrading)
    pub trend: String,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Get current MFA performance metrics
pub async fn get_mfa_metrics(
    State(state): State<Arc<AppState>>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<MetricsResponse>> {
    debug!(
        time_range = query.time_range,
        detailed = query.detailed,
        "Fetching MFA performance metrics"
    );

    // For now, we'll create a simple monitor if not available in state
    // In a real implementation, this would be injected into AppState
    let monitor = Arc::new(MfaPerformanceMonitor::new(None));
    let metrics = monitor.get_metrics().await;

    // Calculate performance summary
    let total_operations = metrics.setup_metrics.total_operations
        + metrics.verification_metrics.total_operations
        + metrics.status_lookup_metrics.total_operations;

    let total_successful = metrics.setup_metrics.successful_operations
        + metrics.verification_metrics.successful_operations
        + metrics.status_lookup_metrics.successful_operations;

    let success_rate = if total_operations > 0 {
        total_successful as f64 / total_operations as f64
    } else {
        1.0
    };

    let avg_response_time = if total_operations > 0 {
        (metrics.setup_metrics.avg_response_time_ms * metrics.setup_metrics.total_operations as f64
            + metrics.verification_metrics.avg_response_time_ms * metrics.verification_metrics.total_operations as f64
            + metrics.status_lookup_metrics.avg_response_time_ms * metrics.status_lookup_metrics.total_operations as f64)
            / total_operations as f64
    } else {
        0.0
    };

    // Calculate health score based on multiple factors
    let health_score = calculate_health_score(&metrics, success_rate, avg_response_time);

    let summary = PerformanceSummary {
        health_score,
        total_operations,
        success_rate,
        avg_response_time_ms: avg_response_time,
        cache_hit_ratio: metrics.cache_metrics.hit_ratio,
    };

    let status = SystemStatus {
        status: if health_score > 0.8 {
            "healthy".to_string()
        } else if health_score > 0.6 {
            "warning".to_string()
        } else {
            "critical".to_string()
        },
        active_alerts: 0, // Would be calculated from actual alerts
        trend: "stable".to_string(), // Would be calculated from historical data
        last_updated: chrono::Utc::now(),
    };

    info!(
        total_operations = total_operations,
        success_rate = success_rate,
        health_score = health_score,
        "MFA performance metrics retrieved"
    );

    Ok(Json(MetricsResponse {
        metrics,
        summary,
        status,
    }))
}

/// Get MFA performance dashboard data
pub async fn get_mfa_dashboard(
    State(state): State<Arc<AppState>>,
) -> Result<Json<MfaDashboardData>> {
    debug!("Fetching MFA performance dashboard data");

    // For now, we'll create a simple monitor if not available in state
    let monitor = Arc::new(MfaPerformanceMonitor::new(None));
    let dashboard_data = monitor.get_dashboard_data().await;

    info!("MFA performance dashboard data retrieved");

    Ok(Json(dashboard_data))
}

/// Get MFA performance alerts
pub async fn get_mfa_alerts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PerformanceAlert>>> {
    debug!("Fetching MFA performance alerts");

    // In a real implementation, this would fetch active alerts from the monitor
    let alerts = Vec::new(); // Placeholder

    info!(alert_count = alerts.len(), "MFA performance alerts retrieved");

    Ok(Json(alerts))
}

/// Trigger manual performance snapshot
pub async fn trigger_performance_snapshot(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>> {
    debug!("Triggering manual performance snapshot");

    // For now, we'll create a simple monitor if not available in state
    let monitor = Arc::new(MfaPerformanceMonitor::new(None));
    monitor.take_snapshot().await?;

    info!("Manual performance snapshot triggered");

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Performance snapshot taken",
        "timestamp": chrono::Utc::now()
    })))
}

/// Get performance statistics by operation type
pub async fn get_operation_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HashMap<String, OperationStats>>> {
    debug!("Fetching operation-specific performance statistics");

    let monitor = Arc::new(MfaPerformanceMonitor::new(None));
    let metrics = monitor.get_metrics().await;

    let mut stats = HashMap::new();

    stats.insert(
        "setup".to_string(),
        OperationStats::from_metrics(&metrics.setup_metrics),
    );

    stats.insert(
        "verification".to_string(),
        OperationStats::from_metrics(&metrics.verification_metrics),
    );

    stats.insert(
        "status_lookup".to_string(),
        OperationStats::from_metrics(&metrics.status_lookup_metrics),
    );

    info!(operation_count = stats.len(), "Operation statistics retrieved");

    Ok(Json(stats))
}

/// Operation statistics for API response
#[derive(Debug, Serialize)]
pub struct OperationStats {
    /// Total operations
    pub total: u64,
    /// Successful operations
    pub successful: u64,
    /// Failed operations
    pub failed: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// 95th percentile response time
    pub p95_response_time_ms: f64,
    /// 99th percentile response time
    pub p99_response_time_ms: f64,
    /// Operations per second
    pub ops_per_second: f64,
}

impl OperationStats {
    fn from_metrics(metrics: &crate::services::mfa_performance_monitor::OperationMetrics) -> Self {
        let success_rate = if metrics.total_operations > 0 {
            metrics.successful_operations as f64 / metrics.total_operations as f64
        } else {
            1.0
        };

        Self {
            total: metrics.total_operations,
            successful: metrics.successful_operations,
            failed: metrics.failed_operations,
            success_rate,
            avg_response_time_ms: metrics.avg_response_time_ms,
            p95_response_time_ms: metrics.p95_response_time_ms,
            p99_response_time_ms: metrics.p99_response_time_ms,
            ops_per_second: metrics.operations_per_second,
        }
    }
}

/// Calculate overall system health score
fn calculate_health_score(metrics: &MfaMetrics, success_rate: f64, avg_response_time: f64) -> f64 {
    let mut score = 1.0;

    // Penalize low success rate
    if success_rate < 0.95 {
        score *= success_rate / 0.95;
    }

    // Penalize high response times
    if avg_response_time > 100.0 {
        score *= 100.0 / avg_response_time.min(1000.0);
    }

    // Penalize low cache hit ratio
    if metrics.cache_metrics.hit_ratio < 0.8 {
        score *= metrics.cache_metrics.hit_ratio / 0.8;
    }

    // Penalize high error rates
    let total_errors = metrics.error_metrics.invalid_otp_errors
        + metrics.error_metrics.rate_limit_errors
        + metrics.error_metrics.database_errors
        + metrics.error_metrics.cache_errors
        + metrics.error_metrics.secreton_errors
        + metrics.error_metrics.other_errors;

    let total_operations = metrics.setup_metrics.total_operations
        + metrics.verification_metrics.total_operations
        + metrics.status_lookup_metrics.total_operations;

    if total_operations > 0 {
        let error_rate = total_errors as f64 / total_operations as f64;
        if error_rate > 0.05 {
            score *= (0.05 / error_rate).min(1.0);
        }
    }

    score.max(0.0).min(1.0)
}

/// Health check endpoint for MFA performance monitoring
pub async fn mfa_performance_health(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>> {
    let monitor = Arc::new(MfaPerformanceMonitor::new(None));
    let metrics = monitor.get_metrics().await;

    let total_operations = metrics.setup_metrics.total_operations
        + metrics.verification_metrics.total_operations
        + metrics.status_lookup_metrics.total_operations;

    let health_status = if total_operations == 0 {
        "no_data"
    } else {
        let total_successful = metrics.setup_metrics.successful_operations
            + metrics.verification_metrics.successful_operations
            + metrics.status_lookup_metrics.successful_operations;

        let success_rate = total_successful as f64 / total_operations as f64;

        if success_rate > 0.95 {
            "healthy"
        } else if success_rate > 0.90 {
            "warning"
        } else {
            "critical"
        }
    };

    Ok(Json(serde_json::json!({
        "status": health_status,
        "total_operations": total_operations,
        "cache_hit_ratio": metrics.cache_metrics.hit_ratio,
        "timestamp": chrono::Utc::now()
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::mfa_performance_monitor::OperationMetrics;

    #[test]
    fn test_calculate_health_score() {
        let mut metrics = MfaMetrics::default();

        // Test perfect health score
        let score = calculate_health_score(&metrics, 1.0, 50.0);
        assert!((score - 1.0).abs() < 0.001);

        // Test degraded performance
        let score = calculate_health_score(&metrics, 0.90, 200.0);
        assert!(score < 1.0);
        assert!(score > 0.0);
    }

    #[test]
    fn test_operation_stats_from_metrics() {
        let mut metrics = OperationMetrics::default();
        metrics.total_operations = 100;
        metrics.successful_operations = 95;
        metrics.failed_operations = 5;
        metrics.avg_response_time_ms = 75.0;

        let stats = OperationStats::from_metrics(&metrics);

        assert_eq!(stats.total, 100);
        assert_eq!(stats.successful, 95);
        assert_eq!(stats.failed, 5);
        assert!((stats.success_rate - 0.95).abs() < 0.001);
        assert_eq!(stats.avg_response_time_ms, 75.0);
    }
}
