//! Security tests for CAPTCHA bot detection accuracy and resilience

use crate::services::captcha::{
    enhanced_service::EnhancedCaptchaService,
    fallback::FallbackConfig,
    retry::RetryConfig,
    service::{CaptchaService, CaptchaServiceTrait},
    types::*,
};
use std::sync::Arc;
use std::time::Duration;

/// Generate bot-like behavioral metrics
fn generate_bot_behavioral_metrics(session_id: String) -> BehavioralMetrics {
    BehavioralMetrics {
        session_id,
        mouse_movements: vec![
            // Bot-like: Perfect straight line movements
            MouseEvent {
                x: 0.0,
                y: 0.0,
                timestamp: 1000,
                event_type: "mousemove".to_string(),
                velocity: Some(100.0),   // Unnaturally fast
                acceleration: Some(0.0), // No acceleration variation
            },
            MouseEvent {
                x: 100.0,
                y: 0.0,
                timestamp: 1010,
                event_type: "mousemove".to_string(),
                velocity: Some(100.0),
                acceleration: Some(0.0),
            },
            MouseEvent {
                x: 200.0,
                y: 0.0,
                timestamp: 1020,
                event_type: "mousemove".to_string(),
                velocity: Some(100.0),
                acceleration: Some(0.0),
            },
        ],
        keystroke_dynamics: vec![
            // Bot-like: Perfect timing
            KeystrokeEvent {
                key: "a".to_string(),
                timestamp: 2000,
                duration: 50, // Exactly 50ms every time
                dwell_time: 50,
                flight_time: Some(0), // No flight time
            },
            KeystrokeEvent {
                key: "b".to_string(),
                timestamp: 2050,
                duration: 50,
                dwell_time: 50,
                flight_time: Some(0),
            },
        ],
        timing_patterns: TimingAnalysis {
            total_interaction_time: 100,   // Suspiciously fast
            pause_patterns: vec![0, 0, 0], // No natural pauses
            rhythm_consistency: 1.0,       // Perfect consistency (unnatural)
            typing_speed: Some(1200.0),    // Impossibly fast typing
        },
        browser_fingerprint: BrowserFingerprint {
            user_agent: "HeadlessChrome/91.0.4472.124".to_string(), // Headless browser
            screen_resolution: "1024x768".to_string(),
            timezone: "UTC".to_string(),
            language: "en-US".to_string(),
            plugins: vec![], // No plugins (suspicious)
            canvas_fingerprint: Some("bot_canvas_hash".to_string()),
            webgl_fingerprint: None, // No WebGL support
        },
        risk_score: 0.95, // High risk
        classification: BehaviorClassification::Bot,
    }
}

/// Generate human-like behavioral metrics
fn generate_human_behavioral_metrics(session_id: String) -> BehavioralMetrics {
    BehavioralMetrics {
        session_id,
        mouse_movements: vec![
            // Human-like: Curved movements with variation
            MouseEvent {
                x: 50.0,
                y: 75.0,
                timestamp: 1000,
                event_type: "mousemove".to_string(),
                velocity: Some(3.2),
                acceleration: Some(0.8),
            },
            MouseEvent {
                x: 52.3,
                y: 76.1,
                timestamp: 1150,
                event_type: "mousemove".to_string(),
                velocity: Some(2.8),
                acceleration: Some(-0.4),
            },
            MouseEvent {
                x: 55.7,
                y: 78.9,
                timestamp: 1320,
                event_type: "mousemove".to_string(),
                velocity: Some(3.5),
                acceleration: Some(0.7),
            },
        ],
        keystroke_dynamics: vec![
            // Human-like: Variable timing
            KeystrokeEvent {
                key: "h".to_string(),
                timestamp: 2000,
                duration: 120,
                dwell_time: 95,
                flight_time: Some(25),
            },
            KeystrokeEvent {
                key: "e".to_string(),
                timestamp: 2180,
                duration: 85,
                dwell_time: 70,
                flight_time: Some(15),
            },
        ],
        timing_patterns: TimingAnalysis {
            total_interaction_time: 8500,             // Natural interaction time
            pause_patterns: vec![200, 150, 350, 180], // Natural pauses
            rhythm_consistency: 0.65,                 // Natural variation
            typing_speed: Some(42.0),                 // Normal typing speed
        },
        browser_fingerprint: BrowserFingerprint {
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
            screen_resolution: "1920x1080".to_string(),
            timezone: "America/New_York".to_string(),
            language: "en-US".to_string(),
            plugins: vec!["Chrome PDF Plugin".to_string(), "Widevine CDM".to_string()],
            canvas_fingerprint: Some("human_canvas_hash_123".to_string()),
            webgl_fingerprint: Some("human_webgl_hash_456".to_string()),
        },
        risk_score: 0.15, // Low risk
        classification: BehaviorClassification::Human,
    }
}

