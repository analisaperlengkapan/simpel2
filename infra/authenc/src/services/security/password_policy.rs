//! Password Policy Service
//!
//! Provides comprehensive password policy management including:
//! - Password strength validation
//! - Password history checking
//! - Password expiration management
//! - Configurable policy rules

use crate::database::Database;
use crate::models::user::User;
use crate::utils::crypto::password::{
    calculate_password_expiration, check_password_expiration, check_password_history,
    validate_password_strength,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicyConfig {
    /// Minimum password length (default: 8)
    pub min_length: usize,

    /// Require uppercase letter (default: true)
    pub require_uppercase: bool,

    /// Require lowercase letter (default: true)
    pub require_lowercase: bool,

    /// Require digit (default: true)
    pub require_digit: bool,

    /// Require special character (default: true)
    pub require_special_char: bool,

    /// Number of previous passwords to check (default: 5)
    pub password_history_count: usize,

    /// Password expiration in days (0 = never expires, default: 90)
    pub password_expiration_days: u32,

    /// Grace period before expiration warning in days (default: 7)
    pub expiration_grace_period_days: u32,

    /// Prevent username in password (default: true)
    pub prevent_username_in_password: bool,

    /// Check for common weak patterns (default: true)
    pub check_weak_patterns: bool,
}

impl Default for PasswordPolicyConfig {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special_char: true,
            password_history_count: 5,
            password_expiration_days: 90,
            expiration_grace_period_days: 7,
            prevent_username_in_password: true,
            check_weak_patterns: true,
        }
    }
}

/// Password policy validation result
#[derive(Debug, Clone, Serialize)]
pub struct PasswordPolicyValidationResult {
    /// Whether the password is valid
    pub is_valid: bool,

    /// List of validation errors
    pub errors: Vec<String>,

    /// Password strength score (0-100)
    pub strength_score: u8,

    /// Whether password was found in history
    pub found_in_history: bool,

    /// Whether password is expired or will expire soon
    pub is_expired: bool,

    /// Days until password expiration (if applicable)
    pub days_until_expiration: Option<i64>,
}

/// Password policy service
pub struct PasswordPolicyService {
    database: Arc<Database>,
    config: PasswordPolicyConfig,
}

impl PasswordPolicyService {
    /// Create a new password policy service
    pub fn new(database: Arc<Database>, config: PasswordPolicyConfig) -> Self {
        Self { database, config }
    }

    /// Create with default configuration
    pub fn with_defaults(database: Arc<Database>) -> Self {
        Self::new(database, PasswordPolicyConfig::default())
    }

    /// Validate a new password against all policy rules
    ///
    /// This performs comprehensive validation including:
    /// - Password strength requirements
    /// - Password history check
    /// - Username inclusion check
    ///
    /// # Arguments
    /// * `user_id` - ID of the user changing their password
    /// * `username` - Username to check for inclusion
    /// * `new_password` - The new password to validate
    ///
    /// # Returns
    /// A `PasswordPolicyValidationResult` with validation status and details
    pub async fn validate_new_password(
        &self,
        user_id: Uuid,
        username: &str,
        new_password: &str,
    ) -> Result<PasswordPolicyValidationResult, Box<dyn std::error::Error>> {
        // Validate password strength
        let username_check = if self.config.prevent_username_in_password {
            Some(username)
        } else {
            None
        };

        let strength_result = validate_password_strength(new_password, username_check);

        // Check password history
        let password_history = self.get_password_history(user_id).await?;
        let found_in_history = check_password_history(
            new_password,
            &password_history,
            self.config.password_history_count,
        );

        // Combine results
        let mut errors = strength_result.errors.clone();
        if found_in_history {
            errors.push(format!(
                "Password cannot be the same as any of your last {} passwords",
                self.config.password_history_count
            ));
        }

        let is_valid = errors.is_empty();

        Ok(PasswordPolicyValidationResult {
            is_valid,
            errors,
            strength_score: strength_result.strength_score,
            found_in_history,
            is_expired: false,
            days_until_expiration: None,
        })
    }

    /// Check if a user's password has expired
    ///
    /// # Arguments
    /// * `user` - The user to check
    ///
    /// # Returns
    /// Tuple of (is_expired, days_until_expiration)
    pub fn check_expiration(&self, user: &User) -> (bool, Option<i64>) {
        check_password_expiration(
            user.password_expires_at,
            self.config.expiration_grace_period_days,
        )
    }

