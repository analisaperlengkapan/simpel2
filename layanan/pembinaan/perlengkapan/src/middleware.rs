//! # JWT Authentication Middleware
//!
//! JWT token validation and user authentication

use axum::{
    extract::{FromRequestParts, Request},
    http::{header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::Response,
};
use async_trait::async_trait;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: Uuid,
    pub username: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub exp: usize,
    pub iat: usize,
}

#[async_trait]
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
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key".to_string());

        let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| AppError::Authentication(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims)
    }
}

pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // For now, we'll extract the claims but let the handlers deal with them
    // This middleware is just for global authentication
    let response = next.run(request).await;
    Ok(response)
}

// Optional middleware for role-based access control
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
pub fn require_permission(required_permission: &str) -> impl Fn(Claims) -> Result<(), AppError> + Clone {
    let permission = required_permission.to_string();
    move |claims: Claims| {
        if claims.permissions.contains(&permission) || claims.permissions.contains(&"*".to_string()) {
            Ok(())
        } else {
            Err(AppError::Authorization(format!(
                "Insufficient permissions. Required permission: {}",
                permission
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_jwt_claims_creation() {
        let claims = Claims {
            user_id: Uuid::new_v4(),
            username: "test_user".to_string(),
            role: "user".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            exp: (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600) as usize,
            iat: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize,
        };

        let jwt_secret = "test-secret";
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());
        let token = encode(&Header::default(), &claims, &encoding_key).unwrap();

        // Test decoding
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
        let validation = Validation::new(Algorithm::HS256);
        let decoded = decode::<Claims>(&token, &decoding_key, &validation).unwrap();

        assert_eq!(decoded.claims.username, claims.username);
        assert_eq!(decoded.claims.role, claims.role);
        assert_eq!(decoded.claims.permissions, claims.permissions);
    }
}
