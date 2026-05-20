//! Health status types

use serde::{Deserialize, Serialize};
use std::fmt;

/// Health status levels
///
/// Represents the health status of a component or the overall system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum HealthStatus {
    /// Component is fully operational
    ///
    /// All checks passed and the component is functioning normally.
    #[default]
    Healthy,

    /// Component is operational but with reduced functionality
    ///
    /// Some non-critical checks failed or performance is degraded,
    /// but the component can still serve requests.
    Degraded,

    /// Component is not operational
    ///
    /// Critical checks failed and the component cannot serve requests.
    Unhealthy,
}

impl HealthStatus {
    /// Returns true if the status is Healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    /// Returns true if the status is Degraded
    pub fn is_degraded(&self) -> bool {
        matches!(self, HealthStatus::Degraded)
    }

    /// Returns true if the status is Unhealthy
    pub fn is_unhealthy(&self) -> bool {
        matches!(self, HealthStatus::Unhealthy)
    }

    /// Returns the severity level as a number (0 = healthy, 1 = degraded, 2 = unhealthy)
    ///
    /// This is useful for comparing statuses or determining the worst status.
    pub fn severity(&self) -> u8 {
        match self {
            HealthStatus::Healthy => 0,
            HealthStatus::Degraded => 1,
            HealthStatus::Unhealthy => 2,
        }
    }

    /// Returns the worst status between self and other
    ///
    /// # Example
    ///
    /// ```rust
    /// use secreton_health::HealthStatus;
    ///
    /// let status1 = HealthStatus::Healthy;
    /// let status2 = HealthStatus::Degraded;
    /// assert_eq!(status1.worst(&status2), HealthStatus::Degraded);
    /// ```
    pub fn worst(&self, other: &HealthStatus) -> HealthStatus {
        if self.severity() >= other.severity() {
            self.clone()
        } else {
            other.clone()
        }
    }
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_checks() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(!HealthStatus::Healthy.is_degraded());
        assert!(!HealthStatus::Healthy.is_unhealthy());

        assert!(!HealthStatus::Degraded.is_healthy());
        assert!(HealthStatus::Degraded.is_degraded());
        assert!(!HealthStatus::Degraded.is_unhealthy());

        assert!(!HealthStatus::Unhealthy.is_healthy());
        assert!(!HealthStatus::Unhealthy.is_degraded());
        assert!(HealthStatus::Unhealthy.is_unhealthy());
    }

    #[test]
    fn test_health_status_severity() {
        assert_eq!(HealthStatus::Healthy.severity(), 0);
        assert_eq!(HealthStatus::Degraded.severity(), 1);
        assert_eq!(HealthStatus::Unhealthy.severity(), 2);
    }

    #[test]
    fn test_health_status_worst() {
        let healthy = HealthStatus::Healthy;
        let degraded = HealthStatus::Degraded;
        let unhealthy = HealthStatus::Unhealthy;

        assert_eq!(healthy.worst(&healthy), HealthStatus::Healthy);
        assert_eq!(healthy.worst(&degraded), HealthStatus::Degraded);
        assert_eq!(healthy.worst(&unhealthy), HealthStatus::Unhealthy);

        assert_eq!(degraded.worst(&healthy), HealthStatus::Degraded);
        assert_eq!(degraded.worst(&degraded), HealthStatus::Degraded);
        assert_eq!(degraded.worst(&unhealthy), HealthStatus::Unhealthy);

        assert_eq!(unhealthy.worst(&healthy), HealthStatus::Unhealthy);
        assert_eq!(unhealthy.worst(&degraded), HealthStatus::Unhealthy);
        assert_eq!(unhealthy.worst(&unhealthy), HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "unhealthy");
    }

    #[test]
    fn test_health_status_default() {
        assert_eq!(HealthStatus::default(), HealthStatus::Healthy);
    }

    #[test]
    fn test_health_status_serialization() {
        let healthy = HealthStatus::Healthy;
        let json = serde_json::to_string(&healthy).unwrap();
        assert_eq!(json, "\"healthy\"");

        let degraded = HealthStatus::Degraded;
        let json = serde_json::to_string(&degraded).unwrap();
        assert_eq!(json, "\"degraded\"");

        let unhealthy = HealthStatus::Unhealthy;
        let json = serde_json::to_string(&unhealthy).unwrap();
        assert_eq!(json, "\"unhealthy\"");
    }

    #[test]
    fn test_health_status_deserialization() {
        let healthy: HealthStatus = serde_json::from_str("\"healthy\"").unwrap();
        assert_eq!(healthy, HealthStatus::Healthy);

        let degraded: HealthStatus = serde_json::from_str("\"degraded\"").unwrap();
        assert_eq!(degraded, HealthStatus::Degraded);

        let unhealthy: HealthStatus = serde_json::from_str("\"unhealthy\"").unwrap();
        assert_eq!(unhealthy, HealthStatus::Unhealthy);
    }
}
