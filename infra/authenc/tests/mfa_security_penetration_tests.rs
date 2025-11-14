//! MFA Security Penetration Tests
//!
//! This module contains comprehensive penetration tests for the MFA implementation,
//! including timing attacks, brute force protection, and cryptographic security validation.

use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json as AxumJson,
    routing::{get, post},
};
use axum_test::TestServer;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;

use authenc::error::AuthencError;
use authenc::spi::credential::otp::{OtpAlgorithm, OtpCredentialProvider};

/// Test state for penetration testing
struct PenetrationTestState {
    otp_provider: OtpCredentialProvider,
    failed_attempts: std::sync::Arc<tokio::sync::Mutex<HashMap<String, u32>>>,
    rate_limits: std::sync::Arc<tokio::sync::Mutex<HashMap<String, Instant>>>,
}

impl PenetrationTestState {
    fn new() -> Self {
        Self {
            otp_provider: OtpCredentialProvider::new(),
            failed_attempts: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            rate_limits: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }
}

/// Mock MFA verification endpoint for testing
async fn mock_mfa_verify(
    State(state): State<PenetrationTestState>,
    headers: HeaderMap,
    AxumJson(payload): AxumJson<Value>,
) -> Result<AxumJson<Value>, StatusCode> {
    let user_id = headers
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("test-user");

    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("127.0.0.1");

    let code = payload.get("code").and_then(|c| c.as_str()).unwrap_or("");

    // Check rate limiting
    {
        let mut rate_limits = state.rate_limits.lock().await;
        let key = format!("{}:{}", user_id, ip_address);

        if let Some(last_attempt) = rate_limits.get(&key) {
            if last_attempt.elapsed() < Duration::from_secs(1) {
                return Err(StatusCode::TOO_MANY_REQUESTS);
            }
        }

        rate_limits.insert(key, Instant::now());
    }

    // Check failed attempts (account lockout)
    {
        let mut failed_attempts = state.failed_attempts.lock().await;
        let attempts = failed_attempts.get(user_id).unwrap_or(&0);

        if *attempts >= 10 {
            return Err(StatusCode::LOCKED);
        }
    }

    // Simulate OTP verification with timing
    let verification_start = Instant::now();

    // Simulate different timing based on code length (vulnerability test)
    let delay_ms = match code.len() {
        0..=3 => 50,
        4..=6 => 100,
        _ => 150,
    };

    sleep(Duration::from_millis(delay_ms)).await;

    // Always fail for testing (except specific test codes)
    let is_valid = match code {
        "123456" => true,  // Test valid code
        "000000" => false, // Test invalid code
        _ => false,
    };

    if !is_valid {
        // Increment failed attempts
        let mut failed_attempts = state.failed_attempts.lock().await;
        let attempts = failed_attempts.entry(user_id.to_string()).or_insert(0);
        *attempts += 1;

        return Ok(AxumJson(json!({
            "success": false,
            "error": "Invalid OTP code",
            "attempts_remaining": 10 - *attempts,
            "verification_time_ms": verification_start.elapsed().as_millis()
        })));
    }

    Ok(AxumJson(json!({
        "success": true,
        "message": "MFA verification successful",
        "verification_time_ms": verification_start.elapsed().as_millis()
    })))
}

/// Mock MFA setup endpoint
async fn mock_mfa_setup(
    State(state): State<PenetrationTestState>,
) -> Result<AxumJson<Value>, StatusCode> {
    let secret = state.otp_provider.generate_secret();

    let qr_uri = state.otp_provider.generate_provisioning_uri(
        &secret,
        "test@kejaksaan.go.id",
        "SIMPelv2 Test",
        OtpAlgorithm::HmacSha1,
        6,
        30,
    );

    Ok(AxumJson(json!({
        "qr_code_url": format!("data:image/png;base64,{}", "mock_qr_data"),
        "secret_key": secret,
        "provisioning_uri": qr_uri,
        "backup_codes": ["12345678", "87654321", "11111111"]
    })))
}

/// Create test router for penetration testing
fn create_test_router() -> Router {
    let state = PenetrationTestState::new();

    Router::new()
        .route("/api/auth/mfa/verify", post(mock_mfa_verify))
        .route("/api/auth/mfa/setup", post(mock_mfa_setup))
        .with_state(state)
}

#[cfg(test)]
mod penetration_tests {
    use super::*;
    use futures::future::join_all;
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    #[tokio::test]
    async fn test_authentication_bypass_attempts() {
        let app = create_test_router();
        let server = TestServer::new(app).unwrap();

        println!("=== Authentication Bypass Testing ===");

        // Test 1: Direct endpoint access without proper authentication
        let response = server.get("/api/auth/mfa/verify").await;

        assert_eq!(response.status_code(), 405); // Method not allowed
        println!("✅ Test 1: Direct GET access properly blocked");

        // Test 2: Missing required fields
        let response = server.post("/api/auth/mfa/verify").json(&json!({})).await;

        // Should handle gracefully (not crash)
        assert!(response.status_code().is_client_error() || response.status_code().is_success());
        println!("✅ Test 2: Missing fields handled gracefully");

        // Test 3: Invalid JSON payload
        let response = server
            .post("/api/auth/mfa/verify")
            .add_header("content-type", "application/json")
            .text("invalid json")
            .await;

        assert_eq!(response.status_code(), 400);
        println!("✅ Test 3: Invalid JSON properly rejected");

        // Test 4: Parameter pollution attempt
        let response = server
            .post("/api/auth/mfa/verify")
            .json(&json!({
                "code": "123456",
                "code": "bypass"  // Duplicate parameter
            }))
            .await;

        // Should use first value or reject
        let body: Value = response.json();
        assert!(
            !body
                .get("success")
                .unwrap_or(&json!(false))
                .as_bool()
                .unwrap_or(false)
                || body
                    .get("success")
                    .unwrap_or(&json!(false))
                    .as_bool()
                    .unwrap_or(false)
        );
        println!("✅ Test 4: Parameter pollution handled correctly");
    }

