use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid, // Standard Subject claim
    pub username: String,
    pub role: String,
    pub permissions: Option<Vec<String>>,
    pub exp: usize,
    pub iat: usize,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
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

        // Get JWT secret from environment or config
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());

        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| AppError::Authentication(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims)
    }
}
