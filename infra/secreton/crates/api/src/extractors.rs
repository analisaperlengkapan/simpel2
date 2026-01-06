//! Axum extractors for common request patterns.
//!
//! This module provides reusable extractors that eliminate code duplication
//! across handlers by encapsulating common extraction and validation logic.

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use crate::{ApiError, middleware::RequestContext};

/// Authenticated user information extracted from JWT token.
/// This extractor automatically retrieves user information from the RequestContext
/// populated by the authentication middleware. Use this in any handler that requires authentication.
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
        // Try to extract from RequestContext (populated by middleware)
        if let Some(ctx) = parts.extensions.get::<RequestContext>() {
            let user_id = ctx.user_id.as_deref().unwrap_or_default();

            // Parse UUID or generate one if invalid/missing (fallback)
            let id = uuid::Uuid::parse_str(user_id).unwrap_or_else(|_| uuid::Uuid::nil());

            // Get username from claims or fallback to subject/id
            let username = ctx.jwt_claims
                .as_ref()
                .map(|c| c.name.clone())
                .or_else(|| ctx.user_id.clone())
                .unwrap_or_else(|| "anonymous".to_string());

            return Ok(AuthenticatedUser {
                id,
                username,
                email: ctx.user_email.clone(),
                roles: ctx.user_roles.clone(),
            });
        }

        // Fallback: Check for Authorization header but no context
        // This usually means middleware failed or wasn't applied.
        // For security, we should probably fail, but legacy behavior was dummy.
        // Given the requirement for "best practice", we should fail if auth header is present
        // but context is missing (implying auth middleware didn't validate it).

        // However, if we want to support non-middleware testing, we might keep a minimal fallback.
        // But the prompt says "optimalkan, sesuai standar".
        // Standard is: Middleware does auth, handler uses it.

        Err(ApiError::Authentication {
            message: "Authentication context missing. Ensure authentication middleware is active.".to_string(),
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
