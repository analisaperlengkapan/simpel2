//! Authentication primitives for the Secreton API.
//!
//! Bearer tokens are verified via [`OidcVerifier`]/[`MultiVerifier`] (RS256/ES256
//! with JWKS from Authenc); authorization is enforced through dynamic capabilities
//! (`DynamicRoleStore`), not a static permission enum.

pub mod oidc_verifier;

use axum::http::HeaderValue;
use jsonwebtoken::TokenData;
use serde::{Deserialize, Serialize};

pub use oidc_verifier::{MultiVerifier, OidcVerifier, OidcVerifierConfig};

/// Token verification trait for abstracting different JWT verification strategies
///
/// Secreton verifies bearer tokens via [`OidcVerifier`] (RS256/ES256 with JWKS
/// from Authenc), composed through [`MultiVerifier`].
#[async_trait::async_trait]
pub trait TokenVerifier: Send + Sync {
    /// Verify and decode a JWT token
    async fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, AuthError>;

    /// Get the issuer this verifier handles
    fn issuer(&self) -> &str;

    /// Get the audience this verifier handles
    fn audience(&self) -> &str;
}

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,              // Subject (user ID)
    pub name: String,             // User name
    pub email: String,            // User email
    pub roles: Vec<String>,       // User roles
    pub permissions: Vec<String>, // Specific permissions
    pub exp: usize,               // Expiration time
    pub iat: usize,               // Issued at
    pub iss: String,              // Issuer
    pub aud: String,              // Audience
    pub jti: String,              // JWT ID
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>, // Additional metadata
}

/// Extract bearer token from Authorization header
pub fn extract_bearer_token(auth_header: &HeaderValue) -> Option<String> {
    let auth_str = auth_header.to_str().ok()?;
    auth_str
        .strip_prefix("Bearer ")
        .map(|stripped| stripped.to_string())
}

// Use consolidated AuthError from error module
pub use crate::error::AuthError;

// Use canonical types from secreton_core::models
// LoginRequest, LoginResponse, UserInfo are now imported at the top

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token() {
        let header = HeaderValue::from_str("Bearer secret-token").expect("valid header");
        let token = extract_bearer_token(&header).expect("token expected");
        assert_eq!(token, "secret-token");
    }

    #[test]
    fn test_extract_bearer_token_invalid_format() {
        let header = HeaderValue::from_str("Basic abc123").expect("valid header");
        assert!(extract_bearer_token(&header).is_none());

        let header = HeaderValue::from_str("Bearer").expect("valid header");
        assert!(extract_bearer_token(&header).is_none());
    }
}
