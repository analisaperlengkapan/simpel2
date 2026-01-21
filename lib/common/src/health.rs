use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
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

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
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

    /// Check if status is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }
}

/// Backward compatible alias for legacy code
pub type ServiceStatus = HealthStatus;

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
    pub fn healthy(name: impl Into<String>, response_time_ms: i64) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            error: None,
            response_time_ms,
        }
    }

    /// Create a new degraded dependency
    pub fn degraded(name: impl Into<String>, response_time_ms: i64, error: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            error: Some(error.into()),
            response_time_ms,
        }
    }

    /// Create a new unhealthy dependency
    pub fn unhealthy(name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            error: Some(error.into()),
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
    /// Create new health info
    pub fn new(service_name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            version: version.into(),
            status: HealthStatus::Healthy,
            uptime_seconds: 0,
            dependencies: HashMap::new(),
        }
    }

    /// Add a dependency check
    pub fn add_dependency(&mut self, dep: DependencyHealth) {
        self.dependencies.insert(dep.name.clone(), dep);
        self.status = Self::determine_overall_status(&self.dependencies);
    }

    /// Determine overall status from dependencies
    pub fn determine_overall_status(dependencies: &HashMap<String, DependencyHealth>) -> HealthStatus {
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

/// Simple health report (backward compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub service: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub checks: Vec<ComponentCheck>,
}

/// Component health check (backward compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentCheck {
    pub component: String,
    pub status: HealthStatus,
    pub message: Option<String>,
}

impl HealthReport {
    pub fn new(service: &str, version: &str) -> Self {
        Self {
            status: HealthStatus::Healthy,
            service: service.to_string(),
            version: version.to_string(),
            timestamp: Utc::now(),
            checks: Vec::new(),
        }
    }

    pub fn add_check(&mut self, check: ComponentCheck) {
        if !check.status.is_healthy() {
            self.status = HealthStatus::Degraded;
        }
        self.checks.push(check);
    }
}
