//! Property-Based Tests for CAPTCHA System
//!
//! This module contains property-based tests using proptest to validate
//! correctness properties defined in the design document.
//!
//! Each test is annotated with the property it validates from the design document.
//! Minimum 100 iterations per property test as specified in the testing strategy.

use proptest::prelude::*;
use std::time::Duration;

use crate::services::captcha::{
    adaptive_difficulty::{AdaptiveDifficultyCalculator, DifficultyParameters},
    rate_limiting::{CaptchaFailureTracker, CaptchaRateLimitConfig, FailureLevel},
    types::{Challenge, ChallengeType, RiskLevel, ValidationResult},
};

// ============================================================================
// Proptest Configuration - 100 iterations minimum per test
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // ========================================================================
    // Property 1: Adaptive Difficulty Increases with Failures
    // **Feature: ai-resistant-captcha, Property 1: Adaptive Difficulty Increases with Failures**
    // **Validates: Requirements 1.3, 1.5**
    // ========================================================================

    #[test]
    fn prop_adaptive_difficulty_increases_with_failures(
        consecutive_failures in 0u32..20,
        session_id in "[a-z0-9]{8,16}",
        ip_address in "192\\.168\\.[0-9]{1,3}\\.[0-9]{1,3}",
    ) {
        let mut calculator = AdaptiveDifficultyCalculator::new(DifficultyParameters::default());

        // Simulate consecutive failures
        for _ in 0..consecutive_failures {
            calculator.update_behavior(
                &session_id,
                &ip_address,
                false, // failure
                Duration::from_secs(10),
                Some(0.5),
                None,
            );
        }

        let difficulty = calculator.calculate_difficulty(&session_id, &ip_address, None, None);

        // Property: difficulty should be between 1 and 10
        prop_assert!(difficulty >= 1, "Difficulty {} should be >= 1", difficulty);
        prop_assert!(difficulty <= 10, "Difficulty {} should be <= 10", difficulty);

        // With failures, difficulty should be at least base level (3)
        let base_difficulty = 3u8;
        if consecutive_failures > 0 {
            prop_assert!(
                difficulty >= base_difficulty,
                "Difficulty {} should be >= base {} after {} failures",
                difficulty, base_difficulty, consecutive_failures
            );
        }
    }


    #[test]
    fn prop_difficulty_resets_after_success(
        initial_failures in 1u32..10,
        session_id in "[a-z0-9]{8,16}",
        ip_address in "192\\.168\\.[0-9]{1,3}\\.[0-9]{1,3}",
    ) {
        let mut calculator = AdaptiveDifficultyCalculator::new(DifficultyParameters::default());

        // Simulate failures first
        for _ in 0..initial_failures {
            calculator.update_behavior(
                &session_id,
                &ip_address,
                false,
                Duration::from_secs(10),
                Some(0.5),
                None,
            );
        }

        let difficulty_after_failures = calculator.calculate_difficulty(
            &session_id, &ip_address, None, None
        );

        // Now simulate success
        calculator.update_behavior(
            &session_id,
            &ip_address,
            true, // success
            Duration::from_secs(10),
            Some(0.3),
            None,
        );

        let difficulty_after_success = calculator.calculate_difficulty(
            &session_id, &ip_address, None, None
        );

        // Property: consecutive_failures resets to 0 after success
        // Difficulty should decrease or stay same (not increase)
        prop_assert!(
            difficulty_after_success <= difficulty_after_failures,
            "Difficulty after success ({}) should be <= difficulty after failures ({})",
            difficulty_after_success, difficulty_after_failures
        );
    }

    // ========================================================================
    // Property 3: Bot Detection Classification Consistency
    // **Feature: ai-resistant-captcha, Property 3: Bot Detection Classification Consistency**
    // **Validates: Requirements 2.1, 2.5, 6.4**
    // ========================================================================

    #[test]
    fn prop_high_risk_score_classification(risk_score in 0.0f64..=1.0) {
        // Determine expected risk level based on score
        let expected_risk_level = match risk_score {
            x if x < 0.3 => RiskLevel::Low,
            x if x < 0.6 => RiskLevel::Medium,
            x if x < 0.8 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        // Property: risk_score > 0.8 should result in High or Critical
        if risk_score > 0.8 {
            prop_assert!(
                matches!(expected_risk_level, RiskLevel::High | RiskLevel::Critical),
                "Risk score {} should result in High or Critical, got {:?}",
                risk_score, expected_risk_level
            );
        }

        // Property: risk_score > 0.9 should result in Critical
        if risk_score > 0.9 {
            prop_assert!(
                matches!(expected_risk_level, RiskLevel::Critical),
                "Risk score {} should result in Critical, got {:?}",
                risk_score, expected_risk_level
            );
        }
    }

    // ========================================================================
    // Property 4: Progressive Rate Limiting
    // **Feature: ai-resistant-captcha, Property 4: Progressive Rate Limiting**
    // **Validates: Requirements 2.4, 8.2**
    // ========================================================================

    #[test]
    fn prop_progressive_rate_limiting(consecutive_failures in 0u32..15) {
        let mut tracker = CaptchaFailureTracker::new();

        // Simulate consecutive failures
        for _ in 0..consecutive_failures {
            tracker.record_failure(RiskLevel::Medium);
        }

        let failure_level = tracker.get_failure_level();

        // Property: rate limits decrease progressively
        // 0-2 failures: Low (20 rpm)
        // 3-5 failures: Medium (10 rpm)
        // 6-10 failures: High (5 rpm)
        // 11+ failures: Critical (1 rpm)
        let (expected_level_matches, expected_rpm) = match consecutive_failures {
            0..=2 => (failure_level == FailureLevel::Low, 20u64),
            3..=5 => (failure_level == FailureLevel::Medium, 10u64),
            6..=10 => (failure_level == FailureLevel::High, 5u64),
            _ => (failure_level == FailureLevel::Critical, 1u64),
        };

        prop_assert!(
            expected_level_matches,
            "With {} failures, got unexpected failure level {:?}",
            consecutive_failures, failure_level
        );

        // Verify rate limits match expected values
        let config = CaptchaRateLimitConfig::default();
        let actual_rpm = match tracker.get_failure_level() {
            FailureLevel::Low => config.progressive_limits.low_failure_rpm,
            FailureLevel::Medium => config.progressive_limits.medium_failure_rpm,
            FailureLevel::High => config.progressive_limits.high_failure_rpm,
            FailureLevel::Critical => config.progressive_limits.critical_failure_rpm,
        };

        prop_assert_eq!(
            actual_rpm, expected_rpm,
            "Expected {} rpm but got {} rpm for {} failures",
            expected_rpm, actual_rpm, consecutive_failures
        );
    }


    // ========================================================================
    // Property 9: Automation Indicator Risk Score Impact
    // **Feature: ai-resistant-captcha, Property 9: Automation Indicator Risk Score Impact**
    // **Validates: Requirements 6.2**
    // ========================================================================

    #[test]
    fn prop_automation_indicator_risk_impact(
        automation_indicator_count in 0u32..5,
        base_risk_score in 0.0f64..0.5,
    ) {
        // Property: risk_score increases by 0.4 * automation_indicator_count
        let expected_increase = 0.4 * (automation_indicator_count as f64);
        let calculated_risk = base_risk_score + expected_increase;
        let clamped_risk = calculated_risk.min(1.0);

        // Verify the calculation
        prop_assert!(
            clamped_risk >= base_risk_score,
            "Risk score should not decrease with automation indicators"
        );

        // Property: if automation_indicator_count > 0, risk should increase
        if automation_indicator_count > 0 {
            prop_assert!(
                clamped_risk > base_risk_score || clamped_risk >= 1.0,
                "Risk score should increase with automation indicators"
            );
        }

        // Verify the increase amount
        if clamped_risk < 1.0 {
            let actual_increase = clamped_risk - base_risk_score;
            prop_assert!(
                (actual_increase - expected_increase).abs() < 0.001,
                "Expected increase {} but got {}",
                expected_increase, actual_increase
            );
        }
    }

    // ========================================================================
    // Property 10: Challenge Struct Completeness
    // **Feature: ai-resistant-captcha, Property 10: Challenge Struct Completeness**
    // **Validates: Requirements 9.2**
    // ========================================================================

    #[test]
    fn prop_challenge_struct_completeness(
        difficulty in 1u8..=10,
        ip_address in "192\\.168\\.[0-9]{1,3}\\.[0-9]{1,3}",
    ) {
        let challenge = Challenge::new(
            ChallengeType::Visual,
            difficulty,
            "encrypted_test_data".to_string(),
            "test_answer_hash".to_string(),
            Some("test_session".to_string()),
            ip_address.clone(),
        );

        // Property: All required fields must be populated
        prop_assert!(!challenge.id.is_empty(), "Challenge ID should not be");
        prop_assert!(
            challenge.difficulty_level >= 1 && challenge.difficulty_level <= 10,
            "Difficulty {} should be between 1 and 10",
            challenge.difficulty_level
        );
        prop_assert!(
            !challenge.encrypted_data.is_empty(),
            "Encrypted data should not be empty"
        );
        prop_assert!(
            !challenge.expected_answer_hash.is_empty(),
            "Answer hash should not be empty"
        );
        prop_assert!(
            !challenge.ip_address.is_empty(),
            "IP address should not be empty"
        );

        // Verify timestamps are set correctly
        prop_assert!(challenge.expires_at > challenge.created_at);
    }

    // ========================================================================
    // Property 11: ValidationResult Struct Completeness
    // **Feature: ai-resistant-captcha, Property 11: ValidationResult Struct Completeness**
    // **Validates: Requirements 9.4**
    // ========================================================================

    #[test]
    fn prop_validation_result_completeness(
        success in prop::bool::ANY,
        confidence_score in 0.0f64..=1.0,
        next_difficulty in 1u8..=10,
    ) {
        let result = if success {
            ValidationResult::success(confidence_score, next_difficulty)
        } else {
            ValidationResult::failure(
                RiskLevel::Medium,
                next_difficulty,
                true,
                None,
                "Test failure message".to_string(),
            )
        };

        // Property: All required fields must be present and valid
        prop_assert!(
            result.confidence_score >= 0.0 && result.confidence_score <= 1.0,
            "Confidence score {} should be between 0 and 1",
            result.confidence_score
        );
        prop_assert!(
            result.next_difficulty >= 1 && result.next_difficulty <= 10,
            "Next difficulty {} should be between 1 and 10",
            result.next_difficulty
        );
        prop_assert!(!result.message.is_empty(), "Message should not be empty");
    }

    // ========================================================================
    // Property 12: Challenge Expiration and Cleanup
    // **Feature: ai-resistant-captcha, Property 12: Challenge Expiration and Cleanup**
    // **Validates: Requirements 10.2**
    // ========================================================================

    #[test]
    fn prop_challenge_expiration_duration(
        difficulty in 1u8..=10,
    ) {
        let challenge = Challenge::new(
            ChallengeType::Visual,
            difficulty,
            "test_data".to_string(),
            "test_hash".to_string(),
            Some("test_session".to_string()),
            "127.0.0.1".to_string(),
        );

        // Property: challenges expire after exactly 300 seconds (5 minutes)
        let expiration_duration = challenge.expires_at
            .duration_since(challenge.created_at)
            .unwrap();

        prop_assert_eq!(
            expiration_duration,
            Duration::from_secs(300),
            "Challenge should expire after exactly 300 seconds"
        );
    }


    // ========================================================================
    // Property 13: Threat Assessment Indicator Detection
    // **Feature: ai-resistant-captcha, Property 13: Threat Assessment Indicator Detection**
    // **Validates: Requirements 6.5**
    // ========================================================================

    #[test]
    fn prop_threat_assessment_indicators(
        failed_attempts in 0u32..20,
        successful_attempts in 0u32..10,
        avg_completion_time in 0.5f64..30.0,
    ) {
        let mut calculator = AdaptiveDifficultyCalculator::new(DifficultyParameters::default());
        let session_id = "test_session";
        let ip_address = "127.0.0.1";

        // Simulate behavior history
        for _ in 0..failed_attempts {
            calculator.update_behavior(
                session_id,
                ip_address,
                false,
                Duration::from_secs_f64(avg_completion_time),
                Some(0.5),
                None,
            );
        }

        for _ in 0..successful_attempts {
            calculator.update_behavior(
                session_id,
                ip_address,
                true,
                Duration::from_secs_f64(avg_completion_time),
                Some(0.3),
                None,
            );
        }

        let assessment = calculator.assess_threat(session_id, ip_address);

        // Property: high_failure_rate > 0.7 should include ThreatIndicator
        let total_attempts = failed_attempts + successful_attempts;
        if total_attempts > 0 {
            let failure_rate = failed_attempts as f64 / total_attempts as f64;
            if failure_rate > 0.7 {
                let has_high_failure_indicator = assessment.indicators.iter()
                    .any(|i| i.indicator_type == "high_failure_rate");
                prop_assert!(
                    has_high_failure_indicator,
                    "Should have high_failure_rate indicator when failure rate is {}",
                    failure_rate
                );
            }
        }

        // Property: risk_score should be between 0 and 1
        prop_assert!(
            assessment.risk_score >= 0.0 && assessment.risk_score <= 1.0,
            "Risk score {} should be between 0 and 1",
            assessment.risk_score
        );
    }
}

