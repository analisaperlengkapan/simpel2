use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use crate::error::CommonError;

#[cfg(feature = "axum")]
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

/// Standard JWT Claims shared across services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (usually User ID)
    pub sub: String,

    /// Preferred username
    #[serde(default)]
    pub preferred_username: Option<String>,

    /// Full name
    #[serde(default)]
    pub name: Option<String>,

    /// Email address
    #[serde(default)]
    pub email: Option<String>,

    /// Realm access (roles)
    #[serde(default)]
    pub realm_access: Option<RealmAccess>,

    /// Resource access (client specific roles)
    #[serde(default)]
    pub resource_access: Option<serde_json::Value>,

    /// MFA enabled status
    #[serde(default)]
    pub mfa_enabled: bool,

    /// MFA setup required
    #[serde(default)]
    pub mfa_setup_required: bool,

    /// Token expiration
    pub exp: usize,

    /// Issued at
    pub iat: usize,

    /// Issuer
    pub iss: String,
}

/// Keycloak-style Realm Access
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealmAccess {
    /// List of roles
    pub roles: Vec<String>,
}

impl Claims {
    /// Decode a JWT token into Claims
    pub fn decode(token: &str, secret: &str) -> Result<Self, CommonError> {
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| CommonError::Internal(format!("JWT decode error: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        if let Some(ra) = &self.realm_access {
            return ra.roles.iter().any(|r| r == role);
        }
        false
    }

    /// Get username (from preferred_username or sub)
    pub fn username(&self) -> String {
        self.preferred_username
            .clone()
            .unwrap_or_else(|| self.sub.clone())
    }

    /// Get primary role (Admin > Supervisor > User)
    pub fn get_primary_role(&self) -> crate::auth::UserRole {
        if self.has_role("admin") {
            crate::auth::UserRole::Admin
        } else if self.has_role("supervisor") {
            crate::auth::UserRole::Supervisor
        } else if self.has_role("guest") {
            crate::auth::UserRole::Guest
        } else {
            crate::auth::UserRole::User
        }
    }
}

#[cfg(feature = "axum")]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (axum::http::StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get(AUTHORIZATION);

        let auth_str = auth_header
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| (axum::http::StatusCode::UNAUTHORIZED, "Missing authorization header".to_string()))?;

        if !auth_str.starts_with("Bearer ") {
            return Err((axum::http::StatusCode::UNAUTHORIZED, "Invalid authorization format".to_string()));
        }

        let token = &auth_str[7..];
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

        Self::decode(token, &jwt_secret).map_err(|e| (axum::http::StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)))
    }
}
