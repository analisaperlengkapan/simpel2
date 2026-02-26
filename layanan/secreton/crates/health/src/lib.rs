//! Health check system for Secreton
//!
//! This module provides a comprehensive health check infrastructure that supports:
//! - Multiple health check implementations via the `HealthCheck` trait
//! - Centralized health check registry
//! - Different health status levels (Healthy, Degraded, Unhealthy)
//! - Detailed health check results with timing and metadata
//!
//! # Example
//!
//! ```rust
//! use secreton_health::{HealthCheck, HealthCheckRegistry, HealthStatus};
//! use async_trait::async_trait;
//!
//! struct DatabaseHealthCheck;
//!
//! #[async_trait]
//! impl HealthCheck for DatabaseHealthCheck {
//!     fn name(&self) -> &str {
//!         "database"
//!     }
//!
//!     async fn check(&self) -> HealthCheckResult {
//!         // Perform database health check
//!         HealthCheckResult {
//!             status: HealthStatus::Healthy,
//!             message: Some("Database connection successful".to_string()),
//!             response_time_ms: 10,
//!             details: None,
//!         }
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut registry = HealthCheckRegistry::new();
//!     registry.register(Box::new(DatabaseHealthCheck));
//!
//!     let results = registry.check_all().await;
//!     println!("Overall status: {:?}", results.overall_status());
//! }
//! ```

use async_trait::async_trait;

pub mod checks;
mod error;
mod registry;
mod result;
mod status;

pub use error::HealthCheckError;
pub use registry::HealthCheckRegistry;
pub use result::{HealthCheckResult, HealthCheckResults};
pub use status::HealthStatus;

/// Health check trait that all health checks must implement
///
/// This trait defines the interface for health checks. Each health check
/// should implement this trait to provide its name and check logic.
///
/// # Example
///
/// ```rust
/// use secreton_health::{HealthCheck, HealthCheckResult, HealthStatus};
/// use async_trait::async_trait;
///
/// struct MyHealthCheck;
///
/// #[async_trait]
/// impl HealthCheck for MyHealthCheck {
///     fn name(&self) -> &str {
///         "my_check"
///     }
///
///     async fn check(&self) -> HealthCheckResult {
///         HealthCheckResult {
///             status: HealthStatus::Healthy,
///             message: Some("All systems operational".to_string()),
///             response_time_ms: 5,
///             details: None,
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Returns the name of this health check
    ///
    /// The name should be unique within a registry and should be
    /// descriptive of what is being checked (e.g., "database", "storage", "crypto").
    fn name(&self) -> &str;

    /// Performs the health check
    ///
    /// This method should execute the actual health check logic and return
    /// a `HealthCheckResult` indicating the status, timing, and any relevant details.
    ///
    /// # Returns
    ///
    /// A `HealthCheckResult` containing:
    /// - `status`: The health status (Healthy, Degraded, or Unhealthy)
    /// - `message`: Optional human-readable message
    /// - `response_time_ms`: Time taken to perform the check in milliseconds
    /// - `details`: Optional additional details as key-value pairs
    async fn check(&self) -> HealthCheckResult;

    /// Returns whether this health check is critical
    ///
    /// Critical health checks affect the overall system health status.
    /// If a critical check is unhealthy, the overall status will be unhealthy.
    /// Non-critical checks only affect the overall status if they are degraded.
    ///
    /// Default implementation returns `true` (all checks are critical by default).
    fn is_critical(&self) -> bool {
        true
    }

    /// Returns optional tags for this health check
    ///
    /// Tags can be used to group or filter health checks.
    /// For example: ["kubernetes", "probe"], ["storage", "backend"], etc.
    fn tags(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Type alias for a boxed health check
pub type BoxedHealthCheck = Box<dyn HealthCheck>;

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHealthCheck {
        name: String,
        status: HealthStatus,
    }

    #[async_trait]
    impl HealthCheck for TestHealthCheck {
        fn name(&self) -> &str {
            &self.name
        }

        async fn check(&self) -> HealthCheckResult {
            HealthCheckResult {
                status: self.status.clone(),
                message: Some(format!("Test check: {}", self.name)),
                response_time_ms: 1,
                details: None,
            }
        }
    }

    #[tokio::test]
    async fn test_health_check_trait() {
        let check = TestHealthCheck {
            name: "test".to_string(),
            status: HealthStatus::Healthy,
        };

        assert_eq!(check.name(), "test");
        assert!(check.is_critical());
        assert!(check.tags().is_empty());

        let result = check.check().await;
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.message, Some("Test check: test".to_string()));
    }
}
