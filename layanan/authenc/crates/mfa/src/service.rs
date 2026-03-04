//! MFA Service Wrapper
//!
//! This service wraps the existing OtpCredentialProvider and integrates with secreton
//! for secure MFA secret storage and management.

use crate::cache::MfaCache;
use authenc_core::error::{AuthencError, Result};
use authenc_core::spi::credential::otp::OtpCredentialProvider;
use authenc_types::domain::mfa::{MfaSetupData, MfaStatusResponse};
use authenc_types::domain::user::User;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Trait for MFA operations that can be performed by a secret client
#[async_trait::async_trait]
pub trait MfaClient: Send + Sync {
    /// Set up MFA for a user by generating a secret and QR code
    async fn setup_mfa(
        &self,
        user_id: &str,
        issuer: &str,
        account_name: &str,
    ) -> Result<MfaSetupData>;

    /// Verify the MFA setup by validating the first TOTP code
    async fn verify_mfa_setup(&self, user_id: &str, code: &str) -> Result<()>;

    /// Verify a TOTP code for MFA authentication
    async fn verify_mfa(&self, user_id: &str, code: &str) -> Result<()>;

    /// Disable MFA for a user (requires admin context)
    async fn disable_mfa(
        &self,
        user_id: &str,
        admin_context: &authenc_types::domain::user::SecurityContext,
    ) -> Result<()>;

    /// Get the current MFA status for a user
    async fn get_mfa_status(&self, user_id: &str) -> Result<MfaStatusResponse>;

    /// Verify a recovery code for emergency access
    async fn verify_recovery_code(&self, user_id: &str, recovery_code: &str) -> Result<()>;

    /// Regenerate backup recovery codes for a user
    async fn regenerate_recovery_codes(&self, user_id: &str) -> Result<Vec<String>>;
}

/// MFA setup response containing QR code and backup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSetupResponse {
    /// QR code data URL for scanning with authenticator apps
    pub qr_code_url: String,
    /// Secret key for manual entry in authenticator apps
    pub secret_key: String,
    /// Backup codes for emergency access
    pub backup_codes: Vec<String>,
}

/// MFA status information from the service layer
/// Note: For comprehensive MFA status, use `crate::policy::MfaStatus`
pub type MfaStatus = crate::policy::MfaStatus;

/// MFA statistics for monitoring and reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaStatistics {
    /// Total number of users
    pub total_users: u64,
    /// Number of users with MFA enabled
    pub mfa_enabled_users: u64,
    /// Number of users who have completed MFA setup
    pub mfa_setup_complete: u64,
    /// Number of users who used MFA in the last 30 days
    pub mfa_active_30d: u64,
    /// Number of users who used MFA in the last 7 days
    pub mfa_active_7d: u64,
    /// Number of users who used MFA in the last 1 day
    pub mfa_active_1d: u64,
}

/// MFA Service that wraps existing OtpCredentialProvider and integrates with secreton
pub struct MfaService {
    /// Existing OTP credential provider for TOTP operations
    otp_provider: OtpCredentialProvider,
    /// Secreton client for encrypted secret storage
    secreton_client: Arc<dyn MfaClient>,
    /// Database connection pool
    db_pool: deadpool_postgres::Pool,
    /// MFA cache for performance optimization (optional)
    mfa_cache: Option<Arc<MfaCache>>,
}

impl MfaService {
    /// Create a new MFA service instance
    pub fn new(secreton_client: Arc<dyn MfaClient>, db_pool: deadpool_postgres::Pool) -> Self {
        Self {
            otp_provider: OtpCredentialProvider::new(),
            secreton_client,
            db_pool,
            mfa_cache: None,
        }
    }

    /// Create a new MFA service instance with caching
    pub fn with_cache(
        secreton_client: Arc<dyn MfaClient>,
        db_pool: deadpool_postgres::Pool,
        mfa_cache: Arc<MfaCache>,
    ) -> Self {
        Self {
            otp_provider: OtpCredentialProvider::new(),
            secreton_client,
            db_pool,
            mfa_cache: Some(mfa_cache),
        }
    }

