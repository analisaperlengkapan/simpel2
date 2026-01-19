//! Axum extractors for common request patterns.
//!
//! This module provides reusable extractors that eliminate code duplication
//! across handlers by encapsulating common extraction and validation logic.

use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{ApiError, middleware::RequestContext};

/// Extracted Namespace from request context
/// Defaults to "default" if no context or specific claims are found
#[derive(Debug, Clone)]
/// Mewakili pub `Namespace(pub`.
pub struct Namespace(pub String);

impl<S> FromRequestParts<S> for Namespace
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            if let Some(ctx) = parts.extensions.get::<RequestContext>() {
                Ok(Namespace(ctx.derive_namespace()))
            } else {
                Ok(Namespace("default".to_string()))
            }
        }
    }
}

/// Authenticated user information extracted from JWT token.
/// This extractor automatically retrieves user information from the RequestContext
/// populated by the authentication middleware. Use this in any handler that requires authentication.
/// # Example
/// ```rust,no_run
/// use secreton_api::{extractors::AuthenticatedUser, ApiError};
/// async fn my_handler(user: AuthenticatedUser) -> Result<String, ApiError> {
///     Ok(format!("Hello, {}!", user.username))
/// }
/// ```
#[derive(Debug, Clone)]
/// Mewakili pub `AuthenticatedUser`.
pub struct AuthenticatedUser {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            // Try to extract from RequestContext (populated by middleware)
            if let Some(ctx) = parts.extensions.get::<RequestContext>() {
                let user_id = ctx.user_id.as_deref().unwrap_or_default();

                // Parse UUID or generate one if invalid/missing (fallback)
                let id = uuid::Uuid::parse_str(user_id).unwrap_or_else(|_| uuid::Uuid::nil());

                // Get username from claims or fallback to subject/id
                let username = ctx
                    .jwt_claims
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
            // For security, we should fail if auth header is present
            // but context is missing (implying auth middleware didn't validate it).
            // Standard is: Middleware does auth, handler uses it.

            Err(ApiError::Authentication {
                message:
                    "Authentication context missing. Ensure authentication middleware is active."
                        .to_string(),
            })
        }
    }
}

/// Optional authenticated user - does not fail if token is missing/invalid.
/// Use this for endpoints that work differently for authenticated vs anonymous users.
#[derive(Debug, Clone)]
/// Mewakili pub `OptionalUser(pub`.
pub struct OptionalUser(pub Option<AuthenticatedUser>);

impl<S> FromRequestParts<S> for OptionalUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            match AuthenticatedUser::from_request_parts(parts, state).await {
                Ok(user) => Ok(OptionalUser(Some(user))),
                Err(_) => Ok(OptionalUser(None)),
            }
        }
    }
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;
    use axum::{Router, body::Body, http::Request, routing::get};
    use secreton_core::namespace::{AdminLevel, JwtClaims};
    use std::collections::HashMap;
    use std::time::Instant;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_namespace_extractor_default() {
        let app = Router::new().route("/", get(|Namespace(ns): Namespace| async move { ns }));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(&body[..], b"default");
    }

    #[tokio::test]
    async fn test_namespace_extractor_with_context() {
        use crate::middleware::RequestContext;

        let app = Router::new().route("/", get(|Namespace(ns): Namespace| async move { ns }));

        let claims = JwtClaims {
            sub: "user".into(),
            name: "User".into(),
            email: "user@example.com".into(),
            satker_code: Some("KJA001".into()),
            wilayah_code: Some("SUMUT".into()),
            admin_level: AdminLevel::Satker,
            roles: vec![],
            permissions: vec![],
            exp: 0,
            iat: 0,
            iss: "test".into(),
            metadata: HashMap::new(),
        };

        let context = RequestContext {
            request_id: "req".into(),
            user_id: Some("user".into()),
            user_email: None,
            user_roles: vec![],
            user_permissions: vec![],
            start_time: Instant::now(),
            jwt_claims: Some(claims),
            policy_names: vec![],
        };

        let request = Request::builder()
            .uri("/")
            .extension(context)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(&body[..], b"satker-kja001");
    }
}
