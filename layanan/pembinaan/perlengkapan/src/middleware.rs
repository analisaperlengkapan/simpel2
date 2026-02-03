//! # JWT Authentication Middleware
//!
//! JWT token validation and user authentication using Authenc Service

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{errors::AppError, grpc_clients::AuthencClient};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: Uuid,
    pub username: String,
    pub role: String,
    pub permissions: Vec<String>,
    // Extended fields for Pakaian Dinas & Kebutuhan BMN modules
    pub nip: Option<String>,
    pub name: Option<String>,
    pub nama: Option<String>,
    pub jabatan: Option<String>,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
    AuthencClient: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let authenc = AuthencClient::from_ref(state);

        // Extract the token from the Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or_else(|| AppError::Authentication("Missing Authorization header".to_string()))?;

        let auth_str = auth_header
            .to_str()
            .map_err(|_| AppError::Authentication("Invalid Authorization header".to_string()))?;

        if !auth_str.starts_with("Bearer ") {
            return Err(AppError::Authentication(
                "Authorization header must start with 'Bearer '".to_string(),
            ));
        }

        let token = &auth_str[7..];

        // Validate token via Authenc gRPC
        match authenc.validate_token(token).await {
            Ok(resp) => {
                if !resp.valid {
                    return Err(AppError::Authentication(
                        resp.error.unwrap_or_else(|| "Invalid token".to_string()),
                    ));
                }

                // Construct claims from response
                // Authenc response: user_id (string), scopes (vec<string>)
                // We map scopes to permissions/roles roughly here
                let user_id = resp
                    .user_id
                    .ok_or_else(|| AppError::Authentication("Token missing user_id".to_string()))?
                    .parse::<Uuid>()
                    .map_err(|_| AppError::Authentication("Invalid user_id format".to_string()))?;

                // TODO: Enhance Authenc ValidateTokenResponse to return more user info (username, role)
                // For now, we stub or infer based on scopes if available, or fetch user info
                // In production, ValidateToken should return richer context or we call GetUser.
                // Assuming "scopes" contains role info for now.

                let role = resp
                    .scopes
                    .iter()
                    .find(|s| s.starts_with("role:"))
                    .map(|s| s.trim_start_matches("role:").to_string())
                    .unwrap_or_else(|| "user".to_string());

                Ok(Claims {
                    user_id,
                    username: "unknown".to_string(), // Missing from ValidateTokenResponse currently
                    role: role.clone(),
                    permissions: resp.scopes.clone(),
                    // Extended fields - try to extract from scopes or use defaults
                    nip: resp.scopes.iter()
                        .find(|s| s.starts_with("nip:"))
                        .map(|s| s.trim_start_matches("nip:").to_string()),
                    name: resp.scopes.iter()
                        .find(|s| s.starts_with("name:"))
                        .map(|s| s.trim_start_matches("name:").to_string()),
                    nama: resp.scopes.iter()
                        .find(|s| s.starts_with("nama:"))
                        .map(|s| s.trim_start_matches("nama:").to_string()),
                    jabatan: resp.scopes.iter()
                        .find(|s| s.starts_with("jabatan:"))
                        .map(|s| s.trim_start_matches("jabatan:").to_string()),
                })
            }
            Err(e) => {
                tracing::error!("Authenc validation failed: {}", e);
                Err(AppError::Internal(
                    "Authentication service unavailable".to_string(),
                ))
            }
        }
    }
}

// Optional middleware for role-based access control
#[allow(dead_code)]
pub fn require_role(required_role: &str) -> impl Fn(Claims) -> Result<(), AppError> + Clone {
    let role = required_role.to_string();
    move |claims: Claims| {
        if claims.role == role || claims.role == "admin" {
            Ok(())
        } else {
            Err(AppError::Authorization(format!(
                "Insufficient permissions. Required role: {}",
                role
            )))
        }
    }
}

// Permission-based access control
#[allow(dead_code)]
pub fn require_permission(
    required_permission: &str,
) -> impl Fn(Claims) -> Result<(), AppError> + Clone {
    let permission = required_permission.to_string();
    move |claims: Claims| {
        if claims.permissions.contains(&permission) || claims.permissions.contains(&"*".to_string())
        {
            Ok(())
        } else {
            Err(AppError::Authorization(format!(
                "Insufficient permissions. Required permission: {}",
                permission
            )))
        }
    }
}
