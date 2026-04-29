//! JWT decoding and Axum extractors

use crate::CommonError;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

// Re-export Claims from lib-core for backward compatibility
pub use lib_core::jwt_claims::{Claims, RealmAccess};

/// Decode a JWT token into Claims
pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, CommonError> {
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    let validation = Validation::new(Algorithm::HS256);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| CommonError::Internal(format!("JWT decode error: {}", e)))?;

    Ok(token_data.claims)
}

/// Newtype wrapper for Claims that can be used as Axum extractor.
/// Use this in handler signatures: `AuthClaims(claims): AuthClaims`
#[cfg(feature = "axum")]
#[derive(Debug, Clone)]
pub struct AuthClaims(pub Claims);

#[cfg(feature = "axum")]
impl std::ops::Deref for AuthClaims {
    type Target = Claims;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "axum")]
impl<S> axum::extract::FromRequestParts<S> for AuthClaims
where
    S: Send + Sync,
{
    type Rejection = (axum::http::StatusCode, String);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get(axum::http::header::AUTHORIZATION);

        let auth_str = auth_header.and_then(|h| h.to_str().ok()).ok_or_else(|| {
            (
                axum::http::StatusCode::UNAUTHORIZED,
                "Missing authorization header".to_string(),
            )
        })?;

        if !auth_str.starts_with("Bearer ") {
            return Err((
                axum::http::StatusCode::UNAUTHORIZED,
                "Invalid authorization format".to_string(),
            ));
        }

        let token = &auth_str[7..];
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

        decode_jwt(token, &jwt_secret).map(AuthClaims).map_err(|e| {
            (
                axum::http::StatusCode::UNAUTHORIZED,
                format!("Invalid token: {}", e),
            )
        })
    }
}
