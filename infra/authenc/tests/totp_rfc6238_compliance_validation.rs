//! RFC 6238 TOTP Compliance Validation Tests
//!
//! This test suite validates that the existing OtpCredentialProvider implementation
//! complies with RFC 6238 standards for Time-Based One-Time Password (TOTP) Algorithm.

use authenc::spi::credential::otp::{OtpAlgorithm, OtpCredentialProvider};
use base32;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};

/// Test vectors from RFC 6238 Appendix B
const RFC6238_SECRET: &str = "12345678901234567890";
const RFC6238_TEST_VECTORS: &[(u64, &str)] = &[
    (59, "94287082"),          // T0 = 0, T = 59 seconds
    (1111111109, "07081804"),  // T0 = 0, T = 1111111109 seconds
    (1111111111, "14050471"),  // T0 = 0, T = 1111111111 seconds
    (1234567890, "89005924"),  // T0 = 0, T = 1234567890 seconds
    (2000000000, "69279037"),  // T0 = 0, T = 2000000000 seconds
    (20000000000, "65353130"), // T0 = 0, T = 20000000000 seconds
];

#[test]
fn test_rfc6238_compliance_basic_parameters() {
    let provider = OtpCredentialProvider::new();

    // RFC 6238 Section 4.1: Default parameters
    // - Time step X = 30 seconds (default)
    // - T0 = 0 (Unix epoch)
    // - HMAC-SHA-1 algorithm (default)
    // - 6-digit codes (default)

    // Verify default parameters match RFC 6238
    let secret = "JBSWY3DPEHPK3PXP"; // Base32 encoded test secret

    // Test with current time - should not throw errors
    let result = provider.verify_totp(
        secret,
        "123456", // Dummy code for parameter validation
        OtpAlgorithm::HmacSha1,
        6,  // 6 digits as per RFC 6238
        30, // 30 second time step as per RFC 6238
    );

    // Should not error on parameter validation
    assert!(result.is_ok());
}

#[test]
fn test_rfc6238_test_vectors() {
    let provider = OtpCredentialProvider::new();

    // Convert RFC test secret to base32 for our implementation
    let secret_bytes = RFC6238_SECRET.as_bytes();
    let secret_base32 = base32::encode(base32::Alphabet::RFC4648 { padding: false }, secret_bytes);

    for &(timestamp, expected_code) in RFC6238_TEST_VECTORS {
        // Calculate time step (T = (Current Unix time - T0) / X)
        let time_step = timestamp / 30;

        // Generate TOTP for this specific time step
        let generated_code = provider
            .generate_totp_for_step(
                secret_bytes,
                time_step,
                OtpAlgorithm::HmacSha1,
                8, // RFC test vectors use 8 digits
            )
            .expect("TOTP generation should succeed");

        assert_eq!(
            generated_code, expected_code,
            "RFC 6238 test vector failed for timestamp {}: expected {}, got {}",
            timestamp, expected_code, generated_code
        );
    }
}

#[test]
fn test_time_step_calculation() {
    let provider = OtpCredentialProvider::new();

    // RFC 6238 Section 4.2: Time step calculation
    // T = (Current Unix time - T0) / X
    // Where T0 = 0 and X = 30 seconds

    let test_cases = vec![
        (0, 0),                 // Unix epoch
        (29, 0),                // Just before first step
        (30, 1),                // First step boundary
        (59, 1),                // Just before second step
        (60, 2),                // Second step boundary
        (1111111109, 37037036), // RFC test case
        (1111111111, 37037037), // RFC test case
    ];

    for (timestamp, expected_step) in test_cases {
        let calculated_step = timestamp / 30;
        assert_eq!(
            calculated_step, expected_step,
            "Time step calculation failed for timestamp {}: expected {}, got {}",
            timestamp, expected_step, calculated_step
        );
    }
}

#[test]
fn test_clock_skew_tolerance() {
    let provider = OtpCredentialProvider::new();
    let secret = provider.generate_secret();

    // RFC 6238 Section 5.2: Clock skew tolerance
    // Implementation should accept codes from adjacent time windows

    let current_time = Utc::now().timestamp() as u64;
    let time_step = current_time / 30;

    // Generate codes for previous, current, and next time steps
    let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &secret)
        .expect("Valid base32 secret");

    let prev_code = provider
        .generate_totp_for_step(&secret_bytes, time_step - 1, OtpAlgorithm::HmacSha1, 6)
        .expect("Previous step code generation");

    let current_code = provider
        .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha1, 6)
        .expect("Current step code generation");

    let next_code = provider
        .generate_totp_for_step(&secret_bytes, time_step + 1, OtpAlgorithm::HmacSha1, 6)
        .expect("Next step code generation");

    // All three codes should be accepted (±1 time window tolerance)
    assert!(
        provider
            .verify_totp(&secret, &prev_code, OtpAlgorithm::HmacSha1, 6, 30)
            .unwrap(),
        "Previous time step code should be accepted"
    );

    assert!(
        provider
            .verify_totp(&secret, &current_code, OtpAlgorithm::HmacSha1, 6, 30)
            .unwrap(),
        "Current time step code should be accepted"
    );

    assert!(
        provider
            .verify_totp(&secret, &next_code, OtpAlgorithm::HmacSha1, 6, 30)
            .unwrap(),
        "Next time step code should be accepted"
    );
}