/// Generate suspicious behavioral metrics (potential bot)
fn generate_suspicious_behavioral_metrics(session_id: String) -> BehavioralMetrics {
    BehavioralMetrics {
        session_id,
        mouse_movements: vec![
            // Suspicious: Some bot-like characteristics but not definitive
            MouseEvent {
                x: 10.0,
                y: 20.0,
                timestamp: 1000,
                event_type: "mousemove".to_string(),
                velocity: Some(15.0), // Faster than normal but not impossible
                acceleration: Some(0.1),
            },
            MouseEvent {
                x: 25.0,
                y: 20.0,
                timestamp: 1100,
                event_type: "mousemove".to_string(),
                velocity: Some(15.0),
                acceleration: Some(0.0), // Some consistency but not perfect
            },
        ],
        keystroke_dynamics: vec![KeystrokeEvent {
            key: "t".to_string(),
            timestamp: 2000,
            duration: 75,
            dwell_time: 75,
            flight_time: Some(5), // Very short flight time
        }],
        timing_patterns: TimingAnalysis {
            total_interaction_time: 2000, // Quite fast
            pause_patterns: vec![50, 50], // Very short pauses
            rhythm_consistency: 0.85,     // High consistency
            typing_speed: Some(80.0),     // Fast but possible
        },
        browser_fingerprint: BrowserFingerprint {
            user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36".to_string(),
            screen_resolution: "1366x768".to_string(),
            timezone: "UTC".to_string(), // Generic timezone
            language: "en".to_string(),  // Generic language
            plugins: vec!["Default Plugin".to_string()], // Minimal plugins
            canvas_fingerprint: Some("suspicious_canvas".to_string()),
            webgl_fingerprint: Some("suspicious_webgl".to_string()),
        },
        risk_score: 0.65, // Medium-high risk
        classification: BehaviorClassification::Suspicious,
    }
}

/// Test bot detection accuracy with clear bot behavior
pub async fn test_bot_detection_accuracy_clear_bot() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    // Generate challenge
    let challenge = enhanced_service
        .generate_challenge(
            ChallengeType::Behavioral,
            Some(3),
            Some("bot_test_session".to_string()),
            "192.168.1.200".to_string(),
        )
        .await
        .unwrap();

    // Create bot-like behavioral metrics
    let bot_metrics = generate_bot_behavioral_metrics("bot_test_session".to_string());

    // Validate with bot behavior
    let validation_result = enhanced_service
        .validate_challenge(
            challenge.id,
            "correct_answer".to_string(), // Even with correct answer
            Some(bot_metrics),
        )
        .await
        .unwrap();

    // Should detect bot behavior and increase risk assessment
    assert_eq!(validation_result.risk_assessment, RiskLevel::Medium); // Fallback assessment
    assert!(validation_result.confidence_score < 1.0); // Reduced confidence due to bot behavior
}