    /// Setup MFA for a user - generates secret, QR code, and backup codes
    pub async fn setup_mfa(&self, user_id: Uuid) -> Result<MfaSetupResponse> {
        // Get user info for MFA setup
        let user = self.get_user(user_id).await?;
        let issuer = "SIMPEL Kejaksaan RI";
        let account_name = format!(
            "{}@kejaksaan.go.id",
            user.nip.unwrap_or_else(|| user.username.clone())
        );

        // Use secreton MfaManager for setup via API
        let setup_data = self
            .secreton_client
            .setup_mfa(&user_id.to_string(), issuer, &account_name)
            .await
            .map_err(|e| AuthencError::internal(format!("Secreton MFA setup failed: {}", e)))?;

        Ok(MfaSetupResponse {
            qr_code_url: setup_data.qr_code_url,
            secret_key: setup_data.secret_key,
            backup_codes: setup_data.backup_codes,
        })
    }

    /// Verify MFA setup by validating the initial OTP code
    pub async fn verify_setup(&self, user_id: Uuid, code: &str) -> Result<()> {
        // Use secreton MfaManager for verification
        self.secreton_client
            .verify_mfa_setup(&user_id.to_string(), code)
            .await?;

        // Mark MFA as enabled in database
        let client = self
            .db_pool
            .get()
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        client
            .execute(
                "UPDATE users SET mfa_enabled = true, mfa_setup_at = NOW() WHERE id = $1",
                &[&user_id],
            )
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        // Invalidate cached MFA status to ensure fresh data
        if let Some(cache) = &self.mfa_cache
            && let Err(e) = cache.invalidate_mfa_status(user_id).await {
                tracing::warn!("Failed to invalidate MFA status cache after setup: {}", e);
            }

        Ok(())
    }

    /// Verify MFA code during authentication
    pub async fn verify_mfa(&self, user_id: Uuid, code: &str) -> Result<()> {
        // Check for OTP replay attack if cache is available
        if let Some(cache) = &self.mfa_cache
            && cache.is_otp_recently_used(user_id, code).await? {
                tracing::warn!("OTP replay attack detected for user: {}", user_id);
                return Err(AuthencError::invalid_otp_code());
            }

        // Use secreton MfaManager for verification with replay protection
        self.secreton_client
            .verify_mfa(&user_id.to_string(), code)
            .await?;

        // Mark OTP as used in cache to prevent replay
        if let Some(cache) = &self.mfa_cache
            && let Err(e) = cache.mark_otp_as_used(user_id, code).await {
                tracing::warn!("Failed to mark OTP as used in cache: {}", e);
                // Don't fail the verification, just log the warning
            }

        // Update last used timestamp in database
        let client = self
            .db_pool
            .get()
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        client
            .execute(
                "UPDATE users SET mfa_last_used = NOW() WHERE id = $1",
                &[&user_id],
            )
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        // Invalidate cached MFA status to ensure fresh data on next request
        if let Some(cache) = &self.mfa_cache
            && let Err(e) = cache.invalidate_mfa_status(user_id).await {
                tracing::warn!("Failed to invalidate MFA status cache: {}", e);
            }

        Ok(())
    }

    /// Get MFA status for a user
    pub async fn get_mfa_status(&self, user_id: Uuid) -> Result<MfaStatus> {
        // Try to get from cache first
        if let Some(cache) = &self.mfa_cache
            && let Some(cached_status) = cache.get_mfa_status(user_id).await? {
                tracing::debug!("MFA status cache hit for user: {}", user_id);
                return Ok(cached_status);
            }

        // Cache miss or no cache - fetch from database using optimized function
        let status = self.get_mfa_status_from_db(user_id).await?;

        // Cache the result for future requests
        if let Some(cache) = &self.mfa_cache
            && let Err(e) = cache.cache_mfa_status(user_id, &status).await {
                tracing::warn!("Failed to cache MFA status: {}", e);
                // Don't fail the request, just log the warning
            }

        Ok(status)
    }

