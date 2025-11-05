//! Password Security Validation Tests
//!
//! Comprehensive tests to validate all password security enhancements:
//! - Argon2 configuration (cost factor >= 10, memory >= 64MB)
//! - Password strength validation (min 8 chars, complexity rules)
//! - Password history check (prevent reuse of last 5 passwords)
//! - Password expiration policy (configurable, default: 90 days)

use authenc::utils::crypto::password::{
    PasswordStrengthResult, calculate_password_expiration, check_password_expiration,
    check_password_history, hash_password, validate_password_strength, verify_password,
};
use chrono::{Duration, Utc};

#[test]
fn test_argon2_configuration() {
    // Test that Argon2 is configured with secure parameters
    // Memory: 64 MB (65536 KiB), Iterations: 10, Parallelism: 4

    let password = "TestPassword123!";
    let hash = hash_password(password).expect("Failed to hash password");

    // Verify the hash format is PHC string format
    assert!(hash.starts_with("$argon2id$"));

    // Verify the hash contains the correct parameters
    // Format: $argon2id$v=19$m=65536,t=10,p=4$...
    assert!(
        hash.contains("m=65536"),
        "Memory cost should be 65536 KiB (64 MB) "
    );
    assert!(hash.contains("t=10"), "Time cost should be 10 iterations ");
    assert!(hash.contains("p=4"), "Parallelism should be 4 ");

    // Verify password can be verified
    let is_valid = verify_password(&hash, password).expect("Failed to verify password");
    assert!(is_valid, "Password verification should succeed ");

    // Verify wrong password fails
    let is_invalid = verify_password(&hash, "WrongPassword").expect("Failed to verify password");
    assert!(!is_invalid, "Wrong password should not verify ");
}

#[test]
fn test_password_strength_minimum_length() {
    // Test minimum 8 characters requirement
    let result = validate_password_strength("Short1!", None);
    assert!(
        !result.is_valid,
        "Password shorter than 8 chars should be invalid "
    );
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("at least 8 characters"))
    );

    let result = validate_password_strength("LongPass1!", None);
    assert!(result.is_valid, "Password with 8+ chars should be valid ");
}

#[test]
fn test_password_strength_uppercase_requirement() {
    // Test uppercase letter requirement
    let result = validate_password_strength("lowercase123!", None);
    assert!(
        !result.is_valid,
        "Password without uppercase should be invalid "
    );
    assert!(result.errors.iter().any(|e| e.contains("uppercase letter")));

    let result = validate_password_strength("Uppercase123!", None);
    assert!(result.is_valid, "Password with uppercase should be valid ");
}

#[test]
fn test_password_strength_lowercase_requirement() {
    // Test lowercase letter requirement
    let result = validate_password_strength("UPPERCASE123!", None);
    assert!(
        !result.is_valid,
        "Password without lowercase should be invalid "
    );
    assert!(result.errors.iter().any(|e| e.contains("lowercase letter")));

    let result = validate_password_strength("Lowercase123!", None);
    assert!(result.is_valid, "Password with lowercase should be valid ");
}

#[test]
fn test_password_strength_digit_requirement() {
    // Test digit requirement
    let result = validate_password_strength("NoDigits!", None);
    assert!(
        !result.is_valid,
        "Password without digit should be invalid "
    );
    assert!(result.errors.iter().any(|e| e.contains("digit")));

    let result = validate_password_strength("WithDigit1!", None);
    assert!(result.is_valid, "Password with digit should be valid ");
}

#[test]
fn test_password_strength_special_char_requirement() {
    // Test special character requirement
    let result = validate_password_strength("NoSpecial123", None);
    assert!(
        !result.is_valid,
        "Password without special char should be invalid "
    );
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("special character"))
    );

    let result = validate_password_strength("WithSpecial123!", None);
    assert!(
        result.is_valid,
        "Password with special char should be valid "
    );
}

#[test]
fn test_password_strength_username_inclusion() {
    // Test that password cannot contain username
    let result = validate_password_strength("MyUsername123!", Some("myusername"));
    assert!(
        !result.is_valid,
        "Password containing username should be invalid "
    );
    assert!(result.errors.iter().any(|e| e.contains("username")));

    let result = validate_password_strength("Diff3!", Some("name"));
    assert!(
        result.is_valid,
        "Password not containing username should be valid "
    );
}

