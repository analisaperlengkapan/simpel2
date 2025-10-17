//! CAPTCHA Retry Mechanisms
//!
//! Intelligent retry logic with exponential backoff and circuit breaker patterns

use crate::services::captcha::error::{CaptchaError, ErrorContext, RecoveryResult};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Normal operation - requests are allowed
    Closed,
    /// Failing state - requests are rejected
    Open,
    /// Testing recovery - limited requests allowed
    HalfOpen,
}

/// Circuit breaker for external service calls
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    /// Current state of the circuit breaker
    state: Arc<RwLock<CircuitState>>,
    /// Number of consecutive failures
    failure_count: Arc<RwLock<u32>>,
    /// Timestamp of the last failure
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    /// Threshold for opening the circuit
    failure_threshold: u32,
    /// Time to wait before attempting recovery
    recovery_timeout: Duration,
    /// Maximum calls allowed in half-open state
    half_open_max_calls: u32,
    /// Current number of calls in half-open state
    half_open_calls: Arc<RwLock<u32>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given failure threshold and recovery timeout
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            failure_threshold,
            recovery_timeout,
            half_open_max_calls: 3,
            half_open_calls: Arc::new(RwLock::new(0)),
        }
    }

    /// Execute operation through circuit breaker
    pub async fn execute<T, F, Fut>(&self, mut operation: F) -> Result<T, CaptchaError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, CaptchaError>>,
    {
        // Check if circuit is open
        if self.is_open().await {
            return Err(CaptchaError::ExternalServiceError {
                service: "circuit_breaker".to_string(),
                error_code: Some("CIRCUIT_OPEN".to_string()),
                recoverable: false,
            });
        }

        // If half-open, check if we can make the call
        if self.is_half_open().await {
            let mut calls = self.half_open_calls.write().await;
            if *calls >= self.half_open_max_calls {
                return Err(CaptchaError::ExternalServiceError {
                    service: "circuit_breaker".to_string(),
                    error_code: Some("HALF_OPEN_LIMIT".to_string()),
                    recoverable: false,
                });
            }
            *calls += 1;
        }

        // Execute operation
        match operation().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                self.on_failure().await;
                Err(error)
            }
        }
    }

    async fn is_open(&self) -> bool {
        let state = self.state.read().await;
        *state == CircuitState::Open
    }

    async fn is_half_open(&self) -> bool {
        let state = self.state.read().await;
        *state == CircuitState::HalfOpen
    }

    async fn on_success(&self) {
        let mut state = self.state.write().await;
        let mut failure_count = self.failure_count.write().await;
        let mut half_open_calls = self.half_open_calls.write().await;

        *failure_count = 0;
        *half_open_calls = 0;
        *state = CircuitState::Closed;

        debug!("Circuit breaker: Success recorded, state reset to Closed");
    }

    async fn on_failure(&self) {
        let mut state = self.state.write().await;
        let mut failure_count = self.failure_count.write().await;
        let mut last_failure_time = self.last_failure_time.write().await;

        *failure_count += 1;
        *last_failure_time = Some(Instant::now());

        if *failure_count >= self.failure_threshold {
            *state = CircuitState::Open;
            warn!("Circuit breaker: Opened due to {} failures", *failure_count);
        }
    }

    /// Check if circuit should transition from Open to HalfOpen
    pub async fn check_recovery(&self) {
        let mut state = self.state.write().await;
        let last_failure = self.last_failure_time.read().await;

        if *state == CircuitState::Open {
            if let Some(failure_time) = *last_failure {
                if failure_time.elapsed() >= self.recovery_timeout {
                    *state = CircuitState::HalfOpen;
                    let mut half_open_calls = self.half_open_calls.write().await;
                    *half_open_calls = 0;
                    info!("Circuit breaker: Transitioned to HalfOpen for recovery testing");
                }
            }
        }
    }

    /// Get current circuit breaker state
    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff base multiplier
    pub exponential_base: f64,
    /// Whether to add random jitter to delays
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            exponential_base: 2.0,
            jitter: true,
        }
    }
}

/// Retry executor with various backoff strategies
#[derive(Clone)]
pub struct RetryExecutor {
    /// Configuration for retry behavior
    config: RetryConfig,
    /// Optional circuit breaker for additional protection
    circuit_breaker: Option<Arc<CircuitBreaker>>,
}

