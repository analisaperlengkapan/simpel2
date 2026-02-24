//! Authentication middleware for Axum 0.8.x
//!
//! This middleware validates JWT tokens from the Authorization header and
//! injects the authenticated user into request extensions.

use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;

use authenc_crypto::jwt_validator::JwtValidator;
use authenc_types::error::AuthencError;

/// The key used to store the authenticated user in request extensions
pub const AUTH_USER_KEY: &str = "authenc.auth_user";

/// Claims extracted from the JWT token
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthUser {
    /// Unique identifier of the authenticated user
    pub id: String,
    /// Email address of the authenticated user
    pub email: String,
    /// List of roles assigned to the authenticated user
    pub roles: Vec<String>,
}

/// State for auth middleware
#[derive(Clone)]
pub struct AuthState {
    /// JWT validator service
    pub jwt_validator: Arc<JwtValidator>,
}

impl AuthState {
    /// Create a new auth state with the given JWT validator
    pub fn new(jwt_validator: Arc<JwtValidator>) -> Self {
        Self { jwt_validator }
    }
}

/// Middleware function that validates JWT tokens from the Authorization header
pub async fn auth_middleware(
    State(state): State<Arc<AuthState>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip authentication for public endpoints
    if is_public_endpoint(request.uri().path()) {
        return Ok(next.run(request).await);
    }

    // Get the token from the Authorization header
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate the token
    let user = validate_token(token, &state.jwt_validator)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Insert the user into the request extensions
    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

/// Extract and validate the JWT token
async fn validate_token(
    token: &str,
    jwt_validator: &JwtValidator,
) -> Result<AuthUser, AuthencError> {
    // Validate JWT using the crypto crate's validator
    let result = jwt_validator.validate_token(token).await.map_err(|e| {
        error!("JWT validation failed: {}", e);
        AuthencError::unauthorized("Invalid token")
    })?;

    if !result.valid {
        let msg = result
            .error
            .unwrap_or_else(|| "Token validation failed".to_string());
        error!("Token validation failed: {}", msg);
        return Err(AuthencError::unauthorized("Invalid token"));
    }

    let user_id = result.user_id.unwrap_or_else(|| "unknown".to_string());

    // Create AuthUser from the validation result
    Ok(AuthUser {
        id: user_id,
        email: "user@example.com".to_string(), // TODO: Extract from token claims
        roles: vec!["user".to_string()],       // TODO: Extract from token claims
    })
}

/// Extension trait to get the authenticated user from a request
pub trait AuthUserExt {
    /// Get the authenticated user if available
    fn auth_user(&self) -> Option<&AuthUser>;
}

impl<B> AuthUserExt for Request<B> {
    fn auth_user(&self) -> Option<&AuthUser> {
        self.extensions().get::<AuthUser>()
    }
}

/// Extension trait to require authentication for a handler
pub trait RequireAuth {
    /// Require the request to be authenticated
    fn require_auth(self) -> Result<AuthUser, AuthencError>;
}

impl<B> RequireAuth for &Request<B> {
    fn require_auth(self) -> Result<AuthUser, AuthencError> {
        self.extensions()
            .get::<AuthUser>()
            .cloned() // Clone instead of unsafe lifetime extension
            .ok_or_else(|| AuthencError::unauthorized("Authentication required"))
    }
}

/// Check if the endpoint is public and doesn't require authentication
fn is_public_endpoint(path: &str) -> bool {
    // Add public endpoints here
    matches!(
        path,
        "/health" | "/health/" | "/health/ready" | "/health/live" | "/metrics"
    )
}

/// Apply authentication middleware to a Router directly.
///
/// This wraps the router with JWT authentication middleware.
/// The middleware validates JWT tokens from Authorization headers.
pub fn apply_auth_layer(router: axum::Router, jwt_validator: Arc<JwtValidator>) -> axum::Router {
    let state = Arc::new(AuthState::new(jwt_validator));
    router.layer(axum::middleware::from_fn_with_state(state, auth_middleware))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_public_endpoints_no_auth_required() {
        // Create a mock JWT validator
        let jwt_validator = Arc::new(JwtValidator::new_for_testing());
        let state = Arc::new(AuthState::new(jwt_validator));

        let app = Router::new()
            .route("/health", get(|| async { "OK" }))
            .route("/health/ready", get(|| async { "Ready" }))
            .route("/health/live", get(|| async { "Live" }))
            .route("/metrics", get(|| async { "Metrics" }))
            .layer(axum::middleware::from_fn_with_state(state, auth_middleware));

        // Test health endpoint
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test health/ready endpoint
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test health/live endpoint
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health/live")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Test metrics endpoint
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_protected_endpoints_require_auth() {
        let jwt_validator = Arc::new(JwtValidator::new_for_testing());
        let state = Arc::new(AuthState::new(jwt_validator));

        let app = Router::new()
            .route("/protected", get(|| async { "Protected content" }))
            .layer(axum::middleware::from_fn_with_state(state, auth_middleware));

        // Request without authorization header should fail
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Request with malformed authorization header should fail
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("authorization", "Invalid token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Request with non-Bearer authorization header should fail
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("authorization", "Basic dXNlcjpwYXNz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_state_creation() {
        let jwt_validator = Arc::new(JwtValidator::new_for_testing());
        let state = AuthState::new(jwt_validator.clone());

        assert!(Arc::ptr_eq(&state.jwt_validator, &jwt_validator));
    }

    #[tokio::test]
    async fn test_auth_user_struct() {
        let user = AuthUser {
            id: "user123".to_string(),
            email: "user@example.com".to_string(),
            roles: vec!["user".to_string(), "admin".to_string()],
        };

        assert_eq!(user.id, "user123");
        assert_eq!(user.email, "user@example.com");
        assert_eq!(user.roles.len(), 2);
        assert!(user.roles.contains(&"user".to_string()));
        assert!(user.roles.contains(&"admin".to_string()));
    }

    #[tokio::test]
    async fn test_is_public_endpoint() {
        assert!(is_public_endpoint("/health"));
        assert!(is_public_endpoint("/health/"));
        assert!(is_public_endpoint("/health/ready"));
        assert!(is_public_endpoint("/health/live"));
        assert!(is_public_endpoint("/metrics"));

        assert!(!is_public_endpoint("/api/users"));
        assert!(!is_public_endpoint("/auth/login"));
        assert!(!is_public_endpoint("/protected"));
        assert!(!is_public_endpoint("/"));
    }
}
