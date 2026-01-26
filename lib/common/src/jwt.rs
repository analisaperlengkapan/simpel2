use crate::error::CommonError;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

// Re-export Claims from jwt_claims module for backward compatibility
pub use crate::jwt_claims::{Claims, RealmAccess};

#[cfg(feature = "axum")]
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

impl Claims {
    /// Decode a JWT token into Claims
    /// This method requires the jsonwebtoken crate and is only available
    /// when the "jwt" feature is enabled.
    pub fn decode(token: &str, secret: &str) -> Result<Self, CommonError> {
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| CommonError::Internal(format!("JWT decode error: {}", e)))?;

        Ok(token_data.claims)
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

        Self::decode(token, &jwt_secret).map_err(|e| {
            (
                axum::http::StatusCode::UNAUTHORIZED,
                format!("Invalid token: {}", e),
            )
        })
    }
}
