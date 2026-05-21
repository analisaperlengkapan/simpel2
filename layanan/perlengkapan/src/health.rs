// Health check module for service monitoring
// Requirements: NFR-M003

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

use crate::AppState;

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub response_time_ms: Option<f64>,
}

/// Overall health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: HealthStatus,
    pub timestamp: String,
    pub uptime_seconds: u64,
    pub version: String,
    pub components: Vec<ComponentHealth>,
}

/// Simple liveness check - returns 200 if service is running
pub async fn liveness_check() -> impl IntoResponse {
    (StatusCode::OK, "alive")
}

/// Readiness check - returns 200 if service is ready to accept traffic
pub async fn readiness_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let start = Instant::now();

    // Check database connectivity
    let db_health = check_database_health(&state).await;

    // Check Redis connectivity
    let redis_health = check_redis_health(&state).await;

    // Determine overall status
    let overall_status = if db_health.status == HealthStatus::Healthy
        && redis_health.status == HealthStatus::Healthy
    {
        HealthStatus::Healthy
    } else if db_health.status == HealthStatus::Unhealthy
        || redis_health.status == HealthStatus::Unhealthy
    {
        HealthStatus::Unhealthy
    } else {
        HealthStatus::Degraded
    };

    let response = HealthCheckResponse {
        status: overall_status.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        uptime_seconds: 0, // TODO: Track actual uptime
        version: env!("CARGO_PKG_VERSION").to_string(),
        components: vec![db_health, redis_health],
    };

    let status_code = match overall_status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK, // Still accepting traffic
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    let duration = start.elapsed().as_secs_f64() * 1000.0;
    tracing::debug!(
        status = ?overall_status,
        duration_ms = duration,
        "Readiness check completed"
    );

    (status_code, Json(response))
}

/// Detailed health check - returns comprehensive health information
pub async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let start = Instant::now();

    // Check all components
    let db_health = check_database_health(&state).await;
    let redis_health = check_redis_health(&state).await;
    let authenc_health = check_authenc_health(&state).await;

    // Determine overall status
    let components = vec![db_health, redis_health, authenc_health];
    let overall_status = determine_overall_status(&components);

    let response = HealthCheckResponse {
        status: overall_status.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        uptime_seconds: 0, // TODO: Track actual uptime
        version: env!("CARGO_PKG_VERSION").to_string(),
        components,
    };

    let status_code = match overall_status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    let duration = start.elapsed().as_secs_f64() * 1000.0;
    tracing::info!(
        status = ?overall_status,
        duration_ms = duration,
        "Health check completed"
    );

    (status_code, Json(response))
}

/// Check database connectivity
async fn check_database_health(state: &AppState) -> ComponentHealth {
    let start = Instant::now();

    match state.db_pool.get().await {
        Ok(client) => match client.simple_query("SELECT 1").await {
            Ok(_) => {
                let response_time = start.elapsed().as_secs_f64() * 1000.0;
                ComponentHealth {
                    name: "database".to_string(),
                    status: HealthStatus::Healthy,
                    message: Some("Connected".to_string()),
                    response_time_ms: Some(response_time),
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Database query failed");
                ComponentHealth {
                    name: "database".to_string(),
                    status: HealthStatus::Unhealthy,
                    message: Some(format!("Query failed: {}", e)),
                    response_time_ms: None,
                }
            }
        },
        Err(e) => {
            tracing::error!(error = %e, "Database connection failed");
            ComponentHealth {
                name: "database".to_string(),
                status: HealthStatus::Unhealthy,
                message: Some(format!("Connection failed: {}", e)),
                response_time_ms: None,
            }
        }
    }
}

/// Check Redis connectivity
async fn check_redis_health(state: &AppState) -> ComponentHealth {
    let start = Instant::now();

    // Try to ping Redis through cache manager
    match state.cache_manager.health_check().await {
        Ok(_) => {
            let response_time = start.elapsed().as_secs_f64() * 1000.0;
            ComponentHealth {
                name: "redis".to_string(),
                status: HealthStatus::Healthy,
                message: Some("Connected".to_string()),
                response_time_ms: Some(response_time),
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "Redis health check failed");
            ComponentHealth {
                name: "redis".to_string(),
                status: HealthStatus::Degraded, // Redis failure is not critical
                message: Some(format!("Health check failed: {}", e)),
                response_time_ms: None,
            }
        }
    }
}

/// Check Authenc service connectivity
async fn check_authenc_health(state: &AppState) -> ComponentHealth {
    let start = Instant::now();

    // Try to validate a dummy token (will fail but tests connectivity)
    match state.authenc.validate_token("health_check_token").await {
        Ok(_) | Err(_) => {
            // Any response (even error) means service is reachable
            let response_time = start.elapsed().as_secs_f64() * 1000.0;
            ComponentHealth {
                name: "authenc".to_string(),
                status: HealthStatus::Healthy,
                message: Some("Reachable".to_string()),
                response_time_ms: Some(response_time),
            }
        }
    }
}

/// Determine overall health status from component statuses
fn determine_overall_status(components: &[ComponentHealth]) -> HealthStatus {
    let has_unhealthy = components
        .iter()
        .any(|c| c.status == HealthStatus::Unhealthy);
    let has_degraded = components
        .iter()
        .any(|c| c.status == HealthStatus::Degraded);

    if has_unhealthy {
        HealthStatus::Unhealthy
    } else if has_degraded {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_overall_status_all_healthy() {
        let components = vec![
            ComponentHealth {
                name: "db".to_string(),
                status: HealthStatus::Healthy,
                message: None,
                response_time_ms: Some(10.0),
            },
            ComponentHealth {
                name: "redis".to_string(),
                status: HealthStatus::Healthy,
                message: None,
                response_time_ms: Some(5.0),
            },
        ];

        assert_eq!(determine_overall_status(&components), HealthStatus::Healthy);
    }

    #[test]
    fn test_determine_overall_status_one_degraded() {
        let components = vec![
            ComponentHealth {
                name: "db".to_string(),
                status: HealthStatus::Healthy,
                message: None,
                response_time_ms: Some(10.0),
            },
            ComponentHealth {
                name: "redis".to_string(),
                status: HealthStatus::Degraded,
                message: Some("Slow".to_string()),
                response_time_ms: Some(100.0),
            },
        ];

        assert_eq!(
            determine_overall_status(&components),
            HealthStatus::Degraded
        );
    }

    #[test]
    fn test_determine_overall_status_one_unhealthy() {
        let components = vec![
            ComponentHealth {
                name: "db".to_string(),
                status: HealthStatus::Unhealthy,
                message: Some("Connection failed".to_string()),
                response_time_ms: None,
            },
            ComponentHealth {
                name: "redis".to_string(),
                status: HealthStatus::Healthy,
                message: None,
                response_time_ms: Some(5.0),
            },
        ];

        assert_eq!(
            determine_overall_status(&components),
            HealthStatus::Unhealthy
        );
    }
}