impl RetryExecutor {
    /// Create a new retry executor with the given configuration
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            circuit_breaker: None,
        }
    }

    /// Add a circuit breaker to this retry executor for additional protection
    pub fn with_circuit_breaker(mut self, circuit_breaker: Arc<CircuitBreaker>) -> Self {
        self.circuit_breaker = Some(circuit_breaker);
        self
    }

    /// Execute operation with retry logic
    pub async fn execute<T, F, Fut>(
        &self,
        mut operation: F,
        context: ErrorContext,
    ) -> RecoveryResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, CaptchaError>>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.config.max_attempts {
            attempts += 1;

            // Execute through circuit breaker if available
            let result = if let Some(cb) = &self.circuit_breaker {
                cb.execute(&mut operation).await
            } else {
                operation().await
            };

            match result {
                Ok(value) => {
                    if attempts > 1 {
                        info!(
                            "Operation succeeded after {} attempts for request {}",
                            attempts, context.request_id
                        );
                        return RecoveryResult::Recovered(value);
                    } else {
                        return RecoveryResult::Recovered(value);
                    }
                }
                Err(error) => {
                    last_error = Some(error.clone());

                    // Check if error is retryable
                    if !self.is_retryable(&error) {
                        warn!(
                            "Non-retryable error encountered: {} for request {}",
                            error, context.request_id
                        );
                        return RecoveryResult::Failed(error);
                    }

                    // Don't retry on last attempt
                    if attempts >= self.config.max_attempts {
                        break;
                    }

                    // Calculate delay
                    let delay = self.calculate_delay(attempts, &error);
                    debug!(
                        "Retrying operation in {:?} (attempt {}/{}) for request {}",
                        delay, attempts, self.config.max_attempts, context.request_id
                    );

                    tokio::time::sleep(delay).await;
                }
            }
        }

        // All retries exhausted
        if let Some(error) = last_error {
            error!(
                "All retry attempts exhausted for request {}: {}",
                context.request_id, error
            );
            RecoveryResult::Failed(error)
        } else {
            RecoveryResult::Failed(CaptchaError::GenerationFailed {
                message: "Unknown error during retry execution".to_string(),
                recoverable: false,
                retry_after: None,
            })
        }
    }

    fn is_retryable(&self, error: &CaptchaError) -> bool {
        match error {
            CaptchaError::GenerationFailed { recoverable, .. } => *recoverable,
            CaptchaError::SecreonUnavailable { .. } => true,
            CaptchaError::DatabaseError { transient, .. } => *transient,
            CaptchaError::NetworkTimeout { .. } => true,
            CaptchaError::SystemOverloaded { .. } => true,
            CaptchaError::ExternalServiceError { recoverable, .. } => *recoverable,
            // Non-retryable errors
            CaptchaError::ValidationFailed { .. } => false,
            CaptchaError::ChallengeNotFound { .. } => false,
            CaptchaError::ChallengeExpired { .. } => false,
            CaptchaError::RateLimitExceeded { .. } => false,
            CaptchaError::UserLockedOut { .. } => false,
            CaptchaError::SuspiciousActivity { .. } => false,
            CaptchaError::ConfigurationError { .. } => false,
            _ => false,
        }
    }

    fn calculate_delay(&self, attempt: u32, error: &CaptchaError) -> Duration {
        // Use error-specific delay if available
        if let Some(delay) = error.retry_delay() {
            return delay;
        }

        // Calculate exponential backoff
        let exponential_delay = self.config.base_delay.as_millis() as f64
            * self.config.exponential_base.powi((attempt - 1) as i32);

        let mut delay = Duration::from_millis(exponential_delay as u64);

        // Apply maximum delay limit
        if delay > self.config.max_delay {
            delay = self.config.max_delay;
        }

        // Add jitter to prevent thundering herd
        if self.config.jitter {
            let jitter_ms = (delay.as_millis() as f64 * 0.1 * rand::random::<f64>()) as u64;
            delay += Duration::from_millis(jitter_ms);
        }

        delay
    }
}

/// Batch retry executor for multiple operations
pub struct BatchRetryExecutor {
    retry_executor: RetryExecutor,
    max_concurrent: usize,
}