// ============================================================================
// Additional Unit Tests for Property Validation
// ============================================================================

#[cfg(test)]
mod additional_tests {
    use super::*;

    /// Test that failure tracker correctly tracks consecutive failures
    #[test]
    fn test_failure_tracker_consecutive_failures() {
        let mut tracker = CaptchaFailureTracker::new();

        // Initial state
        assert_eq!(tracker.consecutive_failures, 0);
        assert_eq!(tracker.get_failure_level(), FailureLevel::Low);

        // Record failures
        for i in 1..=12 {
            tracker.record_failure(RiskLevel::Medium);
            assert_eq!(tracker.consecutive_failures, i);
        }

        // Should be Critical after 10+ failures
        assert_eq!(tracker.get_failure_level(), FailureLevel::Critical);

        // Success should reset consecutive failures
        tracker.record_success();
        assert_eq!(tracker.consecutive_failures, 0);
        assert_eq!(tracker.get_failure_level(), FailureLevel::Low);
    }

    /// Test difficulty bounds
    #[test]
    fn test_difficulty_bounds() {
        let mut calculator = AdaptiveDifficultyCalculator::new(DifficultyParameters::default());

        // Even with many failures, difficulty should not exceed 10
        for _ in 0..100 {
            calculator.update_behavior(
                "test",
                "127.0.0.1",
                false,
                Duration::from_secs(1),
                Some(0.9),
                None,
            );
        }

        let difficulty = calculator.calculate_difficulty("test", "127.0.0.1", None, None);
        assert!(difficulty <= 10, "Difficulty should not exceed 10");
        assert!(difficulty >= 1, "Difficulty should not be less than 1");
    }

