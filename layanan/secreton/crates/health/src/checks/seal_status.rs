//! Seal status health check
//!
//! This health check verifies whether Secreton is sealed or unsealed.
//! When sealed, the health check returns Unhealthy status, indicating
//! that the system cannot serve requests until it is unsealed.
//!
//! **Validates: Requirements 2.6.2** - Health check system must report seal status

use crate::{HealthCheck, HealthCheckResult, HealthStatus};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;

/// Trait for accessing seal status
///
/// This trait abstracts the seal status check to allow for testing
/// and different implementations.
#[async_trait]
pub trait SealStatusProvider: Send + Sync {
    /// Returns true if the system is sealed
    async fn is_sealed(&self) -> bool;
}

/// Seal status health check
///
/// This health check monitors the seal status of Secreton and reports:
/// - `Unhealthy` if Secreton is sealed (cannot serve requests)
/// - `Healthy` if Secreton is unsealed (operational)
///
/// This is a critical health check that affects Kubernetes readiness probes.
/// When sealed, the pod should not receive traffic until it is unsealed.
///
/// # Example
///
/// ```rust
/// use secreton_health::{HealthCheck, HealthCheckRegistry};
/// use secreton_health::checks::SealStatusHealthCheck;
/// use secreton_core::services::seal::SealService;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() {
///     let seal_service = Arc::new(SealService::new(Default::default()));
///     let health_check = SealStatusHealthCheck::new(seal_service);
///
///     let mut registry = HealthCheckRegistry::new();
///     registry.register(Box::new(health_check)).await.unwrap();
///
///     let results = registry.check_all().await;
///     println!("Seal status: {:?}", results.overall_status());
/// }
/// ```
pub struct SealStatusHealthCheck<P: SealStatusProvider> {
    provider: Arc<P>,
    name: String,
}

impl<P: SealStatusProvider> SealStatusHealthCheck<P> {
    /// Creates a new seal status health check
    ///
    /// # Arguments
    ///
    /// * `provider` - The seal status provider (typically a SealService)
    pub fn new(provider: Arc<P>) -> Self {
        Self {
            provider,
            name: "seal_status".to_string(),
        }
    }

    /// Creates a new seal status health check with a custom name
    pub fn with_name(provider: Arc<P>, name: String) -> Self {
        Self { provider, name }
    }
}

#[async_trait]
impl<P: SealStatusProvider + 'static> HealthCheck for SealStatusHealthCheck<P> {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();

        let is_sealed = self.provider.is_sealed().await;
        let response_time_ms = start.elapsed().as_millis() as u64;

        if is_sealed {
            HealthCheckResult {
                status: HealthStatus::Unhealthy,
                message: Some(
                    "Secreton is sealed. Unseal with threshold shares to restore service."
                        .to_string(),
                ),
                response_time_ms,
                details: Some(
                    vec![("sealed".to_string(), serde_json::json!(true))]
                        .into_iter()
                        .collect(),
                ),
            }
        } else {
            HealthCheckResult {
                status: HealthStatus::Healthy,
                message: Some("Secreton is unsealed and operational".to_string()),
                response_time_ms,
                details: Some(
                    vec![("sealed".to_string(), serde_json::json!(false))]
                        .into_iter()
                        .collect(),
                ),
            }
        }
    }

    fn is_critical(&self) -> bool {
        // Seal status is critical - if sealed, the system cannot serve requests
        true
    }

    fn tags(&self) -> Vec<String> {
        vec![
            "seal".to_string(),
            "security".to_string(),
            "kubernetes".to_string(),
            "readiness".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// Mock seal status provider for testing
    struct MockSealStatusProvider {
        sealed: AtomicBool,
    }

    impl MockSealStatusProvider {
        fn new(sealed: bool) -> Self {
            Self {
                sealed: AtomicBool::new(sealed),
            }
        }

        fn set_sealed(&self, sealed: bool) {
            self.sealed.store(sealed, Ordering::SeqCst);
        }
    }

    #[async_trait]
    impl SealStatusProvider for MockSealStatusProvider {
        async fn is_sealed(&self) -> bool {
            self.sealed.load(Ordering::SeqCst)
        }
    }

    #[tokio::test]
    async fn test_seal_status_check_when_sealed() {
        let provider = Arc::new(MockSealStatusProvider::new(true));
        let check = SealStatusHealthCheck::new(provider);

        let result = check.check().await;

        assert_eq!(result.status, HealthStatus::Unhealthy);
        assert!(result.message.unwrap().contains("sealed"));
        assert_eq!(
            result.details.unwrap().get("sealed").unwrap(),
            &serde_json::json!(true)
        );
    }

    #[tokio::test]
    async fn test_seal_status_check_when_unsealed() {
        let provider = Arc::new(MockSealStatusProvider::new(false));
        let check = SealStatusHealthCheck::new(provider);

        let result = check.check().await;

        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.message.unwrap().contains("unsealed"));
        assert_eq!(
            result.details.unwrap().get("sealed").unwrap(),
            &serde_json::json!(false)
        );
    }

    #[tokio::test]
    async fn test_seal_status_check_is_critical() {
        let provider = Arc::new(MockSealStatusProvider::new(true));
        let check = SealStatusHealthCheck::new(provider);

        assert!(check.is_critical());
    }

    #[tokio::test]
    async fn test_seal_status_check_has_correct_tags() {
        let provider = Arc::new(MockSealStatusProvider::new(true));
        let check = SealStatusHealthCheck::new(provider);

        let tags = check.tags();
        assert!(tags.contains(&"seal".to_string()));
        assert!(tags.contains(&"security".to_string()));
        assert!(tags.contains(&"kubernetes".to_string()));
        assert!(tags.contains(&"readiness".to_string()));
    }

    #[tokio::test]
    async fn test_seal_status_check_custom_name() {
        let provider = Arc::new(MockSealStatusProvider::new(true));
        let check = SealStatusHealthCheck::with_name(provider, "custom_seal_check".to_string());

        assert_eq!(check.name(), "custom_seal_check");
    }

    #[tokio::test]
    async fn test_seal_status_check_response_time() {
        let provider = Arc::new(MockSealStatusProvider::new(false));
        let check = SealStatusHealthCheck::new(provider);

        let result = check.check().await;

        // Response time should be very fast (< 100ms for in-memory check)
        assert!(result.response_time_ms < 100);
    }

    #[tokio::test]
    async fn test_seal_status_transitions() {
        let provider = Arc::new(MockSealStatusProvider::new(true));
        let check = SealStatusHealthCheck::new(provider.clone());

        // Initially sealed
        let result = check.check().await;
        assert_eq!(result.status, HealthStatus::Unhealthy);

        // Unseal
        provider.set_sealed(false);
        let result = check.check().await;
        assert_eq!(result.status, HealthStatus::Healthy);

        // Seal again
        provider.set_sealed(true);
        let result = check.check().await;
        assert_eq!(result.status, HealthStatus::Unhealthy);
    }
}
