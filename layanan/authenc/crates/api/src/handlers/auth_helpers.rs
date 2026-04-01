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
    /// Machine-readable error code for programmatic discrimination.
    /// Avoids fragile `message.contains(…)` checks in callers.
    pub error_code: AuthErrorCode,
}

/// Machine-readable error codes for [`AuthError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthErrorCode {
    /// Generic unauthorized (missing/invalid/expired token).
    Unauthorized,
    /// User must change password before accessing protected resources.
    PasswordChangeRequired,
}

impl AuthError {
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            error_code: AuthErrorCode::Unauthorized,
        }
    }

    pub fn password_change_required(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            error_code: AuthErrorCode::PasswordChangeRequired,
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
        .ok_or_else(|| {
            AuthError::unauthorized("Invalid Authorization format - expected 'Bearer <token>'")
        })
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
        .verify_token(&token)
        .map_err(|e| AuthError::unauthorized(format!("Invalid or expired token: {}", e)))?;

    // Extract user ID from subject claim
    Uuid::parse_str(&claims.sub).map_err(|_| AuthError::unauthorized("Invalid user ID in token"))
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
pub async fn verify_token_from_body(state: &Arc<ApiState>, token: &str) -> Result<Uuid, AuthError> {
    // Validate JWT token
    let claims = state
        .jwt_service
        .verify_token(token)
        .map_err(|e| AuthError::unauthorized(format!("Invalid or expired token: {}", e)))?;

    // Extract user ID from subject claim
    Uuid::parse_str(&claims.sub).map_err(|_| AuthError::unauthorized("Invalid user ID in token"))
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

/// Check whether the authenticated user must change their password before
/// accessing any endpoint other than the password-change endpoint itself.
///
/// Extracts the `require_password_change` claim from the JWT.  Returns
/// `Err(AuthError)` when the flag is `true`, signalling the caller to reject
/// the request with an appropriate HTTP 403 response.
pub fn check_password_change_required(
    state: &Arc<ApiState>,
    headers: &HeaderMap,
) -> Result<(), AuthError> {
    let token = extract_bearer_token(headers)?;
    let claims = state
        .jwt_service
        .verify_token(&token)
        .map_err(|e| AuthError::unauthorized(format!("Invalid or expired token: {}", e)))?;

    check_password_change_required_from_claims(&claims)
}

/// Same as [`check_password_change_required`] but accepts pre-verified claims
/// to avoid double JWT verification when the caller also needs the user ID.
pub fn check_password_change_required_from_claims(
    claims: &authenc_crypto::jwt::TokenClaims,
) -> Result<(), AuthError> {
    // Check the custom `require_password_change` claim embedded by
    // `build_user_custom_claims`.
    let must_change = claims
        .custom
        .get("require_password_change")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if must_change {
        return Err(AuthError::password_change_required(
            "Anda harus mengubah password sebelum mengakses fitur lain.",
        ));
    }

    Ok(())
}

/// Extract bearer token, verify it, check password-change requirement, and
/// return the user UUID — all in a single JWT verification pass.
///
/// Use this instead of calling `check_password_change_required` followed by
/// `extract_user_from_token` to avoid verifying the JWT twice.
pub fn extract_user_with_password_check(
    state: &Arc<ApiState>,
    headers: &HeaderMap,
) -> Result<Uuid, AuthError> {
    let token = extract_bearer_token(headers)?;
    let claims = state
        .jwt_service
        .verify_token(&token)
        .map_err(|e| AuthError::unauthorized(format!("Invalid or expired token: {}", e)))?;

    check_password_change_required_from_claims(&claims)?;

    Uuid::parse_str(&claims.sub).map_err(|_| AuthError::unauthorized("Invalid user ID in token"))
}

/// Builds custom JWT claims for a user (NIP, name, jabatan, etc.)
pub fn build_user_custom_claims(
    user: Option<&authenc_types::User>,
) -> std::collections::HashMap<String, serde_json::Value> {
    let mut custom_claims = std::collections::HashMap::new();
    if let Some(u) = user {
        if let Some(ref nip) = u.nip {
            custom_claims.insert("nip".into(), serde_json::json!(nip));
        }
        // Build display name using the same logic as GET /me:
        // first_name + last_name, falling back to nama, then username.
        let display_name = match (&u.first_name, &u.last_name) {
            (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
            (Some(f), None) => Some(f.clone()),
            (None, Some(l)) => Some(l.clone()),
            _ => u.nama.clone(),
        };
        if let Some(ref name) = display_name {
            custom_claims.insert("name".into(), serde_json::json!(name));
        }
        custom_claims.insert("preferred_username".into(), serde_json::json!(u.username));
        if let Some(ref jabatan) = u.jabatan {
            custom_claims.insert("jabatan".into(), serde_json::json!(jabatan));
        }
        if !u.satker_code.is_empty() {
            custom_claims.insert("satker_code".into(), serde_json::json!(u.satker_code));
        }
        if !u.email.is_empty() {
            custom_claims.insert("email".into(), serde_json::json!(u.email));
        }
        // MFA status
        custom_claims.insert("mfa_enabled".into(), serde_json::json!(u.mfa_enabled));
        // NOTE: mfa_setup_required is NOT simply `!mfa_enabled`.  Whether MFA
        // setup is required depends on the realm's MFA policy (Optional /
        // Required / RequiredForAdmins) which is evaluated during the login
        // flow and surfaced via `LoginResponse.mfa_setup_required`.  Setting
        // the JWT claim to `!mfa_enabled` would incorrectly mark every
        // non-MFA user as needing setup even when the policy is Optional.
        // We default to `false` here; the login handler overrides the
        // session flag when MFA setup is actually required.
        custom_claims.insert("mfa_setup_required".into(), serde_json::json!(false));
        // Password change requirement — always embedded in JWT so frontend
        // route guards and middleware can enforce the redirect without an
        // extra API round-trip.  We always emit the claim (even when false)
        // so the frontend never has to rely on serde(default) to infer the
        // value from a missing key.
        custom_claims.insert(
            "require_password_change".into(),
            serde_json::json!(u.require_password_change),
        );
        // Roles
        let roles: Vec<String> = u.roles.iter().map(|r| r.name.clone()).collect();
        custom_claims.insert("realm_access".into(), serde_json::json!({"roles": roles}));
    }
    custom_claims
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
        assert!(result.unwrap_err().message.contains("only access your own"));
    }
}