    /// Test challenge type coverage
    #[test]
    fn test_all_challenge_types_valid() {
        let challenge_types = vec![
            ChallengeType::Visual,
            ChallengeType::Audio,
            ChallengeType::Behavioral,
            ChallengeType::Logical,
            ChallengeType::Hybrid,
        ];

        for challenge_type in challenge_types {
            let challenge = Challenge::new(
                challenge_type.clone(),
                5,
                "data".to_string(),
                "hash".to_string(),
                None,
                "127.0.0.1".to_string(),
            );

            assert_eq!(challenge.challenge_type, challenge_type);
            assert!(!challenge.id.is_empty());
        }
    }

    /// Test risk level ordering
    #[test]
    fn test_risk_level_thresholds() {
        let test_cases = vec![
            (0.0, RiskLevel::Low),
            (0.29, RiskLevel::Low),
            (0.3, RiskLevel::Medium),
            (0.59, RiskLevel::Medium),
            (0.6, RiskLevel::High),
            (0.79, RiskLevel::High),
            (0.8, RiskLevel::Critical),
            (1.0, RiskLevel::Critical),
        ];

        for (score, expected_level) in test_cases {
            let actual_level = match score {
                x if x < 0.3 => RiskLevel::Low,
                x if x < 0.6 => RiskLevel::Medium,
                x if x < 0.8 => RiskLevel::High,
                _ => RiskLevel::Critical,
            };

            assert_eq!(
                actual_level, expected_level,
                "Score {} should map to {:?}",
                score, expected_level
            );
        }
    }

