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

/// Readiness check — K8s probe. Returns 200 if the service is ready to
/// accept traffic; 503 when any hard dependency (DB, Authenc) is down. We
/// treat Redis failures as `Degraded` (still serve, just with cold cache).
pub async fn readiness_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let start = Instant::now();

    let db_health = check_database_health(&state).await;
    let redis_health = check_redis_health(&state).await;
    // Authenc is on the request hot path (every authenticated request hits
    // `validate_token`), so it belongs in readiness — same spec as DB.
    let authenc_health = check_authenc_health(&state).await;

    let components = vec![db_health, redis_health, authenc_health];
    let overall_status = determine_overall_status(&components);

    let response = HealthCheckResponse {
        status: overall_status.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        uptime_seconds: state.boot_time.elapsed().as_secs(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        components,
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
    let mut components = vec![db_health, redis_health, authenc_health];
    // #36: surface circuit-breaker state for the integrasi data sources
    // (SIMAN / MySIMKARI / MonSAKTI). Soft dependencies — an open breaker is
    // Degraded (fail-fast + cache fallback), never Unhealthy.
    components.extend(check_integrasi_breakers(&state));
    let overall_status = determine_overall_status(&components);

    let response = HealthCheckResponse {
        status: overall_status.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        uptime_seconds: state.boot_time.elapsed().as_secs(),
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

/// Check Authenc service connectivity.
///
/// Uses authenc's `HealthCheck` RPC. The previous implementation probed with
/// `validate_token("health_check_token")` and then matched `Ok(_) | Err(_)`,
/// which had two consequences:
///
/// 1. The arm was total, so this function could only ever return `Healthy` —
///    including when authenc was down. `readiness_check` documents that it
///    returns 503 when a hard dependency is down, and for authenc it never
///    could.
/// 2. That string is not a JWT, so every probe made authenc log
///    `Token validation failed: Invalid token: Invalid JWT format`. Three
///    probes across two replicas kept a permanent error stream running with
///    no user in sight, which is exactly the signal one reads authenc's log
///    to find.
async fn check_authenc_health(state: &AppState) -> ComponentHealth {
    let start = Instant::now();
    let result = state.authenc.health_check().await;
    let response_time = start.elapsed().as_secs_f64() * 1000.0;

    match result {
        Ok(()) => ComponentHealth {
            name: "authenc".to_string(),
            status: HealthStatus::Healthy,
            message: Some("Reachable".to_string()),
            response_time_ms: Some(response_time),
        },
        Err(e) => {
            tracing::error!(error = %e, "Authenc health check failed");
            ComponentHealth {
                name: "authenc".to_string(),
                // Hard dependency: `validate_token` is on the path of every
                // authenticated request, so a perlengkapan replica that cannot
                // reach authenc cannot serve. Readiness sheds it; liveness is
                // a separate, dependency-free endpoint and does not restart it.
                status: HealthStatus::Unhealthy,
                message: Some(format!("Unreachable: {}", e)),
                response_time_ms: Some(response_time),
            }
        }
    }
}

/// Map the integrasi circuit breakers to health components.
///
/// SIMAN / MySIMKARI / MonSAKTI are *soft* dependencies (cache + fallback),
/// so an Open breaker reports `Degraded` — the service still serves, just
/// fail-fast against the tripped source — never `Unhealthy`. Returns an empty
/// vec when integrasi is not configured.
fn check_integrasi_breakers(state: &AppState) -> Vec<ComponentHealth> {
    use crate::shared::resilience::CircuitState;

    let Some(client) = state.integrasi_client.as_ref() else {
        return Vec::new();
    };

    client
        .circuit_states()
        .into_iter()
        .map(|(source, circuit)| {
            let (status, message) = match circuit {
                CircuitState::Closed => (HealthStatus::Healthy, "Circuit tertutup (normal)"),
                CircuitState::HalfOpen => {
                    (HealthStatus::Degraded, "Circuit half-open (memulihkan)")
                }
                CircuitState::Open => (
                    HealthStatus::Degraded,
                    "Circuit terbuka (fail-fast, pakai cache/fallback)",
                ),
            };
            ComponentHealth {
                name: format!("integrasi:{source}"),
                status,
                message: Some(message.to_string()),
                response_time_ms: None,
            }
        })
        .collect()
}

/// Per-source circuit-breaker snapshot for the FE banner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrasiCircuitStatus {
    /// Data source key: "mysimkari" | "siman" | "monsakti".
    pub source: String,
    /// Circuit state: "closed" | "open" | "half_open".
    pub state: String,
    /// True only when the breaker is fully closed (source healthy).
    pub healthy: bool,
    /// Human-readable label for the banner.
    pub label: String,
}

/// GET /integrasi/circuit-status — lightweight, auth'd view of the integrasi
/// circuit breakers for the FE staleness banner. Distinct from the root
/// `/health` doc (which is unauthenticated and broader); this lives under the
/// API base so the WASM client can reach it with its bearer token. Any
/// authenticated user may read it — knowing SIMAN is down is not sensitive.
pub async fn integrasi_circuit_status(
    State(client): State<Option<crate::shared::grpc::clients::IntegrasiClient>>,
    _claims: crate::shared::middleware::Claims,
) -> Json<lib_perlengkapan::response::ApiResponse<Vec<IntegrasiCircuitStatus>>> {
    use crate::shared::resilience::CircuitState;

    let statuses = match client {
        Some(c) => c
            .circuit_states()
            .into_iter()
            .map(|(source, circuit)| {
                let (state, healthy, label) = match circuit {
                    CircuitState::Closed => ("closed", true, "normal"),
                    CircuitState::HalfOpen => ("half_open", false, "memulihkan"),
                    CircuitState::Open => ("open", false, "terganggu"),
                };
                IntegrasiCircuitStatus {
                    source: source.to_string(),
                    state: state.to_string(),
                    healthy,
                    label: label.to_string(),
                }
            })
            .collect(),
        None => Vec::new(),
    };

    Json(lib_perlengkapan::response::ApiResponse::success(
        statuses,
        "Status sirkuit integrasi".to_string(),
    ))
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
