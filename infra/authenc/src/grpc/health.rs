//! gRPC health check service implementation
//!
//! Implements the standard gRPC health checking protocol (grpc.health.v1.Health)
//! as well as custom health checks for Authenc dependencies.

use std::sync::Arc;
use std::time::Instant;
use tonic::Status;
use tracing::{debug, error, info};

use crate::app::AppState;
use crate::health::checks::{
    DatabaseHealthCheck, KafkaHealthCheck, RedisHealthCheck, SecretonHealthCheck,
};

// Include common proto for health check types
pub mod common {
    pub mod v1 {
        tonic::include_proto!("common.v1");
    }
}

use common::v1::{DependencyHealth, HealthInfo, HealthStatus};

/// Health check service implementation
pub struct HealthService {
    state: Arc<AppState>,
    start_time: Instant,
}

impl HealthService {
    /// Create a new health service
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            start_time: Instant::now(),
        }
    }

    // Note: The into_service method will be implemented when we integrate
    // with the actual gRPC health check protocol in task 9.1

    /// Check database health
    async fn check_database(&self) -> DependencyHealth {
        // Use shared health check implementation
        let checker = DatabaseHealthCheck::new(self.state.database.clone());
        let result = checker.check().await;

        // Convert to proto DependencyHealth
        DependencyHealth {
            name: result.name,
            status: result.status.as_i32(),
            error: result.error,
            response_time_ms: result.response_time_ms,
        }
    }

    /// Check Redis cache health
    async fn check_redis(&self) -> DependencyHealth {
        // Use shared health check implementation
        let checker = RedisHealthCheck::new();
        let result = checker.check().await;

        // Convert to proto DependencyHealth
        DependencyHealth {
            name: result.name,
            status: result.status.as_i32(),
            error: result.error,
            response_time_ms: result.response_time_ms,
        }
    }

    /// Check Secreton vault health
    async fn check_secreton(&self) -> DependencyHealth {
        // Use shared health check implementation
        let checker = SecretonHealthCheck::new();
        let result = checker.check().await;

        // Convert to proto DependencyHealth
        DependencyHealth {
            name: result.name,
            status: result.status.as_i32(),
            error: result.error,
            response_time_ms: result.response_time_ms,
        }
    }

    /// Check Kafka event bus health
    async fn check_kafka(&self) -> DependencyHealth {
        // Use shared health check implementation
        let checker = KafkaHealthCheck::new();
        let result = checker.check().await;

        // Convert to proto DependencyHealth
        DependencyHealth {
            name: result.name,
            status: result.status.as_i32(),
            error: result.error,
            response_time_ms: result.response_time_ms,
        }
    }

    /// Determine overall health status from dependencies
    fn determine_overall_status(
        &self,
        dependencies: &std::collections::HashMap<String, DependencyHealth>,
    ) -> HealthStatus {
        let mut has_unhealthy = false;
        let mut has_degraded = false;

        for dep in dependencies.values() {
            match dep.status {
                s if s == HealthStatus::Unhealthy as i32 => has_unhealthy = true,
                s if s == HealthStatus::Degraded as i32 => has_degraded = true,
                _ => {}
            }
        }

        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Perform comprehensive health check
    pub async fn check_health(&self) -> HealthInfo {
        debug!("Performing health check");

        // Check all dependencies in parallel
        let (db_health, redis_health, secreton_health, kafka_health) = tokio::join!(
            self.check_database(),
            self.check_redis(),
            self.check_secreton(),
            self.check_kafka(),
        );

        // Build dependencies map
        let mut dependencies = std::collections::HashMap::new();
        dependencies.insert("postgresql".to_string(), db_health);
        dependencies.insert("redis".to_string(), redis_health);
        dependencies.insert("secreton".to_string(), secreton_health);
        dependencies.insert("kafka".to_string(), kafka_health);

        // Determine overall status
        let status = self.determine_overall_status(&dependencies);

        let uptime_seconds = self.start_time.elapsed().as_secs() as i64;

        let health_info = HealthInfo {
            service_name: "authenc".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            status: status as i32,
            uptime_seconds,
            dependencies,
        };

        match status {
            HealthStatus::Healthy => {
                debug!("Health check passed: all systems healthy");
            }
            HealthStatus::Degraded => {
                info!("Health check degraded: some systems experiencing issues");
            }
            HealthStatus::Unhealthy => {
                error!("Health check failed: critical systems unhealthy");
            }
            _ => {}
        }

        health_info
    }
}

/// Standard gRPC health check service
///
/// This implements the grpc.health.v1.Health service protocol
pub struct StandardHealthService {
    authenc_health: Arc<HealthService>,
}

impl StandardHealthService {
    /// Create a new standard health service
    pub fn new(authenc_health: Arc<HealthService>) -> Self {
        Self { authenc_health }
    }

    /// Check health for a specific service
    pub async fn check(&self, service: &str) -> Result<ServingStatus, Status> {
        debug!("Standard health check for service: {}", service);

        // If service is empty or "authenc", check our health
        if service.is_empty() || service == "authenc" {
            let health_info = self.authenc_health.check_health().await;

            let status = match health_info.status {
                s if s == HealthStatus::Healthy as i32 => ServingStatus::Serving,
                s if s == HealthStatus::Degraded as i32 => ServingStatus::Serving,
                _ => ServingStatus::NotServing,
            };

            return Ok(status);
        }

        // Unknown service
        Err(Status::not_found(format!("Unknown service: {}", service)))
    }
}

/// Serving status for standard health check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServingStatus {
    /// Service is healthy and serving requests
    Serving,
    /// Service is not serving requests
    NotServing,
    /// Service status is unknown
    Unknown,
}

impl ServingStatus {
    /// Convert to i32 for proto
    pub fn as_i32(&self) -> i32 {
        match self {
            ServingStatus::Serving => 1,
            ServingStatus::NotServing => 2,
            ServingStatus::Unknown => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serving_status_conversion() {
        assert_eq!(ServingStatus::Serving.as_i32(), 1);
        assert_eq!(ServingStatus::NotServing.as_i32(), 2);
        assert_eq!(ServingStatus::Unknown.as_i32(), 0);
    }

    #[tokio::test]
    async fn test_determine_overall_status() {
        use std::collections::HashMap;

        let state = Arc::new(
            AppState::new(crate::config::AppConfig::default())
                .await
                .unwrap(),
        );
        let service = HealthService::new(state);

        // All healthy
        let mut deps = HashMap::new();
        deps.insert(
            "db".to_string(),
            DependencyHealth {
                name: "db".to_string(),
                status: HealthStatus::Healthy as i32,
                error: None,
                response_time_ms: 10,
            },
        );
        assert_eq!(
            service.determine_overall_status(&deps),
            HealthStatus::Healthy
        );

        // One degraded
        deps.insert(
            "cache".to_string(),
            DependencyHealth {
                name: "cache".to_string(),
                status: HealthStatus::Degraded as i32,
                error: Some("Slow response".to_string()),
                response_time_ms: 500,
            },
        );
        assert_eq!(
            service.determine_overall_status(&deps),
            HealthStatus::Degraded
        );

        // One unhealthy
        deps.insert(
            "vault".to_string(),
            DependencyHealth {
                name: "vault".to_string(),
                status: HealthStatus::Unhealthy as i32,
                error: Some("Connection failed".to_string()),
                response_time_ms: 0,
            },
        );
        assert_eq!(
            service.determine_overall_status(&deps),
            HealthStatus::Unhealthy
        );
    }
}