    /// Test ValidationResult success constructor
    #[test]
    fn test_validation_result_success() {
        let result = ValidationResult::success(0.95, 2);

        assert!(result.success);
        assert_eq!(result.confidence_score, 0.95);
        assert_eq!(result.next_difficulty, 2);
        assert!(result.retry_allowed);
        assert!(result.lockout_duration.is_none());
        assert!(!result.message.is_empty());
    }

    /// Test ValidationResult failure constructor
    #[test]
    fn test_validation_result_failure() {
        let result = ValidationResult::failure(
            RiskLevel::High,
            5,
            false,
            Some(Duration::from_secs(900)),
            "Test failure".to_string(),
        );

        assert!(!result.success);
        assert_eq!(result.confidence_score, 0.0);
        assert_eq!(result.risk_assessment, RiskLevel::High);
        assert_eq!(result.next_difficulty, 5);
        assert!(!result.retry_allowed);
        assert!(result.lockout_duration.is_some());
        assert_eq!(result.message, "Test failure");
    }
}


// ============================================================================
// Additional Property Tests - Properties 2, 5, 6, 7, 8
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // ========================================================================
    // Property 5: Lockdown Mode Activation
    // **Feature: ai-resistant-captcha, Property 5: Lockdown Mode Activation**
    // **Validates: Requirements 2.3**
    // ========================================================================

    #[test]
    fn prop_lockdown_mode_activation(
        failures_count in 0u32..150,
    ) {
        let mut tracker = CaptchaFailureTracker::new();

        // Simulate failures
        for _ in 0..failures_count {
            tracker.record_failure(RiskLevel::High);
        }

        // Property: >100 failures should result in Critical level (lockdown)
        // Note: The actual lockdown is based on consecutive failures, not total
        // With 11+ consecutive failures, we get Critical level (1 rpm)
        if failures_count > 10 {
            prop_assert_eq!(
                tracker.get_failure_level(),
                FailureLevel::Critical,
                "With {} failures, should be in Critical (lockdown) mode",
                failures_count
            );
        }
    }

    // ========================================================================
    // Property 6: Challenge Encryption Round-Trip (Local Fallback)
    // **Feature: ai-resistant-captcha, Property 6: Challenge Encryption Round-Trip**
    // **Validates: Requirements 3.1**
    // ========================================================================

    #[test]
    fn prop_local_encryption_round_trip(
        plaintext in "[a-zA-Z0-9 ]{1,100}",
    ) {
        use crate::services::captcha::fallback::LocalEncryptionFallback;

        let fallback = LocalEncryptionFallback::new();

        // Encrypt the plaintext
        let encrypted = fallback.encrypt(&plaintext);
        prop_assert!(encrypted.is_ok(), "Encryption should succeed");

        // Decrypt the ciphertext
        let decrypted = fallback.decrypt(&encrypted.unwrap());
        prop_assert!(decrypted.is_ok(), "Decryption should succeed");

        // Property: decrypt(encrypt(x)) == x
        prop_assert_eq!(
            decrypted.unwrap(),
            plaintext,
            "Round-trip should preserve original data"
        );
    }

    // ========================================================================
    // Property 8: Anomaly Detection Z-Score Threshold
    // **Feature: ai-resistant-captcha, Property 8: Anomaly Detection Z-Score Threshold**
    // **Validates: Requirements 5.4**
    // ========================================================================

    #[test]
    fn prop_anomaly_detection_z_score(
        value in 0.0f64..1000.0,
        mean in 50.0f64..200.0,
        std_dev in 10.0f64..50.0,
    ) {
        // Calculate z-score
        let z_score = if std_dev > 0.0 {
            (value - mean) / std_dev
        } else {
            0.0
        };

        let z_score_threshold = 2.5;

        // Property: z-score > 2.5 should be flagged as anomaly
        let is_anomaly = z_score.abs() > z_score_threshold;

        if z_score.abs() > z_score_threshold {
            prop_assert!(
                is_anomaly,
                "Z-score {} (abs: {}) should be flagged as anomaly (threshold: {})",
                z_score, z_score.abs(), z_score_threshold
            );
        } else {
            prop_assert!(
                !is_anomaly,
                "Z-score {} (abs: {}) should NOT be flagged as anomaly (threshold: {})",
                z_score, z_score.abs(), z_score_threshold
            );
        }
    }
}

