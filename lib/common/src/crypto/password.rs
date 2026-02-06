//! Password hashing utilities using Argon2id
//!
//! Provides secure password hashing and verification for authentication.
//!
//! # Example
//!
//! ```rust,no_run
//! use lib_common::crypto::password::{hash_password, verify_password};
//!
//! let hash = hash_password("my_password").unwrap();
//! let is_valid = verify_password(&hash, "my_password").unwrap();
//! assert!(is_valid);
//! ```

use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::SaltString,
};
use rand::rngs::OsRng;

/// Hash a password using Argon2id with secure parameters
///
/// # Security Parameters
/// - Algorithm: Argon2id (hybrid mode, resistant to side-channel and GPU attacks)
/// - Memory cost: 64 MB (65536 KiB)
/// - Time cost: 10 iterations
/// - Parallelism: 4 threads
/// - Output length: 32 bytes (256 bits)
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);

    let params = Params::new(
        65536,    // m_cost: 64 MB memory
        10,       // t_cost: 10 iterations
        4,        // p_cost: 4 parallel threads
        Some(32), // output length: 32 bytes
    )?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

/// Verify a password against its hash
///
/// Uses constant-time comparison to prevent timing attacks.
pub fn verify_password(hash: &str, password: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Password strength validation result
#[derive(Debug, Clone)]
pub struct PasswordStrengthResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub strength_score: u8,
}

impl PasswordStrengthResult {
    pub fn new(is_valid: bool, errors: Vec<String>, strength_score: u8) -> Self {
        Self {
            is_valid,
            errors,
            strength_score,
        }
    }

    pub fn valid(strength_score: u8) -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            strength_score,
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            strength_score: 0,
        }
    }
}

/// Validate password strength
///
/// Requirements:
/// - Minimum 8 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one digit
/// - At least one special character
pub fn validate_password_strength(
    password: &str,
    username: Option<&str>,
) -> PasswordStrengthResult {
    let mut errors = Vec::new();
    let mut strength_score = 0u8;

    if password.len() < 8 {
        errors.push("Password must be at least 8 characters long".to_string());
    } else {
        strength_score += 20;
        if password.len() >= 12 {
            strength_score += 10;
        }
        if password.len() >= 16 {
            strength_score += 10;
        }
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        errors.push("Password must contain at least one uppercase letter".to_string());
    } else {
        strength_score += 15;
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        errors.push("Password must contain at least one lowercase letter".to_string());
    } else {
        strength_score += 15;
    }

    if !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push("Password must contain at least one digit".to_string());
    } else {
        strength_score += 15;
    }

    let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?/~`";
    if !password.chars().any(|c| special_chars.contains(c)) {
        errors.push("Password must contain at least one special character".to_string());
    } else {
        strength_score += 15;
    }

    if let Some(user) = username {
        let lower_password = password.to_lowercase();
        let lower_username = user.to_lowercase();
        if lower_password.contains(&lower_username) {
            errors.push("Password cannot contain your username".to_string());
            strength_score = strength_score.saturating_sub(20);
        }
    }

    let lower_password = password.to_lowercase();
    let weak_patterns = ["password", "123456", "qwerty", "admin", "letmein"];

    for pattern in &weak_patterns {
        if lower_password.contains(pattern) {
            errors.push(format!("Password cannot contain '{}'", pattern));
            strength_score = strength_score.saturating_sub(15);
        }
    }

    // Check for 3+ repeated characters (e.g., "aaa", "111")
    if has_repeated_characters(password, 3) {
        errors.push("Password must not contain 3 or more repeated characters in a row".to_string());
        strength_score = strength_score.saturating_sub(10);
    }

    // Check for sequential characters (e.g., "abc", "123", "qwe")
    if has_sequential_characters(&lower_password) {
        errors.push(
            "Password must not contain sequential character patterns (e.g., abc, 123, qwerty)"
                .to_string(),
        );
        strength_score = strength_score.saturating_sub(10);
    }

    let is_valid = errors.is_empty();
    PasswordStrengthResult::new(is_valid, errors, strength_score.min(100))
}

/// Check if a string contains N or more identical characters in a row
///
/// # Examples
/// ```rust,no_run
/// # use lib_common::crypto::password::has_repeated_characters;
/// assert!(has_repeated_characters("Passsword", 3));   // 3x 's'
/// assert!(!has_repeated_characters("Password", 3));    // max 2 consecutive
/// ```
pub fn has_repeated_characters(password: &str, min_repeat: usize) -> bool {
    if min_repeat < 2 || password.len() < min_repeat {
        return false;
    }
    let chars: Vec<char> = password.chars().collect();
    let mut count = 1usize;
    for i in 1..chars.len() {
        if chars[i] == chars[i - 1] {
            count += 1;
            if count >= min_repeat {
                return true;
            }
        } else {
            count = 1;
        }
    }
    false
}

