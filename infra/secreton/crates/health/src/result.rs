//! Health check result types

use crate::HealthStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of a single health check
///
/// Contains the status, timing information, and optional details about the check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// The health status of the check
    pub status: HealthStatus,

    /// Optional human-readable message describing the result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Time taken to perform the check in milliseconds
    pub response_time_ms: u64,

    /// Optional additional details as key-value pairs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, serde_json::Value>>,
}

impl HealthCheckResult {
    /// Creates a new healthy result
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: None,
            response_time_ms: 0,
            details: None,
        }
    }

    /// Creates a new healthy result with a message
    pub fn healthy_with_message(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: Some(message.into()),
            response_time_ms: 0,
            details: None,
        }
    }

    /// Creates a new degraded result with a message
    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            response_time_ms: 0,
            details: None,
        }
    }

    /// Creates a new unhealthy result with a message
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            response_time_ms: 0,
            details: None,
        }
    }

    /// Sets the response time
    pub fn with_response_time(mut self, response_time_ms: u64) -> Self {
        self.response_time_ms = response_time_ms;
        self
    }

    /// Sets the details
    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.details = Some(details);
        self
    }

    /// Adds a detail entry
    pub fn add_detail(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.details
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value);
        self
    }
}

/// Results of multiple health checks
///
/// Contains individual check results and provides methods to determine overall status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResults {
    /// Individual health check results keyed by check name
    pub checks: HashMap<String, HealthCheckResult>,

    /// Timestamp when the checks were performed
    pub timestamp: DateTime<Utc>,

    /// Overall system status
    pub overall_status: HealthStatus,
}

impl HealthCheckResults {
    /// Creates a new empty results collection
    pub fn new() -> Self {
        Self {
            checks: HashMap::new(),
            timestamp: Utc::now(),
            overall_status: HealthStatus::Healthy,
        }
    }

    /// Adds a check result
    pub fn add_check(&mut self, name: impl Into<String>, result: HealthCheckResult) {
        self.checks.insert(name.into(), result);
    }

    /// Calculates and returns the overall status based on all checks
    ///
    /// The overall status is determined by:
    /// - If any critical check is Unhealthy, overall is Unhealthy
    /// - If any check is Degraded, overall is Degraded
    /// - Otherwise, overall is Healthy
    pub fn calculate_overall_status(&mut self, critical_checks: &[String]) {
        let mut worst_status = HealthStatus::Healthy;

        for (name, result) in &self.checks {
            let is_critical = critical_checks.contains(name);

            match (&result.status, is_critical) {
                (HealthStatus::Unhealthy, true) => {
                    // Critical unhealthy check makes overall unhealthy
                    worst_status = HealthStatus::Unhealthy;
                    break;
                }
                (HealthStatus::Unhealthy, false) | (HealthStatus::Degraded, _) => {
                    // Non-critical unhealthy or any degraded makes overall degraded
                    worst_status = worst_status.worst(&HealthStatus::Degraded);
                }
                _ => {}
            }
        }

        self.overall_status = worst_status;
    }

    /// Returns the overall status
    pub fn overall_status(&self) -> &HealthStatus {
        &self.overall_status
    }

    /// Returns true if all checks are healthy
    pub fn is_healthy(&self) -> bool {
        self.overall_status.is_healthy()
    }

    /// Returns true if any check is degraded
    pub fn is_degraded(&self) -> bool {
        self.overall_status.is_degraded()
    }

    /// Returns true if any critical check is unhealthy
    pub fn is_unhealthy(&self) -> bool {
        self.overall_status.is_unhealthy()
    }

    /// Returns checks filtered by status
    pub fn checks_by_status(&self, status: &HealthStatus) -> Vec<(&String, &HealthCheckResult)> {
        self.checks
            .iter()
            .filter(|(_, result)| &result.status == status)
            .collect()
    }

    /// Returns the number of checks
    pub fn check_count(&self) -> usize {
        self.checks.len()
    }