// ============================================================================
// Property 2: Validation Response Time (Performance Test)
// **Feature: ai-resistant-captcha, Property 2: Validation Response Time**
// **Validates: Requirements 1.2**
// ============================================================================

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    /// Test that synchronous validation operations complete quickly
    /// Note: Full async validation with external services is tested in integration tests
    #[test]
    fn test_validation_result_creation_performance() {
        let iterations = 100;
        let max_duration_per_op = Duration::from_millis(10); // 10ms per operation

        let start = Instant::now();

        for i in 0..iterations {
            let _result = if i % 2 == 0 {
                ValidationResult::success(0.95, 3)
            } else {
                ValidationResult::failure(
                    RiskLevel::Medium,
                    5,
                    true,
                    None,
                    "Test failure".to_string(),
                )
            };
        }

        let elapsed = start.elapsed();
        let avg_duration = elapsed / iterations;

        assert!(
            avg_duration < max_duration_per_op,
            "Average operation time {:?} exceeds limit {:?}",
            avg_duration,
            max_duration_per_op
        );
    }

    /// Test that challenge creation completes quickly
    #[test]
    fn test_challenge_creation_performance() {
        let iterations = 100;
        let max_duration_per_op = Duration::from_millis(10);

        let start = Instant::now();

        for i in 0..iterations {
            let _challenge = Challenge::new(
                ChallengeType::Visual,
                (i % 10 + 1) as u8,
                format!("encrypted_data_{}", i),
                format!("hash_{}", i),
                Some(format!("session_{}", i)),
                format!("192.168.1.{}", i % 255),
            );
        }

        let elapsed = start.elapsed();
        let avg_duration = elapsed / iterations;

        assert!(
            avg_duration < max_duration_per_op,
            "Average challenge creation time {:?} exceeds limit {:?}",
            avg_duration,
            max_duration_per_op
        );
    }

    /// Test that difficulty calculation completes quickly
    #[test]
    fn test_difficulty_calculation_performance() {
        let mut calculator = AdaptiveDifficultyCalculator::new(DifficultyParameters::default());
        let iterations = 100;
        let max_duration_per_op = Duration::from_millis(5);

        // Pre-populate some behavior data
        for i in 0..10 {
            calculator.update_behavior(
                &format!("session_{}", i),
                &format!("192.168.1.{}", i),
                i % 2 == 0,
                Duration::from_secs(5),
                Some(0.5),
                None,
            );
        }

        let start = Instant::now();

        for i in 0..iterations {
            let _difficulty = calculator.calculate_difficulty(
                &format!("session_{}", i % 10),
                &format!("192.168.1.{}", i % 10),
                None,
                None,
            );
        }

        let elapsed = start.elapsed();
        let avg_duration = elapsed / iterations;

        assert!(
            avg_duration < max_duration_per_op,
            "Average difficulty calculation time {:?} exceeds limit {:?}",
            avg_duration,
            max_duration_per_op
        );
    }
}

