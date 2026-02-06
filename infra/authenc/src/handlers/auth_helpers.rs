//! Shared authentication and authorization helpers for handlers.
//!
//! This module provides common authorization patterns using capability-based
//! access control, replacing hardcoded role string checks.
//!
//! ## Usage
//!
//! ```ignore
//! use crate::handlers::auth_helpers::verify_capability;
//!
//! async fn admin_handler(
//!     State(state): State<Arc<AppState>>,
//!     headers: HeaderMap,
//! ) -> Result<impl IntoResponse, AuthencError> {
//!     let admin_id = verify_capability(&state, &headers, capabilities::MFA_ADMIN).await?;
//!     // ... handler logic
//! }
//! ```

use crate::app::AppState;
use crate::error::AuthencError;
use crate::services::authorization::capabilities;
use crate::utils::jwt;
use axum::http::HeaderMap;
use std::sync::Arc;
use uuid::Uuid;

/// Verify a JWT token and check that the user has the required capability.
///
/// This is the primary authorization helper that replaces hardcoded role checks.
///
/// # Arguments
/// * `state` - Application state containing user store and capability checker
/// * `headers` - HTTP headers containing Authorization bearer token
/// * `required_capability` - The capability code required (e.g., "mfa:admin")
///
/// # Returns
/// * `Ok(Uuid)` - The user ID if authorization succeeds
/// * `Err(AuthencError)` - If token is invalid or user lacks capability
pub async fn verify_capability(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    required_capability: &str,
) -> Result<Uuid, AuthencError> {
    let token = extract_bearer_token(headers)?;
    let user_id = verify_token_get_user_id(&token)?;

    // Check capability using CapabilityChecker
    state
        .capability_checker
        .require_capability(&user_id, required_capability)
        .await?;

    Ok(user_id)
}

/// Verify a JWT token and check that the user has any of the required capabilities.
///
/// # Arguments
/// * `state` - Application state
/// * `headers` - HTTP headers containing Authorization bearer token
/// * `required_capabilities` - Slice of capability codes (user needs at least one)
///
/// # Returns
/// * `Ok(Uuid)` - The user ID if authorization succeeds
/// * `Err(AuthencError)` - If token is invalid or user lacks all capabilities
pub async fn verify_any_capability(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    required_capabilities: &[&str],
) -> Result<Uuid, AuthencError> {
    let token = extract_bearer_token(headers)?;
    let user_id = verify_token_get_user_id(&token)?;

    state
        .capability_checker
        .require_any_capability(&user_id, required_capabilities)
        .await?;

    Ok(user_id)
}

/// Verify a JWT token and check that the user has all of the required capabilities.
///
/// # Arguments
/// * `state` - Application state
/// * `headers` - HTTP headers containing Authorization bearer token
/// * `required_capabilities` - Slice of capability codes (user needs all)
///
/// # Returns
/// * `Ok(Uuid)` - The user ID if authorization succeeds
/// * `Err(AuthencError)` - If token is invalid or user lacks any capability
pub async fn verify_all_capabilities(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    required_capabilities: &[&str],
) -> Result<Uuid, AuthencError> {
    let token = extract_bearer_token(headers)?;
    let user_id = verify_token_get_user_id(&token)?;

    state
        .capability_checker
        .require_all_capabilities(&user_id, required_capabilities)
        .await?;

    Ok(user_id)
}

/// Verify admin token from request body (for APIs that include token in JSON body).
///
/// This is useful for MFA admin operations that pass the token in the request body.
///
/// # Arguments
/// * `state` - Application state
/// * `token` - The JWT token string from request body
/// * `required_capability` - The capability code required
///
/// # Returns
/// * `Ok(Uuid)` - The admin user ID if authorization succeeds
pub async fn verify_admin_token_with_capability(
    state: &Arc<AppState>,
    token: &str,
    required_capability: &str,
) -> Result<Uuid, AuthencError> {
    let user_id = verify_token_get_user_id(token)?;

    state
        .capability_checker
        .require_capability(&user_id, required_capability)
        .await?;

    Ok(user_id)
}

