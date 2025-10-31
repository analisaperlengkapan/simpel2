// SPDX-License-Identifier: Apache-2.0
//! Input validation and sanitization utilities
//!
//! This module provides comprehensive input validation using the garde crate
//! and sanitization functions to prevent injection attacks and ensure data integrity.

use garde::Validate;
use regex::Regex;
use std::sync::OnceLock;

/// Maximum request body size (1MB)
pub const MAX_REQUEST_BODY_SIZE: usize = 1_048_576;

/// Email validation regex (RFC 5322 simplified)
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();

/// Username validation regex (alphanumeric, underscore, hyphen, 3-50 chars)
static USERNAME_REGEX: OnceLock<Regex> = OnceLock::new();

/// Password complexity regex (at least one uppercase, one lowercase, one digit)
static PASSWORD_COMPLEXITY_REGEX: OnceLock<Regex> = OnceLock::new();

/// Satker code validation regex (alphanumeric, 2-20 chars)
static SATKER_CODE_REGEX: OnceLock<Regex> = OnceLock::new();

/// NIP validation regex (18 digits)
static NIP_REGEX: OnceLock<Regex> = OnceLock::new();

/// Phone number validation regex (international format)
static PHONE_REGEX: OnceLock<Regex> = OnceLock::new();

/// Initialize regex patterns
fn email_regex() -> &'static Regex {
    EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .expect("Invalid email regex")
    })
}

fn username_regex() -> &'static Regex {
    USERNAME_REGEX
        .get_or_init(|| Regex::new(r"^[a-zA-Z0-9_-]{3,50}$").expect("Invalid username regex"))
}

fn password_complexity_regex() -> &'static Regex {
    PASSWORD_COMPLEXITY_REGEX.get_or_init(|| {
        Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).+$")
            .expect("Invalid password complexity regex")
    })
}

fn satker_code_regex() -> &'static Regex {
    SATKER_CODE_REGEX
        .get_or_init(|| Regex::new(r"^[A-Z0-9]{2,20}$").expect("Invalid satker code regex"))
}

fn nip_regex() -> &'static Regex {
    NIP_REGEX.get_or_init(|| Regex::new(r"^\d{18}$").expect("Invalid NIP regex"))
}

fn phone_regex() -> &'static Regex {
    PHONE_REGEX.get_or_init(|| Regex::new(r"^\+?[1-9]\d{1,14}$").expect("Invalid phone regex"))
}

/// Validate email format
pub fn validate_email(email: &str) -> bool {
    email_regex().is_match(email)
}

/// Validate username format
pub fn validate_username(username: &str) -> bool {
    username_regex().is_match(username)
}

/// Validate password complexity
///
/// Requirements:
/// - Minimum 8 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one digit
pub fn validate_password_complexity(password: &str) -> bool {
    password.len() >= 8 && password_complexity_regex().is_match(password)
}

/// Validate satker code format
pub fn validate_satker_code(code: &str) -> bool {
    satker_code_regex().is_match(code)
}

/// Validate NIP (Nomor Induk Pegawai) format
pub fn validate_nip(nip: &str) -> bool {
    nip_regex().is_match(nip)
}

/// Validate phone number format
pub fn validate_phone_number(phone: &str) -> bool {
    phone_regex().is_match(phone)
}

/// Sanitize string input to prevent injection attacks
///
/// This function:
/// - Trims whitespace
/// - Removes null bytes
/// - Removes control characters (except newline and tab)
/// - Limits length to prevent DoS
pub fn sanitize_string(input: &str, max_length: usize) -> String {
    input
        .trim()
        .chars()
        .filter(|c| {
            // Keep printable characters, newlines, and tabs
            !c.is_control() || *c == '\n' || *c == '\t'
        })
        .take(max_length)
        .collect()
}

/// Sanitize username (more restrictive than general string)
pub fn sanitize_username(username: &str) -> String {
    username
        .trim()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(50)
        .collect()
}

/// Sanitize email address
pub fn sanitize_email(email: &str) -> String {
    email
        .trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || "@.-_+".contains(*c))
        .take(255)
        .collect()
}

/// Sanitize satker code (uppercase alphanumeric only)
pub fn sanitize_satker_code(code: &str) -> String {
    code.trim()
        .to_uppercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(20)
        .collect()
}

/// Custom garde validator for email
pub fn email_validator(value: &str, _context: &()) -> garde::Result {
    if validate_email(value) {
        Ok(())
    } else {
        Err(garde::Error::new("invalid email format"))
    }
}

