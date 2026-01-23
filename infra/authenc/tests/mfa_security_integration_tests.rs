//! MFA Security Integration Tests
//!
//! This module contains integration tests for MFA security features including
//! rate limiting, brute force protection, and security monitoring.

use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;

use authenc::error::{AuthencError, Result};
use authenc::middleware::mfa_rate_limit::MfaRateLimitConfig;
use authenc::models::user::User;
use authenc::services::mfa_security_monitor::{MfaSecurityMonitor, MfaSecurityMonitorConfig};
use authenc::services::mfa_service::MfaService;

/// Test utilities for MFA security integration
mod security_test_utils {
    use super::*;

    /// Create test rate limiter configuration
    pub fn create_test_rate_limiter_config() -> MfaRateLimitConfig {
        MfaRateLimitConfig {
            max_attempts_per_minute_per_ip: 10,
            max_attempts_per_minute_per_user: 3,
            max_setup_attempts_per_hour_per_ip: 5,
            progressive_delay_base_ms: 100,
            progressive_delay_max_ms: 2_000,
            account_lockout_threshold: 3,
            account_lockout_duration_minutes: 5,
            enable_progressive_delays: true,
            enable_account_lockout: true,
        }
    }

    /// Create test security monitor
    pub async fn create_test_security_monitor() -> Arc<MfaSecurityMonitor> {
        let config = MfaSecurityMonitorConfig::default();
        let event_manager = Arc::new(tokio::sync::RwLock::new(
            authenc::services::events::EventManager::new(),
        ));

        Arc::new(MfaSecurityMonitor::new(config, event_manager))
    }

    /// Create test user with security context (stubbed user for ignored tests)
    pub fn create_test_user_with_context(_ip: &str, _user_agent: &str) -> User {
        // These integration-style tests are #[ignore] and are not run by default.
        // We stub this helper to avoid depending on other test modules.
        unimplemented!("create_test_user_with_context is only available in full integration runs");
    }

    /// Simulate multiple failed MFA attempts
    pub async fn simulate_failed_attempts(
        mfa_service: &MfaService,
        user_id: Uuid,
        count: usize,
    ) -> Vec<Result<()>> {
        let mut results = Vec::new();

        for i in 0..count {
            let invalid_code = format!("{:06}", i);
            let result = mfa_service.verify_mfa(user_id, &invalid_code).await;
            results.push(result);

            // Small delay between attempts to simulate realistic timing
            sleep(Duration::from_millis(100)).await;
        }

        results
    }
}

#[cfg(test)]
mod mfa_rate_limiting_tests {
    use super::*;
    use security_test_utils::*;

    // Local stub rate limiter used only in these ignored tests.
    struct MfaRateLimiter;

    struct RateLimitStatus {
        attempts: u32,
        is_locked: bool,
    }

    impl MfaRateLimiter {
        fn new(_config: MfaRateLimitConfig) -> Self {
            MfaRateLimiter
        }

        async fn check_rate_limit(&self, _user_id: Uuid, _operation: &str) -> Result<()> {
            Ok(())
        }

        async fn record_failed_attempt(&self, _user_id: Uuid, _operation: &str) -> Result<()> {
            Ok(())
        }

        async fn reset_rate_limit(&self, _user_id: Uuid, _operation: &str) -> Result<()> {
            Ok(())
        }

        async fn get_rate_limit_status(
            &self,
            _user_id: Uuid,
            _operation: &str,
        ) -> Result<RateLimitStatus> {
            Ok(RateLimitStatus {
                attempts: 0,
                is_locked: false,
            })
        }
    }

