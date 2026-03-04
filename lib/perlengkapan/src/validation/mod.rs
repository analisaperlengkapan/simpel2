//! Validation helpers for Perlengkapan domain

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Field '{field}' is required")]
    Required { field: String },

    #[error("Field '{field}' must be at least {min} characters")]
    MinLength { field: String, min: usize },

    #[error("Field '{field}' must be at most {max} characters")]
    MaxLength { field: String, max: usize },

    #[error("Field '{field}' has invalid format")]
    InvalidFormat { field: String },

    #[error("Field '{field}' must be a positive number")]
    MustBePositive { field: String },

    #[error("Invalid date range: start date must be before end date")]
    InvalidDateRange,
}

/// Validate that a string field is not empty
pub fn validate_required(value: &str, field: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::Required {
            field: field.to_string(),
        });
    }
    Ok(())
}

/// Validate string length
pub fn validate_length(
    value: &str,
    field: &str,
    min: Option<usize>,
    max: Option<usize>,
) -> Result<(), ValidationError> {
    let len = value.len();

    if let Some(min_len) = min
        && len < min_len {
            return Err(ValidationError::MinLength {
                field: field.to_string(),
                min: min_len,
            });
        }

    if let Some(max_len) = max
        && len > max_len {
            return Err(ValidationError::MaxLength {
                field: field.to_string(),
                max: max_len,
            });
        }

    Ok(())
}

/// Validate that a number is positive
pub fn validate_positive(value: f64, field: &str) -> Result<(), ValidationError> {
    if value <= 0.0 {
        return Err(ValidationError::MustBePositive {
            field: field.to_string(),
        });
    }
    Ok(())
}