/// Custom garde validator for username
pub fn username_validator(value: &str, _context: &()) -> garde::Result {
    if validate_username(value) {
        Ok(())
    } else {
        Err(garde::Error::new(
            "username must be 3-50 characters, alphanumeric with _ or -",
        ))
    }
}

/// Custom garde validator for password complexity
pub fn password_validator(value: &str, _context: &()) -> garde::Result {
    if validate_password_complexity(value) {
        Ok(())
    } else {
        Err(garde::Error::new(
            "password must be at least 8 characters with uppercase, lowercase, and digit",
        ))
    }
}

/// Custom garde validator for satker code
pub fn satker_code_validator(value: &str, _context: &()) -> garde::Result {
    if validate_satker_code(value) {
        Ok(())
    } else {
        Err(garde::Error::new(
            "satker code must be 2-20 uppercase alphanumeric characters",
        ))
    }
}

/// Custom garde validator for NIP
pub fn nip_validator(value: &str, _context: &()) -> garde::Result {
    if validate_nip(value) {
        Ok(())
    } else {
        Err(garde::Error::new("NIP must be exactly 18 digits"))
    }
}

/// Custom garde validator for phone number
pub fn phone_validator(value: &str, _context: &()) -> garde::Result {
    if validate_phone_number(value) {
        Ok(())
    } else {
        Err(garde::Error::new("invalid phone number format"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(validate_email("user@example.com"));
        assert!(validate_email("test.user+tag@domain.co.id"));
        assert!(!validate_email("invalid"));
        assert!(!validate_email("@example.com"));
        assert!(!validate_email("user@"));
    }

    #[test]
    fn test_username_validation() {
        assert!(validate_username("user123"));
        assert!(validate_username("test_user"));
        assert!(validate_username("user-name"));
        assert!(!validate_username("ab")); // too short
        assert!(!validate_username("user@name")); // invalid char
        assert!(!validate_username("a".repeat(51).as_str())); // too long
    }

    #[test]
    fn test_password_complexity() {
        assert!(validate_password_complexity("Password123"));
        assert!(validate_password_complexity("Secure1Pass"));
        assert!(!validate_password_complexity("password")); // no uppercase
        assert!(!validate_password_complexity("PASSWORD123")); // no lowercase
        assert!(!validate_password_complexity("Password")); // no digit
        assert!(!validate_password_complexity("Pass1")); // too short
    }

    #[test]
    fn test_satker_code_validation() {
        assert!(validate_satker_code("KEJARI"));
        assert!(validate_satker_code("KEJATI01"));
        assert!(!validate_satker_code("kejari")); // lowercase
        assert!(!validate_satker_code("K")); // too short
        assert!(!validate_satker_code("KEJARI-01")); // invalid char
    }

    #[test]
    fn test_nip_validation() {
        assert!(validate_nip("123456789012345678"));
        assert!(!validate_nip("12345678901234567")); // too short
        assert!(!validate_nip("1234567890123456789")); // too long
        assert!(!validate_nip("12345678901234567a")); // non-digit
    }

    #[test]
    fn test_phone_validation() {
        assert!(validate_phone_number("+628123456789"));
        assert!(validate_phone_number("628123456789"));
        assert!(validate_phone_number("+12025551234"));
        assert!(!validate_phone_number("123")); // too short
        assert!(!validate_phone_number("+0123456789")); // starts with 0
    }

    #[test]
    fn test_sanitize_string() {
        assert_eq!(sanitize_string("  hello  ", 100), "hello");
        assert_eq!(sanitize_string("hello\x00world", 100), "helloworld");
        assert_eq!(sanitize_string("test\x01\x02", 100), "test");
        assert_eq!(
            sanitize_string("long".repeat(100).as_str(), 10),
            "longlonglo"
        );
    }

    #[test]
    fn test_sanitize_username() {
        assert_eq!(sanitize_username("  user123  "), "user123");
        assert_eq!(sanitize_username("user@name"), "username");
        assert_eq!(sanitize_username("test_user-01"), "test_user-01");
    }

    #[test]
    fn test_sanitize_email() {
        assert_eq!(sanitize_email("  User@Example.COM  "), "user@example.com");
        assert_eq!(sanitize_email("test+tag@domain.com"), "test+tag@domain.com");
    }

    #[test]
    fn test_sanitize_satker_code() {
        assert_eq!(sanitize_satker_code("  kejari  "), "KEJARI");
        assert_eq!(sanitize_satker_code("kejati-01"), "KEJATI01");
    }
}
