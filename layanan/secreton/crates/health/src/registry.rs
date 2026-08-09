//! Health check registry

use crate::{
    BoxedHealthCheck, HealthCheck, HealthCheckError, HealthCheckResult, HealthCheckResults,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};

/// Registry for managing health checks
///
/// The registry allows registering multiple health checks and executing them
/// all at once or individually. It supports:
/// - Registering and unregistering health checks
/// - Executing all checks or specific checks
/// - Filtering checks by tags
/// - Timeout support for individual checks
///
/// # Example
///
/// ```rust
/// use secreton_health::{HealthCheck, HealthCheckRegistry, HealthCheckResult, HealthStatus};
/// use async_trait::async_trait;
///
/// struct DatabaseCheck;
///
/// #[async_trait]
/// impl HealthCheck for DatabaseCheck {
///     fn name(&self) -> &str {
///         "database"
///     }
///
///     async fn check(&self) -> HealthCheckResult {
///         HealthCheckResult::healthy_with_message("Database connected")
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let mut registry = HealthCheckRegistry::new();
///     registry.register(Box::new(DatabaseCheck));
///
///     let results = registry.check_all().await;
///     assert!(results.is_healthy());
/// }
/// ```
pub struct HealthCheckRegistry {
    /// Registered health checks
    checks: Arc<RwLock<HashMap<String, BoxedHealthCheck>>>,

    /// Default timeout for health checks in milliseconds
    default_timeout_ms: u64,

    /// Names of critical health checks
    critical_checks: Arc<RwLock<Vec<String>>>,
}

impl HealthCheckRegistry {
    /// Creates a new empty health check registry
    ///
    /// Uses a default timeout of 5000ms (5 seconds) for health checks.
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            default_timeout_ms: 5000,
            critical_checks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Creates a new health check registry with a custom default timeout
    pub fn with_timeout(timeout_ms: u64) -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            default_timeout_ms: timeout_ms,
            critical_checks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Registers a health check
    ///
    /// # Errors
    ///
    /// Returns an error if a health check with the same name is already registered.
    ///
    /// # Example
    ///
    /// ```rust
    /// use secreton_health::{HealthCheck, HealthCheckRegistry, HealthCheckResult};
    /// use async_trait::async_trait;
    ///
    /// struct MyCheck;
    ///
    /// #[async_trait]
    /// impl HealthCheck for MyCheck {
    ///     fn name(&self) -> &str { "my_check" }
    ///     async fn check(&self) -> HealthCheckResult {
    ///         HealthCheckResult::healthy()
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut registry = HealthCheckRegistry::new();
    ///     registry.register(Box::new(MyCheck)).await.unwrap();
    /// }
    /// ```
    pub async fn register(&mut self, check: BoxedHealthCheck) -> Result<(), HealthCheckError> {
        let name = check.name().to_string();
        let is_critical = check.is_critical();

        let mut checks = self.checks.write().await;

        if checks.contains_key(&name) {
            return Err(HealthCheckError::AlreadyRegistered(name));
        }

        checks.insert(name.clone(), check);

        // Add to critical checks list if critical
        if is_critical {
            let mut critical = self.critical_checks.write().await;
            critical.push(name);
        }

        Ok(())
    }

    /// Unregisters a health check by name
    ///
    /// # Errors
    ///
    /// Returns an error if the health check is not found.
    pub async fn unregister(&mut self, name: &str) -> Result<(), HealthCheckError> {
        let mut checks = self.checks.write().await;

        if checks.remove(name).is_none() {
            return Err(HealthCheckError::NotFound(name.to_string()));
        }

        // Remove from critical checks list
        let mut critical = self.critical_checks.write().await;
        critical.retain(|n| n != name);

        Ok(())
    }

    /// Returns true if a health check with the given name is registered
    pub async fn contains(&self, name: &str) -> bool {
        let checks = self.checks.read().await;
        checks.contains_key(name)
    }

    /// Returns the number of registered health checks
    pub async fn count(&self) -> usize {
        let checks = self.checks.read().await;
        checks.len()
    }

    /// Returns the names of all registered health checks
    pub async fn check_names(&self) -> Vec<String> {
        let checks = self.checks.read().await;
        checks.keys().cloned().collect()
    }

    /// Executes all registered health checks
    ///
    /// Returns a `HealthCheckResults` containing the results of all checks.
    /// Each check is executed with the default timeout.
    ///
    /// # Example
    ///
    /// ```rust
    /// use secreton_health::HealthCheckRegistry;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let registry = HealthCheckRegistry::new();
    ///     let results = registry.check_all().await;
    ///
    ///     println!("Overall status: {:?}", results.overall_status());
    ///     println!("Total checks: {}", results.check_count());
    /// }
    /// ```
    pub async fn check_all(&self) -> HealthCheckResults {
        let checks = self.checks.read().await;
        let critical = self.critical_checks.read().await;

        let mut results = HealthCheckResults::new();

        for (name, check) in checks.iter() {
            let result = self.execute_check(check.as_ref()).await;
            results.add_check(name.clone(), result);
        }

        results.calculate_overall_status(&critical);
        results
    }

    /// Executes a specific health check by name
    ///
    /// # Errors
    ///
    /// Returns an error if the health check is not found.
    pub async fn check_one(&self, name: &str) -> Result<HealthCheckResult, HealthCheckError> {
        let checks = self.checks.read().await;

        let check = checks
            .get(name)
            .ok_or_else(|| HealthCheckError::NotFound(name.to_string()))?;

        Ok(self.execute_check(check.as_ref()).await)
    }