    /// Disable MFA for a user (admin operation)
    pub async fn disable_mfa(
        &self,
        user_id: Uuid,
        admin_context: &authenc_types::domain::user::SecurityContext,
    ) -> Result<()> {
        // Use secreton MfaManager to disable MFA
        self.secreton_client
            .disable_mfa(&user_id.to_string(), admin_context)
            .await
            .map_err(|e| AuthencError::internal(format!("Secreton MFA disable failed: {}", e)))?;

        // Update database
        let client = self
            .db_pool
            .get()
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        client.execute(
            "UPDATE users SET mfa_enabled = false, mfa_setup_at = NULL, mfa_last_used = NULL WHERE id = $1",
            &[&user_id],
        ).await
        .map_err(|e| AuthencError::database(e.to_string()))?;

        // Invalidate cached MFA status
        if let Some(cache) = &self.mfa_cache
            && let Err(e) = cache.invalidate_mfa_status(user_id).await {
                tracing::warn!("Failed to invalidate MFA status cache after disable: {}", e);
            }

        Ok(())
    }

    /// Verify recovery code for MFA bypass
    pub async fn verify_recovery_code(&self, user_id: Uuid, recovery_code: &str) -> Result<()> {
        // Use secreton MfaManager for recovery code verification
        self.secreton_client
            .verify_recovery_code(&user_id.to_string(), recovery_code)
            .await?;

        // Update last used timestamp and log recovery code usage
        let client = self
            .db_pool
            .get()
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        client
            .execute(
                "UPDATE users SET mfa_last_used = NOW() WHERE id = $1",
                &[&user_id],
            )
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        // Log recovery code usage for audit
        tracing::warn!(
            user_id = %user_id,
            event = "recovery_code_used",
            "User used MFA recovery code for authentication bypass"
        );

        Ok(())
    }

    /// Regenerate recovery codes for a user
    pub async fn regenerate_recovery_codes(&self, user_id: Uuid) -> Result<Vec<String>> {
        // Use secreton MfaManager to regenerate recovery codes
        let new_codes = self
            .secreton_client
            .regenerate_recovery_codes(&user_id.to_string())
            .await
            .map_err(|e| {
                AuthencError::internal(format!("Secreton recovery code regeneration failed: {}", e))
            })?;

        // Log recovery code regeneration for audit
        tracing::info!(
            user_id = %user_id,
            event = "recovery_codes_regenerated",
            codes_count = new_codes.len(),
            "User regenerated MFA recovery codes"
        );

        Ok(new_codes)
    }

    /// Get remaining recovery codes count for a user
    pub async fn get_recovery_codes_count(&self, user_id: Uuid) -> Result<usize> {
        // Use secreton MfaManager to get recovery codes status
        let _status = self
            .secreton_client
            .get_mfa_status(&user_id.to_string())
            .await
            .map_err(|e| AuthencError::internal(format!("Secreton MFA status failed: {}", e)))?;

        // Extract recovery codes count from status
        // Note: Current MfaStatusResponse doesn't include recovery codes count
        // Return 0 as default until this information is available
        Ok(0)
    }

    /// Check if user has any recovery codes remaining
    pub async fn has_recovery_codes(&self, user_id: Uuid) -> Result<bool> {
        let count = self.get_recovery_codes_count(user_id).await?;
        Ok(count > 0)
    }

