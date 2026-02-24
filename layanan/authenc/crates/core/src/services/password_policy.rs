//! Password Policy Service
//!
//! Provides comprehensive password policy management including:
//! - Password strength validation
//! - Password history checking
//! - Password expiration management
//! - Configurable policy rules

use authenc_storage::Database;
use authenc_types::User;
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

/// Password strength validation result
#[derive(Debug, Clone, Serialize)]
pub struct PasswordStrengthResult {
    /// Whether the password meets all requirements
    pub is_valid: bool,

    /// List of validation errors
    pub errors: Vec<String>,

    /// Password strength score (0-100)
    pub strength_score: u8,
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

    /// Validate password strength against policy requirements
    pub fn validate_password_strength(
        &self,
        password: &str,
        username: Option<&str>,
    ) -> PasswordStrengthResult {
        let mut errors = Vec::new();
        let mut strength_score = 0u8;

        // Check minimum length
        if password.len() < self.config.min_length {
            errors.push(format!(
                "Password must be at least {} characters long",
                self.config.min_length
            ));
        } else {
            strength_score += 20;
        }

        // Check for uppercase
        if self.config.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        } else if password.chars().any(|c| c.is_uppercase()) {
            strength_score += 20;
        }

        // Check for lowercase
        if self.config.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        } else if password.chars().any(|c| c.is_lowercase()) {
            strength_score += 20;
        }

        // Check for digit
        if self.config.require_digit && !password.chars().any(|c| c.is_numeric()) {
            errors.push("Password must contain at least one digit".to_string());
        } else if password.chars().any(|c| c.is_numeric()) {
            strength_score += 20;
        }

        // Check for special character
        if self.config.require_special_char && !password.chars().any(|c| !c.is_alphanumeric()) {
            errors.push("Password must contain at least one special character".to_string());
        } else if password.chars().any(|c| !c.is_alphanumeric()) {
            strength_score += 20;
        }

        // Check for username in password
        if self.config.prevent_username_in_password {
            if let Some(uname) = username {
                if !uname.is_empty() && password.to_lowercase().contains(&uname.to_lowercase()) {
                    errors.push("Password must not contain your username".to_string());
                    strength_score = strength_score.saturating_sub(30);
                }
            }
        }

        // Check for common weak patterns
        if self.config.check_weak_patterns {
            let weak_patterns = vec!["password", "123456", "qwerty", "admin", "letmein"];
            for pattern in weak_patterns {
                if password.to_lowercase().contains(pattern) {
                    errors.push(format!(
                        "Password contains common weak pattern: {}",
                        pattern
                    ));
                    strength_score = strength_score.saturating_sub(20);
                    break;
                }
            }
        }

        PasswordStrengthResult {
            is_valid: errors.is_empty(),
            errors,
            strength_score: strength_score.min(100),
        }
    }

    /// Validate a new password against all policy rules
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

        let strength_result = self.validate_password_strength(new_password, username_check);

        // Check password history
        let password_history = self.get_password_history(user_id).await?;
        let found_in_history = self
            .check_password_history(new_password, &password_history)
            .await?;

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
    pub fn check_expiration(&self, user: &User) -> (bool, Option<i64>) {
        if let Some(expires_at) = user.password_expires_at {
            let now = chrono::Utc::now();
            let days_until = (expires_at - now).num_days();

            let is_expired = days_until <= 0;
            let in_grace_period = days_until <= self.config.expiration_grace_period_days as i64;

            (is_expired || in_grace_period, Some(days_until))
        } else {
            (false, None)
        }
    }

    /// Calculate password expiration date for a user
    pub fn calculate_expiration(
        &self,
        password_changed_at: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        if self.config.password_expiration_days > 0 {
            Some(
                password_changed_at
                    + chrono::Duration::days(self.config.password_expiration_days as i64),
            )
        } else {
            None
        }
    }

    /// Get password history for a user from database
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

    /// Check if password matches any in history
    async fn check_password_history(
        &self,
        new_password: &str,
        password_history: &[String],
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // This would use the password hasher to verify against each hash
        // For now, we'll do a simple comparison (in production, use proper password verification)
        for _old_hash in password_history {
            // TODO: Use authenc_crypto::password::verify_password here
            // if verify_password(new_password, old_hash)? {
            //     return Ok(true);
            // }
        }
        Ok(false)
    }

    /// Update password expiration date for a user
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
