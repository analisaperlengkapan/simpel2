//! # JWT Authentication Middleware
//!
//! JWT token validation and user authentication using Authenc Service

pub mod metrics;

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
    /// Satker code ("kdsatker_keu") extracted from the `satker:` scope.
    /// Used for satker-level authorization — handlers must reject writes to
    /// other satker's data unless the caller holds an admin/pusat role.
    pub satker_code: Option<String>,
}

impl Claims {
    /// Roles allowed to read/write data across satker boundaries. Anything
    /// else is treated as satker-scoped.
    pub fn is_cross_satker_role(&self) -> bool {
        matches!(
            self.role.as_str(),
            "admin" | "admin_pusat" | "superadmin" | "validator_pusat" | "pusat" | "analis_pusat"
        )
    }

    /// Returns true when the caller may access data belonging to the given
    /// satker code. Admin/pusat roles always pass; everyone else must match
    /// their own `satker_code`.
    pub fn can_access_satker(&self, target: &str) -> bool {
        if self.is_cross_satker_role() {
            return true;
        }
        match &self.satker_code {
            Some(code) => code == target,
            None => false,
        }
    }

    /// Assert that the caller may act on data scoped to `target_satker`.
    /// Returns `AppError::Authorization` when the satker boundary is violated.
    pub fn require_satker(&self, target_satker: &str) -> Result<(), AppError> {
        if self.can_access_satker(target_satker) {
            Ok(())
        } else {
            Err(AppError::Authorization(format!(
                "Akses ditolak: data milik satker lain ({})",
                target_satker
            )))
        }
    }
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

                // Prefer first-class `realm_roles` from the expanded
                // ValidateTokenResponse (commit 20); fall back to the
                // legacy `role:` scope convention when the issuer has
                // not been updated yet.
                let role = resp
                    .realm_roles
                    .first()
                    .cloned()
                    .or_else(|| {
                        resp.scopes
                            .iter()
                            .find(|s| s.starts_with("role:"))
                            .map(|s| s.trim_start_matches("role:").to_string())
                    })
                    .unwrap_or_else(|| "user".to_string());

                // First-class fields land as of commit 20; scope-prefix
                // fallbacks keep us compatible with tokens minted before
                // the authenc upgrade.
                let scope_lookup = |prefix: &str| -> Option<String> {
                    resp.scopes
                        .iter()
                        .find(|s| s.starts_with(prefix))
                        .map(|s| s.trim_start_matches(prefix).to_string())
                };
                let username = resp
                    .username
                    .clone()
                    .or_else(|| scope_lookup("username:"))
                    .unwrap_or_else(|| user_id.to_string());

                Ok(Claims {
                    user_id,
                    username,
                    role: role.clone(),
                    permissions: resp.scopes.clone(),
                    nip: resp.nip.clone().or_else(|| scope_lookup("nip:")),
                    name: resp.name.clone().or_else(|| scope_lookup("name:")),
                    nama: resp.name.clone().or_else(|| scope_lookup("nama:")),
                    jabatan: resp.jabatan.clone().or_else(|| scope_lookup("jabatan:")),
                    satker_code: resp.satker_code.clone().or_else(|| scope_lookup("satker:")),
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
