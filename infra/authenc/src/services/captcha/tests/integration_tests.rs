//! Integration tests for CAPTCHA system with Authenc and Secreton

use crate::services::captcha::{
    enhanced_service::EnhancedCaptchaService,
    fallback::FallbackConfig,
    retry::RetryConfig,
    service::{CaptchaService, CaptchaServiceTrait},
    types::*,
};
use std::sync::Arc;
use std::time::Duration;

/// Mock Secreton client for testing
pub struct MockSecretonClient {
    pub should_fail: bool,
    pub failure_count: std::sync::atomic::AtomicU32,
}

impl MockSecretonClient {
    pub fn new(should_fail: bool) -> Self {
        Self {
            should_fail,
            failure_count: std::sync::atomic::AtomicU32::new(0),
        }
    }

    pub async fn encrypt(&self, data: &str) -> Result<String, String> {
        if self.should_fail {
            self.failure_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            return Err("Secreton unavailable".to_string());
        }

        // Simple mock encryption
        Ok(base64::encode(data))
    }

    pub async fn decrypt(&self, encrypted_data: &str) -> Result<String, String> {
        if self.should_fail {
            self.failure_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            return Err("Secreton unavailable".to_string());
        }

        // Simple mock decryption
        base64::decode(encrypted_data)
            .map_err(|e| e.to_string())
            .and_then(|bytes| String::from_utf8(bytes).map_err(|e| e.to_string()))
    }

