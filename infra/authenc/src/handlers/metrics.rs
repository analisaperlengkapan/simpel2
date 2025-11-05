//! Prometheus metrics endpoint handler
//!
//! Exposes database connection pool metrics and other system metrics
//! in Prometheus format for monitoring and alerting.

use crate::database::Database;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

/// Prometheus metrics endpoint
///
/// Exposes connection pool metrics in Prometheus format:
/// - authenc_db_pool_size - Current pool size
/// - authenc_db_pool_max_size - Maximum pool size
/// - authenc_db_pool_available - Available connections
/// - authenc_db_pool_waiting - Waiting requests
/// - authenc_db_pool_utilization - Pool utilization percentage
/// - authenc_db_connections_acquired_total - Total connections acquired
/// - authenc_db_connections_failures_total - Total acquisition failures
/// - authenc_db_connections_created_total - Total connections created
/// - authenc_db_connections_reused_total - Total connection reuses
/// - authenc_db_connection_reuse_rate - Connection reuse rate percentage
/// - authenc_db_acquisition_time_avg_microseconds - Average acquisition time
/// - authenc_db_wait_time_avg_microseconds - Average wait time
/// - authenc_db_health_checks_total - Total health checks performed
/// - authenc_db_health_check_success_rate - Health check success rate percentage
pub async fn metrics(State(database): State<Arc<Database>>) -> impl IntoResponse {
    let stats = database.pool_stats();
    let health = database.pool_health();

    let mut output = String::new();

    // Pool size metrics
    output.push_str("# HELP authenc_db_pool_size Current database connection pool size\n");
    output.push_str("# TYPE authenc_db_pool_size gauge\n");
    output.push_str(&format!("authenc_db_pool_size {}\n", stats.size));

    output.push_str("# HELP authenc_db_pool_max_size Maximum database connection pool size\n");
    output.push_str("# TYPE authenc_db_pool_max_size gauge\n");
    output.push_str(&format!("authenc_db_pool_max_size {}\n", stats.max_size));

    output.push_str("# HELP authenc_db_pool_available Available database connections\n");
    output.push_str("# TYPE authenc_db_pool_available gauge\n");
    output.push_str(&format!("authenc_db_pool_available {}\n", stats.available));

    output.push_str("# HELP authenc_db_pool_waiting Requests waiting for database connections\n");
    output.push_str("# TYPE authenc_db_pool_waiting gauge\n");
    output.push_str(&format!("authenc_db_pool_waiting {}\n", stats.waiting));

    output.push_str(
        "# HELP authenc_db_pool_utilization Database connection pool utilization percentage\n",
    );
    output.push_str("# TYPE authenc_db_pool_utilization gauge\n");
    output.push_str(&format!(
        "authenc_db_pool_utilization {:.2}\n",
        stats.utilization
    ));

    // Connection acquisition metrics
    output.push_str(
        "# HELP authenc_db_connections_acquired_total Total database connections acquired\n",
    );
    output.push_str("# TYPE authenc_db_connections_acquired_total counter\n");
    output.push_str(&format!(
        "authenc_db_connections_acquired_total {}\n",
        stats.total_acquired
    ));

    output.push_str("# HELP authenc_db_connections_failures_total Total database connection acquisition failures\n");
    output.push_str("# TYPE authenc_db_connections_failures_total counter\n");
    output.push_str(&format!(
        "authenc_db_connections_failures_total {}\n",
        stats.total_failures
    ));

    output.push_str(
        "# HELP authenc_db_connections_created_total Total new database connections created\n",
    );
    output.push_str("# TYPE authenc_db_connections_created_total counter\n");
    output.push_str(&format!(
        "authenc_db_connections_created_total {}\n",
        stats.total_created
    ));

    output
        .push_str("# HELP authenc_db_connections_reused_total Total database connections reused\n");
    output.push_str("# TYPE authenc_db_connections_reused_total counter\n");
    output.push_str(&format!(
        "authenc_db_connections_reused_total {}\n",
        stats.total_reuses
    ));

    output.push_str(
        "# HELP authenc_db_connection_reuse_rate Database connection reuse rate percentage\n",
    );
    output.push_str("# TYPE authenc_db_connection_reuse_rate gauge\n");
    output.push_str(&format!(
        "authenc_db_connection_reuse_rate {:.2}\n",
        stats.reuse_rate
    ));

    // Timing metrics
    output.push_str("# HELP authenc_db_acquisition_time_avg_microseconds Average database connection acquisition time in microseconds\n");
    output.push_str("# TYPE authenc_db_acquisition_time_avg_microseconds gauge\n");
    output.push_str(&format!(
        "authenc_db_acquisition_time_avg_microseconds {}\n",
        stats.avg_acquisition_time_us
    ));

    output.push_str("# HELP authenc_db_wait_time_avg_microseconds Average database connection wait time in microseconds\n");
    output.push_str("# TYPE authenc_db_wait_time_avg_microseconds gauge\n");
    output.push_str(&format!(
        "authenc_db_wait_time_avg_microseconds {}\n",
        stats.avg_wait_time_us
    ));

    // Health check metrics
    output.push_str(
        "# HELP authenc_db_health_checks_total Total database connection health checks performed\n",
    );
    output.push_str("# TYPE authenc_db_health_checks_total counter\n");
    output.push_str(&format!(
        "authenc_db_health_checks_total {}\n",
        stats.total_health_checks
    ));

    output.push_str("# HELP authenc_db_health_check_success_rate Database connection health check success rate percentage\n");
    output.push_str("# TYPE authenc_db_health_check_success_rate gauge\n");
    output.push_str(&format!(
        "authenc_db_health_check_success_rate {:.2}\n",
        stats.health_check_success_rate
    ));

    // Pool health status (1 = healthy, 0 = unhealthy)
    output.push_str("# HELP authenc_db_pool_healthy Database connection pool health status (1 = healthy, 0 = unhealthy)\n");
    output.push_str("# TYPE authenc_db_pool_healthy gauge\n");
    output.push_str(&format!(
        "authenc_db_pool_healthy {}\n",
        if health.is_healthy() { 1 } else { 0 }
    ));

    (StatusCode::OK, output)
}

/// Enhanced health check endpoint with pool metrics
pub async fn health_with_metrics(State(database): State<Arc<Database>>) -> impl IntoResponse {
    let health = database.pool_health();
    let stats = database.pool_stats();

    let status = if health.is_healthy() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = serde_json::json!({
        "status": health.status(),
        "pool": {
            "size": stats.size,
            "max_size": stats.max_size,
            "available": stats.available,
            "waiting": stats.waiting,
            "utilization": format!("{:.1}%", stats.utilization),
        },
        "metrics": {
            "total_acquired": stats.total_acquired,
            "total_failures": stats.total_failures,
            "reuse_rate": format!("{:.1}%", stats.reuse_rate),
            "avg_acquisition_time_us": stats.avg_acquisition_time_us,
            "avg_wait_time_us": stats.avg_wait_time_us,
            "health_check_success_rate": format!("{:.1}%", stats.health_check_success_rate),
        }
    });

    (status, axum::Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let db = Arc::new(Database::mock().await);

        let (status, _body) = metrics(State(db)).await;

        assert_eq!(status, StatusCode::OK);
        // Body should contain Prometheus metrics
    }
}
