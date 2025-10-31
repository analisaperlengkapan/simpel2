//! Enhanced CAPTCHA Service with Error Handling and Fallback Mechanisms
//!
//! Service implementation with comprehensive error recovery and graceful degradation

use crate::services::captcha::{
    error::{CaptchaError, ErrorContext, ErrorRecovery, RecoveryResult},
    fallback::{FallbackConfig, FallbackService},
    retry::{CircuitBreaker, RetryConfig, RetryExecutor},
    service::{CaptchaService, CaptchaServiceTrait},
    types::*,
};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Enhanced CAPTCHA service with error handling and fallback mechanisms
pub struct EnhancedCaptchaService {
    /// Core CAPTCHA service
    core_service: Arc<CaptchaService>,
    /// Fallback service for degraded mode operations
    fallback_service: Arc<FallbackService>,
    /// Error recovery mechanism
    error_recovery: ErrorRecovery,
    /// Retry executor with circuit breaker
    retry_executor: RetryExecutor,
    /// Circuit breakers for external services
    secreton_circuit_breaker: Arc<CircuitBreaker>,
    authenc_circuit_breaker: Arc<CircuitBreaker>,
    /// Service health status
    service_health: Arc<RwLock<ServiceHealth>>,
}

/// Service health status
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    /// Whether Secreton service is available
    pub secreton_available: bool,
    /// Whether Authenc monitoring is available
    pub authenc_monitoring_available: bool,
    /// Whether database is available
    pub database_available: bool,
    /// Timestamp of last health check
    pub last_health_check: SystemTime,
    /// Whether degraded mode is currently active
    pub degraded_mode_active: bool,
    /// Number of errors since last health check
    pub error_count: u32,
}

impl Default for ServiceHealth {
    fn default() -> Self {
        Self {
            secreton_available: true,
            authenc_monitoring_available: true,
            database_available: true,
            last_health_check: SystemTime::now(),
            degraded_mode_active: false,
            error_count: 0,
        }
    }
}

impl EnhancedCaptchaService {
    /// Create a new enhanced CAPTCHA service with error handling and fallback mechanisms
    pub fn new(
        core_service: Arc<CaptchaService>,
        fallback_config: FallbackConfig,
        retry_config: RetryConfig,
    ) -> Self {
        let fallback_service = Arc::new(FallbackService::new(fallback_config));
        let error_recovery = ErrorRecovery::default();

        // Create circuit breakers for external services
        let secreton_circuit_breaker = Arc::new(CircuitBreaker::new(
            3,                       // failure threshold
            Duration::from_secs(30), // recovery timeout
        ));

        let authenc_circuit_breaker = Arc::new(CircuitBreaker::new(
            5, // higher threshold for monitoring
            Duration::from_secs(60),
        ));

        let retry_executor =
            RetryExecutor::new(retry_config).with_circuit_breaker(secreton_circuit_breaker.clone());

        Self {
            core_service,
            fallback_service,
            error_recovery,
            retry_executor,
            secreton_circuit_breaker,
            authenc_circuit_breaker,
            service_health: Arc::new(RwLock::new(ServiceHealth::default())),
        }
    }

    /// Get fallback service for testing purposes
    pub fn fallback_service(&self) -> &Arc<FallbackService> {
        &self.fallback_service
    }

    /// Create error context for operations
    fn create_error_context(
        &self,
        operation: &str,
        session_id: Option<String>,
        ip_address: Option<String>,
    ) -> ErrorContext {
        ErrorContext {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
            session_id,
            user_id: None, // Would be populated from auth context
            ip_address,
            user_agent: None, // Would be populated from request headers
            request_id: Uuid::new_v4().to_string(),
            component: "captcha_service".to_string(),
            operation: operation.to_string(),
            additional_data: serde_json::json!({}),
        }
    }

