//! Common validation utilities
//!
//! This module provides reusable validation functions.

use crate::error::{CommonError, Result};
use regex::Regex;
use std::sync::OnceLock;

/// Email validation regex (RFC 5322 simplified)
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();

/// Username validation regex (alphanumeric, underscore, hyphen, 3-50 chars)
static USERNAME_REGEX: OnceLock<Regex> = OnceLock::new();

/// Phone number validation regex (international format)
static PHONE_REGEX: OnceLock<Regex> = OnceLock::new();

/// Satker code validation regex (alphanumeric, 2-20 chars)
static SATKER_CODE_REGEX: OnceLock<Regex> = OnceLock::new();

/// NIP validation regex (18 digits)
static NIP_REGEX: OnceLock<Regex> = OnceLock::new();

/// MFA code validation regex (6 digits)
static MFA_CODE_REGEX: OnceLock<Regex> = OnceLock::new();

/// Validate that a collection is not empty
pub fn validate_collection_not_empty<T>(collection: &[T], field_name: &str) -> Result<()> {
    if collection.is_empty() {
        Err(CommonError::Validation {
            message: format!("{} cannot be empty", field_name),
        })
    } else {
        Ok(())
    }
}

/// Validate that a string matches expected format/pattern
pub fn validate_pattern(value: &str, pattern: &Regex, field_name: &str) -> Result<()> {
    if !pattern.is_match(value) {
        Err(CommonError::Validation {
            message: format!("{} format is invalid", field_name),
        })
    } else {
        Ok(())
    }
}

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

fn phone_regex() -> &'static Regex {
    // Matches: optional +, digit 1-9, then 6-14 more digits (7-15 total, minimum valid phone)
    PHONE_REGEX.get_or_init(|| Regex::new(r"^\+?[1-9]\d{6,14}$").expect("Invalid phone regex"))
}

fn satker_code_regex() -> &'static Regex {
    SATKER_CODE_REGEX
        .get_or_init(|| Regex::new(r"^[A-Z0-9]{2,20}$").expect("Invalid satker code regex"))
}

fn nip_regex() -> &'static Regex {
    NIP_REGEX.get_or_init(|| Regex::new(r"^\d{18}$").expect("Invalid NIP regex"))
}

fn mfa_code_regex() -> &'static Regex {
    MFA_CODE_REGEX.get_or_init(|| Regex::new(r"^\d{6}$").expect("Invalid MFA code regex"))
}

/// Validate that a string is not empty
pub fn validate_not_empty(value: &str, field_name: &str) -> Result<()> {
    if value.is_empty() {
        Err(CommonError::Validation {
            message: format!("{} cannot be empty", field_name),
        })
    } else {
        Ok(())
    }
}

/// Validate string length constraints
pub fn validate_length(
    value: &str,
    min_len: Option<usize>,
    max_len: Option<usize>,
    field_name: &str,
) -> Result<()> {
    let len = value.len();

    if let Some(min) = min_len
        && len < min {
            return Err(CommonError::Validation {
                message: format!("{} must be at least {} characters", field_name, min),
            });
        }

    if let Some(max) = max_len
        && len > max {
            return Err(CommonError::Validation {
                message: format!("{} must be at most {} characters", field_name, max),
            });
        }

    Ok(())
}

/// Validate that a value is within a numeric range
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: Option<T>,
    max: Option<T>,
    field_name: &str,
) -> Result<()> {
    if let Some(min_val) = min
        && value < min_val {
            return Err(CommonError::Validation {
                message: format!("{} must be at least {}", field_name, min_val),
            });
        }

    if let Some(max_val) = max
        && value > max_val {
            return Err(CommonError::Validation {
                message: format!("{} must be at most {}", field_name, max_val),
            });
        }

    Ok(())
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
/// Requirements:
/// - Minimum 8 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one digit
pub fn validate_password_complexity(password: &str) -> bool {
    if password.len() < 8 {
        return false;
    }

    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());

    has_lowercase && has_uppercase && has_digit
}

/// Validate phone number format
pub fn validate_phone_number(phone: &str) -> bool {
    phone_regex().is_match(phone)
}

/// Validate satker code format
pub fn validate_satker_code(code: &str) -> bool {
    satker_code_regex().is_match(code)
}

/// Validate NIP (Nomor Induk Pegawai) format
pub fn validate_nip(nip: &str) -> bool {
    nip_regex().is_match(nip)
}

/// Validate MFA code format (6 digits)
pub fn validate_mfa_code(code: &str) -> bool {
    mfa_code_regex().is_match(code)
}

/// Validate URL format
pub fn validate_url(url: &str, field_name: &str) -> Result<()> {
    url::Url::parse(url).map_err(|e| CommonError::Validation {
        message: format!("{} is not a valid URL: {}", field_name, e),
    })?;
    Ok(())
}

/// Validate CIDR notation
#[cfg(feature = "backend")]
pub fn validate_cidr(cidr: &str) -> Result<()> {
    cidr.parse::<ipnetwork::IpNetwork>()
        .map_err(|e| CommonError::Validation {
            message: format!("Invalid CIDR notation: {}", e),
        })?;
    Ok(())
}

/// Validate JWT token format (basic check, not verification)
pub fn validate_jwt_format(token: &str) -> Result<()> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(CommonError::Validation {
            message: "JWT must have 3 parts separated by dots".to_string(),
        });
    }
    Ok(())
}

/// Combine multiple validation results
pub fn combine_validations(validations: Vec<Result<()>>) -> Result<()> {
    for result in validations {
        result?;
    }
    Ok(())
}

/// Sanitize string input to prevent injection attacks
pub fn sanitize_string(input: &str, max_length: usize) -> String {
    input
        .trim()
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .take(max_length)
        .collect()
}

/// Sanitize username
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

// Custom garde validators

#[cfg(feature = "validation")]
pub fn email_validator(value: &str, _context: &()) -> garde::Result {
    if validate_email(value) {
        Ok(())
    } else {
        Err(garde::Error::new("invalid email format"))
    }
}

#[cfg(feature = "validation")]
pub fn username_validator(value: &str, _context: &()) -> garde::Result {
    if validate_username(value) {
        Ok(())
    } else {
        Err(garde::Error::new(
            "username must be 3-50 characters, alphanumeric with _ or -",
        ))
    }
}

#[cfg(feature = "validation")]
pub fn password_validator(value: &str, _context: &()) -> garde::Result {
    if validate_password_complexity(value) {
        Ok(())
    } else {
        Err(garde::Error::new(
            "password must be at least 8 characters with uppercase, lowercase, and digit",
        ))
    }
}

#[cfg(feature = "validation")]
pub fn phone_validator(value: &str, _context: &()) -> garde::Result {
    if validate_phone_number(value) {
        Ok(())
    } else {
        Err(garde::Error::new("invalid phone number format"))
    }
}

#[cfg(feature = "validation")]
pub fn satker_code_validator(value: &str, _context: &()) -> garde::Result {
    if validate_satker_code(value) {
        Ok(())
    } else {
        Err(garde::Error::new(
            "satker code must be 2-20 uppercase alphanumeric characters",
        ))
    }
}

#[cfg(feature = "validation")]
pub fn nip_validator(value: &str, _context: &()) -> garde::Result {
    if validate_nip(value) {
        Ok(())
    } else {
        Err(garde::Error::new("NIP must be exactly 18 digits"))
    }
}
