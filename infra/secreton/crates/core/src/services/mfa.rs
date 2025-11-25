//! MFA Service
//!
//! Multi-Factor Authentication service for user authentication flow.
//! Delegates TOTP operations to TotpEngine for proper RFC 6238 compliance.
//!
//! # Separation of Concerns
//!
//! - **MfaService**: User MFA configuration, recovery codes, authentication flow
//! - **TotpEngine**: TOTP key storage, code generation/validation (RFC 6238 compliant)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::instrument;

use crate::services::secrets::totp::{TotpEngine, TotpKeyCreateRequest, TotpValidationRequest};

/// Error types for MFA
#[derive(Debug, thiserror::Error)]
pub enum MfaError {
    #[error("Invalid TOTP code")]
    InvalidTotp,

    #[error("TOTP code expired")]
    TotpExpired,

    #[error("TOTP code already used")]
    TotpReused,

    #[error("MFA not configured: {0}")]
    NotConfigured(String),

    #[error("MFA already configured: {0}")]
    AlreadyConfigured(String),

    #[error("Invalid secret")]
    InvalidSecret,

    #[error("Recovery code not found")]
    RecoveryCodeNotFound,

    #[error("Recovery code already used")]
    RecoveryCodeUsed,
}

/// MFA method type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MfaMethodType {
    /// TOTP (Google Authenticator, Authy, etc.)
    TOTP,

    /// Push notification
    Push,

    /// SMS
    SMS,

    /// Email
    Email,
}

/// TOTP setup response (returned from TotpEngine)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSetupResponse {
    /// Secret key (base32 encoded)
    pub secret: String,

    /// QR code URL for authenticator apps
    pub qr_code_url: String,

    /// Issuer name
    pub issuer: String,

    /// Account name
    pub account_name: String,

    /// Recovery codes
    pub recovery_codes: Vec<String>,
}

/// MFA configuration for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaConfig {
    /// User ID
    pub user_id: String,

    /// Enabled methods
    pub enabled_methods: Vec<MfaMethodType>,

    /// TOTP key name (references TotpEngine key)
    pub totp_key_name: Option<String>,

    /// Recovery codes
    pub recovery_codes: Vec<String>,

    /// Used recovery codes
    pub used_recovery_codes: Vec<String>,

    /// Created at
    pub created_at: DateTime<Utc>,

    /// Last used at
    pub last_used_at: Option<DateTime<Utc>>,

    /// Is MFA enforced
    pub enforced: bool,
}

impl MfaConfig {
    /// Create new MFA configuration
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            enabled_methods: Vec::new(),
            totp_key_name: None,
            recovery_codes: Self::generate_recovery_codes(),
            used_recovery_codes: Vec::new(),
            created_at: Utc::now(),
            last_used_at: None,
            enforced: false,
        }
    }

    /// Generate recovery codes
    fn generate_recovery_codes() -> Vec<String> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        (0..10)
            .map(|_| {
                format!(
                    "{:04}-{:04}-{:04}",
                    rng.gen_range(0..10000),
                    rng.gen_range(0..10000),
                    rng.gen_range(0..10000)
                )
            })
            .collect()
    }
}

/// MFA service with TotpEngine integration
pub struct MfaService {
    configs: Arc<RwLock<HashMap<String, MfaConfig>>>,
    totp_engine: Arc<TotpEngine>,
}