/// Verify MFA admin privileges.
///
/// Convenience function for MFA administration operations.
/// Requires either `mfa:admin` or `system:admin` capability.
pub async fn verify_mfa_admin(state: &Arc<AppState>, token: &str) -> Result<Uuid, AuthencError> {
    let user_id = verify_token_get_user_id(token)?;

    state
        .capability_checker
        .require_any_capability(
            &user_id,
            &[capabilities::MFA_ADMIN, capabilities::SYSTEM_ADMIN],
        )
        .await?;

    Ok(user_id)
}

/// Verify MFA bypass privileges.
///
/// Required for operations like resetting or disabling other users' MFA.
pub async fn verify_mfa_bypass(state: &Arc<AppState>, token: &str) -> Result<Uuid, AuthencError> {
    let user_id = verify_token_get_user_id(token)?;

    state
        .capability_checker
        .require_any_capability(
            &user_id,
            &[capabilities::MFA_BYPASS, capabilities::SYSTEM_ADMIN],
        )
        .await?;

    Ok(user_id)
}

/// Verify user management privileges.
///
/// Required for creating, updating, or deleting users.
pub async fn verify_users_admin(state: &Arc<AppState>, token: &str) -> Result<Uuid, AuthencError> {
    let user_id = verify_token_get_user_id(token)?;

    state
        .capability_checker
        .require_any_capability(
            &user_id,
            &[capabilities::USERS_ADMIN, capabilities::SYSTEM_ADMIN],
        )
        .await?;

    Ok(user_id)
}

/// Verify audit log access.
pub async fn verify_audit_access(
    state: &Arc<AppState>,
    headers: &HeaderMap,
) -> Result<Uuid, AuthencError> {
    verify_capability(state, headers, capabilities::AUDIT_READ).await
}

/// Verify system admin privileges.
///
/// This is the highest level of access and should be used sparingly.
pub async fn verify_system_admin(state: &Arc<AppState>, token: &str) -> Result<Uuid, AuthencError> {
    let user_id = verify_token_get_user_id(token)?;

    state
        .capability_checker
        .require_capability(&user_id, capabilities::SYSTEM_ADMIN)
        .await?;

    Ok(user_id)
}

/// Check if user has a capability without erroring.
///
/// Useful for conditional logic where you want to check permissions
/// without immediately returning an error.
pub async fn user_has_capability(state: &Arc<AppState>, user_id: &Uuid, capability: &str) -> bool {
    state
        .capability_checker
        .user_has_capability(user_id, capability)
        .await
        .unwrap_or(false)
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Extract bearer token from Authorization header.
fn extract_bearer_token(headers: &HeaderMap) -> Result<String, AuthencError> {
    let auth_header = headers
        .get("Authorization")
        .ok_or_else(|| AuthencError::unauthorized("Missing Authorization header"))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| AuthencError::unauthorized("Invalid Authorization header"))?;

    auth_str
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
        .ok_or_else(|| AuthencError::unauthorized("Invalid Authorization format"))
}

/// Verify JWT token and extract user ID.
fn verify_token_get_user_id(token: &str) -> Result<Uuid, AuthencError> {
    let claims = jwt::verify_jwt(token)
        .map_err(|_| AuthencError::unauthorized("Invalid or expired token"))?;

    // Verify token purpose (must be access token)
    if claims.purpose.as_deref() != Some("access") {
        return Err(AuthencError::unauthorized("Invalid token purpose"));
    }

    Uuid::parse_str(&claims.sub).map_err(|_| AuthencError::internal("Invalid user ID in token"))
}

/// Legacy compatibility: Map old role names to capabilities.
///
/// This helper is provided for backward compatibility during migration.
/// New code should use capabilities directly.
#[deprecated(
    since = "0.2.0",
    note = "Use capability-based authorization instead of role names"
)]
pub fn legacy_role_has_capability(role_name: &str, capability: &str) -> bool {
    use crate::services::authorization::legacy_role_to_capabilities;

    let role_caps = legacy_role_to_capabilities(role_name);
    role_caps.contains(&capability)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Bearer test_token".parse().unwrap());

        let token = extract_bearer_token(&headers).unwrap();
        assert_eq!(token, "test_token");
    }

    #[test]
    fn test_extract_bearer_token_missing() {
        let headers = HeaderMap::new();
        assert!(extract_bearer_token(&headers).is_err());
    }

    #[test]
    fn test_extract_bearer_token_wrong_format() {
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", "Basic credentials".parse().unwrap());

        assert!(extract_bearer_token(&headers).is_err());
    }
}