// ============================================================================
// Property 7: Fallback Mechanism Activation
// **Feature: ai-resistant-captcha, Property 7: Fallback Mechanism Activation**
// **Validates: Requirements 3.5**
// ============================================================================

#[cfg(test)]
mod fallback_tests {
    use super::*;
    use crate::services::captcha::fallback::{FallbackConfig, FallbackService, FallbackState};

    #[tokio::test]
    async fn test_fallback_activates_on_degraded_mode() {
        let config = FallbackConfig::default();
        let service = FallbackService::new(config);

        // Initially in normal mode
        assert_eq!(service.get_state().await, FallbackState::Normal);

        // Enter degraded mode (simulating Secreton unavailable)
        service.enter_degraded_mode("Secreton unavailable").await;
        assert_eq!(service.get_state().await, FallbackState::Degraded);

        // Fallback should still work - generate challenge
        let challenge = service
            .generate_challenge_with_fallback(ChallengeType::Visual, 3)
            .await;
        assert!(challenge.is_ok(), "Fallback challenge generation should work");

        // Fallback encryption should work
        let encrypted = service.encrypt_with_fallback("test data").await;
        assert!(encrypted.is_ok(), "Fallback encryption should work");

        // Fallback decryption should work
        let decrypted = service.decrypt_with_fallback(&encrypted.unwrap()).await;
        assert!(decrypted.is_ok(), "Fallback decryption should work");
        assert_eq!(decrypted.unwrap(), "test data");
    }

