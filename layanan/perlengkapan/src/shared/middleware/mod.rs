//! # JWT Authentication Middleware
//!
//! JWT token validation and user authentication using Authenc Service

pub mod metrics;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{HeaderMap, header::AUTHORIZATION, request::Parts},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::error::AppError;
use crate::shared::grpc::clients::AuthencClient;

/// Axum extractor for the originating client IP, used by audit log /
/// workflow transition records.
///
/// Tries headers in order, falls back to `"unknown"` if none are present:
///   1. `X-Forwarded-For` (first IP in the comma-separated list — the
///      original client, before any proxies)
///   2. `X-Real-IP` (single IP set by nginx-style reverse proxies)
///
/// `ConnectInfo<SocketAddr>` is intentionally NOT consulted here — behind
/// a k8s ingress / nginx the connect-info is always the proxy IP, not the
/// real client, and trusting it would silently log the wrong value.
pub struct ClientIp(pub String);

impl ClientIp {
    pub fn from_headers(headers: &HeaderMap) -> Self {
        if let Some(xff) = headers.get("x-forwarded-for") {
            if let Ok(s) = xff.to_str() {
                if let Some(first) = s.split(',').next() {
                    let ip = first.trim();
                    if !ip.is_empty() {
                        return ClientIp(ip.to_string());
                    }
                }
            }
        }
        if let Some(real) = headers.get("x-real-ip") {
            if let Ok(s) = real.to_str() {
                let ip = s.trim();
                if !ip.is_empty() {
                    return ClientIp(ip.to_string());
                }
            }
        }
        ClientIp("unknown".to_string())
    }
}

impl<S> FromRequestParts<S> for ClientIp
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(ClientIp::from_headers(&parts.headers))
    }
}

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

    /// Assert bahwa role caller adalah salah satu dari `allowed`.
    /// Mengembalikan `AppError::Authorization` (mapped ke 403) jika tidak.
    ///
    /// Match dilakukan case-insensitive utk kompatibilitas dgn variasi
    /// penulisan role di JWT/client header (mis. "Validator_Pusat" vs
    /// "validator_pusat"). Admin/superadmin selalu lolos sbg escape hatch
    /// (lihat [`Claims::is_cross_satker_role`] untuk daftar role pusat).
    ///
    /// # Contoh
    /// ```ignore
    /// // Endpoint yg hanya boleh dipicu Validator Pusat:
    /// claims.require_any_role(&["validator_pusat"])?;
    /// ```
    pub fn require_any_role(&self, allowed: &[&str]) -> Result<(), AppError> {
        // Admin/superadmin: bypass (sudah cross-satker juga).
        if matches!(
            self.role.as_str(),
            "admin" | "admin_pusat" | "superadmin"
        ) {
            return Ok(());
        }
        let me = self.role.to_ascii_lowercase();
        if allowed
            .iter()
            .any(|r| r.to_ascii_lowercase() == me)
        {
            Ok(())
        } else {
            Err(AppError::Authorization(format!(
                "Akses ditolak: role '{}' tidak diizinkan utk aksi ini (perlu salah satu dari: {})",
                self.role,
                allowed.join(", ")
            )))
        }
    }

    /// Alias untuk satu role tunggal.
    pub fn require_role(&self, role: &str) -> Result<(), AppError> {
        self.require_any_role(&[role])
    }

    /// Assert caller adalah admin (atau superadmin). Dipakai utk
    /// endpoint master/referensi yg hanya boleh diubah admin.
    pub fn require_admin(&self) -> Result<(), AppError> {
        if matches!(self.role.as_str(), "admin" | "admin_pusat" | "superadmin") {
            Ok(())
        } else {
            Err(AppError::Authorization(format!(
                "Akses ditolak: aksi ini memerlukan role admin (role caller: '{}')",
                self.role
            )))
        }
    }
}

#[cfg(test)]
mod policy_tests {
    use super::*;

    fn claims_with_role(role: &str) -> Claims {
        Claims {
            user_id: Uuid::nil(),
            username: "u".into(),
            role: role.into(),
            permissions: vec![],
            nip: None,
            name: None,
            nama: None,
            jabatan: None,
            satker_code: None,
        }
    }

    #[test]
    fn require_role_accepts_exact_match() {
        let c = claims_with_role("validator_pusat");
        assert!(c.require_role("validator_pusat").is_ok());
        assert!(c.require_any_role(&["validator_pusat", "admin"]).is_ok());
    }

    #[test]
    fn require_role_rejects_wrong_role() {
        let c = claims_with_role("operator_satker");
        let err = c.require_role("validator_pusat").unwrap_err();
        assert!(matches!(err, AppError::Authorization(_)));
    }

    #[test]
    fn require_role_is_case_insensitive() {
        let c = claims_with_role("Validator_Pusat");
        assert!(c.require_role("validator_pusat").is_ok());
    }

    #[test]
    fn admin_bypasses_require_role() {
        let c = claims_with_role("admin");
        assert!(c.require_role("validator_pusat").is_ok());
        assert!(c.require_admin().is_ok());
    }

    #[test]
    fn superadmin_bypasses_require_role() {
        let c = claims_with_role("superadmin");
        assert!(c.require_role("operator_satker").is_ok());
    }

    #[test]
    fn require_admin_rejects_non_admin() {
        let c = claims_with_role("validator_pusat");
        assert!(c.require_admin().is_err());
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

                // TODO(authenc-claims): Authenc's `ValidateTokenResponse` only
                // carries user_id today; widen it to include username + role
                // + satker_id so this middleware can stop synthesising
                // placeholder values. Tracking dep: a separate PR against
                // layanan/authenc/proto/authenc.proto.
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
