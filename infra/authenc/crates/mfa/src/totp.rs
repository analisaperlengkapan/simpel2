//! TOTP (Time-based One-Time Password) service
//!
//! Provides TOTP setup, QR code generation, and verification with Secreton integration.

use authenc_types::{AuthencError, Result, UserId};
use async_trait::async_trait;
use qrcode::{QrCode, render::svg};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use totp_rs::{Algorithm, Secret, TOTP};
use tracing::{debug, info, warn};

/// TOTP configuration
#[derive(Debug, Clone)]
pub struct TotpConfig {
    /// Issuer name (e.g., "Kejaksaan RI")
    pub issuer: String,
    /// Number of digits in TOTP code (default: 6)
    pub digits: usize,
    /// Time step in seconds (default: 30)
    pub step: u64,
    /// Algorithm (default: SHA1)
    pub algorithm: Algorithm,
    /// Time window tolerance (±N periods, default: 1)
    pub tolerance: u8,
}

impl Default for TotpConfig {
    fn default() -> Self {
        Self {
            issuer: "Kejaksaan RI".to_string(),
            digits: 6,
            step: 30,
            algorithm: Algorithm::SHA1,
            tolerance: 1,
        }
    }
}

/// TOTP setup response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSetupResponse {
    /// Base32-encoded secret
    pub secret: String,
    /// QR code as SVG string
    pub qr_code_svg: String,
    /// Manual entry key (formatted secret)
    pub manual_entry_key: String,
    /// TOTP URI for authenticator apps
    pub totp_uri: String,
}

/// Trait for TOTP secret storage
#[async_trait]
pub trait TotpStore: Send + Sync {
    /// Store TOTP secret for a user
    async fn store_totp_secret(&self, user_id: UserId, secret: &str) -> Result<()>;

    /// Retrieve TOTP secret for a user
    async fn get_totp_secret(&self, user_id: UserId) -> Result<Option<String>>;

    /// Delete TOTP secret for a user
    async fn delete_totp_secret(&self, user_id: UserId) -> Result<()>;
}

/// TOTP service with Secreton integration
pub struct TotpService<S: TotpStore> {
    config: TotpConfig,
    store: Arc<S>,
}

impl<S: TotpStore> TotpService<S> {
    /// Create a new TOTP service
    pub fn new(config: TotpConfig, store: Arc<S>) -> Self {
        Self { config, store }
    }

    /// Generate a new TOTP secret
    pub fn generate_secret(&self) -> String {
        // Generate 20 random bytes (160 bits) for the secret
        let mut rng = rand::thread_rng();
        let secret_bytes: Vec<u8> = (0..20).map(|_| rng.r#gen()).collect();

        // Encode as base32
        Secret::Raw(secret_bytes)
            .to_encoded()
            .to_string()
    }

    /// Setup TOTP for a user
    pub async fn setup_totp(
        &self,
        user_id: UserId,
        username: &str,
    ) -> Result<TotpSetupResponse> {
        info!("Setting up TOTP for user: {}", user_id);

        // Generate secret
        let secret = self.generate_secret();

        // Create TOTP instance (totp-rs 5.7.0 API: 5 parameters)
        // We don't use the instance, just validate the secret is correct
        let _totp = TOTP::new(
            self.config.algorithm,
            self.config.digits,
            self.config.tolerance,
            self.config.step,
            Secret::Encoded(secret.clone())
                .to_bytes()
                .map_err(|e| AuthencError::internal(format!("Failed to decode secret: {}", e)))?,
        )
        .map_err(|e| AuthencError::internal(format!("Failed to create TOTP: {}", e)))?;

        // Generate QR code URI manually
        let totp_uri = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}",
            urlencoding::encode(&self.config.issuer),
            urlencoding::encode(username),
            secret,
            urlencoding::encode(&self.config.issuer)
        );

        let qr_code = QrCode::new(&totp_uri)
            .map_err(|e| AuthencError::internal(format!("Failed to generate QR code: {}", e)))?;

        let qr_code_svg = qr_code
            .render()
            .min_dimensions(200, 200)
            .dark_color(svg::Color("#000000"))
            .light_color(svg::Color("#ffffff"))
            .build();

        // Format manual entry key (add spaces every 4 characters)
        let manual_entry_key = secret
            .chars()
            .collect::<Vec<_>>()
            .chunks(4)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ");

        // Store secret in Secreton
        self.store.store_totp_secret(user_id, &secret).await?;

        info!("TOTP setup completed for user: {}", user_id);