impl MfaService {
    /// Create new MFA service
    pub fn new() -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            totp_engine: Arc::new(TotpEngine::new()),
        }
    }

    /// Create MFA service with existing TotpEngine
    pub fn with_totp_engine(totp_engine: Arc<TotpEngine>) -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            totp_engine,
        }
    }

    /// Enable TOTP for user (delegates to TotpEngine)
    #[instrument(skip(self), fields(
        user_id = %user_id,
        issuer = %issuer,
        account_name = %account_name,
        operation = "enable_totp"
    ))]
    pub async fn enable_totp(
        &self,
        user_id: &str,
        issuer: String,
        account_name: String,
    ) -> Result<TotpSetupResponse, MfaError> {
        let mut configs = self.configs.write().await;

        let config = configs
            .entry(user_id.to_string())
            .or_insert_with(|| MfaConfig::new(user_id.to_string()));

        if config.totp_key_name.is_some() {
            return Err(MfaError::AlreadyConfigured("TOTP".to_string()));
        }

        // Create TOTP key in TotpEngine
        let key_name = format!("user:{}", user_id);
        let request = TotpKeyCreateRequest {
            name: key_name.clone(),
            issuer: issuer.clone(),
            account_name: account_name.clone(),
            algorithm: None, // Use default SHA1
            digits: None,    // Use default 6
            period: None,    // Use default 30
            skew: None,      // Use default 1
        };

        let totp_response = self
            .totp_engine
            .create_key(request)
            .await
            .map_err(|e| MfaError::InvalidSecret)?;

        // Update MFA config
        config.totp_key_name = Some(key_name);
        config.enabled_methods.push(MfaMethodType::TOTP);

        Ok(TotpSetupResponse {
            secret: totp_response.secret,
            qr_code_url: totp_response.qr_code_url,
            issuer: totp_response.issuer,
            account_name: totp_response.account_name,
            recovery_codes: config.recovery_codes.clone(),
        })
    }

    /// Verify TOTP code (delegates to TotpEngine)
    #[instrument(skip(self, code), fields(
        user_id = %user_id,
        operation = "verify_totp"
    ))]
    pub async fn verify_totp(&self, user_id: &str, code: &str) -> Result<bool, MfaError> {
        let key_name = {
            let configs = self.configs.read().await;
            let config = configs
                .get(user_id)
                .ok_or_else(|| MfaError::NotConfigured(user_id.to_string()))?;

            config
                .totp_key_name
                .as_ref()
                .ok_or_else(|| MfaError::NotConfigured("TOTP".to_string()))?
                .clone()
        };

        // Delegate to TotpEngine for validation
        let request = TotpValidationRequest {
            key_name,
            code: code.to_string(),
            skew: None,
        };

        let response = self
            .totp_engine
            .validate_code(request)
            .await
            .map_err(|e| match e {
                crate::services::secrets::totp::TotpError::CodeReused => MfaError::TotpReused,
                crate::services::secrets::totp::TotpError::KeyNotFound(_) => {
                    MfaError::NotConfigured(user_id.to_string())
                }
                _ => MfaError::InvalidTotp,
            })?;

        // Update last used if valid
        if response.valid {
            let mut configs = self.configs.write().await;
            if let Some(config) = configs.get_mut(user_id) {
                config.last_used_at = Some(Utc::now());
            }
        }

        Ok(response.valid)
    }

    /// Verify recovery code
    #[instrument(skip(self, code), fields(
        user_id = %user_id,
        operation = "verify_recovery_code"
    ))]
    pub async fn verify_recovery_code(&self, user_id: &str, code: &str) -> Result<bool, MfaError> {
        let mut configs = self.configs.write().await;
        let config = configs
            .get_mut(user_id)
            .ok_or_else(|| MfaError::NotConfigured(user_id.to_string()))?;

        // Check if already used
        if config.used_recovery_codes.contains(&code.to_string()) {
            return Err(MfaError::RecoveryCodeUsed);
        }

        // Check if valid
        if !config.recovery_codes.contains(&code.to_string()) {
            return Err(MfaError::RecoveryCodeNotFound);
        }

        // Mark as used
        config.used_recovery_codes.push(code.to_string());
        config.last_used_at = Some(Utc::now());

        Ok(true)
    }

    /// Disable TOTP for user
    #[instrument(skip(self), fields(
        user_id = %user_id,
        operation = "disable_totp"
    ))]
    pub async fn disable_totp(&self, user_id: &str) -> Result<(), MfaError> {
        let key_name = {
            let configs = self.configs.read().await;
            let config = configs
                .get(user_id)
                .ok_or_else(|| MfaError::NotConfigured(user_id.to_string()))?;
            config.totp_key_name.clone()
        };

        // Delete TOTP key from TotpEngine if exists
        if let Some(key_name) = key_name {
            let _ = self.totp_engine.delete_key(&key_name).await;
        }

        // Update MFA config
        let mut configs = self.configs.write().await;
        let config = configs
            .get_mut(user_id)
            .ok_or_else(|| MfaError::NotConfigured(user_id.to_string()))?;

        config.totp_key_name = None;
        config.enabled_methods.retain(|m| *m != MfaMethodType::TOTP);

        Ok(())
    }

    /// Get MFA configuration
    pub async fn get_config(&self, user_id: &str) -> Option<MfaConfig> {
        let configs = self.configs.read().await;
        configs.get(user_id).cloned()
    }

    /// Check if MFA is configured for user
    pub async fn is_configured(&self, user_id: &str) -> bool {
        let configs = self.configs.read().await;
        configs
            .get(user_id)
            .map(|c| !c.enabled_methods.is_empty())
            .unwrap_or(false)
    }

    /// Regenerate recovery codes
    #[instrument(skip(self), fields(
        user_id = %user_id,
        operation = "regenerate_recovery_codes"
    ))]
    pub async fn regenerate_recovery_codes(&self, user_id: &str) -> Result<Vec<String>, MfaError> {
        let mut configs = self.configs.write().await;
        let config = configs
            .get_mut(user_id)
            .ok_or_else(|| MfaError::NotConfigured(user_id.to_string()))?;

        config.recovery_codes = MfaConfig::generate_recovery_codes();
        config.used_recovery_codes.clear();

        Ok(config.recovery_codes.clone())
    }

    /// Cleanup old TOTP history (delegates to TotpEngine)
    pub async fn cleanup_history(&self, max_age_seconds: i64) -> usize {
        self.totp_engine.cleanup_history(max_age_seconds).await
    }
}