#[test]
fn test_password_strength_weak_patterns() {
    // Test detection of common weak patterns
    let weak_passwords = vec![
        ("Password123!", "password"),
        ("Admin123456!", "admin"),
        ("Qwerty123!", "qwerty"),
        ("Letmein123!", "letmein"),
        ("Test123456!", "123456"),
    ];

    for (password, pattern) in weak_passwords {
        let result = validate_password_strength(password, None);
        assert!(
            !result.is_valid,
            "Password with weak pattern \"{}\" should be invalid ",
            pattern
        );
    }
}

#[test]
fn test_password_strength_repeated_characters() {
    // Test detection of repeated characters
    let result = validate_password_strength("Passsword123!", None);
    assert!(
        !result.is_valid,
        "Password with 3+ repeated chars should be invalid "
    );
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("repeated characters"))
    );

    let result = validate_password_strength("Password123!", None);
    assert!(
        result.is_valid,
        "Password without repeated chars should be valid "
    );
}

#[test]
fn test_password_strength_sequential_characters() {
    // Test detection of sequential characters
    let sequential_passwords = vec!["Pass123word!", "Passabc456!", "Passqwe789!"];

    for password in sequential_passwords {
        let result = validate_password_strength(password, None);
        assert!(
            !result.is_valid,
            "Password with sequential chars \"{}\" should be invalid",
            password
        );
    }
}

#[test]
fn test_password_strength_score() {
    // Test password strength scoring
    let weak = validate_password_strength("Pass123!", None);
    let medium = validate_password_strength("MyPassword123!", None);
    let strong = validate_password_strength("MyVerySecureP@ssw0rd2024!", None);

    assert!(weak.strength_score < medium.strength_score);
    assert!(medium.strength_score < strong.strength_score);
    assert!(
        strong.strength_score >= 70,
        "Strong password should have score >= 70"
    );
}

#[test]
fn test_password_history_check() {
    // Test password history checking
    let old_pass1 = hash_password("OldPassword1!").expect("Failed to hash");
    let old_pass2 = hash_password("OldPassword2!").expect("Failed to hash");
    let old_pass3 = hash_password("OldPassword3!").expect("Failed to hash");

    let history = vec![old_pass1, old_pass2, old_pass3];

    // Test that old password is detected
    let is_reused = check_password_history("OldPassword1!", &history, 5);
    assert!(is_reused, "Old password should be detected in history");

    let is_reused = check_password_history("OldPassword2!", &history, 5);
    assert!(is_reused, "Old password should be detected in history");

    // Test that new password is not in history
    let is_new = check_password_history("NewPassword4!", &history, 5);
    assert!(!is_new, "New password should not be in history");
}

#[test]
fn test_password_history_limit() {
    // Test that history limit is respected
    let mut history = Vec::new();
    for i in 1..=10 {
        let hash = hash_password(&format!("OldPassword{}!", i)).expect("Failed to hash");
        history.push(hash);
    }

    // With limit of 5, only last 5 should be checked
    let is_reused = check_password_history("OldPassword1!", &history, 5);
    assert!(
        !is_reused,
        "Password beyond history limit should not be detected"
    );

    let is_reused = check_password_history("OldPassword7!", &history, 5);
    assert!(
        is_reused,
        "Password within history limit should be detected"
    );
}

#[test]
fn test_password_expiration_calculation() {
    // Test password expiration calculation
    let changed_at = Utc::now();

    // Test with 90 days expiration
    let expires_at = calculate_password_expiration(changed_at, 90);
    assert!(expires_at.is_some(), "Expiration should be calculated");

    let expected = changed_at + Duration::days(90);
    let actual = expires_at.unwrap();

    // Allow 1 second tolerance for test execution time
    let diff = (actual - expected).num_seconds().abs();
    assert!(diff <= 1, "Expiration should be 90 days from change date");

    // Test with 0 days (never expires)
    let expires_at = calculate_password_expiration(changed_at, 0);
    assert!(exs_none(), "Password with 0 days should never expire");
}