    /// Calculate password expiration date for a user
    ///
    /// # Arguments
    /// * `password_changed_at` - When the password was changed
    ///
    /// # Returns
    /// Optional expiration date
    pub fn calculate_expiration(
        &self,
        password_changed_at: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        calculate_password_expiration(password_changed_at, self.config.password_expiration_days)
    }

    /// Get password history for a user from database
    ///
    /// # Arguments
    /// * `user_id` - ID of the user
    ///
    /// # Returns
    /// Vector of password hashes from history
    async fn get_password_history(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let client = self.database.get_connection().await?;

        let query = "
            SELECT password_hash
            FROM password_history
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
        ";

        let rows = client
            .query(
                query,
                &[&user_id, &(self.config.password_history_count as i32)],
            )
            .await?;

        let history: Vec<String> = rows
            .iter()
            .map(|row| row.get::<_, String>("password_hash"))
            .collect();

        Ok(history)
    }

    /// Update password expiration date for a user
    ///
    /// # Arguments
    /// * `user_id` - ID of the user
    /// * `password_changed_at` - When the password was changed
    ///
    /// # Returns
    /// The calculated expiration date
    pub async fn update_password_expiration(
        &self,
        user_id: Uuid,
        password_changed_at: DateTime<Utc>,
    ) -> Result<Option<DateTime<Utc>>, Box<dyn std::error::Error>> {
        let expires_at = self.calculate_expiration(password_changed_at);

        let client = self.database.get_connection().await?;

        let query = "
            UPDATE users
            SET password_expires_at = $1,
                password_changed_at = $2,
                require_password_change = FALSE
            WHERE id = $3
        ";

        client
            .execute(query, &[&expires_at, &password_changed_at, &user_id])
            .await?;

        Ok(expires_at)
    }

    /// Force password change on next login
    ///
    /// # Arguments
    /// * `user_id` - ID of the user
    pub async fn require_password_change(
        &self,
        user_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.database.get_connection().await?;

        let query = "
            UPDATE users
            SET require_password_change = TRUE
            WHERE id = $1
        ";

        client.execute(query, &[&user_id]).await?;

        Ok(())
    }

    /// Get users with expiring passwords
    ///
    /// # Arguments
    /// * `days_threshold` - Number of days before expiration to include
    ///
    /// # Returns
    /// List of user IDs with expiring passwords
    pub async fn get_users_with_expiring_passwords(
        &self,
        days_threshold: i32,
    ) -> Result<Vec<Uuid>, Box<dyn std::error::Error>> {
        let client = self.database.get_connection().await?;

        let query = "
            SELECT id
            FROM users
            WHERE password_expires_at IS NOT NULL
            AND password_expires_at <= NOW() + INTERVAL '1 day' * $1
            AND password_expires_at > NOW()
            AND enabled = TRUE
        ";

        let rows = client.query(query, &[&days_threshold]).await?;

        let user_ids: Vec<Uuid> = rows.iter().map(|row| row.get::<_, Uuid>("id")).collect();

        Ok(user_ids)
    }

    /// Get users with expired passwords
    ///
    /// # Returns
    /// List of user IDs with expired passwords
    pub async fn get_users_with_expired_passwords(
        &self,
    ) -> Result<Vec<Uuid>, Box<dyn std::error::Error>> {
        let client = self.database.get_connection().await?;

        let query = "
            SELECT id
            FROM users
            WHERE password_expires_at IS NOT NULL
            AND password_expires_at <= NOW()
            AND enabled = TRUE
        ";

        let rows = client.query(query, &[]).await?;

        let user_ids: Vec<Uuid> = rows.iter().map(|row| row.get::<_, Uuid>("id")).collect();

        Ok(user_ids)
    }

    /// Get password policy configuration
    pub fn get_config(&self) -> &PasswordPolicyConfig {
        &self.config
    }

    /// Update password policy configuration
    pub fn update_config(&mut self, config: PasswordPolicyConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PasswordPolicyConfig::default();
        assert_eq!(config.min_length, 8);
        assert_eq!(config.password_history_count, 5);
        assert_eq!(config.password_expiration_days, 90);
        assert!(config.require_uppercase);
        assert!(config.require_lowercase);
        assert!(config.require_digit);
        assert!(config.require_special_char);
    }

    #[test]
    fn test_custom_config() {
        let config = PasswordPolicyConfig {
            min_length: 12,
            password_history_count: 10,
            password_expiration_days: 60,
            ..Default::default()
        };

        assert_eq!(config.min_length, 12);
        assert_eq!(config.password_history_count, 10);
        assert_eq!(config.password_expiration_days, 60);
    }
}