    /// Check and update service health
    async fn update_service_health(&self) {
        let mut health = self.service_health.write().await;
        health.last_health_check = SystemTime::now();

        // Check circuit breaker states
        let secreton_state = {
            let state = self.secreton_circuit_breaker.get_state().await;
            state
        };

        let authenc_state = {
            let state = self.authenc_circuit_breaker.get_state().await;
            state
        };

        health.secreton_available =
            secreton_state != crate::services::captcha::retry::CircuitState::Open;
        health.authenc_monitoring_available =
            authenc_state != crate::services::captcha::retry::CircuitState::Open;

        // Check if we should enter degraded mode
        let should_degrade = !health.secreton_available || !health.database_available;

        if should_degrade && !health.degraded_mode_active {
            health.degraded_mode_active = true;
            self.fallback_service
                .enter_degraded_mode("Service unavailability detected")
                .await;
            warn!("Entering degraded mode due to service health issues");
        } else if !should_degrade && health.degraded_mode_active {
            health.degraded_mode_active = false;
            self.fallback_service.return_to_normal().await;
            info!("Returning to normal mode - services recovered");
        }
    }

    /// Execute operation with comprehensive error handling
    async fn execute_with_recovery<T, F, Fut>(
        &self,
        operation: F,
        context: ErrorContext,
    ) -> Result<T, CaptchaError>
    where
        F: Fn() -> Fut + Clone + Send + 'static,
        Fut: std::future::Future<Output = Result<T, CaptchaError>> + Send,
        T: Send + 'static,
    {
        match self
            .retry_executor
            .execute(operation, context.clone())
            .await
        {
            RecoveryResult::Recovered(result) => Ok(result),
            RecoveryResult::FallbackSucceeded(result) => Ok(result),
            RecoveryResult::Failed(error) => {
                // Update error count
                let mut health = self.service_health.write().await;
                health.error_count += 1;
                drop(health);

                // Update service health
                self.update_service_health().await;

                Err(error)
            }
            RecoveryResult::RequiresIntervention(error) => {
                error!(
                    "Manual intervention required for operation {}: {}",
                    context.operation, error
                );
                Err(error)
            }
        }
    }

    /// Generate challenge with fallback support
    async fn generate_challenge_with_fallback(
        &self,
        challenge_type: ChallengeType,
        difficulty: Option<u8>,
        session_id: Option<String>,
        ip_address: String,
    ) -> Result<Challenge, CaptchaError> {
        let context = self.create_error_context(
            "generate_challenge",
            session_id.clone(),
            Some(ip_address.clone()),
        );

        // Try normal generation first
        let core_service = self.core_service.clone();
        let challenge_type_clone = challenge_type.clone();
        let session_id_clone = session_id.clone();
        let ip_address_clone = ip_address.clone();

        let normal_operation = move || {
            let core_service = core_service.clone();
            let challenge_type = challenge_type_clone.clone();
            let session_id = session_id_clone.clone();
            let ip_address = ip_address_clone.clone();

            async move {
                core_service
                    .generate_challenge(challenge_type, difficulty, session_id, ip_address)
                    .await
            }
        };

        match self.execute_with_recovery(normal_operation, context).await {
            Ok(challenge) => Ok(challenge),
            Err(error) => {
                warn!(
                    "Normal challenge generation failed: {}, attempting fallback",
                    error
                );

                // Try fallback generation
                let fallback_difficulty = difficulty.unwrap_or(1);
                self.fallback_service
                    .generate_challenge_with_fallback(challenge_type, fallback_difficulty)
                    .await
            }
        }
    }

