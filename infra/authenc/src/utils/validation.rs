// SPDX-License-Identifier: Apache-2.0
//! Input validation and sanitization utilities
//!
//! This module provides comprehensive input validation using the garde crate
//! and sanitization functions to prevent injection attacks and ensure data integrity.
//!
//! Note: Most core validation logic has been moved to `lib_common::validation`
//! to allow sharing with other services.

// Re-export common validation logic
pub use lib_common::validation::{
    validate_email, validate_username, validate_password_complexity,
    validate_phone_number, validate_satker_code, validate_nip,
    sanitize_string, sanitize_username, sanitize_email, sanitize_satker_code,
};

// Re-export garde validators
pub use lib_common::validation::{
    email_validator, username_validator, password_validator, phone_validator,
    satker_code_validator, nip_validator,
};

/// Maximum request body size (1MB)
pub const MAX_REQUEST_BODY_SIZE: usize = 1_048_576;

// Optional validators wrapping common validators

pub fn phone_validator_optional(value: &Option<String>, _context: &()) -> garde::Result {
    match value {
        Some(v) => phone_validator(v, _context),
        None => Ok(()),
    }
}

pub fn nip_validator_optional(value: &Option<String>, _context: &()) -> garde::Result {
    match value {
        Some(v) => nip_validator(v, _context),
        None => Ok(()),
    }
}

pub fn email_validator_optional(value: &Option<String>, _context: &()) -> garde::Result {
    match value {
        Some(v) => email_validator(v, _context),
        None => Ok(()),
    }
}

pub fn username_validator_optional(value: &Option<String>, _context: &()) -> garde::Result {
    match value {
        Some(v) => username_validator(v, _context),
        None => Ok(()),
    }
}


pub fn satker_code_validator_optional(value: &Option<String>, _context: &()) -> garde::Result {
    match value {
        Some(v) => satker_code_validator(v, _context),
        None => Ok(()),
    }
}