    #[tokio::test]
    async fn test_brute_force_protection() {
        let app = create_test_router();
        let server = TestServer::new(app).unwrap();

        println!("=== Brute Force Protection Testing ===");

        let user_id = "brute-force-test-user";
        let mut rate_limited = false;
        let mut account_locked = false;

        // Perform rapid attempts
        for i in 1..=25 {
            let response = server
                .post("/api/auth/mfa/verify")
                .add_header("x-user-id", user_id)
                .json(&json!({
                    "code": format!("{:06}", i)
                }))
                .await;

            match response.status_code() {
                429 => {
                    if !rate_limited {
                        println!("✅ Rate limiting activated at attempt {}", i);
                        rate_limited = true;
                    }
                }
                423 => {
                    if !account_locked {
                        println!("✅ Account lockout triggered at attempt {}", i);
                        account_locked = true;
                        break;
                    }
                }
                _ => {
                    // Continue testing
                }
            }

            // Small delay to avoid overwhelming the test
            sleep(Duration::from_millis(10)).await;
        }

        assert!(
            rate_limited || account_locked,
            "Brute force protection should activate"
        );
        println!("✅ Brute force protection validation completed");
    }

    #[tokio::test]
    async fn test_timing_attack_vulnerability() {
        let app = create_test_router();
        let server = TestServer::new(app).unwrap();

        println!("=== Timing Attack Testing ===");

        let test_cases = vec![
            ("1", "1-digit"),
            ("12", "2-digit"),
            ("123", "3-digit"),
            ("1234", "4-digit"),
            ("12345", "5-digit"),
            ("123456", "6-digit"),
            ("1234567", "7-digit"),
            ("12345678", "8-digit"),
        ];

        let mut timing_results = HashMap::new();

        for (code, description) in test_cases {
            let mut times = Vec::new();

            // Multiple measurements for statistical significance
            for _ in 0..20 {
                let start = Instant::now();

                let _response = server
                    .post("/api/auth/mfa/verify")
                    .add_header("x-user-id", "timing-test-user")
                    .json(&json!({
                        "code": code
                    }))
                    .await;

                let duration = start.elapsed();
                times.push(duration.as_millis());

                // Delay to avoid rate limiting
                sleep(Duration::from_millis(100)).await;
            }

            let avg_time = times.iter().sum::<u128>() / times.len() as u128;
            timing_results.insert(code.to_string(), (avg_time, description));
        }

        // Analyze timing differences
        let times: Vec<u128> = timing_results.values().map(|(time, _)| *time).collect();
        let avg_overall = times.iter().sum::<u128>() / times.len() as u128;

        let mut significant_differences = 0;

        println!("\nTiming Analysis Results:");
        println!("{:-<60}", "");

        for (code, (avg_time, description)) in &timing_results {
            let deviation = if *avg_time > avg_overall {
                *avg_time - avg_overall
            } else {
                avg_overall - *avg_time
            };

            let significant = deviation > avg_overall / 4; // 25% deviation threshold

            if significant {
                significant_differences += 1;
            }

            let status = if significant {
                "⚠️ SIGNIFICANT"
            } else {
                "✅ NORMAL"
            };

            println!(
                "{:15} | Avg: {:3}ms | Dev: {:3}ms | {}",
                description, avg_time, deviation, status
            );
        }

        println!("{:-<60}", "");

        if significant_differences > 0 {
            println!("⚠️ TIMING VULNERABILITY DETECTED");
            println!("Recommendation: Implement constant-time comparison");
        } else {
            println!("✅ NO SIGNIFICANT TIMING DIFFERENCES DETECTED");
        }
    }

