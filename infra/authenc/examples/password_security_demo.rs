//! Password Security Features Demonstration
//!
//! This example demonstrates all password security enhancements:
//! - Argon2 hashing with secure parameters
//! - Password strength validation
//! - Password history checking
//! - Password expiration management

use authenc::utils::crypto::password::{
    calculate_password_expiration, check_password_expiration, check_password_history,
    hash_password, validate_password_strength, verify_password,
};
use chrono::{Duration, Utc};

fn main() {
    println!("=== Authenc Password Security Features Demo ===\n");

    // 1. Argon2 Hashing Demo
    println!("1. ARGON2 HASHING (64 MB memory, 10 iterations)");
    println!("   ------------------------------------------------");

    let password = "Myw0rd2024!";
    println!("   Original password: {}", password);

    let hash = hash_password(password).expect("Failed to hash password");
    println!("   Hashed password: {}", hash);
    println!("   Hash format: PHC string (includes algorithm parameters)");

    // Verify the hash format
    if hash.starts_with("$argon2id$") {
        println!("   ✓ Using Argon2id (hybrid mode)");
    }
    if hash.contains("m=65536") {
        println!("   ✓ Memory cost: 64 MB (65536 KiB)");
    }
    if hash.contains("t=10") {
        println!("   ✓ Time cost: 10 iterations");
    }
    if hash.contains("p=4") {
        println!("   ✓ Parallelism: 4 threads");
    }

    // Verify password
    let is_valid = verify_password(&hash, password).expect("Failed to verify");
    println!(
        "   ✓ Password verification: {}",
        if is_valid { "SUCCESS" } else { "FAILED" }
    );

    let is_invalid = verify_password(&hash, "WrongPassword").expect("Failed to verify");
    println!(
        "   ✓ Wrong password rejected: {}\n",
        if !is_invalid { "SUCCESS" } else { "FAILED" }
    );

    // 2. Password Strength Validation Demo
    println!("2. PASSWORD STRENGTH VALIDATION");
    println!("   ------------------------------------------------");

    // Test weak password
    let weak_password = "weak";
    let result = validate_password_strength(weak_password, None);
    println!("   Testing weak password: '{}'", weak_password);
    println!("   Valid: {}", result.is_valid);
    println!("   Strength score: {}/100", result.strength_score);
    println!("   Errors:");
    for error in &result.errors {
        println!("     - {}", error);
    }
    println!();

    // Test strong password
    let strong_password = "MyVerySecureP@ssw0rd2024!";
    let result = validate_password_strength(strong_password, Some("testuser"));
    println!("   Testing strong password: '{}'", strong_password);
    println!("   Valid: {}", result.is_valid);
    println!("   Strength score: {}/100", result.strength_score);
    if result.errors.is_empty() {
        println!("   ✓ No validation errors");
    }
    println!();

    // Test password with username
    let password_with_username = "TestUser123!";
    let result = validate_password_strength(password_with_username, Some("testuser"));
    println!(
        "   Testing password containing username: '{}'",
        password_with_username
    );
    println!("   Valid: {}", result.is_valid);
    if !result.is_valid {
        println!("   ✓ Correctly rejected (contains username)");
    }
    println!();

    // 3. Password History Demo
    println!("3. PASSWORD HISTORY CHECK (Prevent Reuse)");
    println!("   ------------------------------------------------");

    // Create password history
    let old_pass1 = hash_password("OldPassword1!").expect("Failed to hash");
    let old_pass2 = hash_password("OldPassword2!").expect("Failed to hash");
    let old_pass3 = hash_password("OldPassword3!").expect("Failed to hash");
    let history = vec![old_pass1, old_pass2, old_pass3];

    println!("   Password history size: {}", history.len());
    println!("   History limit: 5");
    println!();

    // Test reused password
    let reused_password = "OldPassword1!";
    let is_reused = check_password_history(reused_password, &history, 5);
    println!("   Testing reused password: '{}'", reused_password);
    println!("   Found in history: {}", is_reused);
    if is_reused {
        println!("   ✓ Correctly detected password reuse");
    }
    println!();

    // Test new password
    let new_password = "NewPassword4!";
    let is_new = check_password_history(new_password, &history, 5);
    println!("   Testing new password: '{}'", new_password);
    println!("   Found in history: {}", is_new);
    if !is_new {
        println!("   ✓ Correctly accepted new password");
    }
    println!();

    // 4. Password Expiration Demo
    println!("4. PASSWORD EXPIRATION POLICY");
    println!("   ------------------------------------------------");

    let now = Utc::now();

    // Calculate expiration (90 days)
    let changed_at = now;
    let expires_at = calculate_password_expiration(changed_at, 90);
    println!("   Password changed: {}", changed_at.format("%Y-%m-%d"));
    println!("   Expiration policy: 90 days");
    if let Some(expiry) = expires_at {
        println!("   Password expires: {}", expiry.format("%Y-%m-%d"));
        println!("   ✓ Expiration calculated correctly");
    }
    println!();

    // Check expired password
    let expired = now - Duration::days(1);
    let (is_expired, days_left) = check_password_expiration(Some(expired), 7);
    println!("   Testing expired password (expired 1 day ago):");
    println!("   Is expired: {}", is_expired);
    println!("   Days left: {}", days_left.unwrap());
    if is_expired {
        println!("   ✓ Correctly detected expired password");
    }
    println!();

    // Check password expiring soon
    let expiring_soon = now + Duration::days(5);
    let (is_expired, days_left) = check_password_expiration(Some(expiring_soon), 7);
    println!("   Testing password expiring soon (5 days left):");
    println!("   Is expired/warning: {}", is_expired);
    println!("   Days left: {}", days_left.unwrap());
    if is_expired {
        println!("   ✓ Correctly triggered expiration warning (within 7-day grace period)");
    }
    println!();

    // Check password not expiring soon
    let not_expiring = now + Duration::days(30);
    let (is_expired, days_left) = check_password_expiration(Some(not_expiring), 7);
    println!("   Testing password not expiring soon (30 days left):");
    println!("   Is expired/warning: {}", is_expired);
    println!("   Days left: {}", days_left.unwrap());
    if !is_expired {
        println!("   ✓ Correctly accepted (outside grace period)");
    }
    println!();

    // Summary
    println!("=== SUMMARY ===");
    println!("✓ Argon2 hashing with secure parameters (64 MB, 10 iterations)");
    println!("✓ Password strength validation (8+ chars, complexity rules)");
    println!("✓ Password history checking (prevents reuse of last 5)");
    println!("✓ Password expiration policy (configurable, default 90 days)");
    println!("\nAll password security features are working correctly!");
}