    /// Validate challenge with fallback support
    async fn validate_challenge_with_fallback(
        &self,
        challenge_id: String,
        answer: String,
        behavioral_data: Option<BehavioralMetrics>,
    ) -> Result<ValidationResult, CaptchaError> {
        let context = self.create_error_context("validate_challenge", None, None);

        // Check cache first
        let cache_key = format!("validation:{}:{}", challenge_id, answer);
        if let Some(cached_result) = self
            .fallback_service
            .validate_with_cache_fallback(&cache_key)
            .await
        {
            debug!(
                "Using cached validation result for challenge {}",
                challenge_id
            );
            return Ok(cached_result);
        }

        // Try normal validation
        let core_service = self.core_service.clone();
        let challenge_id_clone = challenge_id.clone();
        let answer_clone = answer.clone();
        let behavioral_data_clone = behavioral_data.clone();

        let normal_operation = move || {
            let core_service = core_service.clone();
            let challenge_id = challenge_id_clone.clone();
            let answer = answer_clone.clone();
            let behavioral_data = behavioral_data_clone.clone();

            async move {
                core_service
                    .validate_challenge(challenge_id, answer, behavioral_data)
                    .await
            }
        };

        match self.execute_with_recovery(normal_operation, context).await {
            Ok(result) => {
                // Cache successful validation
                self.fallback_service
                    .cache_validation_result(cache_key, result.clone())
                    .await;
                Ok(result)
            }
            Err(error) => {
                warn!(
                    "Normal challenge validation failed: {}, using fallback",
                    error
                );

                // Fallback validation (simplified)
                Ok(ValidationResult::failure(
                    RiskLevel::Medium,
                    1,    // Reset to easiest difficulty
                    true, // Allow retry
                    None, // No lockout
                    "Validation failed - please try again".to_string(),
                ))
            }
        }
    }

    /// Get service health status
    pub async fn get_health_status(&self) -> ServiceHealth {
        self.update_service_health().await;
        let health = self.service_health.read().await;
        health.clone()
    }

    /// Get fallback service status
    pub async fn get_fallback_status(&self) -> crate::services::captcha::fallback::FallbackStatus {
        self.fallback_service.get_status().await
    }

    /// Force enter degraded mode (for testing or manual intervention)
    pub async fn force_degraded_mode(&self, reason: &str) {
        self.fallback_service.enter_degraded_mode(reason).await;
        let mut health = self.service_health.write().await;
        health.degraded_mode_active = true;
    }

    /// Force return to normal mode
    pub async fn force_normal_mode(&self) {
        self.fallback_service.return_to_normal().await;
        let mut health = self.service_health.write().await;
        health.degraded_mode_active = false;
        health.error_count = 0;
    }

    /// Cleanup expired data and perform maintenance
    pub async fn perform_maintenance(&self) -> Result<(), CaptchaError> {
        // Cleanup fallback service
        self.fallback_service.cleanup().await;

        // Cleanup core service
        let _ = self.core_service.cleanup_expired_challenges().await;

        // Update circuit breaker states
        self.secreton_circuit_breaker.check_recovery().await;
        self.authenc_circuit_breaker.check_recovery().await;

        // Update service health
        self.update_service_health().await;

        Ok(())
    }

    /// Get comprehensive service metrics including error rates
    pub async fn get_comprehensive_metrics(&self) -> Result<ComprehensiveMetrics, CaptchaError> {
        let health = self.get_health_status().await;
        let fallback_status = self.get_fallback_status().await;

        // Get core metrics if available
        let core_metrics = if health.database_available {
            self.core_service.get_real_time_metrics().await.ok()
        } else {
            None
        };

        Ok(ComprehensiveMetrics {
            service_health: health,
            fallback_status,
            core_metrics,
            circuit_breaker_states: CircuitBreakerStates {
                secreton_state: {
                    let state = self.secreton_circuit_breaker.get_state().await;
                    format!("{:?}", state)
                },
                authenc_state: {
                    let state = self.authenc_circuit_breaker.get_state().await;
                    format!("{:?}", state)
                },
            },
        })
    }
}

/// Comprehensive metrics including error handling status
#[derive(Debug, Clone)]
pub struct ComprehensiveMetrics {
    /// Current service health status
    pub service_health: ServiceHealth,
    /// Current fallback service status
    pub fallback_status: crate::services::captcha::fallback::FallbackStatus,
    /// Optional core service metrics summary
    pub core_metrics: Option<crate::services::captcha::metrics::MetricsSummary>,
    /// Current circuit breaker states
    pub circuit_breaker_states: CircuitBreakerStates,
}

/// Circuit breaker states for external services
#[derive(Debug, Clone)]
pub struct CircuitBreakerStates {
    /// Current state of Secreton circuit breaker
    pub secreton_state: String,
    /// Current state of Authenc circuit breaker
    pub authenc_state: String,
}