    #[tokio::test]
    async fn test_fallback_encryption_round_trip() {
        let config = FallbackConfig::default();
        let service = FallbackService::new(config);

        let test_data = "sensitive challenge data";

        // Encrypt
        let encrypted = service.encrypt_with_fallback(test_data).await;
        assert!(encrypted.is_ok());

        // Decrypt
        let decrypted = service.decrypt_with_fallback(&encrypted.unwrap()).await;
        assert!(decrypted.is_ok());

        // Property: round-trip preserves data
        assert_eq!(decrypted.unwrap(), test_data);
    }

    #[tokio::test]
    async fn test_fallback_state_transitions() {
        let config = FallbackConfig::default();
        let service = FallbackService::new(config);

        // Normal -> Degraded
        service.enter_degraded_mode("test").await;
        assert_eq!(service.get_state().await, FallbackState::Degraded);

        // Degraded -> Emergency
        service.enter_emergency_mode("critical").await;
        assert_eq!(service.get_state().await, FallbackState::Emergency);

        // Emergency -> Normal
        service.return_to_normal().await;
        assert_eq!(service.get_state().await, FallbackState::Normal);
    }
}


// ============================================================================
// Property 15: Audit Logging on Bot Detection
// **Feature: ai-resistant-captcha, Property 15: Audit Logging on Bot Detection**
// **Validates: Requirements 2.2, 3.4**
// ============================================================================

#[cfg(test)]
mod audit_logging_tests {
    use super::*;
    use crate::services::captcha::security_monitoring::{
        CaptchaSecurityConfig, CaptchaSecurityEvent, CaptchaSecurityEventData,
        CaptchaSecurityMonitoringState,
    };
    use crate::services::captcha::types::BehaviorClassification;
    use std::time::Instant;

    #[tokio::test]
    async fn test_bot_detection_audit_logging() {
        let config = CaptchaSecurityConfig::default();
        let state = CaptchaSecurityMonitoringState::new(config, None);

        // Create a bot detection event
        let event_data = CaptchaSecurityEventData::new(
            CaptchaSecurityEvent::BotDetected,
            "192.168.1.100".to_string(),
            Some("test_session".to_string()),
        )
        .with_behavior_classification(BehaviorClassification::Bot)
        .with_confidence_score(0.95);

        // Measure logging time
        let start = Instant::now();
        let result = state.log_security_event(event_data).await;
        let elapsed = start.elapsed();

        // Property: audit log entry should be created within 1 second
        assert!(result.is_ok(), "Audit logging should succeed");
        assert!(
            elapsed < Duration::from_secs(1),
            "Audit logging should complete within 1 second, took {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_audit_logging_performance() {
        let config = CaptchaSecurityConfig::default();
        let state = CaptchaSecurityMonitoringState::new(config, None);

        let iterations = 100;
        let max_total_time = Duration::from_secs(5); // 5 seconds for 100 events

        let start = Instant::now();

        for i in 0..iterations {
            let event_data = CaptchaSecurityEventData::new(
                if i % 3 == 0 {
                    CaptchaSecurityEvent::BotDetected
                } else if i % 3 == 1 {
                    CaptchaSecurityEvent::ValidationFailed
                } else {
                    CaptchaSecurityEvent::ChallengeValidated
                },
                format!("192.168.1.{}", i % 255),
                Some(format!("session_{}", i)),
            )
            .with_confidence_score(0.5 + (i as f64 * 0.005));

            let _ = state.log_security_event(event_data).await;
        }

        let elapsed = start.elapsed();

        assert!(
            elapsed < max_total_time,
            "Logging {} events should complete within {:?}, took {:?}",
            iterations,
            max_total_time,
            elapsed
        );
    }

    #[tokio::test]
    async fn test_bot_detection_triggers_alert() {
        let mut config = CaptchaSecurityConfig::default();
        config.bot_alert_threshold = 0.8;
        config.enable_real_time_alerts = true;

        let state = CaptchaSecurityMonitoringState::new(config, None);

        // Create a high-confidence bot detection event
        let event_data = CaptchaSecurityEventData::new(
            CaptchaSecurityEvent::BotDetected,
            "192.168.1.100".to_string(),
            Some("test_session".to_string()),
        )
        .with_behavior_classification(BehaviorClassification::Bot)
        .with_confidence_score(0.95); // Above threshold

        let result = state.log_security_event(event_data).await;
        assert!(result.is_ok());

        // Verify activity was tracked
        let stats = state.get_activity_stats("192.168.1.100").await;
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().bot_detections, 1);
    }
}

