//! Common validation utilities for Secreton
//!
//! This module provides reusable validation functions to reduce code duplication
//! across auth methods, secret engines, and other services.

use crate::error::CoreError;

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, CoreError>;

/// Validate that a string is not empty
/// # Arguments
/// * `value` - String to validate
/// * `field_name` - Name of the field for error messages
/// # Returns
/// Ok(()) if valid, Err with descriptive message if empty
#[inline]
pub fn validate_not_empty(value: &str, field_name: &str) -> ValidationResult<()> {
    if value.is_empty() {
        Err(CoreError::Validation {
            message: format!("{} cannot be empty", field_name),
        })
    } else {
        Ok(())
    }
}

/// Validate that a collection is not empty
/// # Arguments
/// * `collection` - Collection to validate
/// * `field_name` - Name of the field for error messages
#[inline]
pub fn validate_collection_not_empty<T>(
    collection: &[T],
    field_name: &str,
) -> ValidationResult<()> {
    if collection.is_empty() {
        Err(CoreError::Validation {
            message: format!("{} cannot be empty", field_name),
        })
    } else {
        Ok(())
    }
}

/// Validate that a string matches expected format/pattern
/// # Arguments
/// * `value` - String to validate
/// * `pattern` - Regex pattern to match
/// * `field_name` - Name of the field for error messages
pub fn validate_pattern(
    value: &str,
    pattern: &regex::Regex,
    field_name: &str,
) -> ValidationResult<()> {
    if !pattern.is_match(value) {
        Err(CoreError::Validation {
            message: format!("{} format is invalid", field_name),
        })
    } else {
        Ok(())
    }
}

/// Validate string length constraints
/// # Arguments
/// * `value` - String to validate
/// * `min_len` - Minimum length (inclusive), None for no minimum
/// * `max_len` - Maximum length (inclusive), None for no maximum
/// * `field_name` - Name of the field for error messages
pub fn validate_length(
    value: &str,
    min_len: Option<usize>,
    max_len: Option<usize>,
    field_name: &str,
) -> ValidationResult<()> {
    let len = value.len();

    if let Some(min) = min_len {
        if len < min {
            return Err(CoreError::Validation {
                message: format!("{} must be at least {} characters", field_name, min),
            });
        }
    }

    if let Some(max) = max_len {
        if len > max {
            return Err(CoreError::Validation {
                message: format!("{} must be at most {} characters", field_name, max),
            });
        }
    }

    Ok(())
}

/// Validate that a value is within a numeric range
/// # Arguments
/// * `value` - Value to validate
/// * `min` - Minimum value (inclusive), None for no minimum
/// * `max` - Maximum value (inclusive), None for no maximum
/// * `field_name` - Name of the field for error messages
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: Option<T>,
    max: Option<T>,
    field_name: &str,
) -> ValidationResult<()> {
    if let Some(min_val) = min {
        if value < min_val {
            return Err(CoreError::Validation {
                message: format!("{} must be at least {}", field_name, min_val),
            });
        }
    }

    if let Some(max_val) = max {
        if value > max_val {
            return Err(CoreError::Validation {
                message: format!("{} must be at most {}", field_name, max_val),
            });
        }
    }

    Ok(())
}

/// Validate CIDR notation
/// # Arguments
/// * `cidr` - CIDR string to validate
pub fn validate_cidr(cidr: &str) -> ValidationResult<()> {
    cidr.parse::<ipnetwork::IpNetwork>()
        .map_err(|e| CoreError::Validation {
            message: format!("Invalid CIDR notation: {}", e),
        })?;
    Ok(())
}

/// Validate JWT token format (basic check, not verification)
/// # Arguments
/// * `token` - JWT token string
pub fn validate_jwt_format(token: &str) -> ValidationResult<()> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(CoreError::Validation {
            message: "JWT must have 3 parts separated by dots".to_string(),
        });
    }
    Ok(())
}

/// Validate URL format
/// # Arguments
/// * `url` - URL string to validate
/// * `field_name` - Name of the field for error messages
pub fn validate_url(url: &str, field_name: &str) -> ValidationResult<()> {
    url::Url::parse(url).map_err(|e| CoreError::Validation {
        message: format!("{} is not a valid URL: {}", field_name, e),
    })?;
    Ok(())
}

/// Combine multiple validation results
/// # Arguments
/// * `validations` - Vector of validation results to combine
/// # Returns
/// Ok(()) if all validations pass, Err with first error otherwise
pub fn combine_validations(validations: Vec<ValidationResult<()>>) -> ValidationResult<()> {
    for result in validations {
        result?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_not_empty() {
        assert!(validate_not_empty("test", "field").is_ok());
        assert!(validate_not_empty("", "field").is_err());
    }

    #[test]
    fn test_validate_length() {
        assert!(validate_length("test", Some(2), Some(10), "field").is_ok());
        assert!(validate_length("a", Some(2), Some(10), "field").is_err());
        assert!(validate_length("12345678901", Some(2), Some(10), "field").is_err());
    }

    #[test]
    fn test_validate_range() {
        assert!(validate_range(5, Some(1), Some(10), "field").is_ok());
        assert!(validate_range(0, Some(1), Some(10), "field").is_err());
        assert!(validate_range(11, Some(1), Some(10), "field").is_err());
    }

    #[test]
    fn test_validate_jwt_format() {
        assert!(validate_jwt_format("header.payload.signature").is_ok());
        assert!(validate_jwt_format("invalid").is_err());
        assert!(validate_jwt_format("only.two").is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com", "url").is_ok());
        assert!(validate_url("not-a-url", "url").is_err());
    }

    #[test]
    fn test_combine_validations() {
        let valid = vec![
            validate_not_empty("test", "field1"),
            validate_length("test", Some(1), Some(10), "field2"),
        ];
        assert!(combine_validations(valid).is_ok());

        let invalid = vec![
            validate_not_empty("test", "field1"),
            validate_not_empty("", "field2"),
        ];
        assert!(combine_validations(invalid).is_err());
    }
}