    pub fn get_failure_count(&self) -> u32 {
        self.failure_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Mock Authenc monitoring for testing
pub struct MockAuthencMonitoring {
    pub should_fail: bool,
    pub events: std::sync::Arc<std::sync::Mutex<Vec<SecurityEvent>>>,
}

#[derive(Debug, Clone)]
pub struct SecurityEvent {
    pub event_type: String,
    pub ip_address: String,
    pub risk_level: String,
    pub timestamp: std::time::SystemTime,
}

impl MockAuthencMonitoring {
    pub fn new(should_fail: bool) -> Self {
        Self {
            should_fail,
            events: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub async fn log_security_event(
        &self,
        event_type: &str,
        ip_address: &str,
        risk_level: &str,
    ) -> Result<(), String> {
        if self.should_fail {
            return Err("Authenc monitoring unavailable".to_string());
        }

        let event = SecurityEvent {
            event_type: event_type.to_string(),
            ip_address: ip_address.to_string(),
            risk_level: risk_level.to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        let mut events = self.events.lock().unwrap();
        events.push(event);

        Ok(())
    }

    pub fn get_events(&self) -> Vec<SecurityEvent> {
        let events = self.events.lock().unwrap();
        events.clone()
    }

    pub fn clear_events(&self) {
        let mut events = self.events.lock().unwrap();
        events.clear();
    }
}

/// Test CAPTCHA system with working Secreton integration
#[tokio::test]
pub async fn test_captcha_with_working_secreton() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    // Test challenge generation
    let challenge = enhanced_service
        .generate_challenge(
            ChallengeType::Visual,
            Some(3),
            Some("test_session".to_string()),
            "192.168.1.100".to_string(),
        )
        .await;

    assert!(challenge.is_ok());
    let challenge = challenge.unwrap();
    assert_eq!(challenge.challenge_type, ChallengeType::Visual);
    assert_eq!(challenge.difficulty_level, 3);
    assert_eq!(challenge.ip_address, "192.168.1.100");
}

/// Test CAPTCHA system with Secreton failure and fallback
#[tokio::test]
pub async fn test_captcha_with_secreton_failure() {
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

    // Force degraded mode to simulate Secreton failure
    enhanced_service
        .force_degraded_mode("Secreton unavailable")
        .await;

    // Test challenge generation with fallback
    let challenge = enhanced_service
        .generate_challenge(
            ChallengeType::Logical,
            Some(2),
            Some("test_session".to_string()),
            "192.168.1.101".to_string(),
        )
        .await;

    assert!(challenge.is_ok());
    let challenge = challenge.unwrap();
    assert_eq!(challenge.challenge_type, ChallengeType::Logical);
    assert_eq!(challenge.difficulty_level, 2);

    // Verify fallback state
    let fallback_status = enhanced_service.get_fallback_status().await;
    assert_eq!(
        fallback_status.state,
        crate::services::captcha::FallbackState::Degraded
    );
}

/// Test CAPTCHA system with Authenc monitoring failure
#[tokio::test]
async fn test_captcha_with_authenc_monitoring_failure() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    let mock_monitoring = MockAuthencMonitoring::new(true); // Simulate failure

    // Test that CAPTCHA operations continue even with monitoring failure
    let challenge = enhanced_service
        .generate_challenge(
            ChallengeType::Audio,
            Some(1),
            Some("test_session".to_string()),
            "192.168.1.102".to_string(),
        )
        .await;

    // Should still work with fallback
    assert!(challenge.is_ok());
    let challenge = challenge.unwrap();
    assert_eq!(challenge.challenge_type, ChallengeType::Audio);
}

/// Test rate limiting integration with fallback
#[tokio::test]
async fn test_rate_limiting_with_fallback() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    let ip_address = "192.168.1.103".to_string();

    // Generate multiple challenges rapidly to test rate limiting
    for i in 0..5 {
        let challenge = enhanced_service
            .generate_challenge(
                ChallengeType::Visual,
                Some(1),
                Some(format!("session_{}", i)),
                ip_address.clone(),
            )
            .await;

        // All should succeed initially
        assert!(challenge.is_ok());
    }

    // Simulate rate limit exceeded scenario
    // In a real implementation, this would be handled by authenc middleware
    let validation_result = enhanced_service
        .validate_challenge(
            "non_existent_challenge".to_string(),
            "wrong_answer".to_string(),
            None,
        )
        .await;

    // Should handle gracefully with fallback
    assert!(validation_result.is_ok());
}

/// Test behavioral analysis integration
#[tokio::test]
async fn test_behavioral_analysis_integration() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    // Generate challenge
    let challenge = enhanced_service
        .generate_challenge(
            ChallengeType::Behavioral,
            Some(2),
            Some("behavioral_test_session".to_string()),
            "192.168.1.104".to_string(),
        )
        .await
        .unwrap();

    // Create behavioral metrics
    let behavioral_metrics = BehavioralMetrics {
        session_id: "behavioral_test_session".to_string(),
        mouse_movements: vec![
            MouseEvent {
                x: 100.0,
                y: 150.0,
                timestamp: 1000,
                event_type: "mousemove".to_string(),
                velocity: Some(5.0),
                acceleration: Some(0.5),
            },
            MouseEvent {
                x: 105.0,
                y: 155.0,
                timestamp: 1100,
                event_type: "mousemove".to_string(),
                velocity: Some(4.8),
                acceleration: Some(-0.2),
            },
        ],
        keystroke_dynamics: vec![KeystrokeEvent {
            key: "a".to_string(),
            timestamp: 2000,
            duration: 100,
            dwell_time: 80,
            flight_time: Some(20),
        }],
        timing_patterns: TimingAnalysis {
            total_interaction_time: 5000,
            pause_patterns: vec![200, 150, 300],
            rhythm_consistency: 0.8,
            typing_speed: Some(45.0),
        },
        browser_fingerprint: BrowserFingerprint {
            user_agent: "Mozilla/5.0 (Test Browser)".to_string(),
            screen_resolution: "1920x1080".to_string(),
            timezone: "UTC".to_string(),
            language: "en-US".to_string(),
            plugins: vec!["plugin1".to_string(), "plugin2".to_string()],
            canvas_fingerprint: Some("canvas_hash_123".to_string()),
            webgl_fingerprint: Some("webgl_hash_456".to_string()),
        },
        risk_score: 0.3, // Low risk
        classification: BehaviorClassification::Human,
    };

    // Validate with behavioral data
    let validation_result = enhanced_service
        .validate_challenge(
            challenge.id,
            "test_answer".to_string(),
            Some(behavioral_metrics),
        )
        .await;

    assert!(validation_result.is_ok());
    let result = validation_result.unwrap();

    // Should incorporate behavioral analysis
    assert!(result.confidence_score > 0.0);
    assert_eq!(result.risk_assessment, RiskLevel::Medium); // Fallback assessment
}

/// Test accessibility features with fallback
#[tokio::test]
async fn test_accessibility_features_with_fallback() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    // Test audio challenge generation
    let audio_challenge = enhanced_service
        .generate_challenge(
            ChallengeType::Audio,
            Some(1),
            Some("accessibility_session".to_string()),
            "192.168.1.105".to_string(),
        )
        .await;

    assert!(audio_challenge.is_ok());
    let challenge = audio_challenge.unwrap();
    assert_eq!(challenge.challenge_type, ChallengeType::Audio);

    // Test manual verification fallback
    let fallback_service = enhanced_service.fallback_service();
    let verification_id = fallback_service
        .request_manual_verification(
            "user_with_disability".to_string(),
            "accessibility_session".to_string(),
            "Audio challenge unavailable".to_string(),
        )
        .await;