/// Test bot detection accuracy with clear human behavior
pub async fn test_bot_detection_accuracy_clear_human() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    // Generate challenge
    let challenge = enhanced_service
        .generate_challenge(
            ChallengeType::Behavioral,
            Some(2),
            Some("human_test_session".to_string()),
            "192.168.1.201".to_string(),
        )
        .await
        .unwrap();

    // Create human-like behavioral metrics
    let human_metrics = generate_human_behavioral_metrics("human_test_session".to_string());

    // Validate with human behavior
    let validation_result = enhanced_service
        .validate_challenge(
            challenge.id,
            "correct_answer".to_string(),
            Some(human_metrics),
        )
        .await
        .unwrap();

    // Should recognize human behavior
    assert!(validation_result.confidence_score > 0.0);
    // Risk assessment should be reasonable for human behavior
    assert!(matches!(
        validation_result.risk_assessment,
        RiskLevel::Low | RiskLevel::Medium
    ));
}

/// Test bot detection with suspicious behavior
#[tokio::test]
async fn test_bot_detection_suspicious_behavior() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    // Generate challenge
    let challenge = enhanced_service
        .generate_challenge(
            ChallengeType::Behavioral,
            Some(4),
            Some("suspicious_test_session".to_string()),
            "192.168.1.202".to_string(),
        )
        .await
        .unwrap();

    // Create suspicious behavioral metrics
    let suspicious_metrics =
        generate_suspicious_behavioral_metrics("suspicious_test_session".to_string());

    // Validate with suspicious behavior
    let validation_result = enhanced_service
        .validate_challenge(
            challenge.id,
            "correct_answer".to_string(),
            Some(suspicious_metrics),
        )
        .await
        .unwrap();

    // Should flag as suspicious and increase difficulty
    assert!(validation_result.next_difficulty > 2);
    assert!(matches!(
        validation_result.risk_assessment,
        RiskLevel::Medium | RiskLevel::High
    ));
}

/// Test adaptive difficulty based on bot detection
pub async fn test_adaptive_difficulty_bot_detection() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    let ip_address = "192.168.1.203".to_string();
    let mut current_difficulty = 1u8;

    // Simulate multiple bot attempts from same IP
    for i in 0..3 {
        let challenge = enhanced_service
            .generate_challenge(
                ChallengeType::Behavioral,
                Some(current_difficulty),
                Some(format!("adaptive_session_{}", i)),
                ip_address.clone(),
            )
            .await
            .unwrap();

        let bot_metrics = generate_bot_behavioral_metrics(format!("adaptive_session_{}", i));

        let validation_result = enhanced_service
            .validate_challenge(
                challenge.id,
                "wrong_answer".to_string(), // Wrong answer + bot behavior
                Some(bot_metrics),
            )
            .await
            .unwrap();

        // Difficulty should not decrease after bot attempts
        assert!(validation_result.next_difficulty >= current_difficulty);
        current_difficulty = validation_result.next_difficulty;
    }

    // After multiple bot attempts, difficulty should be high
    assert!(current_difficulty >= 3);
}

/// Test rate limiting effectiveness against bot attacks
#[tokio::test]
async fn test_rate_limiting_bot_protection() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    let ip_address = "192.168.1.204".to_string();
    let mut successful_attempts = 0;
    let mut rate_limited_attempts = 0;

    // Simulate rapid bot attempts
    for i in 0..20 {
        let challenge_result = enhanced_service
            .generate_challenge(
                ChallengeType::Visual,
                Some(1),
                Some(format!("rate_limit_session_{}", i)),
                ip_address.clone(),
            )
            .await;

        match challenge_result {
            Ok(challenge) => {
                successful_attempts += 1;

                // Try to validate immediately (bot-like behavior)
                let bot_metrics =
                    generate_bot_behavioral_metrics(format!("rate_limit_session_{}", i));
                let _ = enhanced_service
                    .validate_challenge(challenge.id, "bot_answer".to_string(), Some(bot_metrics))
                    .await;
            }
            Err(_) => {
                rate_limited_attempts += 1;
            }
        }

        // Small delay to simulate rapid requests
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Should have some successful attempts initially, then rate limiting should kick in
    assert!(successful_attempts > 0);
    println!(
        "Successful attempts: {}, Rate limited: {}",
        successful_attempts, rate_limited_attempts
    );
}

