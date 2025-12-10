//! Application Configuration Storage
//!
//! All application configuration (auth, database, MFA, rate limiting, etc.)
//! is stored ENCRYPTED in the storage backend using the master key.
//! This configuration is only accessible when the vault is unsealed.

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use chrono::{DateTime, Utc};
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::services::seal::SealService;
use secreton_storage::StorageBackend;

/// Application configuration stored encrypted in storage backend
/// Contains all sensitive application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub auth: AuthConfig,
    pub database: DatabaseConfig,
    pub mfa: MfaConfig,
    pub rate_limit: RateLimitConfig,
    pub cors: CorsConfig,
    pub logging: LoggingConfig,
    pub audit: AuditConfig,
    pub backup: BackupConfig,

    #[serde(default)]
    pub metadata: ConfigMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<String>,
}

impl Default for ConfigMetadata {
    fn default() -> Self {
        Self {
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            updated_by: None,
        }
    }
}

// ================================
// Configuration Sections
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub require_auth: bool,
    pub jwt_expiration_hours: u32,
    pub issuer: String,
    pub audience: String,
    pub admin_roles: Vec<String>,

    // JWT secrets - stored encrypted!
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt_secret: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_secret: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            require_auth: true,
            jwt_expiration_hours: 24,
            issuer: "secreton-vault".to_string(),
            audience: "secreton-api".to_string(),
            admin_roles: vec!["admin".to_string(), "vault-admin".to_string()],
            jwt_secret: None,
            refresh_secret: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    // Connection URL from environment or encrypted storage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    pub max_connections: u32,
    pub connection_timeout_seconds: u32,
    pub ssl_mode: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: None,
            max_connections: 50,
            connection_timeout_seconds: 30,
            ssl_mode: "require".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaConfig {
    pub enabled: bool,
    pub require_for_admin: bool,
    pub require_for_secrets: bool,
    pub allowed_methods: Vec<String>,
    pub grace_period_hours: u32,
    pub allow_backup_codes: bool,
    pub backup_code_count: u32,
}

impl Default for MfaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            require_for_admin: true,
            require_for_secrets: false,
            allowed_methods: vec!["totp".to_string(), "webauthn".to_string()],
            grace_period_hours: 24,
            allow_backup_codes: true,
            backup_code_count: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub max_requests_per_minute: u32,
    pub unseal_max_attempts: u32,
    pub unseal_window_seconds: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_requests_per_minute: 100,
            unseal_max_attempts: 10,
            unseal_window_seconds: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age_seconds: u32,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
            ],
            allowed_headers: vec!["*".to_string()],
            max_age_seconds: 3600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            output: "stdout".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub log_path: String,
    pub rotation: String,
    pub retention_days: u32,
    pub log_reads: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_path: "/var/log/secreton/audit.log".to_string(),
            rotation: "daily".to_string(),
            retention_days: 90,
            log_reads: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub schedule: String,
    pub retention_days: u32,
    pub path: String,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            schedule: "0 2 * * *".to_string(), // Daily at 2 AM
            retention_days: 30,
            path: "/var/backups/secreton".to_string(),
        }
    }
}

// ================================
// Implementation
// ================================

impl ApplicationConfig {
    /// Create default configuration
    pub fn default() -> Self {
        Self {
            auth: AuthConfig::default(),
            database: DatabaseConfig::default(),
            mfa: MfaConfig::default(),
            rate_limit: RateLimitConfig::default(),
            cors: CorsConfig::default(),
            logging: LoggingConfig::default(),
            audit: AuditConfig::default(),
            backup: BackupConfig::default(),
            metadata: ConfigMetadata::default(),
        }
    }

    /// Load from encrypted storage
    /// REQUIRES: Vault must be unsealed
    pub async fn load_from_storage(
        storage: &dyn StorageBackend,
        seal_service: &SealService,
    ) -> anyhow::Result<Self> {
        // Get master key from SealService (only available when unsealed)
        let master_key = seal_service
            .get_master_key()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get master key: {}", e))?;

        // Read encrypted config from storage
        let entry = storage
            .get_by_path("config/system")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read config from storage: {}", e))?
            .ok_or_else(|| anyhow::anyhow!("Config not found in storage"))?;

        // Decrypt using AES-256-GCM
        let decrypted = Self::decrypt_config(&entry.encrypted_data, &master_key)?;

        // Deserialize
        let config: Self = serde_json::from_slice(&decrypted)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize config: {}", e))?;

        Ok(config)
    }