    /// Returns the total response time of all checks
    pub fn total_response_time_ms(&self) -> u64 {
        self.checks.values().map(|r| r.response_time_ms).sum()
    }
}

impl Default for HealthCheckResults {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_result_builders() {
        let healthy = HealthCheckResult::healthy();
        assert_eq!(healthy.status, HealthStatus::Healthy);
        assert!(healthy.message.is_none());

        let healthy_msg = HealthCheckResult::healthy_with_message("All good");
        assert_eq!(healthy_msg.status, HealthStatus::Healthy);
        assert_eq!(healthy_msg.message, Some("All good".to_string()));

        let degraded = HealthCheckResult::degraded("Slow response");
        assert_eq!(degraded.status, HealthStatus::Degraded);
        assert_eq!(degraded.message, Some("Slow response".to_string()));

        let unhealthy = HealthCheckResult::unhealthy("Connection failed");
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
        assert_eq!(unhealthy.message, Some("Connection failed".to_string()));
    }

    #[test]
    fn test_health_check_result_with_details() {
        let result = HealthCheckResult::healthy()
            .with_response_time(100)
            .add_detail("connections", serde_json::json!(10))
            .add_detail("latency_ms", serde_json::json!(5));

        assert_eq!(result.response_time_ms, 100);
        assert!(result.details.is_some());
        let details = result.details.unwrap();
        assert_eq!(details.len(), 2);
        assert_eq!(details.get("connections"), Some(&serde_json::json!(10)));
    }

    #[test]
    fn test_health_check_results_overall_status() {
        let mut results = HealthCheckResults::new();

        results.add_check("db", HealthCheckResult::healthy());
        results.add_check("cache", HealthCheckResult::healthy());
        results.calculate_overall_status(&vec!["db".to_string(), "cache".to_string()]);
        assert!(results.is_healthy());

        results.add_check("storage", HealthCheckResult::degraded("Slow"));
        results.calculate_overall_status(&vec!["db".to_string(), "cache".to_string()]);
        assert!(results.is_degraded());

        results.add_check("db", HealthCheckResult::unhealthy("Connection failed"));
        results.calculate_overall_status(&vec!["db".to_string(), "cache".to_string()]);
        assert!(results.is_unhealthy());
    }

    #[test]
    fn test_health_check_results_non_critical() {
        let mut results = HealthCheckResults::new();

        results.add_check("db", HealthCheckResult::healthy());
        results.add_check("metrics", HealthCheckResult::unhealthy("Metrics down"));

        // Only db is critical
        results.calculate_overall_status(&vec!["db".to_string()]);

        // Non-critical unhealthy should make overall degraded, not unhealthy
        assert!(results.is_degraded());
    }

    #[test]
    fn test_health_check_results_filters() {
        let mut results = HealthCheckResults::new();

        results.add_check("db", HealthCheckResult::healthy());
        results.add_check("cache", HealthCheckResult::degraded("Slow"));
        results.add_check("storage", HealthCheckResult::unhealthy("Failed"));

        let healthy_checks = results.checks_by_status(&HealthStatus::Healthy);
        assert_eq!(healthy_checks.len(), 1);

        let degraded_checks = results.checks_by_status(&HealthStatus::Degraded);
        assert_eq!(degraded_checks.len(), 1);

        let unhealthy_checks = results.checks_by_status(&HealthStatus::Unhealthy);
        assert_eq!(unhealthy_checks.len(), 1);
    }

    #[test]
    fn test_health_check_results_metrics() {
        let mut results = HealthCheckResults::new();

        results.add_check(
            "db",
            HealthCheckResult::healthy().with_response_time(10),
        );
        results.add_check(
            "cache",
            HealthCheckResult::healthy().with_response_time(5),
        );

        assert_eq!(results.check_count(), 2);
        assert_eq!(results.total_response_time_ms(), 15);
    }
}