impl Default for MfaService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enable_totp() {
        let service = MfaService::new();

        let response = service
            .enable_totp(
                "user1",
                "Secreton".to_string(),
                "user1@example.com".to_string(),
            )
            .await
            .unwrap();

        assert!(!response.secret.is_empty());
        assert!(!response.qr_code_url.is_empty());
        assert_eq!(response.recovery_codes.len(), 10);
        assert!(service.is_configured("user1").await);
    }

    #[tokio::test]
    async fn test_recovery_codes() {
        let service = MfaService::new();

        service
            .enable_totp(
                "user1",
                "Secreton".to_string(),
                "user1@example.com".to_string(),
            )
            .await
            .unwrap();

        let config = service.get_config("user1").await.unwrap();
        assert_eq!(config.recovery_codes.len(), 10);

        // Each code should be properly formatted
        for code in &config.recovery_codes {
            assert!(code.contains('-'));
            assert!(code.len() > 10);
        }
    }

    #[tokio::test]
    async fn test_verify_recovery_code() {
        let service = MfaService::new();

        service
            .enable_totp(
                "user1",
                "Secreton".to_string(),
                "user1@example.com".to_string(),
            )
            .await
            .unwrap();

        let config = service.get_config("user1").await.unwrap();
        let recovery_code = config.recovery_codes[0].clone();

        // First use should succeed
        let result = service.verify_recovery_code("user1", &recovery_code).await;
        assert!(result.is_ok());

        // Second use should fail
        let result = service.verify_recovery_code("user1", &recovery_code).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_regenerate_recovery_codes() {
        let service = MfaService::new();

        service
            .enable_totp(
                "user1",
                "Secreton".to_string(),
                "user1@example.com".to_string(),
            )
            .await
            .unwrap();

        let old_codes = service.get_config("user1").await.unwrap().recovery_codes;
        let new_codes = service.regenerate_recovery_codes("user1").await.unwrap();

        assert_ne!(old_codes, new_codes);
        assert_eq!(new_codes.len(), 10);
    }
}
