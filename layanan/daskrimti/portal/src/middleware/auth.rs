//! Authentication middleware
//!
//! Extracts and validates JWT tokens from requests, injecting user context

use axum::{
    Json,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::state::AppState;

/// User context extracted from validated token
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub username: String,
    pub roles: Vec<String>,
    pub token: String,
}

/// Error response for auth failures
#[derive(Debug, Serialize)]
pub struct AuthError {
    pub error: String,
    pub code: String,
}

impl AuthError {
    fn unauthorized(message: &str) -> (StatusCode, Json<Self>) {
        (
            StatusCode::UNAUTHORIZED,
            Json(AuthError {
                error: message.to_string(),
                code: "UNAUTHORIZED".to_string(),
            }),
        )
    }

    fn forbidden(message: &str) -> (StatusCode, Json<Self>) {
        (
            StatusCode::FORBIDDEN,
            Json(AuthError {
                error: message.to_string(),
                code: "FORBIDDEN".to_string(),
            }),
        )
    }
}

/// Extract Bearer token from Authorization header
fn extract_bearer_token(request: &Request<Body>) -> Option<String> {
    request
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            if value.starts_with("Bearer ") {
                Some(value[7..].to_string())
            } else {
                None
            }
        })
}

/// Authentication middleware - validates token and injects user context
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    info!(
        "Auth middleware seeing request: {} {}",
        request.method(),
        request.uri()
    );

    // Extract bearer token
    let token = match extract_bearer_token(&request) {
        Some(token) => token,
        None => {
            warn!("PORTAL_AUTH_FAIL: Missing or invalid Authorization header");
            return AuthError::unauthorized("Missing or invalid Authorization header")
                .into_response();
        }
    };

    // Validate token via Authenc
    match state.authenc.validate_token(&token).await {
        Ok(validation) => {
            if !validation.valid {
                warn!("Invalid token: {:?}", validation.error);
                return AuthError::unauthorized("Invalid or expired token").into_response();
            }

            // Extract user info
            let user_id = match validation.user_id {
                Some(id) => id,
                None => {
                    error!("Token valid but no user_id present");
                    return AuthError::unauthorized("Invalid token payload").into_response();
                }
            };

            // Create auth context
            let auth_context = AuthContext {
                user_id: user_id.clone(),
                username: user_id.clone(), // Would be extracted from full token claims
                roles: validation.scopes,
                token: token.clone(),
            };

            // Insert context into request extensions
            request.extensions_mut().insert(auth_context);
            request.extensions_mut().insert(user_id);
            request.extensions_mut().insert(token);

            // Continue to handler
            next.run(request).await
        }
        Err(err) => {
            error!("Token validation error: {}", err);
            AuthError::unauthorized("Token validation failed").into_response()
        }
    }
}

/// Permission checking middleware factory
/// Creates a middleware that checks if user has required permission
pub fn require_permission(
    resource: &'static str,
    action: &'static str,
) -> impl Fn(
    State<Arc<AppState>>,
    Request<Body>,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
+ Clone {
    move |State(state): State<Arc<AppState>>, request: Request<Body>, next: Next| {
        let resource = resource;
        let action = action;

        Box::pin(async move {
            // Get auth context from extensions (must be after auth_middleware)
            let auth_context = match request.extensions().get::<AuthContext>() {
                Some(ctx) => ctx.clone(),
                None => {
                    error!("AuthContext not found - auth_middleware not applied?");
                    return AuthError::unauthorized("Not authenticated").into_response();
                }
            };

            // Check permission via Authenc
            match state
                .authenc
                .check_permission(&auth_context.token, &auth_context.user_id, resource, action)
                .await
            {
                Ok(result) => {
                    if result.allowed {
                        info!(
                            "Permission granted: user={} resource={} action={}",
                            auth_context.user_id, resource, action
                        );
                        next.run(request).await
                    } else {
                        warn!(
                            "Permission denied: user={} resource={} action={} reason={:?}",
                            auth_context.user_id, resource, action, result.reason
                        );
                        AuthError::forbidden(&format!(
                            "Access denied: {}",
                            result
                                .reason
                                .unwrap_or_else(|| "Insufficient permissions".to_string())
                        ))
                        .into_response()
                    }
                }
                Err(err) => {
                    error!("Permission check error: {}", err);
                    AuthError::forbidden("Permission check failed").into_response()
                }
            }
        })
    }
}

/// Optional auth middleware - extracts user context if token present, but doesn't fail
pub async fn optional_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    // Try to extract bearer token
    if let Some(token) = extract_bearer_token(&request) {
        // Try to validate token
        if let Ok(validation) = state.authenc.validate_token(&token).await
            && validation.valid
            && let Some(user_id) = validation.user_id
        {
            let auth_context = AuthContext {
                user_id: user_id.clone(),
                username: user_id.clone(),
                roles: validation.scopes,
                token: token.clone(),
            };

            request.extensions_mut().insert(auth_context);
            request.extensions_mut().insert(user_id);
            request.extensions_mut().insert(token);
        }
    }

    // Continue regardless of auth status
    next.run(request).await
}
