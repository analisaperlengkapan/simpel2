//! gRPC Health Check Types
//!
//! Standard health check types for the grpc.health.v1.Health protocol.

use serde::{Deserialize, Serialize};

/// Health status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy = 0,
    /// Service is degraded but functional
    Degraded = 1,
    /// Service is unhealthy
    Unhealthy = 2,
    /// Health status unknown
    Unknown = 3,
}

impl HealthStatus {
    /// Convert to i32 for proto
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// Create from i32
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => HealthStatus::Healthy,
            1 => HealthStatus::Degraded,
            2 => HealthStatus::Unhealthy,
            _ => HealthStatus::Unknown,
        }
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "HEALTHY"),
            HealthStatus::Degraded => write!(f, "DEGRADED"),
            HealthStatus::Unhealthy => write!(f, "UNHEALTHY"),
            HealthStatus::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Standard gRPC serving status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServingStatus {
    /// Service is healthy and serving
    Serving = 1,
    /// Service is not serving
    NotServing = 2,
    /// Service status is unknown
    Unknown = 0,
}

impl ServingStatus {
    /// Convert to i32 for proto
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}

/// Dependency health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    /// Dependency name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Error message if unhealthy
    pub error: Option<String>,
    /// Response time in milliseconds
    pub response_time_ms: i64,
}

impl DependencyHealth {
    /// Create a healthy dependency
    pub fn healthy(name: &str, response_time_ms: i64) -> Self {
        Self {
            name: name.to_string(),
            status: HealthStatus::Healthy,
            error: None,
            response_time_ms,
        }
    }

    /// Create an unhealthy dependency
    pub fn unhealthy(name: &str, error: &str) -> Self {
        Self {
            name: name.to_string(),
            status: HealthStatus::Unhealthy,
            error: Some(error.to_string()),
            response_time_ms: 0,
        }
    }

    /// Create a degraded dependency
    pub fn degraded(name: &str, error: &str, response_time_ms: i64) -> Self {
        Self {
            name: name.to_string(),
            status: HealthStatus::Degraded,
            error: Some(error.to_string()),
            response_time_ms,
        }
    }
}

/// Overall service health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthInfo {
    /// Service name
    pub service_name: String,
    /// Service version
    pub version: String,
    /// Overall status
    pub status: HealthStatus,
    /// Uptime in seconds
    pub uptime_seconds: i64,
    /// Dependencies health
    pub dependencies: std::collections::HashMap<String, DependencyHealth>,
}

impl HealthInfo {
    /// Create new health info
    pub fn new(service_name: &str, version: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            version: version.to_string(),
            status: HealthStatus::Unknown,
            uptime_seconds: 0,
            dependencies: std::collections::HashMap::new(),
        }
    }

    /// Determine overall status from dependencies
    pub fn determine_status(&mut self) {
        let mut has_unhealthy = false;
        let mut has_degraded = false;

        for dep in self.dependencies.values() {
            match dep.status {
                HealthStatus::Unhealthy => has_unhealthy = true,
                HealthStatus::Degraded => has_degraded = true,
                _ => {}
            }
        }

        self.status = if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_conversion() {
        assert_eq!(HealthStatus::Healthy.as_i32(), 0);
        assert_eq!(HealthStatus::Unhealthy.as_i32(), 2);
        assert_eq!(HealthStatus::from_i32(1), HealthStatus::Degraded);
    }

    #[test]
    fn test_serving_status() {
        assert_eq!(ServingStatus::Serving.as_i32(), 1);
        assert_eq!(ServingStatus::NotServing.as_i32(), 2);
    }

    #[test]
    fn test_dependency_health_constructors() {
        let healthy = DependencyHealth::healthy("db", 10);
        assert_eq!(healthy.status, HealthStatus::Healthy);

        let unhealthy = DependencyHealth::unhealthy("cache", "Connection failed");
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_info_determine_status() {
        let mut info = HealthInfo::new("test-service", "1.0.0");
        info.dependencies
            .insert("db".to_string(), DependencyHealth::healthy("db", 5));
        info.dependencies.insert(
            "cache".to_string(),
            DependencyHealth::degraded("cache", "Slow", 500),
        );

        info.determine_status();
        assert_eq!(info.status, HealthStatus::Degraded);
    }
}