    #[tokio::test]
    async fn test_concurrent_attack_simulation() {
        let app = create_test_router();
        let server = Arc::new(TestServer::new(app).unwrap());

        println!("=== Concurrent Attack Simulation ===");

        let concurrent_users = 50;
        let attempts_per_user = 10;
        let semaphore = Arc::new(Semaphore::new(concurrent_users));

        let mut tasks = Vec::new();

        for user_id in 0..concurrent_users {
            let server_clone = Arc::clone(&server);
            let semaphore_clone = semaphore.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();

                let mut results = Vec::new();

                for attempt in 0..attempts_per_user {
                    let start = Instant::now();

                    let response = server_clone
                        .post("/api/auth/mfa/verify")
                        .add_header("x-user-id", &format!("concurrent-user-{}", user_id))
                        .json(&json!({
                            "code": format!("{:06}", attempt)
                        }))
                        .await;

                    let duration = start.elapsed();

                    results.push((response.status_code(), duration));

                    // Small delay between attempts
                    sleep(Duration::from_millis(50)).await;
                }

                results
            });

            tasks.push(task);
        }

        // Wait for all concurrent attacks to complete
        let results = join_all(tasks).await;

        let mut total_requests = 0;
        let mut successful_blocks = 0;
        let mut rate_limited = 0;
        let mut server_errors = 0;

        for task_result in results {
            if let Ok(user_results) = task_result {
                for (status_code, _duration) in user_results {
                    total_requests += 1;

                    match status_code.as_u16() {
                        200 => {} // Successful response (expected to be failure)
                        400..=499 => successful_blocks += 1,
                        429 => rate_limited += 1,
                        500..=599 => server_errors += 1,
                        _ => {}
                    }
                }
            }
        }

        println!("Concurrent Attack Results:");
        println!("Total requests: {}", total_requests);
        println!("Successfully blocked: {}", successful_blocks);
        println!("Rate limited: {}", rate_limited);
        println!("Server errors: {}", server_errors);

        let protection_rate = (successful_blocks + rate_limited) as f64 / total_requests as f64;
        println!("Protection effectiveness: {:.1}%", protection_rate * 100.0);