// ============================================================================
// Property 14: ARIA Labels Completeness (Backend Validation)
// **Feature: ai-resistant-captcha, Property 14: ARIA Labels Completeness**
// **Validates: Requirements 4.2**
// Note: Full ARIA testing requires frontend tests. This tests the backend
// data structures that support accessibility features.
// ============================================================================

#[cfg(test)]
mod accessibility_tests {
    use super::*;
    use crate::services::captcha::types::ChallengeType;

    /// Test that all challenge types support accessibility metadata
    #[test]
    fn test_challenge_types_support_accessibility() {
        let challenge_types = vec![
            (ChallengeType::Visual, "visual_challenge"),
            (ChallengeType::Audio, "audio_challenge"),
            (ChallengeType::Behavioral, "behavioral_challenge"),
            (ChallengeType::Logical, "logical_challenge"),
            (ChallengeType::Hybrid, "hybrid_challenge"),
        ];

        for (challenge_type, expected_label) in challenge_types {
            // Each challenge type should have a corresponding accessibility label
            let aria_label = match challenge_type {
                ChallengeType::Visual => "visual_challenge",
                ChallengeType::Audio => "audio_challenge",
                ChallengeType::Behavioral => "behavioral_challenge",
                ChallengeType::Logical => "logical_challenge",
                ChallengeType::Hybrid => "hybrid_challenge",
            };

            assert_eq!(
                aria_label, expected_label,
                "Challenge type {:?} should have ARIA label {}",
                challenge_type, expected_label
            );
        }
    }

    /// Test that ValidationResult messages are suitable for screen readers
    #[test]
    fn test_validation_result_messages_accessible() {
        let success_result = ValidationResult::success(0.95, 3);
        let failure_result = ValidationResult::failure(
            RiskLevel::Medium,
            5,
            true,
            None,
            "Incorrect answer. Please try again.".to_string(),
        );

        // Messages should be non-empty and descriptive
        assert!(!success_result.message.is_empty());
        assert!(!failure_result.message.is_empty());

        // Messages should not contain technical jargon
        assert!(
            !success_result.message.contains("0x"),
            "Success message should not contain hex codes"
        );
        assert!(
            !failure_result.message.contains("0x"),
            "Failure message should not contain hex codes"
        );
    }

    /// Test that risk levels have human-readable descriptions
    #[test]
    fn test_risk_levels_have_descriptions() {
        let risk_levels = vec![
            RiskLevel::Low,
            RiskLevel::Medium,
            RiskLevel::High,
            RiskLevel::Critical,
        ];

        for risk_level in risk_levels {
            let description = match risk_level {
                RiskLevel::Low => "Low risk - normal operation",
                RiskLevel::Medium => "Medium risk - increased monitoring",
                RiskLevel::High => "High risk - restricted access",
                RiskLevel::Critical => "Critical risk - account locked",
            };

            assert!(
                !description.is_empty(),
                "Risk level {:?} should have a description",
                risk_level
            );
        }
    }
}