#[test]
fn test_password_expiration_check() {
    // Test password expiration checking
    let now = Utc::now();

    // Test expired password
    let expired = now - Duration::days(1);
    let (is_expired, days_left) = check_password_expiration(Some(expired), 7);
    assert!(is_expired, "Password past expiration should be expired");
    assert!(days_left.unwrap() < 0, "Days left should be negative");

    // Test password expiring soon (within grace period)
    let expiring_soon = now + Duration::days(5);
    let (is_expired, days_left) = check_password_expiration(Some(expiring_soon), 7);
    assert!(is_expired, "Password within grace period should be flagged");
    assert_eq!(days_left.unwrap(), 5, "Days left should be 5");

    // Test password not expiring soon
    let not_expiring = now + Duration::days(30);
    let (is_expired, days_left) = check_password_expiration(Some(not_expiring), 7);
    assert!(
        !is_expired,
        "Password outside grace period should not be flagged"
    );
    assert_eq!(days_left.unwrap(), 30, "Days left should be 30");

    // Test no expiration set
    let (is_expired, days_left) = check_password_expiration(None, 7);
    assert!(
        !is_expired,
        "Password with no expiration should not be expired"
    );
    assert!(days_left.is_none(), "Days left should be None");
}

#[test]
fn test_comprehensive_password_validation() {
    // Test a comprehensive valid password
    let password = "MySecureP@ssw0rd2024!";
    let result = validate_password_strength(password, Some("testuser"));

    assert!(result.is_valid, "Comprehensive valid password should pass");
    assert!(
        result.errors.is_empty(),
        "Valid password should have no errors"
    );
    assert!(
        result.strength_score >= 70,
        "Strong password should have high score"
    );
}

#[test]
fn test_comprehensive_password_rejection() {
    // Test a password that fails multiple requirements
    let password = "weak";
    let result = validate_password_strength(password, None);

    assert!(!result.is_valid, "Weak password should fail");
    assert!(
        result.errors.len() >= 4,
        "Weak password should have multiple errors"
    );
    assert!(
        result.strength_score < 50,
        "Weak password should have low score"
    );
}

#[test]
fn test_argon2_performance() {
    // Test that Argon2 hashing takes reasonable time (should be > 50ms for security)
    use std::time::Instant;

    let password = "TestPassword123!";
    let start = Instant::now();
    let _hash = hash_password(password).expect("Failed to hash password");
    let duration = start.elapsed();

    // Argon2 with our parameters should take at least 50ms
    assert!(
        duration.as_millis() >= 50,
        "Argon2 hashing should take at least 50ms for security (took {}ms)",
        duration.as_millis()
    );

    // But not too long (< 500ms for usability)
    assert!(
        duration.as_millis() < 500,
        "Argon2 hashing should complete within 500ms for usability (took {}ms)",
        duration.as_millis()
    );
}

#[test]
fn test_password_hash_uniqueness() {
    // Test that same password produces different hashes (due to random salt)
    let password = "TestPassword123!";

    let hash1 = hash_password(password).expect("Failed to hash password");
    let hash2 = hash_password(password).expect("Failed to hash password");

    assert_ne!(
        hash1, hash2,
        "Same password should produce different hashes due to random salt"
    );

    // But both should verify correctly
    assert!(verify_password(&hash1, password).unwrap());
    assert!(verify_password(&hash2, password).unwrap());
}

#[test]
fn test_password_verification_constant_time() {
    // Test that password verification is constant-time (timing attack resistant)
    use std::time::Instant;

    let password = "TestPassword123!";
    let hash = hash_password(password).expect("Failed to hash password");

    // Measure time for correct password
    let start = Instant::now();
    let _ = verify_password(&hash, password);
    let correct_duration = start.elapsed();

    // Measure time for incorrect password
    let start = Instant::now();
    let _ = verify_password(&hash, "WrongPassword123!");
    let incorrect_duration = start.elapsed();

    // The difference should be minimal (< 10ms) for constant-time comparison
    let diff = if correct_duration > incorrect_duration {
        correct_duration - incorrect_duration
    } else {
        incorrect_duration - correct_duration
    };

    assert!(
        diff.as_millis() < 10,
        "Password verification should be constant-time (diff: {}ms)",
        diff.as_millis()
    );
}