    /// Save to encrypted storage
    /// REQUIRES: Vault must be unsealed
    pub async fn save_to_storage(
        &self,
        storage: &dyn StorageBackend,
        seal_service: &SealService,
    ) -> anyhow::Result<()> {
        // Get master key from SealService
        let master_key = seal_service
            .get_master_key()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get master key: {}", e))?;

        // Serialize
        let serialized = serde_json::to_vec(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;

        // Encrypt using AES-256-GCM
        let encrypted = Self::encrypt_config(&serialized, &master_key)?;

        // Create VaultEntry
        use secreton_storage::VaultEntry;
        use secreton_types::SecurityLevel;
        let entry = VaultEntry {
            id: uuid::Uuid::new_v4(),
            path: "config/system".to_string(),
            encrypted_data: encrypted,
            encryption_metadata: serde_json::json!({
                "algorithm": "aes-256-gcm",
                "version": 1
            }),
            security_level: SecurityLevel::Secret,
            metadata: serde_json::json!({
                "type": "application_config",
                "updated_by": self.metadata.updated_by.clone().unwrap_or_else(|| "system".to_string())
            }),
            tags: vec!["config".to_string(), "system".to_string()],
            version: self.metadata.version,
            owner_id: "system".to_string(),
            created_at: self.metadata.created_at,
            updated_at: Utc::now(),
            expires_at: None,
        };

        // Store
        storage
            .store(&entry)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to store config: {}", e))?;

        Ok(())
    }

    /// Encrypt configuration data with AES-256-GCM
    fn encrypt_config(data: &[u8], master_key: &[u8]) -> anyhow::Result<Vec<u8>> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        // Create cipher
        let cipher = Aes256Gcm::new_from_slice(&master_key[..32])
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

        // Encrypt
        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt configuration data with AES-256-GCM
    fn decrypt_config(encrypted: &[u8], master_key: &[u8]) -> anyhow::Result<Vec<u8>> {
        if encrypted.len() < 12 {
            return Err(anyhow::anyhow!("Invalid encrypted data: too short"));
        }

        // Extract nonce (first 12 bytes)
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes.copy_from_slice(&encrypted[..12]);
        let nonce = Nonce::from(nonce_bytes);

        // Extract ciphertext (rest)
        let ciphertext = &encrypted[12..];

        // Create cipher
        let cipher = Aes256Gcm::new_from_slice(&master_key[..32])
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

        // Decrypt
        let plaintext = cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }

    /// Create from legacy TOML files (for migration)
    pub fn from_legacy_toml(
        default_toml: &str,
        production_toml: Option<&str>,
    ) -> anyhow::Result<Self> {
        // Parse default config
        let default: toml::Value = toml::from_str(default_toml)
            .map_err(|e| anyhow::anyhow!("Failed to parse default.toml: {}", e))?;

        // Parse production config if provided
        let production: Option<toml::Value> = production_toml
            .map(toml::from_str)
            .transpose()
            .map_err(|e| anyhow::anyhow!("Failed to parse production.toml: {}", e))?;

        // Merge configs (production overrides default)
        let merged = if let Some(prod) = production {
            Self::merge_toml_values(default, prod)
        } else {
            default
        };

        // Convert to ApplicationConfig
        let config = Self::from_toml_value(&merged)?;

        Ok(config)
    }

    fn merge_toml_values(mut base: toml::Value, override_val: toml::Value) -> toml::Value {
        if let (toml::Value::Table(base_table), toml::Value::Table(override_table)) =
            (&mut base, &override_val)
        {
            for (key, value) in override_table {
                base_table.insert(key.clone(), value.clone());
            }
        }
        base
    }

    fn from_toml_value(value: &toml::Value) -> anyhow::Result<Self> {
        let mut config = Self::default();

        if let toml::Value::Table(table) = value {
            // Parse auth section
            if let Some(toml::Value::Table(auth)) = table.get("auth") {
                if let Some(toml::Value::Boolean(v)) = auth.get("require_auth") {
                    config.auth.require_auth = *v;
                }
                if let Some(toml::Value::Integer(v)) = auth.get("jwt_expiration_hours") {
                    config.auth.jwt_expiration_hours = *v as u32;
                }
                if let Some(toml::Value::String(v)) = auth.get("issuer") {
                    config.auth.issuer = v.clone();
                }
                // Note: Secrets should come from environment, not TOML
            }

            // Parse MFA section
            if let Some(toml::Value::Table(mfa)) = table.get("mfa") {
                if let Some(toml::Value::Boolean(v)) = mfa.get("enabled") {
                    config.mfa.enabled = *v;
                }
                if let Some(toml::Value::Boolean(v)) = mfa.get("require_for_admin") {
                    config.mfa.require_for_admin = *v;
                }
            }

            // Parse rate_limiting section
            if let Some(toml::Value::Table(rl)) = table.get("rate_limiting") {
                if let Some(toml::Value::Boolean(v)) = rl.get("enabled") {
                    config.rate_limit.enabled = *v;
                }
                if let Some(toml::Value::Integer(v)) = rl.get("max_requests_per_minute") {
                    config.rate_limit.max_requests_per_minute = *v as u32;
                }
            }

            // Parse audit section
            if let Some(toml::Value::Table(audit)) = table.get("audit") {
                if let Some(toml::Value::Boolean(v)) = audit.get("enabled") {
                    config.audit.enabled = *v;
                }
                if let Some(toml::Value::String(v)) = audit.get("log_path") {
                    config.audit.log_path = v.clone();
                }
            }
        }

        Ok(config)
    }
}

// ================================
// Tests
// ================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_encryption_roundtrip() {
        let config = ApplicationConfig::default();
        let serialized = serde_json::to_vec(&config).unwrap();

        // Generate test key
        let mut master_key = [0u8; 32];
        OsRng.fill_bytes(&mut master_key);

        // Encrypt
        let encrypted = ApplicationConfig::encrypt_config(&serialized, &master_key).unwrap();

        // Decrypt
        let decrypted = ApplicationConfig::decrypt_config(&encrypted, &master_key).unwrap();

        // Should match original
        assert_eq!(serialized, decrypted);
    }

    #[test]
    fn test_default_config() {
        let config = ApplicationConfig::default();

        assert!(config.auth.require_auth);
        assert_eq!(config.auth.jwt_expiration_hours, 24);
        assert!(config.mfa.enabled);
        assert!(config.rate_limit.enabled);
        assert!(config.audit.enabled);
    }

    #[test]
    fn test_from_legacy_toml() {
        let toml_str = r#"
[auth]
require_auth = true
jwt_expiration_hours = 48
issuer = "test-vault"

[mfa]
enabled = true
require_for_admin = true
"#;

        let config = ApplicationConfig::from_legacy_toml(toml_str, None).unwrap();

        assert_eq!(config.auth.jwt_expiration_hours, 48);
        assert_eq!(config.auth.issuer, "test-vault");
        assert!(config.mfa.require_for_admin);
    }
}
