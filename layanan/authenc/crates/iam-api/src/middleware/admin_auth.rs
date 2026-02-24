//! Admin authentication middleware

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::state::IamApiState;

/// Admin user extracted from JWT token
#[derive(Debug, Clone)]
pub struct AdminUser {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub roles: Vec<String>,
}

/// Admin authentication middleware
///
/// Verifies JWT token and checks for admin role
pub async fn admin_auth_middleware(
    State(state): State<Arc<IamApiState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (StatusCode::UNAUTHORIZED, "Missing Authorization header").into_response()
        })?;

    // Extract Bearer token
    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            "Invalid Authorization header format",
        )
            .into_response()
    })?;

    // Validate JWT token
    let claims = state
        .jwt_service
        .verify_token(token)
        .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)).into_response())?;

    // TODO: Extract roles from claims
    // For now, assume roles are in the "roles" claim
    let roles: Vec<String> = vec![]; // claims.roles or similar

    // Check for admin role
    if !roles.contains(&"admin".to_string()) {
        return Err((StatusCode::FORBIDDEN, "Admin role required").into_response());
    }

    // Create AdminUser and insert into request extensions
    let admin_user = AdminUser {
        user_id: uuid::Uuid::parse_str(&claims.sub).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Invalid user ID in token",
            )
                .into_response()
        })?,
        username: claims.sub.clone(), // TODO: Get actual username
        roles,
    };

    request.extensions_mut().insert(admin_user);

    Ok(next.run(request).await)
}

/// Permission-based authorization middleware
///
/// Checks if user has specific permission
pub async fn require_permission(
    permission: &'static str,
) -> impl Fn(
    Request,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Response>> + Send>>
+ Clone {
    move |request: Request, next: Next| {
        Box::pin(async move {
            // Extract AdminUser from extensions
            let admin_user = request.extensions().get::<AdminUser>().ok_or_else(|| {
                (StatusCode::UNAUTHORIZED, "Admin authentication required").into_response()
            })?;

            // TODO: Implement permission checking
            // For now, just check if user has admin role
            if !admin_user.roles.contains(&"admin".to_string()) {
                return Err((
                    StatusCode::FORBIDDEN,
                    format!("Permission '{}' required", permission),
                )
                    .into_response());
            }

            Ok(next.run(request).await)
        })
    }
}
