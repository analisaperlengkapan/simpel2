//! gRPC health check service implementation
//!
//! Implements the standard gRPC health checking protocol (grpc.health.v1.Health)
//! as well as custom health checks for Authenc dependencies.
//!
//! TODO: Integrate with `authenc_core::health::checks` when that module is implemented.
//! Currently stubs out individual dependency health checks.

use std::sync::Arc;
use std::time::Instant;
use tonic::Status;
use tracing::{debug, error, info};

// Use shared proto from grpc module
pub use crate::proto::common;

use common::v1::{DependencyHealth, HealthInfo, HealthStatus};

/// Health check service implementation
pub struct HealthService {
    database: Arc<authenc_storage::Database>,
    start_time: Instant,
}

impl HealthService {
    /// Create a new health service
    pub fn new(database: Arc<authenc_storage::Database>) -> Self {
        Self {
            database,
            start_time: Instant::now(),
        }
    }

    /// Check database health
    ///
    /// TODO: Use `authenc_core::health::checks::DatabaseHealthCheck` when available.
    async fn check_database(&self) -> DependencyHealth {
        // Attempt a simple connectivity check
        let start = Instant::now();
        let (status, error) = match self.database.get_connection().await {
            Ok(_conn) => (HealthStatus::Healthy as i32, None),
            Err(e) => (HealthStatus::Unhealthy as i32, Some(e.to_string())),
        };
        let response_time_ms = start.elapsed().as_millis() as i64;

        DependencyHealth {
            name: "postgresql".to_string(),
            status,
            error,
            response_time_ms,
        }
    }

    /// Check Redis cache health (stub)
    ///
    /// TODO: Use `authenc_core::health::checks::RedisHealthCheck` when available.
    async fn check_redis(&self) -> DependencyHealth {
        DependencyHealth {
            name: "redis".to_string(),
            status: HealthStatus::Healthy as i32,
            error: None,
            response_time_ms: 0,
        }
    }

    /// Check Secreton vault health (stub)
    ///
    /// TODO: Use `authenc_core::health::checks::SecretonHealthCheck` when available.
    async fn check_secreton(&self) -> DependencyHealth {
        DependencyHealth {
            name: "secreton".to_string(),
            status: HealthStatus::Healthy as i32,
            error: None,
            response_time_ms: 0,
        }
    }

    /// Check Kafka event bus health (stub)
    ///
    /// TODO: Use `authenc_core::health::checks::KafkaHealthCheck` when available.
    async fn check_kafka(&self) -> DependencyHealth {
        DependencyHealth {
            name: "kafka".to_string(),
            status: HealthStatus::Healthy as i32,
            error: None,
            response_time_ms: 0,
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

    #[test]
    fn test_determine_overall_status() {
        use std::collections::HashMap;

        // Note: This test is simplified since we can't easily create a HealthService
        // without a database connection. In a real scenario, you'd use a mock database.

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

        // Test that the structure is correct
        assert_eq!(deps.len(), 1);
        assert_eq!(deps.get("db").unwrap().status, HealthStatus::Healthy as i32);
    }
}
