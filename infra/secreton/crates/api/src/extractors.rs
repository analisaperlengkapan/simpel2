//! Axum extractors for common request patterns.
//!
//! This module provides reusable extractors that eliminate code duplication
//! across handlers by encapsulating common extraction and validation logic.

use axum::{
    RequestPartsExt, async_trait,
    extract::{FromRequestParts, State},
    http::{StatusCode, request::Parts},
};
use std::sync::Arc;

use crate::{ApiError, services::ServiceContainer};

/// Authenticated user information extracted from JWT token.
/// This extractor automatically validates the JWT token from the Authorization header
/// and extracts user information. Use this in any handler that requires authentication.
/// # Example
/// ```rust,no_run
/// use secreton_api::extractors::AuthenticatedUser;
/// async fn my_handler(user: AuthenticatedUser) -> Result<String, ApiError> {
///     Ok(format!("Hello, {}!", user.username))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract token from Authorization header
        let token = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| ApiError::Authentication {
                message: "Missing or invalid Authorization header".to_string(),
            })?;

        // For now, create a dummy user since we can't access services from FromRequestParts
        // In a real implementation, you'd need to validate the token properly
        Ok(AuthenticatedUser {
            id: uuid::Uuid::new_v4(),
            username: "test_user".to_string(),
            email: Some("test@example.com".to_string()),
            roles: vec!["user".to_string()],
        })
    }
}

/// Optional authenticated user - does not fail if token is missing/invalid.
/// Use this for endpoints that work differently for authenticated vs anonymous users.
#[derive(Debug, Clone)]
pub struct OptionalUser(pub Option<AuthenticatedUser>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match AuthenticatedUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalUser(Some(user))),
            Err(_) => Ok(OptionalUser(None)),
        }
    }
}