    /// Executes health checks filtered by tags
    ///
    /// Only checks that have at least one of the specified tags will be executed.
    pub async fn check_by_tags(&self, tags: &[String]) -> HealthCheckResults {
        let checks = self.checks.read().await;
        let critical = self.critical_checks.read().await;

        let mut results = HealthCheckResults::new();

        for (name, check) in checks.iter() {
            let check_tags = check.tags();
            let has_matching_tag = tags.iter().any(|tag| check_tags.contains(tag));

            if has_matching_tag {
                let result = self.execute_check(check.as_ref()).await;
                results.add_check(name.clone(), result);
            }
        }

        results.calculate_overall_status(&critical);
        results
    }

    /// Executes a single health check with timeout
    async fn execute_check(&self, check: &dyn HealthCheck) -> HealthCheckResult {
        let start = Instant::now();
        let timeout_duration = Duration::from_millis(self.default_timeout_ms);

        match timeout(timeout_duration, check.check()).await {
            Ok(mut result) => {
                // Update response time if not already set
                if result.response_time_ms == 0 {
                    result.response_time_ms = start.elapsed().as_millis() as u64;
                }
                result
            }
            Err(_) => {
                // Timeout occurred
                HealthCheckResult::unhealthy(format!(
                    "Health check timed out after {}ms",
                    self.default_timeout_ms
                ))
                .with_response_time(self.default_timeout_ms)
            }
        }
    }

    /// Sets the default timeout for health checks
    pub fn set_default_timeout(&mut self, timeout_ms: u64) {
        self.default_timeout_ms = timeout_ms;
    }

    /// Gets the default timeout for health checks
    pub fn default_timeout(&self) -> u64 {
        self.default_timeout_ms
    }
}

impl Default for HealthCheckRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HealthStatus;
    use async_trait::async_trait;

    struct TestCheck {
        name: String,
        status: HealthStatus,
        delay_ms: u64,
    }

    #[async_trait]
    impl HealthCheck for TestCheck {
        fn name(&self) -> &str {
            &self.name
        }

        async fn check(&self) -> HealthCheckResult {
            if self.delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
            }

            HealthCheckResult {
                status: self.status.clone(),
                message: Some(format!("Test check: {}", self.name)),
                response_time_ms: self.delay_ms,
                details: None,
            }
        }
    }

    #[tokio::test]
    async fn test_registry_register() {
        let mut registry = HealthCheckRegistry::new();

        let check = Box::new(TestCheck {
            name: "test".to_string(),
            status: HealthStatus::Healthy,
            delay_ms: 0,
        });

        assert!(registry.register(check).await.is_ok());
        assert!(registry.contains("test").await);
        assert_eq!(registry.count().await, 1);
    }

    #[tokio::test]
    async fn test_registry_duplicate_registration() {
        let mut registry = HealthCheckRegistry::new();

        let check1 = Box::new(TestCheck {
            name: "test".to_string(),
            status: HealthStatus::Healthy,
            delay_ms: 0,
        });

        let check2 = Box::new(TestCheck {
            name: "test".to_string(),
            status: HealthStatus::Healthy,
            delay_ms: 0,
        });

        assert!(registry.register(check1).await.is_ok());
        assert!(registry.register(check2).await.is_err());
    }

    #[tokio::test]
    async fn test_registry_unregister() {
        let mut registry = HealthCheckRegistry::new();

        let check = Box::new(TestCheck {
            name: "test".to_string(),
            status: HealthStatus::Healthy,
            delay_ms: 0,
        });

        registry.register(check).await.unwrap();
        assert!(registry.contains("test").await);

        assert!(registry.unregister("test").await.is_ok());
        assert!(!registry.contains("test").await);
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_registry_check_all() {
        let mut registry = HealthCheckRegistry::new();

        registry
            .register(Box::new(TestCheck {
                name: "check1".to_string(),
                status: HealthStatus::Healthy,
                delay_ms: 0,
            }))
            .await
            .unwrap();

        registry
            .register(Box::new(TestCheck {
                name: "check2".to_string(),
                status: HealthStatus::Healthy,
                delay_ms: 0,
            }))
            .await
            .unwrap();

        let results = registry.check_all().await;
        assert_eq!(results.check_count(), 2);
        assert!(results.is_healthy());
    }

    #[tokio::test]
    async fn test_registry_check_one() {
        let mut registry = HealthCheckRegistry::new();

        registry
            .register(Box::new(TestCheck {
                name: "test".to_string(),
                status: HealthStatus::Healthy,
                delay_ms: 0,
            }))
            .await
            .unwrap();

        let result = registry.check_one("test").await.unwrap();
        assert_eq!(result.status, HealthStatus::Healthy);

        let err = registry.check_one("nonexistent").await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn test_registry_timeout() {
        let mut registry = HealthCheckRegistry::with_timeout(100);

        registry
            .register(Box::new(TestCheck {
                name: "slow".to_string(),
                status: HealthStatus::Healthy,
                delay_ms: 200, // Longer than timeout
            }))
            .await
            .unwrap();

        let result = registry.check_one("slow").await.unwrap();
        assert_eq!(result.status, HealthStatus::Unhealthy);
        assert!(result.message.unwrap().contains("timed out"));
    }

    #[tokio::test]
    async fn test_registry_check_names() {
        let mut registry = HealthCheckRegistry::new();

        registry
            .register(Box::new(TestCheck {
                name: "check1".to_string(),
                status: HealthStatus::Healthy,
                delay_ms: 0,
            }))
            .await
            .unwrap();

        registry
            .register(Box::new(TestCheck {
                name: "check2".to_string(),
                status: HealthStatus::Healthy,
                delay_ms: 0,
            }))
            .await
            .unwrap();

        let names = registry.check_names().await;
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"check1".to_string()));
        assert!(names.contains(&"check2".to_string()));
    }
}