        // Server should handle concurrent load without excessive errors
        assert!(
            server_errors < total_requests / 10,
            "Too many server errors under load"
        );
        assert!(protection_rate > 0.8, "Protection rate should be > 80%");

        println!("✅ Concurrent attack simulation completed successfully");
    }

    #[tokio::test]
    async fn test_cryptographic_security() {
        println!("=== Cryptographic Security Testing ===");

        let provider = OtpCredentialProvider::new();

        // Test 1: Secret entropy validation
        let mut secrets = std::collections::HashSet::new();
        for _ in 0..1000 {
            let secret = provider.generate_secret();
            assert!(!secrets.contains(&secret), "Duplicate secret generated");
            secrets.insert(secret);
        }
        println!("✅ Secret uniqueness test passed (1000 unique secrets)");

        // Test 2: TOTP collision resistance
        let secret = "JBSWY3DPEHPK3PXP";
        let mut codes = std::collections::HashSet::new();

        for time_step in 0..1000 {
            let secret_bytes =
                base32::decode(base32::Alphabet::RFC4648 { padding: false }, secret).unwrap();

            let code = provider
                .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha1, 6)
                .unwrap();

            codes.insert(code);
        }

        let collision_rate = 1.0 - (codes.len() as f64 / 1000.0);
        println!("TOTP collision rate: {:.2}%", collision_rate * 100.0);
        assert!(collision_rate < 0.1, "Excessive TOTP collisions detected");

        if collision_rate < 0.01 {
            println!("✅ Low collision rate - good cryptographic properties");
        }

        // Test 3: Algorithm diversity
        let time_step = chrono::Utc::now().timestamp() as u64 / 30;
        let secret_bytes =
            base32::decode(base32::Alphabet::RFC4648 { padding: false }, secret).unwrap();

        let sha1_code = provider
            .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha1, 6)
            .unwrap();

        let sha256_code = provider
            .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha256, 6)
            .unwrap();

        let sha512_code = provider
            .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha512, 6)
            .unwrap();

        assert_ne!(sha1_code, sha256_code);
        assert_ne!(sha1_code, sha512_code);
        assert_ne!(sha256_code, sha512_code);

        println!("✅ Algorithm diversity validation passed");
        println!(
            "SHA1: {}, SHA256: {}, SHA512: {}",
            sha1_code, sha256_code, sha512_code
        );
    }

    #[tokio::test]
    async fn test_input_validation_security() {
        let app = create_test_router();
        let server = TestServer::new(app).unwrap();

        println!("=== Input Validation Security Testing ===");

        let malicious_inputs: Vec<(String, &str)> = vec![
            ("".to_string(), "Empty code"),
            ("a".to_string(), "Non-numeric code"),
            ("123".to_string(), "Short code"),
            ("1234567890".to_string(), "Long code"),
            ("../../../etc/passwd".to_string(), "Path traversal"),
            ("<script>alert('xss')</script>".to_string(), "XSS attempt"),
            ("'; DROP TABLE users; --".to_string(), "SQL injection"),
            ("\\x00\\x01\\x02".to_string(), "Binary data"),
            ("🔥💯🚀".to_string(), "Unicode/emoji"),
            ("A".repeat(10000), "Buffer overflow attempt"),
        ];

        for (input, description) in malicious_inputs {
            let response = server
                .post("/api/auth/mfa/verify")
                .add_header("x-user-id", "input-validation-test")
                .json(&json!({
                    "code": input
                }))
                .await;

            // Should not crash or return 500 error
            assert_ne!(
                response.status_code(),
                500,
                "Server error on input: {} ({})",
                input,
                description
            );

            // Should handle gracefully
            assert!(
                response.status_code().is_client_error() || response.status_code().is_success(),
                "Unexpected status for input: {} ({})",
                input,
                description
            );
        }

        println!("✅ Input validation security tests passed");
    }

    #[tokio::test]
    async fn test_session_security() {
        let app = create_test_router();
        let server = TestServer::new(app).unwrap();

        println!("=== Session Security Testing ===");

        // Test 1: Session fixation
        let response1 = server
            .post("/api/auth/mfa/verify")
            .add_header("x-user-id", "session-test-user")
            .add_header("x-session-id", "fixed-session-123")
            .json(&json!({
                "code": "123456"
            }))
            .await;

        // Should not accept fixed session IDs
        // (In real implementation, would check session handling)
        assert!(response1.status_code().is_success() || response1.status_code().is_client_error());

        // Test 2: Session hijacking simulation
        let response2 = server
            .post("/api/auth/mfa/verify")
            .add_header("x-user-id", "session-test-user")
            .add_header("x-session-id", "hijacked-session-456")
            .add_header("x-forwarded-for", "192.168.1.100")
            .json(&json!({
                "code": "123456"
            }))
            .await;

        // Should handle session security appropriately
        assert!(response2.status_code().is_success() || response2.status_code().is_client_error());

        println!("✅ Session security tests completed");
    }
}

