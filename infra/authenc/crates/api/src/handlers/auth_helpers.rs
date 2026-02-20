//! Shared authentication and authorization helpers for API handlers.
//!
//! This module provides common authorization patterns for REST API endpoints.
//! It includes JWT token extraction, validation, and user identification.
//!
//! ## Usage
//!
//! ```ignore
//! use crate::handlers::auth_helpers::extract_user_from_token;
//!
//! async fn protected_handler(
//!     State(state): State<Arc<ApiState>>,
//!     headers: HeaderMap,
//! ) -> Result<impl IntoResponse, ErrorResponse> {
//!     let user_id = extract_user_from_token(&state, &headers).await?;
//!     // ... handler logic
//! }
//! ```

use std::sync::Arc;

use axum::http::HeaderMap;
use uuid::Uuid;

use crate::state::ApiState;

/// Error response for authorization failures
#[derive(Debug)]
pub struct AuthError {
    pub message: String,
}

impl AuthError {
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Extract bearer token from Authorization header.
///
/// # Arguments
/// * `headers` - HTTP headers containing Authorization bearer token
///
/// # Returns
/// * `Ok(String)` - The extracted JWT token
/// * `Err(AuthError)` - If token is missing or invalid format
pub fn extract_bearer_token(headers: &HeaderMap) -> Result<String, AuthError> {
    let auth_header = headers
        .get("Authorization")
        .ok_or_else(|| AuthError::unauthorized("Missing Authorization header"))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| AuthError::unauthorized("Invalid Authorization header"))?;

    auth_str
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
        .ok_or_else(|| AuthError::unauthorized("Invalid Authorization format - expected 'Bearer <token>'"))
}

/// Extract and validate JWT token, returning the user ID.
///
/// This is the primary helper for protected endpoints that need to identify
/// the authenticated user.
///
/// # Arguments
/// * `state` - API state containing JWT service
/// * `headers` - HTTP headers containing Authorization bearer token
///
/// # Returns
/// * `Ok(Uuid)` - The authenticated user ID
/// * `Err(AuthError)` - If token is invalid or expired
pub async fn extract_user_from_token(
    state: &Arc<ApiState>,
    headers: &HeaderMap,
) -> Result<Uuid, AuthError> {
    let token = extract_bearer_token(headers)?;

    // Validate JWT token
    let claims = state
        .jwt_service
        .validate_token(&token)
        .map_err(|e| AuthError::unauthorized(format!("Invalid or expired token: {}", e)))?;

    // Extract user ID from subject claim
    Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::unauthorized("Invalid user ID in token"))
}

/// Verify JWT token from request body (for APIs that include token in JSON body).
///
/// This is useful for operations that pass the token in the request body
/// instead of the Authorization header.
///
/// # Arguments
/// * `state` - API state containing JWT service
/// * `token` - The JWT token string from request body
///
/// # Returns
/// * `Ok(Uuid)` - The authenticated user ID
/// * `Err(AuthError)` - If token is invalid or expired
pub async fn verify_token_from_body(
    state: &Arc<ApiState>,
    token: &str,
) -> Result<Uuid, AuthError> {
    // Validate JWT token
    let claims = state
        .jwt_service
        .validate_token(token)
        .map_err(|e| AuthError::unauthorized(format!("Invalid or expired token: {}", e)))?;

    // Extract user ID from subject claim
    Uuid::parse_str(&claims.sub)
        .map_err(|_| AuthError::unauthorized("Invalid user ID in token"))
}

/// Check if the authenticated user is the same as the target user.
///
/// This is useful for self-service operations where users can only
/// modify their own data.
///
/// # Arguments
/// * `authenticated_user_id` - The user ID from the JWT token
/// * `target_user_id` - The user ID being accessed/modified
///
/// # Returns
/// * `Ok(())` - If the user IDs match
/// * `Err(AuthError)` - If the user IDs don't match
pub fn verify_self_or_admin(
    authenticated_user_id: Uuid,
    target_user_id: Uuid,
) -> Result<(), AuthError> {
    if authenticated_user_id != target_user_id {
        return Err(AuthError::unauthorized(
            "You can only access your own resources",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_bearer_token_valid() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("Bearer test_token_123"),
        );

        let token = extract_bearer_token(&headers).unwrap();
        assert_eq!(token, "test_token_123");
    }

    #[test]
    fn test_extract_bearer_token_missing() {
        let headers = HeaderMap::new();
        let result = extract_bearer_token(&headers);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Missing"));
    }

    #[test]
    fn test_extract_bearer_token_wrong_format() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("Basic credentials"),
        );

        let result = extract_bearer_token(&headers);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("expected 'Bearer"));
    }

    #[test]
    fn test_verify_self_or_admin_same_user() {
        let user_id = Uuid::new_v4();
        let result = verify_self_or_admin(user_id, user_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_self_or_admin_different_user() {
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let result = verify_self_or_admin(user1, user2);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("only access your own"));
    }
}
</content>
