use axum::extract::State;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::vault::VaultClient;
use crate::config::Config;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub vault_client: VaultClient,
    pub config: Config,
}

impl AppState {
    pub async fn insert_audit_log(
        &self,
        user_id: Option<Uuid>,
        action: &str,
        resource: &str,
        resource_id: Option<&str>,
        details: &Value,
        ip_address: &str,
        user_agent: &str,
    ) -> Result<(), sqlx::Error> {
        let timestamp = Utc::now();
        let id = Uuid::new_v4();
        let hash = format!("{}:{}:{}:{}:{}:{}:{}:{}", id, user_id.map(|u| u.to_string()).unwrap_or_default(), action, resource, resource_id.unwrap_or(""), ip_address, user_agent, timestamp);
        sqlx::query!(
            r#"INSERT INTO keamanan.audit_logs (id, user_id, action, resource, resource_id, details, ip_address, user_agent, timestamp, hash)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
            id,
            user_id,
            action,
            resource,
            resource_id,
            details,
            ip_address,
            user_agent,
            timestamp,
            hash
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn is_refresh_token_revoked(&self, refresh_token_hash: &str) -> Result<bool, sqlx::Error> {
        let rec = sqlx::query!(
            "SELECT is_revoked FROM keamanan.sessions WHERE refresh_token_hash = $1",
            refresh_token_hash
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec.map(|r| r.is_revoked).unwrap_or(true))
    }
    pub async fn revoke_refresh_token(&self, refresh_token_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE keamanan.sessions SET is_revoked = TRUE WHERE refresh_token_hash = $1",
            refresh_token_hash
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn save_new_refresh_token(&self, user_id: Uuid, refresh_token_hash: &str, expires_at: DateTime<Utc>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO keamanan.sessions (user_id, refresh_token_hash, expires_at) VALUES ($1, $2, $3)",
            user_id,
            refresh_token_hash,
            expires_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn revoke_all_sessions_for_user(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE keamanan.sessions SET is_revoked = TRUE WHERE user_id = $1",
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub nip: Option<String>,
    pub nik: Option<String>,
    pub password_hash: String,
    pub mfa_secret: Option<String>,
    pub mfa_enabled: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub name: String,
    pub resource: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub details: serde_json::Value,
    pub ip_address: String,
    pub user_agent: String,
    pub timestamp: DateTime<Utc>,
    pub hash: String, // Immutable hash for integrity
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub mfa_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub nip: Option<String>,
    pub nik: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaSetupRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaSetupResponse {
    pub qr_code: String,
    pub secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MfaVerifyRequest {
    pub user_id: Uuid,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: Uuid, // user_id
    pub username: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogQuery {
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
} 