//! MFA administration service for managing account lockouts and MFA settings

use crate::rate_limiter::MfaRateLimiterState;
use authenc_core::error::AuthencError;
use authenc_core::stores::UserStoreTrait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// MFA administration service for managing lockouts and settings
pub struct MfaAdminService {
    user_store: Arc<dyn UserStoreTrait>,
    mfa_rate_limiter: Arc<MfaRateLimiterState>,
    db_pool: deadpool_postgres::Pool,
}

/// Account lockout information for admin interface
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountLockoutInfo {
    /// ID of the locked user
    pub user_id: Uuid,
    /// Username of the locked user
    pub username: String,
    /// Time until the account lockout expires
    pub locked_until: DateTime<Utc>,
    /// Number of failed attempts that triggered the lockout
    pub failed_attempts: u32,
    /// Reason for the account lockout
    pub lockout_reason: String,
    /// Timestamp when the account was locked
    pub locked_at: DateTime<Utc>,
}

/// MFA admin operation result
#[derive(Debug, Serialize, Deserialize)]
pub struct MfaAdminResult {
    /// Whether the admin operation was successful
    pub success: bool,
    /// Message describing the result of the operation
    pub message: String,
    /// ID of the user affected by the operation (if applicable)
    pub user_id: Option<Uuid>,
}

/// Request to unlock an account
#[derive(Debug, Deserialize)]
pub struct UnlockAccountRequest {
    /// ID of the user account to unlock
    pub user_id: Uuid,
    /// Reason for unlocking the account (for audit logging)
    pub admin_reason: String,
}

/// Request to reset MFA for a user
#[derive(Debug, Deserialize)]
pub struct ResetMfaRequest {
    /// ID of the user whose MFA should be reset
    pub user_id: Uuid,
    /// Reason for resetting MFA (for audit logging)
    pub admin_reason: String,
    /// Whether to force immediate reactivation of MFA
    pub force_reactivation: bool,
}

impl MfaAdminService {
    /// Create a new MFA admin service
    pub fn new(
        user_store: Arc<dyn UserStoreTrait>,
        mfa_rate_limiter: Arc<MfaRateLimiterState>,
        db_pool: deadpool_postgres::Pool,
    ) -> Self {
        Self {
            user_store,
            mfa_rate_limiter,
            db_pool,
        }
    }

    /// Get all currently locked accounts
    pub async fn get_locked_accounts(&self) -> Result<Vec<AccountLockoutInfo>, AuthencError> {
        let mut locked_accounts = Vec::new();

        // Iterate through locked accounts in the rate limiter
        for entry in self.mfa_rate_limiter.get_locked_accounts().iter() {
            let user_id = entry.0;
            let lockout = &entry.1;

            // Get user information
            if let Ok(Some(user)) = self.user_store.get_user(user_id).await {
                let lockout_info = AccountLockoutInfo {
                    user_id,
                    username: user.username.clone(),
                    locked_until: self.instant_to_datetime(lockout.locked_until),
                    failed_attempts: lockout.failed_attempts,
                    lockout_reason: lockout.lockout_reason.clone(),
                    locked_at: self.instant_to_datetime(
                        lockout.locked_until
                            - std::time::Duration::from_secs(
                                self.mfa_rate_limiter
                                    .get_config()
                                    .account_lockout_duration_minutes
                                    as u64
                                    * 60,
                            ),
                    ),
                };
                locked_accounts.push(lockout_info);
            }
        }

        Ok(locked_accounts)
    }