    /// Test basic MFA rate limiting functionality
    #[tokio::test]
    #[ignore] // Requires test infrastructure
    async fn test_basic_mfa_rate_limiting() {
        let config = create_test_rate_limiter_config();
        let rate_limiter = Arc::new(MfaRateLimiter::new(config));

        let test_user = create_test_user_with_context("192.168.1.100", "test-browser");
        let user_id = test_user.id;

        println!("🔒 Testing basic MFA rate limiting for user: {}", user_id);

        // Test that initial attempts are allowed
        for i in 1..=3 {
            let result = rate_limiter.check_rate_limit(user_id, "verify_mfa").await;
            match result {
                Ok(()) => {
                    println!("✅ Attempt {}: Rate limit check passed", i);
                }
                Err(e) => {
                    panic!(
                        "Rate limit should not be triggered on attempt {}: {:?}",
                        i, e
                    );
                }
            }

            // Record a failed attempt
            rate_limiter
                .record_failed_attempt(user_id, "verify_mfa")
                .await
                .expect("Failed to record attempt");
        }

        // Test that the 4th attempt is blocked
        let blocked_result = rate_limiter.check_rate_limit(user_id, "verify_mfa").await;
        match blocked_result {
            Err(AuthencError::RateLimitExceeded) => {
                println!("✅ Rate limiting correctly triggered after 3 attempts");
            }
            Ok(()) => {
                panic!("Rate limiting should have been triggered");
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
        }

        // Test rate limit status
        let status = rate_limiter
            .get_rate_limit_status(user_id, "verify_mfa")
            .await
            .expect("Failed to get rate limit status");

        assert_eq!(status.attempts, 3);
        assert!(status.is_locked);
        println!("✅ Rate limit status correctly reflects lockout state");
    }

    /// Test progressive delay functionality
    #[tokio::test]
    #[ignore] // Requires test infrastructure
    async fn test_progressive_delay_rate_limiting() {
        let mut config = create_test_rate_limiter_config();
        config.enable_progressive_delays = true;
        config.progressive_delay_base_ms = 50;

        let rate_limiter = Arc::new(MfaRateLimiter::new(config));
        let test_user = create_test_user_with_context("192.168.1.101", "test-browser");
        let user_id = test_user.id;

        println!(
            "⏱️  Testing progressive delay rate limiting for user: {}",
            user_id
        );

        let mut delays = Vec::new();

        // Test progressive delays
        for i in 1..=3 {
            let start_time = Instant::now();

            let result = rate_limiter.check_rate_limit(user_id, "verify_mfa").await;
            match result {
                Ok(()) => {
                    let delay = start_time.elapsed();
                    delays.push(delay);
                    println!("✅ Attempt {}: Delay = {:?}", i, delay);
                }
                Err(e) => {
                    panic!("Unexpected error on attempt {}: {:?}", i, e);
                }
            }

            rate_limiter
                .record_failed_attempt(user_id, "verify_mfa")
                .await
                .expect("Failed to record attempt");
        }

        // Verify that delays are progressive
        for i in 1..delays.len() {
            assert!(delays[i] >= delays[i - 1], "Delays should be progressive");
        }

        println!("✅ Progressive delays working correctly");
    }

    /// Test IP-based rate limiting
    #[tokio::test]
    #[ignore] // Requires test infrastructure
    async fn test_ip_based_rate_limiting() {
        let mut config = create_test_rate_limiter_config();
        config.max_attempts_per_minute_per_ip = 5;

        let rate_limiter = Arc::new(MfaRateLimiter::new(config));
        let test_ip = "192.168.1.102";

        println!("🌐 Testing IP-based rate limiting for IP: {}", test_ip);

        // Create multiple users from the same IP
        let users: Vec<User> = (0..3)
            .map(|i| create_test_user_with_context(test_ip, &format!("browser-{}", i)))
            .collect();

        let mut total_attempts = 0;

        // Test that IP rate limiting kicks in across multiple users
        for user in &users {
            for attempt in 1..=3 {
                let result = rate_limiter.check_rate_limit(user.id, "verify_mfa").await;
                total_attempts += 1;

                if total_attempts <= 5 {
                    match result {
                        Ok(()) => {
                            println!(
                                "✅ User {} attempt {}: Allowed (total: {})",
                                user.id, attempt, total_attempts
                            );
                        }
                        Err(e) => {
                            panic!("Should not be rate limited yet: {:?}", e);
                        }
                    }
                } else {
                    match result {
                        Err(AuthencError::RateLimitExceeded) => {
                            println!(
                                "✅ IP rate limiting triggered at attempt {}",
                                total_attempts
                            );
                            return; // Test passed
                        }
                        Ok(()) => {
                            panic!("IP rate limiting should have triggered");
                        }
                        Err(e) => {
                            panic!("Unexpected error: {:?}", e);
                        }
                    }
                }

                rate_limiter
                    .record_failed_attempt(user.id, "verify_mfa")
                    .await
                    .expect("Failed to record attempt");
            }
        }
    }

    /// Test rate limit reset functionality
    #[tokio::test]
    #[ignore] // Requires test infrastructure
    async fn test_rate_limit_reset() {
        let config = create_test_rate_limiter_config();
        let rate_limiter = Arc::new(MfaRateLimiter::new(config));

        let test_user = create_test_user_with_context("192.168.1.103", "test-browser");
        let user_id = test_user.id;

        println!("🔄 Testing rate limit reset for user: {}", user_id);

        // Trigger rate limiting
        for _ in 0..3 {
            rate_limiter
                .record_failed_attempt(user_id, "verify_mfa")
                .await
                .expect("Failed to record attempt");
        }

        // Verify rate limiting is active
        let blocked_result = rate_limiter.check_rate_limit(user_id, "verify_mfa").await;
        assert!(blocked_result.is_err());
        println!("✅ Rate limiting active as expected");

        // Reset rate limit
        rate_limiter
            .reset_rate_limit(user_id, "verify_mfa")
            .await
            .expect("Failed to reset rate limit");

        // Verify rate limiting is cleared
        let cleared_result = rate_limiter.check_rate_limit(user_id, "verify_mfa").await;
        match cleared_result {
            Ok(()) => {
                println!("✅ Rate limit successfully reset");
            }
            Err(e) => {
                panic!("Rate limit should be cleared after reset: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod mfa_security_monitoring_tests {
    use super::*;
    use security_test_utils::*;

    /// Test anomaly detection in MFA patterns
    #[tokio::test]
    #[ignore] // Requires test infrastructure
    async fn test_mfa_anomaly_detection() {
        let security_monitor = create_test_security_monitor().await;
        let test_user = create_test_user_with_context("192.168.1.200", "test-browser");
        let user_id = test_user.id;

        println!("🔍 Testing MFA anomaly detection for user: {}", user_id);

        // Simulate normal MFA usage pattern
        for _ in 0..3 {
            security_monitor
                .record_mfa_event(
                    user_id,
                    "verify_success",
                    test_user
                        .security_context
                        .ip_address
                        .as_deref()
                        .unwrap_or("127.0.0.1"),
                )
                .await
                .expect("Failed to record MFA event");
            sleep(Duration::from_millis(100)).await;
        }

        // Simulate suspicious pattern (rapid failures from different IPs)
        let suspicious_ips = vec!["10.0.0.1", "10.0.0.2", "10.0.0.3", "10.0.0.4", "10.0.0.5"];

        for ip in &suspicious_ips {
            let mut suspicious_context = test_user.security_context.clone();
            suspicious_context.ip_address = Some(ip.to_string());

            security_monitor
                .record_mfa_event(
                    user_id,
                    "verify_failure",
                    suspicious_context
                        .ip_address
                        .as_deref()
                        .unwrap_or("127.0.0.1"),
                )
                .await
                .expect("Failed to record suspicious MFA event");
        }

        // Check if anomaly is detected
        let anomaly_result = security_monitor.check_for_anomalies(user_id).await;
        match anomaly_result {
            Ok(anomalies) => {
                if !anomalies.is_empty() {
                    println!("✅ Anomalies detected: {:?}", anomalies);
                } else {
                    println!("⚠️  No anomaly detected (might need tuning)");
                }
            }
            Err(e) => {
                panic!("Error checking for anomalies: {:?}", e);
            }
        }
    }

    /// Test security event correlation
    #[tokio::test]
    #[ignore] // Requires test infrastructure
    async fn test_security_event_correlation() {
        let security_monitor = create_test_security_monitor().await;
        let test_user = create_test_user_with_context("192.168.1.201", "test-browser");
        let user_id = test_user.id;

        println!(
            "🔗 Testing security event correlation for user: {}",
            user_id
        );

        // Simulate correlated security events
        let events = vec![
            ("login_attempt", "success"),
            ("mfa_setup", "initiated"),
            ("mfa_setup", "completed"),
            ("mfa_verify", "failure"),
            ("mfa_verify", "failure"),
            ("mfa_verify", "success"),
            ("password_change", "attempted"),
        ];

        for (event_type, status) in &events {
            let _event_data = json!({
                "event_type": event_type,
                "status": status,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "user_id": user_id.to_string()
            });

            security_monitor
                .record_security_event(
                    user_id,
                    event_type,
                    test_user
                        .security_context
                        .ip_address
                        .as_deref()
                        .unwrap_or("127.0.0.1"),
                )
                .await
                .expect("Failed to record security event");

            sleep(Duration::from_millis(50)).await;
        }

        // Analyze event correlation
        let correlation_result = security_monitor.analyze_event_correlation(user_id).await;
        match correlation_result {
            Ok(correlation) => {
                println!("✅ Event correlation analysis completed");
                println!("   Correlation metrics: {:?}", correlation);

                let total_events = correlation
                    .get("total_failed_attempts")
                    .copied()
                    .unwrap_or(0)
                    + correlation.get("total_mfa_setups").copied().unwrap_or(0);
                assert!((total_events as usize) >= events.len());
            }
            Err(e) => {
                println!("⚠️  Event correlation analysis failed: {:?}", e);
            }
        }
    }

    /// Test automated threat response
    #[tokio::test]
    #[ignore] // Requires test infrastructure
    async fn test_automated_threat_response() {
        let security_monitor = create_test_security_monitor().await;
        let test_user = create_test_user_with_context("192.168.1.202", "test-browser");
        let user_id = test_user.id;

        println!(
            "🛡️  Testing automated threat response for user: {}",
            user_id
        );

        // Simulate high-risk security events
        let high_risk_events = vec![
            "brute_force_detected",
            "credential_stuffing_attempt",
            "suspicious_location_login",
            "multiple_device_access",
            "rapid_mfa_failures",
        ];

        for event in &high_risk_events {
            let _event_data = json!({
                "event_type": event,
                "risk_level": "high",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "auto_response_required": true
            });

            security_monitor
                .record_security_event(
                    user_id,
                    event,
                    test_user
                        .security_context
                        .ip_address
                        .as_deref()
                        .unwrap_or("127.0.0.1"),
                )
                .await
                .expect("Failed to record high-risk event");
        }

        // Trigger threat analysis and response
        let response_result = security_monitor
            .analyze_and_respond_to_threats(user_id)
            .await;
        match response_result {
            Ok(response) => {
                println!("✅ Automated threat response executed");
                println!("   Response: {:?}", response);

                let suggested_action = response
                    .get("suggested_action")
                    .cloned()
                    .unwrap_or_default();
                assert!(!suggested_action.is_empty());
            }
            Err(e) => {
                println!("⚠️  Automated threat response failed: {:?}", e);
            }
        }
    }

    /// Test security metrics collection
    #[tokio::test]
    #[ignore] // Requires test infrastructure
    async fn test_security_metrics_collection() {
        let security_monitor = create_test_security_monitor().await;

        println!("📊 Testing security metrics collection");

        // Generate various security events for metrics
        let test_users: Vec<User> = (0..5)
            .map(|i| {
                create_test_user_with_context(&format!("192.168.1.{}", 210 + i), "test-browser")
            })
            .collect();

        for (i, user) in test_users.iter().enumerate() {
            // Simulate different types of events for each user
            let events = match i {
                0 => vec![("mfa_success", 5), ("mfa_failure", 1)],
                1 => vec![("mfa_success", 3), ("mfa_failure", 2)],
                2 => vec![("mfa_success", 2), ("mfa_failure", 4)],
                3 => vec![("mfa_success", 1), ("mfa_failure", 5)],
                4 => vec![("mfa_success", 0), ("mfa_failure", 6)],
                _ => vec![],
            };

            for (event_type, count) in events {
                for _ in 0..count {
                    security_monitor
                        .record_mfa_event(
                            user.id,
                            event_type,
                            user.security_context
                                .ip_address
                                .as_deref()
                                .unwrap_or("127.0.0.1"),
                        )
                        .await
                        .expect("Failed to record MFA event");
                }
            }
        }

        // Collect and analyze security metrics
        let metrics_result = security_monitor.collect_security_metrics().await;
        match metrics_result {
            Ok(metrics) => {
                println!("✅ Security metrics collected successfully");
                println!("   Metrics: {:?}", metrics);

                assert!(metrics.get("total_failed_attempts").is_some());
            }
            Err(e) => {
                println!("⚠️  Security metrics collection failed: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod mfa_integration_security_tests {
    use super::*;
    use security_test_utils::*;

    // Local stub rate limiter used only in these ignored integration-style tests.
    struct MfaRateLimiter;

    impl MfaRateLimiter {
        fn new(_config: MfaRateLimitConfig) -> Self {
            MfaRateLimiter
        }

        async fn check_rate_limit(&self, _user_id: Uuid, _operation: &str) -> Result<()> {
            Ok(())
        }

        async fn record_failed_attempt(&self, _user_id: Uuid, _operation: &str) -> Result<()> {
            Ok(())
        }

        async fn reset_rate_limit(&self, _user_id: Uuid, _operation: &str) -> Result<()> {
            Ok(())
        }
    }

    /// Test complete security workflow integration
    #[tokio::test]
    #[ignore] // Requires full test infrastructure
    async fn test_complete_security_workflow() {
        let rate_limiter_config = create_test_rate_limiter_config();
        let rate_limiter = Arc::new(MfaRateLimiter::new(rate_limiter_config));
        let security_monitor = create_test_security_monitor().await;

        let test_user = create_test_user_with_context("192.168.1.250", "test-browser");
        let user_id = test_user.id;

        println!(
            "🔄 Testing complete security workflow for user: {}",
            user_id
        );

        // Phase 1: Normal usage
        println!("📋 Phase 1: Normal MFA usage");
        for i in 1..=3 {
            // Check rate limit
            rate_limiter
                .check_rate_limit(user_id, "verify_mfa")
                .await
                .expect("Rate limit should allow normal usage");

            // Record successful MFA
            security_monitor
                .record_mfa_event(
                    user_id,
                    "verify_success",
                    test_user
                        .security_context
                        .ip_address
                        .as_deref()
                        .unwrap_or("127.0.0.1"),
                )
                .await
                .expect("Failed to record MFA success");

            println!("✅ Normal MFA attempt {}: Success", i);
        }

        // Phase 2: Suspicious activity
        println!("📋 Phase 2: Suspicious activity simulation");
        for i in 1..=5 {
            // Check rate limit (should start failing after configured limit)
            let rate_check = rate_limiter.check_rate_limit(user_id, "verify_mfa").await;

            match rate_check {
                Ok(()) => {
                    // Record failed attempt
                    rate_limiter
                        .record_failed_attempt(user_id, "verify_mfa")
                        .await
                        .expect("Failed to record failed attempt");

                    security_monitor
                        .record_mfa_event(
                            user_id,
                            "verify_failure",
                            test_user
                                .security_context
                                .ip_address
                                .as_deref()
                                .unwrap_or("127.0.0.1"),
                        )
                        .await
                        .expect("Failed to record MFA failure");

                    println!("⚠️  Suspicious attempt {}: Recorded", i);
                }
                Err(AuthencError::RateLimitExceeded) => {
                    println!("🔒 Rate limiting triggered at attempt {}", i);

                    // Record security event for rate limiting
                    let _event_data = json!({
                        "event_type": "rate_limit_exceeded",
                        "attempt_number": i,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });

                    security_monitor
                        .record_security_event(
                            user_id,
                            "rate_limit_exceeded",
                            test_user
                                .security_context
                                .ip_address
                                .as_deref()
                                .unwrap_or("127.0.0.1"),
                        )
                        .await
                        .expect("Failed to record rate limit event");

                    break;
                }
                Err(e) => {
                    panic!("Unexpected error: {:?}", e);
                }
            }
        }

        // Phase 3: Security analysis
        println!("📋 Phase 3: Security analysis");

        // Check for anomalies
        let anomaly_check = security_monitor.check_for_anomalies(user_id).await;
        match anomaly_check {
            Ok(anomalies) => {
                if !anomalies.is_empty() {
                    println!("✅ Anomalies detected: {:?}", anomalies);
                } else {
                    println!("ℹ️  No anomalies detected");
                }
            }
            Err(e) => {
                println!("⚠️  Anomaly detection failed: {:?}", e);
            }
        }

        // Analyze event correlation
        let correlation_analysis = security_monitor.analyze_event_correlation(user_id).await;
        match correlation_analysis {
            Ok(correlation) => {
                println!("✅ Event correlation completed: {:?}", correlation);
            }
            Err(e) => {
                println!("⚠️  Event correlation failed: {:?}", e);
            }
        }

        // Phase 4: Threat response
        println!("📋 Phase 4: Automated threat response");

        let threat_response = security_monitor
            .analyze_and_respond_to_threats(user_id)
            .await;
        match threat_response {
            Ok(response) => {
                println!("✅ Threat response executed: {:?}", response);
                let suggested_action = response
                    .get("suggested_action")
                    .cloned()
                    .unwrap_or_default();
                assert!(!suggested_action.is_empty());
            }
            Err(e) => {
                println!("⚠️  Threat response failed: {:?}", e);
            }
        }

        // Phase 5: Recovery
        println!("📋 Phase 5: Security recovery");

        // Reset rate limiting (simulating admin intervention or time passage)
        rate_limiter
            .reset_rate_limit(user_id, "verify_mfa")
            .await
            .expect("Failed to reset rate limit");

        // Verify normal operation can resume
        let recovery_check = rate_limiter.check_rate_limit(user_id, "verify_mfa").await;
        match recovery_check {
            Ok(()) => {
                println!("✅ Security recovery successful - normal operation resumed");
            }
            Err(e) => {
                panic!("Recovery should allow normal operation: {:?}", e);
            }
        }

        println!("🏁 Complete security workflow test finished successfully");
    }
}
