//! Shared health check types and enums

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Overall health status of a service or dependency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Service is fully operational
    Healthy,
    /// Service is operational but experiencing issues
    Degraded,
    /// Service is not operational
    Unhealthy,
    /// Status cannot be determined
    Unknown,
}

impl HealthStatus {
    /// Convert to i32 for protobuf compatibility
    pub fn as_i32(&self) -> i32 {
        match self {
            HealthStatus::Healthy => 1,
            HealthStatus::Degraded => 2,
            HealthStatus::Unhealthy => 3,
            HealthStatus::Unknown => 0,
        }
    }

    /// Convert from i32 (protobuf)
    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => HealthStatus::Healthy,
            2 => HealthStatus::Degraded,
            3 => HealthStatus::Unhealthy,
            _ => HealthStatus::Unknown,
        }
    }

    /// Convert to string for HTTP responses
    pub fn as_str(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "healthy",
            HealthStatus::Degraded => "degraded",
            HealthStatus::Unhealthy => "unhealthy",
            HealthStatus::Unknown => "unknown",
        }
    }
}

/// Health status of a specific dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    /// Name of the dependency (e.g., "postgresql", "redis")
    pub name: String,
    /// Current health status
    pub status: HealthStatus,
    /// Error message if unhealthy
    pub error: Option<String>,
    /// Response time in milliseconds
    pub response_time_ms: i64,
}

impl DependencyHealth {
    /// Create a new healthy dependency
    pub fn healthy(name: String, response_time_ms: i64) -> Self {
        Self {
            name,
            status: HealthStatus::Healthy,
            error: None,
            response_time_ms,
        }
    }

    /// Create a new degraded dependency
    pub fn degraded(name: String, response_time_ms: i64, error: String) -> Self {
        Self {
            name,
            status: HealthStatus::Degraded,
            error: Some(error),
            response_time_ms,
        }
    }

    /// Create a new unhealthy dependency
    pub fn unhealthy(name: String, error: String) -> Self {
        Self {
            name,
            status: HealthStatus::Unhealthy,
            error: Some(error),
            response_time_ms: 0,
        }
    }
}

/// Complete health information for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthInfo {
    /// Service name
    pub service_name: String,
    /// Service version
    pub version: String,
    /// Overall health status
    pub status: HealthStatus,
    /// Uptime in seconds
    pub uptime_seconds: i64,
    /// Health status of dependencies
    pub dependencies: HashMap<String, DependencyHealth>,
}

impl HealthInfo {
    /// Determine overall status from dependencies
    pub fn determine_overall_status(
        dependencies: &HashMap<String, DependencyHealth>,
    ) -> HealthStatus {
        let mut has_unhealthy = false;
        let mut has_degraded = false;

        for dep in dependencies.values() {
            match dep.status {
                HealthStatus::Unhealthy => has_unhealthy = true,
                HealthStatus::Degraded => has_degraded = true,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_conversion() {
        assert_eq!(HealthStatus::Healthy.as_i32(), 1);
        assert_eq!(HealthStatus::Degraded.as_i32(), 2);
        assert_eq!(HealthStatus::Unhealthy.as_i32(), 3);
        assert_eq!(HealthStatus::Unknown.as_i32(), 0);

        assert_eq!(HealthStatus::from_i32(1), HealthStatus::Healthy);
        assert_eq!(HealthStatus::from_i32(2), HealthStatus::Degraded);
        assert_eq!(HealthStatus::from_i32(3), HealthStatus::Unhealthy);
        assert_eq!(HealthStatus::from_i32(999), HealthStatus::Unknown);
    }

    #[test]
    fn test_health_status_strings() {
        assert_eq!(HealthStatus::Healthy.as_str(), "healthy");
        assert_eq!(HealthStatus::Degraded.as_str(), "degraded");
        assert_eq!(HealthStatus::Unhealthy.as_str(), "unhealthy");
        assert_eq!(HealthStatus::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_dependency_health_constructors() {
        let healthy = DependencyHealth::healthy("test".to_string(), 10);
        assert_eq!(healthy.status, HealthStatus::Healthy);
        assert!(healthy.error.is_none());
        assert_eq!(healthy.response_time_ms, 10);

        let degraded = DependencyHealth::degraded("test".to_string(), 500, "Slow".to_string());
        assert_eq!(degraded.status, HealthStatus::Degraded);
        assert_eq!(degraded.error, Some("Slow".to_string()));

        let unhealthy = DependencyHealth::unhealthy("test".to_string(), "Failed".to_string());
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
        assert_eq!(unhealthy.response_time_ms, 0);
    }

    #[test]
    fn test_determine_overall_status() {
        let mut deps = HashMap::new();

        // All healthy
        deps.insert(
            "db".to_string(),
            DependencyHealth::healthy("db".to_string(), 10),
        );
        assert_eq!(
            HealthInfo::determine_overall_status(&deps),
            HealthStatus::Healthy
        );

        // One degraded
        deps.insert(
            "cache".to_string(),
            DependencyHealth::degraded("cache".to_string(), 500, "Slow".to_string()),
        );
        assert_eq!(
            HealthInfo::determine_overall_status(&deps),
            HealthStatus::Degraded
        );

        // One unhealthy
        deps.insert(
            "vault".to_string(),
            DependencyHealth::unhealthy("vault".to_string(), "Failed".to_string()),
        );
        assert_eq!(
            HealthInfo::determine_overall_status(&deps),
            HealthStatus::Unhealthy
        );
    }
}