impl BatchRetryExecutor {
    /// Create a new batch retry executor with the given retry executor and concurrency limit
    pub fn new(retry_executor: RetryExecutor, max_concurrent: usize) -> Self {
        Self {
            retry_executor,
            max_concurrent,
        }
    }

    /// Execute multiple operations with retry logic
    pub async fn execute_batch<T, F, Fut>(
        &self,
        operations: Vec<(F, ErrorContext)>,
    ) -> Vec<RecoveryResult<T>>
    where
        F: Fn() -> Fut + Clone + Send + 'static,
        Fut: std::future::Future<Output = Result<T, CaptchaError>> + Send,
        T: Send + 'static,
    {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrent));
        let mut handles = Vec::new();

        for (operation, context) in operations {
            let executor = self.retry_executor.clone();
            let semaphore = semaphore.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                executor.execute(operation, context).await
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Task join error: {}", e);
                    results.push(RecoveryResult::Failed(CaptchaError::GenerationFailed {
                        message: format!("Task execution failed: {}", e),
                        recoverable: false,
                        retry_after: None,
                    }));
                }
            }
        }

        results
    }
}

/// Health check for external services
pub struct ServiceHealthChecker {
    circuit_breakers: std::collections::HashMap<String, Arc<CircuitBreaker>>,
}

impl ServiceHealthChecker {
    /// Create a new service health checker
    pub fn new() -> Self {
        Self {
            circuit_breakers: std::collections::HashMap::new(),
        }
    }

    /// Add a service to monitor with its circuit breaker
    pub fn add_service(&mut self, name: String, circuit_breaker: Arc<CircuitBreaker>) {
        self.circuit_breakers.insert(name, circuit_breaker);
    }

    /// Check health of all services and update circuit breakers
    pub async fn check_health(&self) {
        for (service_name, cb) in &self.circuit_breakers {
            cb.check_recovery().await;
            debug!("Health check completed for service: {}", service_name);
        }
    }

    /// Get service health status
    pub async fn get_service_status(&self, service_name: &str) -> Option<CircuitState> {
        if let Some(cb) = self.circuit_breakers.get(service_name) {
            let state = cb.state.read().await;
            Some(state.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(100));
        let counter = Arc::new(AtomicU32::new(0));

        // First failure
        let result = cb
            .execute(|| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>(CaptchaError::GenerationFailed {
                        message: "Test failure".to_string(),
                        recoverable: true,
                        retry_after: None,
                    })
                }
            })
            .await;
        assert!(result.is_err());

        // Second failure should open circuit
        let result = cb
            .execute(|| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>(CaptchaError::GenerationFailed {
                        message: "Test failure".to_string(),
                        recoverable: true,
                        retry_after: None,
                    })
                }
            })
            .await;
        assert!(result.is_err());

        // Third call should be rejected by open circuit
        let result = cb
            .execute(|| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok::<(), _>(())
                }
            })
            .await;
        assert!(result.is_err());

        // Counter should only be 2 (third call was rejected)
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_retry_executor_with_exponential_backoff() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_secs(1),
            exponential_base: 2.0,
            jitter: false,
        };

        let executor = RetryExecutor::new(config);
        let counter = Arc::new(AtomicU32::new(0));

        let context = ErrorContext {
            timestamp: "2023-01-01T00:00:00Z".to_string(),
            session_id: Some("test_session".to_string()),
            user_id: None,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: None,
            request_id: "test_request".to_string(),
            component: "test".to_string(),
            operation: "test_operation".to_string(),
            additional_data: serde_json::json!({}),
        };

        let start_time = Instant::now();
        let result = executor
            .execute(
                || {
                    let counter = counter.clone();
                    async move {
                        let count = counter.fetch_add(1, Ordering::SeqCst);
                        if count < 2 {
                            Err(CaptchaError::GenerationFailed {
                                message: "Test failure".to_string(),
                                recoverable: true,
                                retry_after: None,
                            })
                        } else {
                            Ok("success")
                        }
                    }
                },
                context,
            )
            .await;

        match result {
            RecoveryResult::Recovered(value) => {
                assert_eq!(value, "success");
                assert_eq!(counter.load(Ordering::SeqCst), 3);
                // Should have taken at least base_delay + 2*base_delay = 30ms
                assert!(start_time.elapsed() >= Duration::from_millis(25));
            }
            _ => panic!("Expected successful recovery"),
        }
    }
}
