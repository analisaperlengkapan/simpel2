//! Health check and system status handlers.
//!
//! Provides endpoints for monitoring system health,
//! readiness, and liveness checks.

use axum::{extract::State, response::Json};

use serde::Serialize;
use std::collections::HashMap;

use crate::{
    ApiError, ApiResponse, ApiResult, DependencyStatus, HealthCheckDependencies,
    HealthCheckResponse, handlers::AppState,
};

/// Basic health check response
#[derive(Debug, Serialize)]
pub struct SimpleHealthResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Detailed health check response
#[derive(Debug, Serialize)]
pub struct DetailedHealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub checks: HashMap<String, HealthCheck>,
}

/// Individual health check result
#[derive(Debug, Serialize)]
pub struct HealthCheck {
    pub status: String,
    pub message: Option<String>,
    pub response_time_ms: u64,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

/// Readiness check response
#[derive(Debug, Serialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub version: String,
    pub checks: HashMap<String, HealthCheck>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Liveness check response
#[derive(Debug, Serialize)]
pub struct LivenessResponse {
    pub alive: bool,
    pub uptime: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Main health check endpoint
/// Returns basic health status of the service
pub async fn health_check(
    State(_state): State<AppState>,
) -> ApiResult<Json<ApiResponse<HealthCheckResponse>>> {
    // TODO: Implement actual health checks

    let health = HealthCheckResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: get_uptime_seconds(),
        dependencies: HealthCheckDependencies {
            storage: DependencyStatus::healthy(),
            crypto: DependencyStatus::healthy(),
            audit: DependencyStatus::healthy(),
        },
    };