    /// Batch get MFA status for multiple users (optimized for performance)
    pub async fn batch_get_mfa_status(&self, user_ids: &[Uuid]) -> Result<Vec<(Uuid, MfaStatus)>> {
        if user_ids.is_empty() {
            return Ok(vec![]);
        }

        // Try cache first if available
        if let Some(cache) = &self.mfa_cache {
            let cached_results = cache.batch_get_mfa_status(user_ids).await?;
            let mut results = Vec::new();
            let mut missing_ids = Vec::new();

            for (user_id, cached_status) in cached_results {
                match cached_status {
                    Some(status) => results.push((user_id, status)),
                    None => missing_ids.push(user_id),
                }
            }

            // If we have all results from cache, return them
            if missing_ids.is_empty() {
                return Ok(results);
            }

            // Fetch missing results from database
            let db_results = self.batch_get_mfa_status_from_db(&missing_ids).await?;

            // Cache the results we fetched from database
            let cache_items: Vec<(Uuid, MfaStatus)> = db_results.to_vec();
            if let Err(e) = cache.batch_cache_mfa_status(&cache_items).await {
                tracing::warn!("Failed to batch cache MFA status: {}", e);
            }

            results.extend(db_results);
            return Ok(results);
        }

        // No cache available, fetch all from database
        self.batch_get_mfa_status_from_db(user_ids).await
    }

    /// Batch get MFA status from database using optimized query
    async fn batch_get_mfa_status_from_db(
        &self,
        user_ids: &[Uuid],
    ) -> Result<Vec<(Uuid, MfaStatus)>> {
        let client = self
            .db_pool
            .get()
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        // Use the optimized batch function from migration
        let rows = client.query(
            "SELECT user_id, mfa_enabled, mfa_setup_at, mfa_last_used FROM get_batch_mfa_status($1)",
            &[&user_ids],
        ).await
        .map_err(|e| AuthencError::database(e.to_string()))?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            let user_id: Uuid = row.get(0);
            let enabled: bool = row.get(1);
            let _setup_at: Option<DateTime<Utc>> = row.get(2);
            let _last_used: Option<DateTime<Utc>> = row.get(3);

            // For batch operations, we'll use a default backup codes count
            // In a real implementation, this could be optimized further
            let backup_codes_remaining = if enabled { 10 } else { 0 };

            let status = MfaStatus {
                enabled,
                totp_configured: enabled,
                backup_codes_available: backup_codes_remaining > 0,
                webauthn_configured: false,
                remaining_backup_codes: backup_codes_remaining as usize,
            };

            results.push((user_id, status));
        }