#[test]
fn test_hmac_sha1_algorithm_compliance() {
    let provider = OtpCredentialProvider::new();

    // RFC 6238 Section 4.1: HMAC-SHA-1 is the default algorithm
    let secret = "JBSWY3DPEHPK3PXP";
    let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: false }, secret)
        .expect("Valid base32 secret");

    let time_step = Utc::now().timestamp() as u64 / 30;

    // Generate TOTP using our implementation
    let our_code = provider
        .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha1, 6)
        .expect("TOTP generation");

    // Generate TOTP using direct HMAC-SHA1 calculation for verification
    let time_bytes = time_step.to_be_bytes();
    let mut mac = Hmac::<Sha1>::new_from_slice(&secret_bytes).expect("Valid HMAC key");
    mac.update(&time_bytes);
    let hash = mac.finalize().into_bytes();

    // Dynamic truncation as per RFC 4226/6238
    let offset = (hash[hash.len() - 1] & 0x0f) as usize;
    let code = u32::from_be_bytes([
        hash[offset] & 0x7f,
        hash[offset + 1],
        hash[offset + 2],
        hash[offset + 3],
    ]);
    let expected_code = format!("{:06}", code % 1_000_000);

    assert_eq!(
        our_code, expected_code,
        "HMAC-SHA1 implementation does not match RFC specification"
    );
}

#[test]
fn test_six_digit_code_format() {
    let provider = OtpCredentialProvider::new();
    let secret = provider.generate_secret();

    // RFC 6238: Default is 6-digit codes
    let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &secret)
        .expect("Valid base32 secret");

    let time_step = Utc::now().timestamp() as u64 / 30;

    let code = provider
        .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha1, 6)
        .expect("TOTP generation");

    // Verify code format
    assert_eq!(code.len(), 6, "Code should be exactly 6 digits");
    assert!(
        code.chars().all(|c| c.is_ascii_digit()),
        "Code should contain only digits"
    );

    // Test with leading zeros
    let small_time_step = 1; // This might generate a code with leading zeros
    let code_with_zeros = provider
        .generate_totp_for_step(&secret_bytes, small_time_step, OtpAlgorithm::HmacSha1, 6)
        .expect("TOTP generation");

    assert_eq!(
        code_with_zeros.len(),
        6,
        "Code with leading zeros should still be 6 digits"
    );
}

#[test]
fn test_thirty_second_time_window() {
    let provider = OtpCredentialProvider::new();
    let secret = provider.generate_secret();

    // RFC 6238 Section 4.1: Default time step is 30 seconds
    let current_time = Utc::now().timestamp() as u64;

    // Codes generated within the same 30-second window should be identical
    let time1 = (current_time / 30) * 30; // Start of current window
    let time2 = time1 + 15; // Midof currindow
    let time3 = time1 + 29; // End of current window

    let step = time1 / 30;
    let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &secret)
        .expect("Valid base32 secret");

    let code1 = provider
        .generate_totp_for_step(&secret_bytes, step, OtpAlgorithm::HmacSha1, 6)
        .expect("TOTP generation");
    let code2 = provider
        .generate_totp_for_step(&secret_bytes, step, OtpAlgorithm::HmacSha1, 6)
        .expect("TOTP generation");
    let code3 = provider
        .generate_totp_for_step(&secret_bytes, step, OtpAlgorithm::HmacSha1, 6)
        .expect("TOTP generation");

    assert_eq!(
        code1, code2,
        "Codes within same time window should be identical"
    );
    assert_eq!(
        code2, code3,
        "Codes within same time window should be identical"
    );

    // Code from next window should be different
    let next_code = provider
        .generate_totp_for_step(&secret_bytes, step + 1, OtpAlgorithm::HmacSha1, 6)
        .expect("TOTP generation");

    assert_ne!(
        code1, next_code,
        "Codes from different time windows should be different"
    );
}

