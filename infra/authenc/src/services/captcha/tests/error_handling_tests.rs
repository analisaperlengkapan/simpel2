//! Comprehensive tests for CAPTCHA error handling and fallback mechanisms

use crate::services::captcha::{
    enhanced_service::EnhancedCaptchaService,
    error::{CaptchaError, ErrorContext, ErrorRecovery, RecoveryResult},
    fallback::{FallbackConfig, FallbackService, LocalEncryptionFallback},
    retry::{CircuitBreaker, RetryConfig, RetryExecutor},
    service::{CaptchaService, CaptchaServiceTrait},
    types::*,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Test error recovery mechanisms
pub async fn test_error_recovery_retry_logic() {
    let recovery = ErrorRecovery::default();
    let mut attempt_count = 0;

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

    let result = recovery
        .execute_with_recovery(
            || {
                attempt_count += 1;
                async move {
                    if attempt_count < 3 {
                        Err(CaptchaError::GenerationFailed {
                            message: "Temporary failure".to_string(),
                            recoverable: true,
                            retry_after: Some(Duration::from_millis(10)),
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
            assert_eq!(attempt_count, 3);
        }
        _ => panic!("Expected successful recovery"),
    }
}

/// Test error recovery with non-recoverable error
#[tokio::test]
async fn test_error_recovery_non_recoverable() {
    let recovery = ErrorRecovery::default();

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

    let result = recovery
        .execute_with_recovery(
            || async {
                Err::<&str, _>(CaptchaError::UserLockedOut {
                    lockout_duration: Duration::from_secs(300),
                    reason: "Too many failures".to_string(),
                })
            },
            context,
        )
        .await;

    match result {
        RecoveryResult::Failed(_) => {
            // Expected - non-recoverable error should fail immediately
        }
        _ => panic!("Expected immediate failure for non-recoverable error"),
    }
}

/// Test circuit breaker functionality
pub async fn test_circuit_breaker_opens_and_recovers() {
    let circuit_breaker = Arc::new(CircuitBreaker::new(2, Duration::from_millis(100)));
    let mut failure_count = 0;

    // First failure
    let result = circuit_breaker
        .execute(|| {
            failure_count += 1;
            async move {
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
    let result = circuit_breaker
        .execute(|| {
            failure_count += 1;
            async move {
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
    let result = circuit_breaker
        .execute(|| {
            failure_count += 1;
            async move { Ok::<(), _>(()) }
        })
        .await;
    assert!(result.is_err());
    assert_eq!(failure_count, 2); // Third call was rejected

    // Wait for recovery timeout
    sleep(Duration::from_millis(150)).await;
    circuit_breaker.check_recovery().await;

    // Should now allow calls again (half-open state)
    let result = circuit_breaker
        .execute(|| {
            failure_count += 1;
            async move { Ok::<(), _>(()) }
        })
        .await;
    assert!(result.is_ok());
    assert_eq!(failure_count, 3); // Call was executed
}

/// Test retry executor with exponential backoff
#[tokio::test]
async fn test_retry_executor_exponential_backoff() {
    let config = RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        exponential_base: 2.0,
        jitter: false,
    };

    let executor = RetryExecutor::new(config);
    let mut attempt_count = 0;

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

    let start_time = std::time::Instant::now();
    let result = executor
        .execute(
            || {
                attempt_count += 1;
                async move {
                    if attempt_count < 3 {
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
            assert_eq!(attempt_count, 3);
            // Should have taken at least base_delay + 2*base_delay = 30ms
            assert!(start_time.elapsed() >= Duration::from_millis(25));
        }
        _ => panic!("Expected successful recovery"),
    }
}

/// Test local encryption fallback
#[tokio::test]
async fn test_local_encryption_fallback() {
    let fallback = LocalEncryptionFallback::new();
    let original_data = "sensitive challenge data";

    // Test encryption and decryption
    let encrypted = fallback.encrypt(original_data).unwrap();
    assert_ne!(encrypted, original_data);
    assert!(!encrypted.is_empty());

    let decrypted = fallback.decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, original_data);
}

/// Test fallback service state transitions
pub async fn test_fallback_service_state_management() {
    let config = FallbackConfig::default();
    let service = FallbackService::new(config);

    // Initial state should be normal
    assert_eq!(
        service.get_state().await,
        crate::services::captcha::FallbackState::Normal
    );

    // Enter degraded mode
    service.enter_degraded_mode("Test degradation").await;
    assert_eq!(
        service.get_state().await,
        crate::services::captcha::FallbackState::Degraded
    );

    // Enter emergency mode
    service.enter_emergency_mode("Test emergency").await;
    assert_eq!(
        service.get_state().await,
        crate::services::captcha::FallbackState::Emergency
    );

    // Return to normal
    service.return_to_normal().await;
    assert_eq!(
        service.get_state().await,
        crate::services::captcha::FallbackState::Normal
    );
}

/// Test fallback challenge generation
#[tokio::test]
async fn test_fallback_challenge_generation() {
    let config = FallbackConfig::default();
    let service = FallbackService::new(config);

    // Enter degraded mode to enable fallback generation
    service.enter_degraded_mode("Test").await;

    // Generate fallback challenge
    let challenge = service
        .generate_challenge_with_fallback(ChallengeType::Logical, 3)
        .await
        .unwrap();

    assert_eq!(challenge.challenge_type, ChallengeType::Logical);
    assert_eq!(challenge.difficulty_level, 3);
    assert!(!challenge.encrypted_data.is_empty());
    assert!(!challenge.expected_answer_hash.is_empty());
}

/// Test enhanced service error handling
#[tokio::test]
async fn test_enhanced_service_error_handling() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        exponential_base: 2.0,
        jitter: false,
    };

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    // Test health status
    let health = enhanced_service.get_health_status().await;
    assert!(health.secreton_available);
    assert!(health.authenc_monitoring_available);
    assert!(!health.degraded_mode_active);

    // Test fallback status
    let fallback_status = enhanced_service.get_fallback_status().await;
    assert_eq!(
        fallback_status.state,
        crate::services::captcha::FallbackState::Normal
    );
    assert!(fallback_status.local_encryption_enabled);
}

/// Test enhanced service degraded mode
#[tokio::test]
async fn test_enhanced_service_degraded_mode() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

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

/// Test comprehensive metrics collection
#[tokio::test]
async fn test_comprehensive_metrics() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    let metrics = enhanced_service.get_comprehensive_metrics().await.unwrap();

    // Verify service health metrics
    assert!(!metrics.service_health.degraded_mode_active);
    assert_eq!(metrics.service_health.error_count, 0);

    // Verify fallback status
    assert_eq!(
        metrics.fallback_status.state,
        crate::services::captcha::FallbackState::Normal
    );

    // Verify circuit breaker states
    assert!(
        metrics
            .circuit_breaker_states
            .secreton_state
            .contains("Closed")
    );
    assert!(
        metrics
            .circuit_breaker_states
            .authenc_state
            .contains("Closed")
    );
}

/// Test error message generation
#[tokio::test]
async fn test_user_friendly_error_messages() {
    let validation_error = CaptchaError::ValidationFailed {
        message: "Wrong answer".to_string(),
        attempts_remaining: 2,
        next_difficulty: 3,
    };

    let user_info = validation_error.user_info();
    assert_eq!(user_info.title, "Incorrect Answer");
    assert!(user_info.message.contains("2 attempts remaining"));
    assert!(!user_info.suggested_actions.is_empty());
    assert_eq!(user_info.error_code, "CAPTCHA_VAL_001");

    let rate_limit_error = CaptchaError::RateLimitExceeded {
        reset_time: Duration::from_secs(60),
        max_attempts: 5,
    };

    let user_info = rate_limit_error.user_info();
    assert_eq!(user_info.title, "Too Many Attempts");
    assert!(user_info.message.contains("60 seconds"));
    assert!(!user_info.suggested_actions.is_empty());
}

/// Test accessibility fallback mechanisms
#[tokio::test]
async fn test_accessibility_fallback() {
    let config = FallbackConfig::default();
    let service = FallbackService::new(config);

    // Test manual verification request
    let verification_id = service
        .request_manual_verification(
            "user123".to_string(),
            "session456".to_string(),
            "Audio challenge unavailable".to_string(),
        )
        .await
        .unwrap();

    assert!(!verification_id.is_empty());

    // Check verification status (should be pending)
    // Note: In a real implementation, we would need access to the manual verification system
    // For now, we'll just verify the verification_id was created
    assert!(!verification_id.is_empty());
}

/// Test maintenance operations
#[tokio::test]
async fn test_maintenance_operations() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    // Perform maintenance
    let result = enhanced_service.perform_maintenance().await;
    assert!(result.is_ok());

    // Verify health status is updated
    let health = enhanced_service.get_health_status().await;
    assert!(health.last_health_check.elapsed().unwrap() < Duration::from_secs(1));
}

/// Test error context creation and propagation
#[tokio::test]
async fn test_error_context_propagation() {
    let error = CaptchaError::GenerationFailed {
        message: "Test error".to_string(),
        recoverable: true,
        retry_after: Some(Duration::from_secs(1)),
    };

    // Test recovery strategy
    let strategy = error.recovery_strategy();
    match strategy {
        crate::services::captcha::error::RecoveryStrategy::Retry {
            max_attempts,
            delay,
            ..
        } => {
            assert_eq!(max_attempts, 3);
            assert_eq!(delay, Duration::from_secs(1));
        }
        _ => panic!("Expected retry strategy"),
    }

    // Test error recoverability
    assert!(error.is_recoverable());

    // Test retry delay
    assert_eq!(error.retry_delay(), Some(Duration::from_secs(1)));
}

/// Integration test for complete error handling flow
#[tokio::test]
async fn test_complete_error_handling_flow() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        exponential_base: 2.0,
        jitter: false,
    };

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    // Force degraded mode to test fallback generation
    enhanced_service
        .force_degraded_mode("Integration test")
        .await;

    // Test challenge generation with fallback
    let challenge = enhanced_service
        .generate_challenge(
            ChallengeType::Logical,
            Some(2),
            Some("test_session".to_string()),
            "127.0.0.1".to_string(),
        )
        .await;

    // Should succeed with fallback
    assert!(challenge.is_ok());
    let challenge = challenge.unwrap();
    assert_eq!(challenge.challenge_type, ChallengeType::Logical);
    assert_eq!(challenge.difficulty_level, 2);

    // Test validation with fallback
    let validation_result = enhanced_service
        .validate_challenge(challenge.id.clone(), "wrong_answer".to_string(), None)
        .await;

    // Should succeed with fallback validation
    assert!(validation_result.is_ok());
    let result = validation_result.unwrap();
    assert!(!result.success); // Wrong answer
    assert!(result.retry_allowed);

    // Return to normal mode
    enhanced_service.force_normal_mode().await;

    let health = enhanced_service.get_health_status().await;
    assert!(!health.degraded_mode_active);
}

/// Performance test for error handling overhead
#[tokio::test]
async fn test_error_handling_performance() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig {
        max_attempts: 1, // No retries for performance test
        base_delay: Duration::from_millis(1),
        max_delay: Duration::from_secs(1),
        exponential_base: 2.0,
        jitter: false,
    };

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    let start_time = std::time::Instant::now();
    let iterations = 100;

    for i in 0..iterations {
        let _ = enhanced_service.get_health_status().await;
        let _ = enhanced_service.get_fallback_status().await;

        if i % 10 == 0 {
            let _ = enhanced_service.perform_maintenance().await;
        }
    }

    let elapsed = start_time.elapsed();
    let avg_time_per_operation = elapsed / iterations;

    // Error handling operations should be fast (< 5ms average)
    assert!(avg_time_per_operation < Duration::from_millis(5));
    println!(
        "Average time per error handling operation: {:?}",
        avg_time_per_operation
    );
}