    /// Unlock a specific account (admin override)
    pub async fn unlock_account(
        &self,
        request: UnlockAccountRequest,
        admin_user_id: Uuid,
    ) -> Result<MfaAdminResult, AuthencError> {
        // Verify the user exists
        let user = self
            .user_store
            .get_user(request.user_id)
            .await?
            .ok_or_else(|| AuthencError::not_found("User not found"))?;

        // Check if account is actually locked
        let was_locked = self
            .mfa_rate_limiter
            .is_account_locked(request.user_id)
            .is_some();

        if !was_locked {
            return Ok(MfaAdminResult {
                success: false,
                message: "Account is not currently locked".to_string(),
                user_id: Some(request.user_id),
            });
        }

        // Unlock the account
        let unlocked = self.mfa_rate_limiter.unlock_account(request.user_id);

        if unlocked {
            // Log the admin action
            self.log_admin_action(
                admin_user_id,
                request.user_id,
                "account_unlock",
                &request.admin_reason,
            )
            .await?;

            Ok(MfaAdminResult {
                success: true,
                message: format!("Account {} unlocked successfully", user.username),
                user_id: Some(request.user_id),
            })
        } else {
            Ok(MfaAdminResult {
                success: false,
                message: "Failed to unlock account".to_string(),
                user_id: Some(request.user_id),
            })
        }
    }

    /// Reset MFA configuration for a user (admin operation)
    pub async fn reset_mfa(
        &self,
        request: ResetMfaRequest,
        admin_user_id: Uuid,
    ) -> Result<MfaAdminResult, AuthencError> {
        // Verify the user exists
        let user = self
            .user_store
            .get_user(request.user_id)
            .await?
            .ok_or_else(|| AuthencError::not_found("User not found"))?;

        // Disable MFA in database
        let client = self
            .db_pool
            .get()
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        let update_query = if request.force_reactivation {
            // Force user to set up MFA again on next login
            "UPDATE users SET mfa_enabled = false, mfa_setup_at = NULL WHERE id = $1"
        } else {
            // Just disable MFA
            "UPDATE users SET mfa_enabled = false WHERE id = $1"
        };

        client
            .execute(update_query, &[&request.user_id])
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        // Also unlock the account if it's locked
        self.mfa_rate_limiter.unlock_account(request.user_id);

        // Log the admin action
        let action = if request.force_reactivation {
            "mfa_reset_with_reactivation"
        } else {
            "mfa_reset"
        };

        self.log_admin_action(
            admin_user_id,
            request.user_id,
            action,
            &request.admin_reason,
        )
        .await?;

        Ok(MfaAdminResult {
            success: true,
            message: format!("MFA reset for user {}", user.username),
            user_id: Some(request.user_id),
        })
    }

    /// Get account lockout status for a specific user
    pub async fn get_account_lockout_status(
        &self,
        user_id: Uuid,
    ) -> Result<Option<AccountLockoutInfo>, AuthencError> {
        if let Some(lockout) = self.mfa_rate_limiter.is_account_locked(user_id) {
            // Get user information
            let user = self
                .user_store
                .get_user(user_id)
                .await?
                .ok_or_else(|| AuthencError::not_found("User not found"))?;

            Ok(Some(AccountLockoutInfo {
                user_id,
                username: user.username,
                locked_until: self.instant_to_datetime(lockout.locked_until),
                failed_attempts: lockout.failed_attempts,
                lockout_reason: lockout.lockout_reason,
                locked_at: self.instant_to_datetime(
                    lockout.locked_until
                        - std::time::Duration::from_secs(
                            self.mfa_rate_limiter
                                .get_config()
                                .account_lockout_duration_minutes
                                as u64
                                * 60,
                        ),
                ),
            }))
        } else {
            Ok(None)
        }
    }