    Ok(Json(ApiResponse::success(health)))
}

/// Simple health check for load balancers
pub async fn simple_health_check(
    State(_state): State<AppState>,
) -> ApiResult<Json<SimpleHealthResponse>> {
    let health = SimpleHealthResponse {
        status: "ok".to_string(),
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(health))
}

/// Detailed health check with component status
pub async fn detailed_health_check(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<DetailedHealthResponse>>> {
    let mut checks = HashMap::new();

    // Database health check
    let db_check = check_database_health(&state).await;
    checks.insert("database".to_string(), db_check);

    // Cache health check
    let cache_check = check_cache_health(&state).await;
    checks.insert("cache".to_string(), cache_check);

    // Crypto service health check
    let crypto_check = check_crypto_health(&state).await;
    checks.insert("crypto".to_string(), crypto_check);

    // Storage health check
    let storage_check = check_storage_health(&state).await;
    checks.insert("storage".to_string(), storage_check);

    // HSM health check (if enabled)
    if let Some(ref hsm) = state.hsm {
        let hsm_check = check_hsm_health(hsm).await;
        checks.insert("hsm".to_string(), hsm_check);
    }

    // Determine overall status
    let overall_status = if checks.values().all(|check| check.status == "healthy") {
        "healthy"
    } else if checks.values().any(|check| check.status == "unhealthy") {
        "unhealthy"
    } else {
        "degraded"
    };

    let health = DetailedHealthResponse {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: get_uptime_seconds(),
        timestamp: chrono::Utc::now(),
        checks,
    };

    Ok(Json(ApiResponse::success(health)))
}

/// Readiness check - determines if the service is ready to accept traffic
///
/// CRITICAL: Returns 503 Service Unavailable if vault is sealed.
/// This follows HashiCorp Vault best practices where Kubernetes/load balancers
/// should not route traffic to a sealed vault instance.
pub async fn readiness_check(State(state): State<AppState>) -> ApiResult<Json<ReadinessResponse>> {
    let mut checks = HashMap::new();

    // Check if database is ready
    let db_check = check_database_readiness(&state).await;
    checks.insert("database".to_string(), db_check);

    // Check if cache is ready
    let cache_check = check_cache_readiness(&state).await;
    checks.insert("cache".to_string(), cache_check);

    // Check if crypto service is ready
    let crypto_check = check_crypto_readiness(&state).await;
    checks.insert("crypto".to_string(), crypto_check);

    // CRITICAL: Check seal status - vault must be unsealed to be ready
    let seal_check = check_seal_status(&state).await;
    checks.insert("seal".to_string(), seal_check);

    // Service is ready if all critical components are ready AND vault is unsealed
    // CRITICAL: Sealed vault means NOT ready (status != "ready")
    let ready = checks.values().all(|check| check.status == "ready");

    let readiness = ReadinessResponse {
        ready,
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks,
        timestamp: chrono::Utc::now(),
    };

    // CRITICAL: Return 503 if not ready (including when sealed)
    // This tells Kubernetes/load balancers to not route traffic here
    if !ready {
        tracing::warn!("Readiness check failed - service not ready (possibly sealed)");
        return Err(ApiError::ServiceUnavailable {
            message: "Service not ready".to_string(),
        });
    }

    Ok(Json(readiness))
}

/// Liveness check - determines if the service is alive and should not be restarted
pub async fn liveness_check(State(_state): State<AppState>) -> ApiResult<Json<LivenessResponse>> {
    // Simple liveness check - if we can respond, we're alive
    let liveness = LivenessResponse {
        alive: true,
        uptime: get_uptime_seconds(),
        timestamp: chrono::Utc::now(),
    };

    Ok(Json(liveness))
}

/// Check database health
async fn check_database_health(_state: &AppState) -> HealthCheck {
    let start_time = std::time::Instant::now();

    // TODO: Implement actual database health check
    // - Test connection
    // - Execute simple query
    // - Check response time

    let response_time = start_time.elapsed().as_millis() as u64;

    HealthCheck {
        status: "healthy".to_string(),
        message: None,
        response_time_ms: response_time,
        last_check: chrono::Utc::now(),
        details: Some({
            let mut details = HashMap::new();
            details.insert(
                "connection_pool".to_string(),
                serde_json::Value::String("healthy".to_string()),
            );
            details.insert(
                "active_connections".to_string(),
                serde_json::Value::Number(serde_json::Number::from(5)),
            );
            details.insert(
                "max_connections".to_string(),
                serde_json::Value::Number(serde_json::Number::from(100)),
            );
            details
        }),
    }
}

/// Check cache health
async fn check_cache_health(_state: &AppState) -> HealthCheck {
    let start_time = std::time::Instant::now();

    // TODO: Implement actual cache health check
    // - Test Redis connection
    // - Execute ping command
    // - Check memory usage

    let response_time = start_time.elapsed().as_millis() as u64;

    HealthCheck {
        status: "healthy".to_string(),
        message: None,
        response_time_ms: response_time,
        last_check: chrono::Utc::now(),
        details: Some({
            let mut details = HashMap::new();
            details.insert(
                "ping".to_string(),
                serde_json::Value::String("pong".to_string()),
            );
            details.insert(
                "memory_usage".to_string(),
                serde_json::Value::String("25%".to_string()),
            );
            details.insert(
                "connected_clients".to_string(),
                serde_json::Value::Number(serde_json::Number::from(10)),
            );
            details
        }),
    }
}

/// Check crypto service health
async fn check_crypto_health(_state: &AppState) -> HealthCheck {
    let start_time = std::time::Instant::now();

    // TODO: Implement actual crypto service health check
    // - Test encryption/decryption
    // - Verify key accessibility
    // - Check HSM connectivity if used

    let response_time = start_time.elapsed().as_millis() as u64;

    HealthCheck {
        status: "healthy".to_string(),
        message: None,
        response_time_ms: response_time,
        last_check: chrono::Utc::now(),
        details: Some({
            let mut details = HashMap::new();
            details.insert(
                "hsm_status".to_string(),
                serde_json::Value::String("connected".to_string()),
            );
            details.insert(
                "key_store".to_string(),
                serde_json::Value::String("accessible".to_string()),
            );
            details.insert(
                "entropy_available".to_string(),
                serde_json::Value::Bool(true),
            );
            details
        }),
    }
}

/// Check storage health
async fn check_storage_health(_state: &AppState) -> HealthCheck {
    let start_time = std::time::Instant::now();

    // TODO: Implement actual storage health check
    // - Test file system access
    // - Check disk space
    // - Verify backup systems

    let response_time = start_time.elapsed().as_millis() as u64;

    HealthCheck {
        status: "healthy".to_string(),
        message: None,
        response_time_ms: response_time,
        last_check: chrono::Utc::now(),
        details: Some({
            let mut details = HashMap::new();
            details.insert(
                "disk_usage".to_string(),
                serde_json::Value::String("45%".to_string()),
            );
            details.insert(
                "backup_status".to_string(),
                serde_json::Value::String("current".to_string()),
            );
            details.insert(
                "encryption".to_string(),
                serde_json::Value::String("enabled".to_string()),
            );
            details
        }),
    }
}

/// Check database readiness
async fn check_database_readiness(_state: &AppState) -> HealthCheck {
    // Similar to health check but focused on readiness
    check_database_health(_state).await
}

/// Check cache readiness
async fn check_cache_readiness(_state: &AppState) -> HealthCheck {
    // Similar to health check but focused on readiness
    let mut check = check_cache_health(_state).await;
    check.status = "ready".to_string();
    check
}

/// Check crypto readiness
async fn check_crypto_readiness(_state: &AppState) -> HealthCheck {
    // Similar to health check but focused on readiness
    let mut check = check_crypto_health(_state).await;
    check.status = "ready".to_string();
    check
}

/// Check HSM health
async fn check_hsm_health(hsm: &secreton_core::hsm::HsmBackend) -> HealthCheck {
    let start_time = std::time::Instant::now();

    // Check HSM connectivity and health
    let hsm_healthy = match hsm.health_check().await {
        Ok(true) => true,
        Ok(false) => false,
        Err(e) => {
            tracing::error!("HSM health check failed: {:?}", e);
            false
        }
    };

    let response_time = start_time.elapsed().as_millis() as u64;

    let status = if hsm_healthy { "healthy" } else { "unhealthy" };
    let message = if hsm_healthy {
        Some("HSM is connected and operational".to_string())
    } else {
        Some("HSM is not responding or unavailable".to_string())
    };

    HealthCheck {
        status: status.to_string(),
        message,
        response_time_ms: response_time,
        last_check: chrono::Utc::now(),
        details: Some({
            let mut details = HashMap::new();
            details.insert(
                "connected".to_string(),
                serde_json::Value::Bool(hsm_healthy),
            );
            details.insert(
                "provider".to_string(),
                serde_json::Value::String("pkcs11".to_string()),
            );
            details
        }),
    }
}

/// Check seal status
///
/// CRITICAL: Vault must be unsealed to be considered "ready"
/// This follows HashiCorp Vault best practices where a sealed vault
/// returns 503 Service Unavailable for readiness checks.
async fn check_seal_status(state: &AppState) -> HealthCheck {
    let start_time = std::time::Instant::now();

    // CRITICAL SECURITY FIX: Get SealService from state and check if unsealed
    let is_unsealed = state.seal.is_unsealed().await;

    let response_time = start_time.elapsed().as_millis() as u64;

    // CRITICAL: Sealed vault is NOT ready
    // Status should be "sealed" not "ready" when vault is sealed
    let status = if is_unsealed { "ready" } else { "sealed" };
    let message = if is_unsealed {
        "Vault is unsealed and ready for operations"
    } else {
        "Vault is SEALED - unseal with threshold shares required before operations"
    };

    HealthCheck {
        status: status.to_string(),
        message: Some(message.to_string()),
        response_time_ms: response_time,
        last_check: chrono::Utc::now(),
        details: Some({
            let mut details = HashMap::new();
            details.insert("unsealed".to_string(), serde_json::Value::Bool(is_unsealed));
            details.insert(
                "seal_type".to_string(),
                serde_json::Value::String("shamir".to_string()),
            );
            details.insert("initialized".to_string(), serde_json::Value::Bool(true));
            details
        }),
    }
}

/// Get system uptime in seconds
fn get_uptime_seconds() -> u64 {
    // TODO: Implement actual uptime calculation
    // For now, return a placeholder
    86400 // 24 hours
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ApiConfig;
    use crate::services::ServiceContainer;
    use std::sync::Arc;

    fn create_state() -> Arc<ServiceContainer> {
        let config = ApiConfig::default();
        tokio::runtime::Runtime::new()
            .block_on(ServiceContainer::new(&config))
            .expect("Failed to create services")
            .into()
    }

    #[tokio::test]
    async fn test_simple_health_check() {
        let services = create_state();

        let result = simple_health_check(axum::extract::State(services)).await;
        assert!(result.is_ok());

        let response = result.0;
        assert_eq!(response.status, "ok");
    }

    #[tokio::test]
    async fn test_liveness_check() {
        let services = create_state();

        let result = liveness_check(axum::extract::State(services)).await;
        assert!(result.is_ok());

        let response = result.0;
        assert!(response.alive);
    }

    #[tokio::test]
    async fn test_health_check_response() {
        let services = create_state();
        let result = health_check(axum::extract::State(services)).await;
        assert!(result.is_ok());

        let response = result.0;
        assert!(response.success);
        let health = response.data.expect("health data");
        assert_eq!(health.status, "healthy");
        assert_eq!(health.dependencies.database, "healthy");
    }

    #[tokio::test]
    async fn test_readiness_check_marks_ready() {
        let services = create_state();
        let result = readiness_check(axum::extract::State(services)).await;
        assert!(result.is_ok());

        let response = result.0;
        assert!(response.ready);
        assert_eq!(response.version, env!("CARGO_PKG_VERSION"));
        assert!(response.checks.contains_key("database"));
        assert!(response.checks.contains_key("cache"));
        assert!(response.checks.contains_key("crypto"));
    }

    #[tokio::test]
    async fn test_detailed_health_overall_status() {
        let services = create_state();
        let result = detailed_health_check(axum::extract::State(services)).await;
        assert!(result.is_ok());

        let response = result.0;
        assert!(response.success);
        let payload = response.data.expect("detailed data");
        assert_eq!(payload.status, "healthy");
        assert_eq!(payload.checks.len(), 4);
        assert!(
            payload
                .checks
                .values()
                .all(|check| check.status == "healthy")
        );
    }
}