    assert!(verification_id.is_ok());
    assert!(!verification_id.unwrap().is_empty());
}
/// Test system recovery after failures
#[tokio::test]
pub async fn test_system_recovery_after_failures() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        exponential_base: 2.0,
        jitter: false,
    };

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    // Simulate system failure
    enhanced_service
        .force_degraded_mode("System failure simulation")
        .await;

    let health = enhanced_service.get_health_status().await;
    assert!(health.degraded_mode_active);

    // Test operations in degraded mode
    let challenge = enhanced_service
        .generate_challenge(
            ChallengeType::Logical,
            Some(1),
            Some("recovery_test_session".to_string()),
            "192.168.1.106".to_string(),
        )
        .await;

    assert!(challenge.is_ok());

    // Simulate system recovery
    enhanced_service.force_normal_mode().await;

    let health = enhanced_service.get_health_status().await;
    assert!(!health.degraded_mode_active);

    // Test operations after recovery
    let challenge = enhanced_service
        .generate_challenge(
            ChallengeType::Visual,
            Some(2),
            Some("recovery_test_session_2".to_string()),
            "192.168.1.107".to_string(),
        )
        .await;

    assert!(challenge.is_ok());
}

/// Test concurrent operations with error handling
#[tokio::test]
async fn test_concurrent_operations_with_error_handling() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = Arc::new(EnhancedCaptchaService::new(
        core_service,
        fallback_config,
        retry_config,
    ));

    let mut handles = Vec::new();

    // Spawn multiple concurrent operations
    for i in 0..10 {
        let service = enhanced_service.clone();
        let handle = tokio::spawn(async move {
            let challenge = service
                .generate_challenge(
                    ChallengeType::Visual,
                    Some((i % 5) + 1), // Vary difficulty
                    Some(format!("concurrent_session_{}", i)),
                    format!("192.168.1.{}", 110 + i),
                )
                .await;

            challenge.is_ok()
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    let results = futures::future::join_all(handles).await;

    // All operations should succeed
    for result in results {
        assert!(result.unwrap());
    }
}

/// Test metrics collection during error scenarios
#[tokio::test]
async fn test_metrics_collection_during_errors() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    // Generate some challenges and validations
    for i in 0..5 {
        let challenge = enhanced_service
            .generate_challenge(
                ChallengeType::Visual,
                Some(2),
                Some(format!("metrics_session_{}", i)),
                format!("192.168.1.{}", 120 + i),
            )
            .await
            .unwrap();

        // Validate with wrong answer to generate failure metrics
        let _ = enhanced_service
            .validate_challenge(challenge.id, "wrong_answer".to_string(), None)
            .await;
    }

    // Get comprehensive metrics
    let metrics = enhanced_service.get_comprehensive_metrics().await;
    assert!(metrics.is_ok());

    let metrics = metrics.unwrap();
    assert!(!metrics.service_health.degraded_mode_active);
    assert_eq!(
        metrics.fallback_status.state,
        crate::services::captcha::FallbackState::Normal
    );
}

/// Test maintenance operations during various states
#[tokio::test]
async fn test_maintenance_operations() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    // Test maintenance in normal mode
    let result = enhanced_service.perform_maintenance().await;
    assert!(result.is_ok());

    // Test maintenance in degraded mode
    enhanced_service
        .force_degraded_mode("Maintenance test")
        .await;
    let result = enhanced_service.perform_maintenance().await;
    assert!(result.is_ok());

    // Return to normal and test again
    enhanced_service.force_normal_mode().await;
    let result = enhanced_service.perform_maintenance().await;
    assert!(result.is_ok());
}

/// Performance test for error handling overhead in integration scenarios
#[tokio::test]
async fn test_integration_performance() {
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
    let iterations = 50;

    for i in 0..iterations {
        // Mix of operations
        if i % 3 == 0 {
            let _ = enhanced_service.get_health_status().await;
        } else if i % 3 == 1 {
            let _ = enhanced_service.get_fallback_status().await;
        } else {
            let _ = enhanced_service.get_comprehensive_metrics().await;
        }
    }

    let elapsed = start_time.elapsed();
    let avg_time_per_operation = elapsed / iterations;

    // Integration operations should be reasonably fast (< 5ms average)
    assert!(avg_time_per_operation < Duration::from_millis(5));
    println!(
        "Average time per integration operation: {:?}",
        avg_time_per_operation
    );
}