    /// Bulk unlock accounts (for emergency situations)
    pub async fn bulk_unlock_accounts(
        &self,
        user_ids: Vec<Uuid>,
        admin_user_id: Uuid,
        admin_reason: String,
    ) -> Result<Vec<MfaAdminResult>, AuthencError> {
        let mut results = Vec::new();

        for user_id in user_ids {
            let request = UnlockAccountRequest {
                user_id,
                admin_reason: admin_reason.clone(),
            };

            match self.unlock_account(request, admin_user_id).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(MfaAdminResult {
                        success: false,
                        message: format!("Failed to unlock {}: {}", user_id, e),
                        user_id: Some(user_id),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Check if automatic unlock should be performed
    pub async fn check_automatic_unlocks(&self) -> Result<u32, AuthencError> {
        let now = std::time::Instant::now();
        let mut unlocked_count = 0;

        // Get list of locked accounts that should be automatically unlocked
        let mut accounts_to_unlock = Vec::new();

        for entry in self.mfa_rate_limiter.get_locked_accounts().iter() {
            let user_id = entry.0;
            let lockout = &entry.1;

            if now >= lockout.locked_until {
                accounts_to_unlock.push(user_id);
            }
        }

        // Unlock expired accounts
        for user_id in accounts_to_unlock {
            if self.mfa_rate_limiter.unlock_account(user_id) {
                unlocked_count += 1;

                // Log automatic unlock
                self.log_admin_action(
                    Uuid::nil(), // System action
                    user_id,
                    "automatic_unlock",
                    "Account lockout period expired",
                )
                .await?;
            }
        }

        Ok(unlocked_count)
    }

    /// Log admin actions for audit purposes
    async fn log_admin_action(
        &self,
        admin_user_id: Uuid,
        target_user_id: Uuid,
        action: &str,
        reason: &str,
    ) -> Result<(), AuthencError> {
        let client = self
            .db_pool
            .get()
            .await
            .map_err(|e| AuthencError::database(e.to_string()))?;

        client.execute(
            "INSERT INTO mfa_admin_actions (admin_user_id, target_user_id, action, reason, created_at) VALUES ($1, $2, $3, $4, NOW())",
            &[&admin_user_id, &target_user_id, &action, &reason],
        ).await
        .map_err(|e| AuthencError::database(e.to_string()))?;

        Ok(())
    }

    /// Convert `std::time::Instant` to `chrono::DateTime<Utc>`
    fn instant_to_datetime(&self, instant: std::time::Instant) -> DateTime<Utc> {
        let now = std::time::Instant::now();
        let system_now = std::time::SystemTime::now();

        if instant > now {
            let duration = instant - now;
            DateTime::from(system_now + duration)
        } else {
            let duration = now - instant;
            DateTime::from(system_now - duration)
        }
    }
}

/// Background service for automatic account unlocking
pub struct AutoUnlockService {
    mfa_admin_service: Arc<MfaAdminService>,
}

impl AutoUnlockService {
    /// Create a new auto-unlock service
    pub fn new(mfa_admin_service: Arc<MfaAdminService>) -> Self {
        Self { mfa_admin_service }
    }

    /// Start the background auto-unlock task
    pub async fn start(&self) {
        let service = self.mfa_admin_service.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes

            loop {
                interval.tick().await;

                match service.check_automatic_unlocks().await {
                    Ok(count) => {
                        if count > 0 {
                            tracing::info!("Automatically unlocked {} accounts", count);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to check automatic unlocks: {}", e);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rate_limiter::MfaRateLimitConfig;

    #[tokio::test]
    async fn test_account_unlock() {
        // This would require setting up test database and user store
        // Implementation would depend on your test infrastructure
    }

    #[tokio::test]
    async fn test_automatic_unlock_timing() {
        let config = MfaRateLimitConfig {
            account_lockout_duration_minutes: 1, // 1 minute for testing
            ..Default::default()
        };

        let mfa_rate_limiter = MfaRateLimiterState::new(config);
        let user_id = Uuid::new_v4();

        // Lock account
        mfa_rate_limiter.lock_account(user_id, "Test lockout".to_string());

        // Verify account is locked
        assert!(mfa_rate_limiter.is_account_locked(user_id).is_some());

        // Wait for lockout to expire (in real test, you'd mock time)
        tokio::time::sleep(std::time::Duration::from_secs(61)).await;

        // Check if account should be unlocked
        // (This would need the full service setup to test properly)
    }
}