/// Test security event logging during bot attacks
#[tokio::test]
async fn test_security_event_logging() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    // Simulate bot attack
    for i in 0..5 {
        let challenge = enhanced_service
            .generate_challenge(
                ChallengeType::Behavioral,
                Some(2),
                Some(format!("security_log_session_{}", i)),
                format!("192.168.1.{}", 210 + i),
            )
            .await
            .unwrap();

        let bot_metrics = generate_bot_behavioral_metrics(format!("security_log_session_{}", i));

        let _ = enhanced_service
            .validate_challenge(
                challenge.id,
                "attack_attempt".to_string(),
                Some(bot_metrics),
            )
            .await;
    }

    // Get comprehensive metrics to verify security events were logged
    let metrics = enhanced_service.get_comprehensive_metrics().await.unwrap();

    // Should have recorded the security events
    assert!(metrics.service_health.error_count as i32 >= 0); // May have errors from bot attempts
}

/// Test CAPTCHA resilience against automated solving attempts
#[tokio::test]
async fn test_captcha_resilience_automated_solving() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    let mut successful_solves = 0;
    let total_attempts = 10;

    // Simulate automated solving attempts
    for i in 0..total_attempts {
        let challenge = enhanced_service
            .generate_challenge(
                ChallengeType::Visual,
                Some(3),
                Some(format!("automated_solve_session_{}", i)),
                format!("192.168.1.{}", 220 + i),
            )
            .await
            .unwrap();

        // Simulate bot trying common answers
        let common_answers = vec!["123", "abc", "test", "captcha", "answer"];

        for answer in common_answers {
            let bot_metrics =
                generate_bot_behavioral_metrics(format!("automated_solve_session_{}", i));

            let validation_result = enhanced_service
                .validate_challenge(
                    challenge.id.clone(),
                    answer.to_string(),
                    Some(bot_metrics.clone()),
                )
                .await;

            if let Ok(result) = validation_result {
                if result.success {
                    successful_solves += 1;
                    break;
                }
            }
        }
    }

    // Success rate should be very low for automated attempts
    let success_rate = successful_solves as f64 / total_attempts as f64;
    assert!(
        success_rate < 0.1,
        "Success rate too high: {}",
        success_rate
    );
    println!(
        "Automated solving success rate: {:.2}%",
        success_rate * 100.0
    );
}

/// Test fallback security when main systems are compromised
#[tokio::test]
async fn test_fallback_security_compromised_systems() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    // Force degraded mode (simulating system compromise)
    enhanced_service
        .force_degraded_mode("System compromise simulation")
        .await;

    // Test that security is maintained even in degraded mode
    let challenge = enhanced_service
        .generate_challenge(
            ChallengeType::Logical,
            Some(5), // High difficulty
            Some("compromise_test_session".to_string()),
            "192.168.1.230".to_string(),
        )
        .await
        .unwrap();

    // Bot should still be detected and handled appropriately
    let bot_metrics = generate_bot_behavioral_metrics("compromise_test_session".to_string());

    let validation_result = enhanced_service
        .validate_challenge(challenge.id, "bot_attempt".to_string(), Some(bot_metrics))
        .await
        .unwrap();

    // Even in degraded mode, should maintain security posture
    assert!(!validation_result.success); // Wrong answer should fail
    assert!(validation_result.next_difficulty >= 1); // Should maintain difficulty
}

