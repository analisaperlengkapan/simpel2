//! Comprehensive test suite for CAPTCHA error handling and fallback mechanisms

pub mod error_handling_tests;
pub mod integration_tests;
pub mod security_tests;

// Re-export test utilities for use in other test modules
pub use error_handling_tests::*;
pub use integration_tests::*;
pub use security_tests::*;

#[cfg(test)]
mod test_utils {
    use crate::services::captcha::ChallengeType;
    use crate::services::captcha::types::*;
    use std::time::Duration;

    /// Create a test challenge for testing purposes
    pub fn create_test_challenge(
        challenge_type: ChallengeType,
        difficulty: u8,
        session_id: Option<String>,
        ip_address: String,
    ) -> Challenge {
        Challenge::new(
            challenge_type,
            difficulty,
            "encrypted_test_data".to_string(),
            "test_answer_hash".to_string(),
            session_id,
            ip_address,
        )
    }

    /// Create test behavioral metrics with specified classification
    pub fn create_test_behavioral_metrics(
        session_id: String,
        classification: BehaviorClassification,
        risk_score: f64,
    ) -> BehavioralMetrics {
        BehavioralMetrics {
            session_id,
            mouse_movements: vec![MouseEvent {
                x: 100.0,
                y: 150.0,
                timestamp: 1000,
                event_type: "mousemove".to_string(),
                velocity: Some(5.0),
                acceleration: Some(0.5),
            }],
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
                plugins: vec!["plugin1".to_string()],
                canvas_fingerprint: Some("test_canvas_hash".to_string()),
                webgl_fingerprint: Some("test_webgl_hash".to_string()),
            },
            risk_score,
            classification,
        }
    }

    /// Create a validation result for testing
    pub fn create_test_validation_result(
        success: bool,
        confidence_score: f64,
        risk_assessment: RiskLevel,
    ) -> ValidationResult {
        ValidationResult {
            success,
            confidence_score,
            risk_assessment,
            next_difficulty: if success { 1 } else { 3 },
            retry_allowed: true,
            lockout_duration: None,
            message: if success {
                "Test validation successful".to_string()
            } else {
                "Test validation failed".to_string()
            },
        }
    }

    /// Measure execution time of an async operation
    pub async fn measure_execution_time<F, Fut, T>(operation: F) -> (T, Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let start_time = std::time::Instant::now();
        let result = operation().await;
        let elapsed = start_time.elapsed();
        (result, elapsed)
    }

    /// Generate a series of test IP addresses
    pub fn generate_test_ips(count: usize) -> Vec<String> {
        (0..count)
            .map(|i| format!("192.168.{}.{}", i / 254 + 1, i % 254 + 1))
            .collect()
    }

    /// Generate test session IDs
    pub fn generate_test_session_ids(count: usize) -> Vec<String> {
        (0..count)
            .map(|i| format!("test_session_{:04}", i))
            .collect()
    }
}

#[cfg(test)]
pub use test_utils::*;

/// Run all CAPTCHA tests
#[cfg(test)]
mod all_tests {
    use super::*;
    use crate::services::captcha::service::CaptchaServiceTrait;
    use crate::services::captcha::types::ChallengeType;
    use std::time::Duration;

    /// Integration test that runs a comprehensive test suite
    #[tokio::test]
    async fn run_comprehensive_test_suite() {
        println!("Running comprehensive CAPTCHA test suite...");

        // Test error handling
        println!("Testing error handling mechanisms...");
        error_handling_tests::test_error_recovery_retry_logic().await;
        error_handling_tests::test_circuit_breaker_opens_and_recovers().await;
        error_handling_tests::test_fallback_service_state_management().await;

        // Test integration
        println!("Testing integration with external services...");
        integration_tests::test_captcha_with_working_secreton();
        integration_tests::test_captcha_with_secreton_failure();
        integration_tests::test_system_recovery_after_failures();

        // Test security
        println!("Testing security and bot detection...");
        security_tests::test_bot_detection_accuracy_clear_bot().await;
        security_tests::test_bot_detection_accuracy_clear_human().await;
        security_tests::test_adaptive_difficulty_bot_detection().await;

        println!("Comprehensive test suite completed successfully!");
    }

    /// Performance benchmark test
    #[tokio::test]
    async fn benchmark_captcha_operations() {
        use crate::services::captcha::{
            enhanced_service::EnhancedCaptchaService, fallback::FallbackConfig, retry::RetryConfig,
            service::CaptchaService,
        };
        use std::sync::Arc;

        let core_service = Arc::new(CaptchaService::simple().await);
        let fallback_config = FallbackConfig::default();
        let retry_config = RetryConfig::default();

        let enhanced_service =
            EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

        let iterations = 100;
        let start_time = std::time::Instant::now();

        for i in 0..iterations {
            let challenge = enhanced_service
                .generate_challenge(
                    ChallengeType::Visual,
                    Some(2),
                    Some(format!("benchmark_session_{}", i)),
                    format!("192.168.100.{}", i % 254 + 1),
                )
                .await
                .unwrap();

            let _ = enhanced_service
                .validate_challenge(challenge.id, "benchmark_answer".to_string(), None)
                .await;
        }

        let elapsed = start_time.elapsed();
        let avg_time = elapsed / iterations;

        println!("Benchmark results:");
        println!("  Total time: {:?}", elapsed);
        println!("  Average time per operation: {:?}", avg_time);
        println!(
            "  Operations per second: {:.2}",
            1000.0 / avg_time.as_millis() as f64
        );

        // Performance should be reasonable
        assert!(avg_time < Duration::from_millis(250));
    }

    /// Stress test with concurrent operations
    #[tokio::test]
    async fn stress_test_concurrent_operations() {
        use crate::services::captcha::{
            enhanced_service::EnhancedCaptchaService, fallback::FallbackConfig, retry::RetryConfig,
            service::CaptchaService,
        };
        use std::sync::Arc;

        let core_service = Arc::new(CaptchaService::simple().await);
        let fallback_config = FallbackConfig::default();
        let retry_config = RetryConfig::default();

        let enhanced_service = Arc::new(EnhancedCaptchaService::new(
            core_service,
            fallback_config,
            retry_config,
        ));

        let concurrent_operations = 50;
        let mut handles = Vec::new();

        let start_time = std::time::Instant::now();

        for i in 0..concurrent_operations {
            let service = enhanced_service.clone();
            let handle = tokio::spawn(async move {
                let challenge = service
                    .generate_challenge(
                        ChallengeType::Visual,
                        Some((i % 5) + 1),
                        Some(format!("stress_session_{}", i)),
                        format!("10.0.{}.{}", i / 254, i % 254 + 1),
                    )
                    .await?;

                service
                    .validate_challenge(challenge.id, "stress_test_answer".to_string(), None)
                    .await
            });
            handles.push(handle);
        }

        let results = futures::future::join_all(handles).await;
        let elapsed = start_time.elapsed();

        let successful_operations = results
            .into_iter()
            .filter(|result| result.is_ok() && result.as_ref().unwrap().is_ok())
            .count();

        println!("Stress test results:");
        println!("  Concurrent operations: {}", concurrent_operations);
        println!("  Successful operations: {}", successful_operations);
        println!("  Total time: {:?}", elapsed);
        println!(
            "  Success rate: {:.2}%",
            (successful_operations as f64 / concurrent_operations as f64) * 100.0
        );

        // Should handle concurrent operations well
        assert!(successful_operations as f64 / concurrent_operations as f64 > 0.8);
    }
}
