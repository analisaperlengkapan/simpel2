// SPDX-License-Identifier: Apache-2.0
//! Validation helper utilities for handlers
//!
//! Provides utilities to validate request payloads and return appropriate error responses.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use garde::Validate;
use serde_json::json;

/// Validation error response
#[derive(Debug)]
pub struct ValidationError {
    pub errors: Vec<String>,
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": {
                "code": "VALIDATION_ERROR",
                "message": "Request validation failed",
                "details": self.errors,
            }
        }));

        (StatusCode::BAD_REQUEST, body).into_response()
    }
}

/// Validate a request payload and return validation errors if any
pub fn validate_request<T>(request: &T) -> Result<(), ValidationError>
where
    T: Validate,
    T::Context: Default,
{
    match request.validate() {
        Ok(_) => Ok(()),
        Err(errors) => {
            // In garde 0.22.0, Report doesn't have flatten() - iterate over errors directly
            let error_messages: Vec<String> = errors
                .iter()
                .map(|(path, error)| format!("{}: {}", path, error))
                .collect();

            Err(ValidationError {
                errors: error_messages,
            })
        }
    }
}

/// Macro to validate and sanitize a request in one line
///
/// Usage:
/// ```ignore
/// validate_and_sanitize!(request)?;
/// ```
#[macro_export]
macro_rules! validate_and_sanitize {
    ($request:expr) => {{
        // Validate first
        $crate::handlers::validation_helper::validate_request(&$request)?;
        // Then sanitize
        $request.sanitize();
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use garde::Validate;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, Validate)]
    struct TestRequest {
        #[garde(length(min = 3, max = 10))]
        username: String,

        #[garde(range(min = 18, max = 120))]
        age: u32,
    }

    #[test]
    fn test_validate_request_success() {
        let request = TestRequest {
            username: "user123".to_string(),
            age: 25,
        };

        assert!(validate_request(&request).is_ok());
    }

    #[test]
    fn test_validate_request_failure() {
        let request = TestRequest {
            username: "ab".to_string(), // too short
            age: 150,                   // too old
        };

        let result = validate_request(&request);
        assert!(result.is_err());

        if let Err(err) = result {
            assert!(!err.errors.is_empty());
        }
    }
}