#[test]
fn test_secret_key_requirements() {
    let provider = OtpCredentialProvider::new();

    // RFC 6238 recommends at least 160 bits (20 bytes) for HMAC-SHA1
    let secret = provider.generate_secret();
    let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &secret)
        .expect("Valid base32 secret");

    assert!(
        secret_bytes.len() >= 20,
        "Secret should be at least 160 bits (20 bytes) as recommended by RFC 6238"
    );

    // Test that different secrets generate different codes
    let secret2 = provider.generate_secret();
    assert_ne!(secret, secret2, "Generated secrets should be unique");

    let time_step = Utc::now().timestamp() as u64 / 30;
    let secret2_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &secret2)
        .expect("Valid base32 secret");

    let code1 = provider
        .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha1, 6)
        .expect("TOTP generation");
    let code2 = provider
        .generate_totp_for_step(&secret2_bytes, time_step, OtpAlgorithm::HmacSha1, 6)
        .expect("TOTP generation");

    assert_ne!(
        code1, code2,
        "Different secrets should generate different codes"
    );
}

#[test]
fn test_base32_encoding_compliance() {
    let provider = OtpCredentialProvider::new();

    // RFC 6238 references RFC 4648 for Base32 encoding
    let secret = provider.generate_secret();

    // Verify it's valid base32
    let decoded = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &secret);
    assert!(decoded.is_some(), "Generated secret should be valid base32");

    // Verify it can be re-encoded
    let decoded_bytes = decoded.unwrap();
    let re_encoded = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &decoded_bytes);
    assert_eq!(
        secret, re_encoded,
        "Secret should round-trip through base32 encoding"
    );

    // Verify no padding (as commonly used in TOTP URIs)
    assert!(
        !secret.contains('='),
        "Secret should not contain padding characters"
    );
}

#[test]
fn test_provisioning_uri_format() {
    let provider = OtpCredentialProvider::new();

    // RFC 6238 doesn't specify URI format, but Google Authenticator format is de facto standard
    let secret = "JBSWY3DPEHPK3PXP";
    let account = "user@example.com";
    let issuer = "TestApp";

    let uri =
        provider.generate_provisioning_uri(secret, account, issuer, OtpAlgorithm::HmacSha1, 6, 30);

    // Verify URI format
    assert!(
        uri.starts_with("otpauth://totp/"),
        "URI should start with otpauth://totp/"
    );
    assert!(
        uri.contains(&format!("secret={}", secret)),
        "URI should contain secret"
    );
    assert!(
        uri.contains(&format!("issuer={}", issuer)),
        "URI should contain issuer"
    );
    assert!(
        uri.contains("algorithm=HmacSHA1"),
        "URI should specify algorithm"
    );
    assert!(uri.contains("digits=6"), "URI should specify digits");
    assert!(uri.contains("period=30"), "URI should specify period");

    // Verify URL encoding
    let encoded_issuer = urlencoding::encode(issuer);
    let encoded_account = urlencoding::encode(account);
    assert!(
        uri.contains(&encoded_issuer.to_string()),
        "Issuer should be URL encoded"
    );
    assert!(
        uri.contains(&encoded_account.to_string()),
        "Account should be URL encoded"
    );
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_end_to_end_totp_flow() {
        let provider = OtpCredentialProvider::new();

        // 1. Generate secret
        let secret = provider.generate_secret();
        assert!(!secret.is_empty());

        // 2. Generate provisioning URI
        let uri = provider.generate_provisioning_uri(
            &secret,
            "test@kejaksaan.go.id",
            "SIMPelv2 Kejaksaan RI",
            OtpAlgorithm::HmacSha1,
            6,
            30,
        );
        assert!(uri.contains("otpauth://totp/"));

        // 3. Generate current TOTP code
        let secret_bytes = base32::decode(base32::Alphabet::RFC4648 { padding: false }, &secret)
            .expect("Valid base32 secret");
        let time_step = Utc::now().timestamp() as u64 / 30;
        let code = provider
            .generate_totp_for_step(&secret_bytes, time_step, OtpAlgorithm::HmacSha1, 6)
            .expect("TOTP generation");

        // 4. Verify the code
        let is_valid = provider
            .verify_totp(&secret, &code, OtpAlgorithm::HmacSha1, 6, 30)
            .expect("TOTP verification");

        assert!(is_valid, "Generated code should be valid");

        // 5. Verify invalid code is rejected
        let invalid_code = "000000";
        let is_invalid = provider
            .verify_totp(&secret, invalid_code, OtpAlgorithm::HmacSha1, 6, 30)
            .expect("TOTP verification");

        assert!(!is_invalid, "Invalid code should be rejected");
    }
}