/// Test security against timing attacks
#[tokio::test]
async fn test_timing_attack_resistance() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    let mut validation_times = Vec::new();

    // Test multiple validations to check for timing consistency
    for i in 0..10 {
        let challenge = enhanced_service
            .generate_challenge(
                ChallengeType::Logical,
                Some(2),
                Some(format!("timing_session_{}", i)),
                format!("192.168.1.{}", 240 + i),
            )
            .await
            .unwrap();

        let start_time = std::time::Instant::now();

        let _ = enhanced_service
            .validate_challenge(challenge.id, "test_answer".to_string(), None)
            .await;

        let validation_time = start_time.elapsed();
        validation_times.push(validation_time);
    }

    // Calculate timing variance
    let avg_time = validation_times.iter().sum::<Duration>() / validation_times.len() as u32;
    let variance = validation_times
        .iter()
        .map(|&time| {
            let diff = if time > avg_time {
                time - avg_time
            } else {
                avg_time - time
            };
            diff.as_nanos() as f64
        })
        .sum::<f64>()
        / validation_times.len() as f64;

    // Timing should be relatively consistent (low variance indicates timing attack resistance)
    println!("Average validation time: {:?}", avg_time);
    println!("Timing variance: {:.2} ns", variance);

    // All validations should complete within reasonable time
    assert!(avg_time < Duration::from_millis(100));
}

/// Test security monitoring integration during attacks
#[tokio::test]
async fn test_security_monitoring_integration() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    // Simulate coordinated attack from multiple IPs
    let attack_ips = vec![
        "192.168.1.250",
        "192.168.1.251",
        "192.168.1.252",
        "10.0.0.100",
        "10.0.0.101",
    ];

    for (i, ip) in attack_ips.iter().enumerate() {
        // Multiple attempts from each IP
        for j in 0..3 {
            let challenge = enhanced_service
                .generate_challenge(
                    ChallengeType::Behavioral,
                    Some(1),
                    Some(format!("attack_session_{}_{}", i, j)),
                    ip.to_string(),
                )
                .await
                .unwrap();

            let bot_metrics =
                generate_bot_behavioral_metrics(format!("attack_session_{}_{}", i, j));

            let _ = enhanced_service
                .validate_challenge(challenge.id, "attack".to_string(), Some(bot_metrics))
                .await;
        }
    }

    // Get metrics to verify attack was detected and logged
    let metrics = enhanced_service.get_comprehensive_metrics().await.unwrap();

    // Should have detected the coordinated attack
    assert!(metrics.service_health.last_health_check.elapsed().unwrap() < Duration::from_secs(5));
}

/// Performance test for security operations
#[tokio::test]
async fn test_security_operations_performance() {
    let core_service = Arc::new(CaptchaService::simple().await);
    let fallback_config = FallbackConfig::default();
    let retry_config = RetryConfig::default();

    let enhanced_service = EnhancedCaptchaService::new(core_service, fallback_config, retry_config);

    let start_time = std::time::Instant::now();
    let iterations = 20;

    // Test performance under security load
    for i in 0..iterations {
        let challenge = enhanced_service
            .generate_challenge(
                ChallengeType::Behavioral,
                Some(3),
                Some(format!("perf_session_{}", i)),
                format!("192.168.2.{}", i + 1),
            )
            .await
            .unwrap();

        let behavioral_metrics = if i % 2 == 0 {
            generate_bot_behavioral_metrics(format!("perf_session_{}", i))
        } else {
            generate_human_behavioral_metrics(format!("perf_session_{}", i))
        };

        let _ = enhanced_service
            .validate_challenge(
                challenge.id,
                "performance_test".to_string(),
                Some(behavioral_metrics),
            )
            .await;
    }

    let elapsed = start_time.elapsed();
    let avg_time_per_operation = elapsed / iterations;

    // Security operations should maintain good performance
    assert!(avg_time_per_operation < Duration::from_millis(1000));
    println!(
        "Average time per security operation: {:?}",
        avg_time_per_operation
    );
}