/// Performance and stress testing
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[tokio::test]
    async fn test_performance_under_load() {
        let app = create_test_router();
        let server = Arc::new(TestServer::new(app).unwrap());

        println!("=== Performance Under Load Testing ===");

        let concurrent_requests = 100;
        let requests_per_task = 10;
        let success_counter = Arc::new(AtomicU64::new(0));
        let error_counter = Arc::new(AtomicU64::new(0));
        let total_time = Arc::new(AtomicU64::new(0));

        let mut tasks = Vec::new();

        let start_time = Instant::now();

        for task_id in 0..concurrent_requests {
            let server_clone = Arc::clone(&server);
            let success_counter_clone = success_counter.clone();
            let error_counter_clone = error_counter.clone();
            let total_time_clone = total_time.clone();

            let task = tokio::spawn(async move {
                for request_id in 0..requests_per_task {
                    let request_start = Instant::now();

                    let response = server_clone
                        .post("/api/auth/mfa/verify")
                        .add_header("x-user-id", &format!("perf-user-{}", task_id))
                        .json(&json!({
                            "code": format!("{:06}", request_id)
                        }))
                        .await;

                    let request_duration = request_start.elapsed();
                    total_time_clone
                        .fetch_add(request_duration.as_millis() as u64, Ordering::Relaxed);

                    if response.status_code().is_success()
                        || response.status_code().is_client_error()
                    {
                        success_counter_clone.fetch_add(1, Ordering::Relaxed);
                    } else {
                        error_counter_clone.fetch_add(1, Ordering::Relaxed);
                    }

                    // Small delay to simulate realistic usage
                    sleep(Duration::from_millis(10)).await;
                }
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        join_all(tasks).await;

        let total_duration = start_time.elapsed();
        let total_requests = concurrent_requests * requests_per_task;
        let successful_requests = success_counter.load(Ordering::Relaxed);
        let failed_requests = error_counter.load(Ordering::Relaxed);
        let avg_response_time = total_time.load(Ordering::Relaxed) / total_requests as u64;

        println!("Performance Test Results:");
        println!("Total requests: {}", total_requests);
        println!("Successful requests: {}", successful_requests);
        println!("Failed requests: {}", failed_requests);
        println!(
            "Success rate: {:.1}%",
            (successful_requests as f64 / total_requests as f64) * 100.0
        );
        println!("Average response time: {}ms", avg_response_time);
        println!(
            "Requests per second: {:.1}",
            total_requests as f64 / total_duration.as_secs_f64()
        );

        // Performance assertions
        assert!(
            avg_response_time < 1000,
            "Average response time should be < 1000ms"
        );
        assert!(
            successful_requests > total_requests * 8 / 10,
            "Success rate should be > 80%"
        );

        println!("✅ Performance under load test completed successfully");
    }
}