#[async_trait]
impl CaptchaServiceTrait for EnhancedCaptchaService {
    async fn generate_challenge(
        &self,
        challenge_type: ChallengeType,
        difficulty: Option<u8>,
        session_id: Option<String>,
        ip_address: String,
    ) -> Result<Challenge, CaptchaError> {
        self.generate_challenge_with_fallback(challenge_type, difficulty, session_id, ip_address)
            .await
    }

    async fn validate_challenge(
        &self,
        challenge_id: String,
        answer: String,
        behavioral_data: Option<BehavioralMetrics>,
    ) -> Result<ValidationResult, CaptchaError> {
        self.validate_challenge_with_fallback(challenge_id, answer, behavioral_data)
            .await
    }

    async fn refresh_challenge(&self, challenge_id: String) -> Result<Challenge, CaptchaError> {
        let context = self.create_error_context("refresh_challenge", None, None);

        let core_service = self.core_service.clone();
        let challenge_id_clone = challenge_id.clone();

        let operation = move || {
            let core_service = core_service.clone();
            let challenge_id = challenge_id_clone.clone();

            async move { core_service.refresh_challenge(challenge_id).await }
        };

        self.execute_with_recovery(operation, context).await
    }

    async fn get_challenge(&self, challenge_id: String) -> Result<Challenge, CaptchaError> {
        let context = self.create_error_context("get_challenge", None, None);

        let core_service = self.core_service.clone();
        let challenge_id_clone = challenge_id.clone();

        let operation = move || {
            let core_service = core_service.clone();
            let challenge_id = challenge_id_clone.clone();

            async move { core_service.get_challenge(challenge_id).await }
        };

        self.execute_with_recovery(operation, context).await
    }

    async fn cleanup_expired_challenges(&self) -> Result<u64, CaptchaError> {
        let context = self.create_error_context("cleanup_expired_challenges", None, None);

        let core_service = self.core_service.clone();

        let operation = move || {
            let core_service = core_service.clone();

            async move { core_service.cleanup_expired_challenges().await }
        };

        self.execute_with_recovery(operation, context).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::captcha::service::CaptchaService;

    #[tokio::test]
    async fn test_enhanced_service_creation() {
        let core_service = Arc::new(CaptchaService::simple().await);
        let fallback_config = FallbackConfig::default();
        let retry_config = RetryConfig::default();

        let enhanced_service =
            EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

        let health = enhanced_service.get_health_status().await;
        assert!(health.secreton_available);
        assert!(health.authenc_monitoring_available);
        assert!(health.database_available);
        assert!(!health.degraded_mode_active);
    }

    #[tokio::test]
    async fn test_degraded_mode_transition() {
        let core_service = Arc::new(CaptchaService::simple().await);
        let fallback_config = FallbackConfig::default();
        let retry_config = RetryConfig::default();

        let enhanced_service =
            EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

        // Force degraded mode
        enhanced_service
            .force_degraded_mode("Test degraded mode")
            .await;

        let health = enhanced_service.get_health_status().await;
        assert!(health.degraded_mode_active);

        let fallback_status = enhanced_service.get_fallback_status().await;
        assert_eq!(
            fallback_status.state,
            crate::services::captcha::FallbackState::Degraded
        );

        // Return to normal
        enhanced_service.force_normal_mode().await;

        let health = enhanced_service.get_health_status().await;
        assert!(!health.degraded_mode_active);
    }

    #[tokio::test]
    async fn test_comprehensive_metrics() {
        let core_service = Arc::new(CaptchaService::simple().await);
        let fallback_config = FallbackConfig::default();
        let retry_config = RetryConfig::default();

        let enhanced_service =
            EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

        let metrics = enhanced_service.get_comprehensive_metrics().await.unwrap();

        assert!(!metrics.service_health.degraded_mode_active);
        assert_eq!(
            metrics.fallback_status.state,
            crate::services::captcha::FallbackState::Normal
        );
        assert!(
            metrics
                .circuit_breaker_states
                .secreton_state
                .contains("Closed")
        );
    }
}
