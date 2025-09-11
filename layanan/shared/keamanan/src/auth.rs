use crate::error::AppError;
use crate::models::{JwtClaims, LoginRequest, LoginResponse, User, UserInfo};
use axum::Json;
use chrono::{Duration, Utc};
use deadpool_postgres::Pool;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::collections::HashMap;
use tracing::{error, info};

pub struct AuthService {
    pool: Pool,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(pool: Pool, jwt_secret: String) -> Self {
        Self { pool, jwt_secret }
    }

    pub async fn authenticate(&self, login_req: LoginRequest) -> Result<LoginResponse, AppError> {
        // Get user from database
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT id, username, password_hash, email, is_active, mfa_enabled, mfa_secret, created_at, updated_at
                 FROM keamanan.users WHERE username = $1 AND is_active = true",
                &[&login_req.username]
            )
            .await?;

        let user = if let Some(row) = rows.first() {
            User::from(row)
        } else {
            return Err(AppError::InvalidCredentials);
        };

        // Verify password
        if !self.verify_password(&login_req.password, &user.password_hash)? {
            return Err(AppError::InvalidCredentials);
        }

        // Verify MFA if enabled
        if user.mfa_enabled {
            if let Some(mfa_code) = login_req.mfa_code {
                if let Some(secret) = &user.mfa_secret {
                    if !self.verify_mfa(secret, &mfa_code)? {
                        return Err(AppError::InvalidMfaCode);
                    }
                } else {
                    return Err(AppError::MfaRequired);
                }
            } else {
                return Err(AppError::MfaRequired);
            }
        }

        // Get user roles and permissions
        let roles = self.get_user_roles(user.id).await?;
        let permissions = self.get_user_permissions(user.id).await?;

        // Generate tokens
        let access_token = self.generate_access_token(&user, &roles, &permissions)?;
        let refresh_token = self.generate_refresh_token(&user)?;

        // Create user info
        let user_info = UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
            roles,
            permissions,
        };

        Ok(LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600, // 1 hour
            user: user_info,
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<JwtClaims, AppError> {
        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    pub fn generate_access_token(
        &self,
        user: &User,
        roles: &[String],
        permissions: &[String],
    ) -> Result<String, AppError> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(1);

        let claims = JwtClaims {
            sub: user.id,
            username: user.username.clone(),
            roles: roles.to_vec(),
            permissions: permissions.to_vec(),
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        Ok(token)
    }

    pub fn generate_refresh_token(&self, user: &User) -> Result<String, AppError> {
        let now = Utc::now();
        let expires_at = now + Duration::days(30);

        let claims = JwtClaims {
            sub: user.id,
            username: user.username.clone(),
            roles: vec![],
            permissions: vec![],
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        Ok(token)
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};

        let parsed_hash = PasswordHash::new(hash)?;
        let argon2 = Argon2::default();

        Ok(PasswordVerifier::verify_password(&argon2, password.as_bytes(), &parsed_hash).is_ok())
    }

    fn verify_mfa(&self, secret: &str, code: &str) -> Result<bool, AppError> {
        use totp_lite::{totp_custom, Algorithm, Sha1};

        let totp = totp_custom!(secret.as_bytes(), 30, 6, Sha1::sha1());

        Ok(totp == code)
    }

    async fn get_user_roles(&self, user_id: uuid::Uuid) -> Result<Vec<String>, AppError> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT r.name FROM keamanan.roles r
                 JOIN keamanan.user_roles ur ON r.id = ur.role_id
                 WHERE ur.user_id = $1",
                &[&user_id],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| row.get::<_, String>("name"))
            .collect())
    }

    async fn get_user_permissions(&self, user_id: uuid::Uuid) -> Result<Vec<String>, AppError> {
        let client = self.pool.get().await?;
        let rows = client
            .query(
                "SELECT DISTINCT p.name FROM keamanan.permissions p
                 JOIN keamanan.role_permissions rp ON p.id = rp.permission_id
                 JOIN keamanan.user_roles ur ON rp.role_id = ur.role_id
                 WHERE ur.user_id = $1",
                &[&user_id],
            )
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| row.get::<_, String>("name"))
            .collect())
    }
}