        Ok(TotpSetupResponse {
            secret: secret.clone(),
            qr_code_svg,
            manual_entry_key,
            totp_uri,
        })
    }

    /// Verify TOTP code
    pub async fn verify_totp(&self, user_id: UserId, code: &str) -> Result<bool> {
        debug!("Verifying TOTP code for user: {}", user_id);

        // Retrieve secret from Secreton
        let secret = self
            .store
            .get_totp_secret(user_id)
            .await?
            .ok_or_else(|| AuthencError::not_found("TOTP secret not found"))?;

        // Create TOTP instance (totp-rs 5.7.0 API: 5 parameters)
        let totp = TOTP::new(
            self.config.algorithm,
            self.config.digits,
            self.config.tolerance,
            self.config.step,
            Secret::Encoded(secret)
                .to_bytes()
                .map_err(|e| AuthencError::internal(format!("Failed to decode secret: {}", e)))?,
        )
        .map_err(|e| AuthencError::internal(format!("Failed to create TOTP: {}", e)))?;

        // Verify code
        let is_valid = totp
            .check_current(code)
            .map_err(|e| AuthencError::internal(format!("Failed to verify TOTP: {}", e)))?;

        if is_valid {
            info!("TOTP verification successful for user: {}", user_id);
        } else {
            warn!("TOTP verification failed for user: {}", user_id);
        }

        Ok(is_valid)
    }

    /// Disable TOTP for a user
    pub async fn disable_totp(&self, user_id: UserId) -> Result<()> {
        info!("Disabling TOTP for user: {}", user_id);
        self.store.delete_totp_secret(user_id).await?;
        Ok(())
    }

    /// Check if TOTP is enabled for a user
    pub async fn is_totp_enabled(&self, user_id: UserId) -> Result<bool> {
        let secret = self.store.get_totp_secret(user_id).await?;
        Ok(secret.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::RwLock;

    /// Mock TOTP store for testing
    struct MockTotpStore {
        secrets: Arc<RwLock<HashMap<UserId, String>>>,
    }

    impl MockTotpStore {
        fn new() -> Self {
            Self {
                secrets: Arc::new(RwLock::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl TotpStore for MockTotpStore {
        async fn store_totp_secret(&self, user_id: UserId, secret: &str) -> Result<()> {
            self.secrets.write().await.insert(user_id, secret.to_string());
            Ok(())
        }

        async fn get_totp_secret(&self, user_id: UserId) -> Result<Option<String>> {
            Ok(self.secrets.read().await.get(&user_id).cloned())
        }

        async fn delete_totp_secret(&self, user_id: UserId) -> Result<()> {
            self.secrets.write().await.remove(&user_id);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_totp_setup() {
        let store = Arc::new(MockTotpStore::new());
        let service = TotpService::new(TotpConfig::default(), store.clone());

        let user_id = UserId::new();
        let response = service.setup_totp(user_id, "test@example.com").await.unwrap();

        // Verify response
        assert!(!response.secret.is_empty());
        assert!(!response.qr_code_svg.is_empty());
        assert!(!response.manual_entry_key.is_empty());
        assert!(response.totp_uri.contains("otpauth://totp/"));

        // Verify secret was stored
        let stored_secret = store.get_totp_secret(user_id).await.unwrap();
        assert_eq!(stored_secret, Some(response.secret));
    }

    #[tokio::test]
    async fn test_totp_verification() {
        let store = Arc::new(MockTotpStore::new());
        let service = TotpService::new(TotpConfig::default(), store.clone());

        let user_id = UserId::new();
        let response = service.setup_totp(user_id, "test@example.com").await.unwrap();

        // Generate current TOTP code
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(response.secret).to_bytes().unwrap(),
        )
        .unwrap();

        let code = totp.generate_current().unwrap();

        // Verify code
        let is_valid = service.verify_totp(user_id, &code).await.unwrap();
        assert!(is_valid);

        // Verify invalid code
        let is_valid = service.verify_totp(user_id, "000000").await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_totp_disable() {
        let store = Arc::new(MockTotpStore::new());
        let service = TotpService::new(TotpConfig::default(), store.clone());

        let user_id = UserId::new();
        service.setup_totp(user_id, "test@example.com").await.unwrap();

        // Verify TOTP is enabled
        assert!(service.is_totp_enabled(user_id).await.unwrap());

        // Disable TOTP
        service.disable_totp(user_id).await.unwrap();

        // Verify TOTP is disabled
        assert!(!service.is_totp_enabled(user_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_secret_generation() {
        let store = Arc::new(MockTotpStore::new());
        let service = TotpService::new(TotpConfig::default(), store);

        let secret1 = service.generate_secret();
        let secret2 = service.generate_secret();

        // Verify secrets are different
        assert_ne!(secret1, secret2);

        // Verify secrets are base32 encoded
        assert!(secret1.chars().all(|c| "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567".contains(c)));
    }
}