        Ok(results)
    }

    /// Get MFA status using optimized database function
    async fn get_mfa_status_from_db(&self, user_id: Uuid) -> Result<MfaStatus> {
        let client = self
            .db_pool
            .get()
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        // Use the optimized function from migration
        let row = client
            .query_one(
                "SELECT mfa_enabled, mfa_setup_at, mfa_last_used FROM get_user_mfa_status($1)",
                &[&user_id],
            )
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        let enabled: bool = row.get(0);
        let _setup_at: Option<DateTime<Utc>> = row.get(1);
        let _last_used: Option<DateTime<Utc>> = row.get(2);

        // Get backup codes count from secreton if enabled
        let backup_codes_remaining = if enabled {
            match self
                .secreton_client
                .get_mfa_status(&user_id.to_string())
                .await
            {
                Ok(_status) => 10, // Default count
                Err(_) => 0,
            }
        } else {
            0
        };

        Ok(MfaStatus {
            enabled,
            totp_configured: enabled,
            backup_codes_available: backup_codes_remaining > 0,
            webauthn_configured: false,
            remaining_backup_codes: backup_codes_remaining as usize,
        })
    }

    /// Log MFA admin action using optimized database function
    pub async fn log_admin_action(
        &self,
        admin_user_id: Option<Uuid>,
        target_user_id: Uuid,
        action: &str,
        reason: &str,
    ) -> Result<Uuid> {
        let client = self
            .db_pool
            .get()
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        let action_id: Uuid = client
            .query_one(
                "SELECT log_mfa_admin_action($1, $2, $3, $4)",
                &[&admin_user_id, &target_user_id, &action, &reason],
            )
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?
            .get(0);

        tracing::info!(
            admin_user_id = ?admin_user_id,
            target_user_id = %target_user_id,
            action = %action,
            action_id = %action_id,
            "MFA admin action logged"
        );

        Ok(action_id)
    }

    /// Get MFA statistics for monitoring and reporting
    pub async fn get_mfa_statistics(&self, satker_code: Option<&str>) -> Result<MfaStatistics> {
        let client = self
            .db_pool
            .get()
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        let query = if let Some(_satker) = satker_code {
            "SELECT total_users, mfa_enabled_users, mfa_setup_complete, mfa_active_30d, mfa_active_7d, mfa_active_1d FROM mfa_statistics WHERE satker_code = $1"
        } else {
            "SELECT SUM(total_users), SUM(mfa_enabled_users), SUM(mfa_setup_complete), SUM(mfa_active_30d), SUM(mfa_active_7d), SUM(mfa_active_1d) FROM mfa_statistics"
        };

        let row = if let Some(satker) = satker_code {
            client.query_one(query, &[&satker]).await
        } else {
            client.query_one(query, &[]).await
        }
        .map_err(|e| AuthencError::database(e.to_string()))?;

        Ok(MfaStatistics {
            total_users: row.get::<_, Option<i64>>(0).unwrap_or(0) as u64,
            mfa_enabled_users: row.get::<_, Option<i64>>(1).unwrap_or(0) as u64,
            mfa_setup_complete: row.get::<_, Option<i64>>(2).unwrap_or(0) as u64,
            mfa_active_30d: row.get::<_, Option<i64>>(3).unwrap_or(0) as u64,
            mfa_active_7d: row.get::<_, Option<i64>>(4).unwrap_or(0) as u64,
            mfa_active_1d: row.get::<_, Option<i64>>(5).unwrap_or(0) as u64,
        })
    }

    /// Refresh MFA statistics materialized view
    pub async fn refresh_mfa_statistics(&self) -> Result<()> {
        let client = self
            .db_pool
            .get()
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        client
            .execute("SELECT refresh_mfa_statistics()", &[])
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        tracing::info!("MFA statistics materialized view refreshed");
        Ok(())
    }

    /// Get user information from database
    async fn get_user(&self, user_id: Uuid) -> Result<User> {
        let client = self
            .db_pool
            .get()
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        let row = client.query_one(
            "SELECT id, username, email, nip, nama, satker_code, mfa_enabled, mfa_setup_at FROM users WHERE id = $1",
            &[&user_id],
        ).await
        .map_err(|e| AuthencError::database(e.to_string()))?;

        Ok(User {
            id: row.get(0),
            username: row.get(1),
            email: row.get(2),
            email_verified: true, // Default for government employees
            first_name: None,
            last_name: None,
            nip: row.get(3),
            nama: row.get(4),
            jabatan: None,
            satker_code: row.get(5),
            phone_number: None,
            phone_verified: false,
            password_hash: None,
            totp_secret: None,
            totp_backup_codes: None,
            mfa_enabled: row.get(6),
            mfa_setup_at: row.get(7),
            mfa_last_used: None,
            webauthn_enabled: false,
            account_locked: false,
            account_locked_until: None,
            failed_login_attempts: 0,
            last_login_at: None,
            last_failed_login_at: None,
            password_changed_at: None,
            password_expires_at: None,
            require_password_change: false,
            realm_id: None,
            organization_id: None,
            roles: Vec::new(),
            permissions: Vec::new(),
            session_data: None,
            security_context: authenc_types::domain::user::SecurityContext {
                ip_address: None,
                user_agent: None,
                session_id: None,
                timestamp: Utc::now(),
                risk_score: None,
                metadata: None,
            },
            attributes: None,
            enabled: true,
            federated: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
            login_count: 0,
        })
    }
}