/// Check if a string contains common sequential character patterns
///
/// Detects:
/// - Numeric sequences: "123", "234", "345", ...
/// - Alphabetic sequences: "abc", "bcd", "cde", ...
/// - Keyboard row patterns: "qwe", "wer", "asd", "zxc", etc.
///
/// Checks forward sequences of 3+ consecutive characters.
pub fn has_sequential_characters(password: &str) -> bool {
    let lower = password.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();

    // Check keyboard row patterns (common sequences typed on QWERTY layout)
    let keyboard_rows = [
        "qwertyuiop",
        "asdfghjkl",
        "zxcvbnm",
        "1234567890",
    ];

    for row in &keyboard_rows {
        // Check for 3+ consecutive chars from same keyboard row in order
        let row_chars: Vec<char> = row.chars().collect();
        for window in row_chars.windows(3) {
            let pattern: String = window.iter().collect();
            if lower.contains(&pattern) {
                return true;
            }
        }
    }

    // Check ascending/descending alphabetic sequences (e.g., "abc", "cba")
    if chars.len() >= 3 {
        for i in 0..chars.len() - 2 {
            let a = chars[i] as i32;
            let b = chars[i + 1] as i32;
            let c = chars[i + 2] as i32;

            // All must be lowercase letters
            if chars[i].is_ascii_lowercase()
                && chars[i + 1].is_ascii_lowercase()
                && chars[i + 2].is_ascii_lowercase()
            {
                // Ascending: a, b, c (diff = +1, +1)
                if b - a == 1 && c - b == 1 {
                    return true;
                }
                // Descending: c, b, a (diff = -1, -1)
                if b - a == -1 && c - b == -1 {
                    return true;
                }
            }

            // Also check ascending/descending digit sequences
            if chars[i].is_ascii_digit()
                && chars[i + 1].is_ascii_digit()
                && chars[i + 2].is_ascii_digit()
            {
                if b - a == 1 && c - b == 1 {
                    return true;
                }
                if b - a == -1 && c - b == -1 {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if a password matches any in the password history
pub fn check_password_history(
    password: &str,
    password_history: &[String],
    history_limit: usize,
) -> bool {
    let check_count = password_history.len().min(history_limit);
    let skip_count = password_history.len().saturating_sub(check_count);

    for hash in password_history.iter().skip(skip_count) {
        if let Ok(matches) = verify_password(hash, password)
            && matches
        {
            return true;
        }
    }

    false
}

/// Calculate password expiration date based on policy
pub fn calculate_password_expiration(
    password_changed_at: chrono::DateTime<chrono::Utc>,
    expiration_days: u32,
) -> Option<chrono::DateTime<chrono::Utc>> {
    if expiration_days == 0 {
        return None;
    }

    Some(password_changed_at + chrono::Duration::days(expiration_days as i64))
}

/// Check if a password has expired
pub fn check_password_expiration(
    password_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    grace_period_days: u32,
) -> (bool, Option<i64>) {
    if let Some(expires_at) = password_expires_at {
        let now = chrono::Utc::now();
        let duration = expires_at.signed_duration_since(now);
        let days_left = duration.num_days();

        // Check if expired or within grace period
        let is_expired = days_left <= grace_period_days as i64;

        (is_expired, Some(days_left))
    } else {
        // No expiration set
        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "SecureP@ssw0rd!";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(&hash, password).unwrap());
        assert!(!verify_password(&hash, "wrong_password").unwrap());
    }

    #[test]
    fn test_password_strength_valid() {
        let result = validate_password_strength("MyP@ssw0rd!", None);
        assert!(result.is_valid);
        assert!(result.strength_score >= 70);
    }

    #[test]
    fn test_password_strength_too_short() {
        let result = validate_password_strength("Ab1!", None);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("8 characters")));
    }

    #[test]
    fn test_password_strength_no_uppercase() {
        let result = validate_password_strength("myp@ssw0rd!", None);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("uppercase")));
    }

    #[test]
    fn test_password_with_username() {
        let result = validate_password_strength("JohnDoe123!", Some("johndoe"));
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("username")));
    }
}
