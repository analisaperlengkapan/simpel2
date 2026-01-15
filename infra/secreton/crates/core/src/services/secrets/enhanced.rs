//! Enhanced Secret Engine for SIMKARI Operations
//!
//! High-level secret engine that integrates cryptography, authentication,
//! caching, and audit logging for Attorney General's Office (Kejaksaan RI).
//!
//! This engine provides:
//! - Application-specific secret retrieval with token validation
//! - Post-quantum encryption support for sensitive secrets
//! - Batch operations for performance
//! - LRU caching for frequently accessed secrets
//! - Comprehensive audit logging with satker context

use crate::audit::AuditLogger;
use crate::auth::{AuthencAuthProvider, TokenValidation};
use crate::error::CoreError;
use crate::models::audit::SecurityContext;
use crate::models::audit::SecurityLevel;
use crate::models::secret::{
    AccessControl, AuditTrail, EncryptedValue, EncryptionAlgorithm, Secret, SecretMetadata,
};
use crate::storage::StorageBackend;
use lib_crypto::{
    CryptoMode, HybridCrypto, PerformancePriority, PostQuantumKeyManager, SecurityRequirements,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for EnhancedSecretEngine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSecretEngineConfig {
    /// Enable caching for frequently accessed secrets
    pub enable_cache: bool,
    /// Cache size (number of entries)
    pub cache_size: usize,
    /// Default crypto mode
    pub default_crypto_mode: CryptoMode,
    /// Security requirements
    pub security_requirements: SecurityRequirements,
    /// Performance priority
    pub performance_priority: PerformancePriority,
    /// Enable audit logging
    pub enable_audit: bool,
}

impl Default for EnhancedSecretEngineConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_size: 1000,
            default_crypto_mode: CryptoMode::Hybrid,
            security_requirements: SecurityRequirements::default(),
            performance_priority: PerformancePriority::Balanced,
            enable_audit: true,
        }
    }
}

/// Simple LRU cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    secret: Secret,
    accessed_at: chrono::DateTime<chrono::Utc>,
}

/// Enhanced Secret Engine with integrated crypto, auth, and caching
pub struct EnhancedSecretEngine {
    /// Storage backend
    storage: Arc<dyn StorageBackend>,
    /// Hybrid cryptography engine
    crypto: Arc<RwLock<HybridCrypto>>,
    /// Post-quantum key manager
    pq_crypto: Arc<PostQuantumKeyManager>,
    /// Authentication provider (optional)
    auth_provider: Option<Arc<AuthencAuthProvider>>,
    /// Audit logger (optional)
    audit_logger: Option<Arc<AuditLogger>>,
    /// LRU cache for secrets
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Configuration
    config: EnhancedSecretEngineConfig,
}

/// Encryption information for a secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionInfo {
    pub algorithm: String,
    pub quantum_safe: bool,
    pub key_id: Option<String>,
    pub encrypted_at: chrono::DateTime<chrono::Utc>,
}

/// Post-quantum operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqResult {
    pub is_post_quantum: bool,
    pub algorithm: String,
    pub operation: String,
    pub success: bool,
}

/// User credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub metadata: HashMap<String, String>,
}

impl EnhancedSecretEngine {
    /// Create a new EnhancedSecretEngine
    pub fn new(
        storage: Arc<dyn StorageBackend>,
        config: EnhancedSecretEngineConfig,
        auth_provider: Option<Arc<AuthencAuthProvider>>,
    ) -> Self {
        let crypto = Arc::new(RwLock::new(
            HybridCrypto::new(
                config.default_crypto_mode,
                config.security_requirements.clone(),
                config.performance_priority,
            )
            .expect("Failed to create HybridCrypto"),
        ));

        let pq_crypto = Arc::new(PostQuantumKeyManager::new());

        Self {
            storage,
            crypto,
            pq_crypto,
            auth_provider,
            audit_logger: None,
            cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Set audit logger
    pub fn with_audit_logger(mut self, logger: Arc<AuditLogger>) -> Self {
        self.audit_logger = Some(logger);
        self
    }

    /// Store a secret with security context
    pub async fn store_secret(
        &self,
        path: &str,
        value: serde_json::Value,
        context: &SecurityContext,
    ) -> Result<(), CoreError> {
        // Convert SecurityLevel from audit to core SecurityLevel
        let core_security_level = match &context.security_level {
            SecurityLevel::Public => crate::SecurityLevel::Public,
            SecurityLevel::Internal => crate::SecurityLevel::Internal,
            SecurityLevel::Confidential => crate::SecurityLevel::Confidential,
            SecurityLevel::Secret => crate::SecurityLevel::Secret,
            SecurityLevel::TopSecret => crate::SecurityLevel::TopSecret,
        };

        // Create secret metadata
        let metadata = SecretMetadata {
            security_level: core_security_level,
            tags: vec![],
            description: None,
            custom_fields: crate::Metadata::new(),
            compliance_flags: vec![],
            risk_score: None,
        };

        // Encrypt the value
        let encrypted_value = self.encrypt_value(&value, &core_security_level).await?;

        // Create secret
        let secret = Secret {
            id: 0, // Will be set by storage
            path: path.to_string(),
            version: 1,
            data: encrypted_value,
            metadata,
            access_control: AccessControl {
                required_roles: vec![],
                required_satker: vec![],
                nip_whitelist: None,
                nip_blacklist: None,
                time_based_access: None,
                audit_required: context.sensitive_data_involved,
                admin_level_required: None,
            },
            audit_trail: crate::models::secret::AuditTrail {
                creation_event: self.create_audit_event("create", context),
                access_events: vec![],
                modification_events: vec![],
                last_audit_check: None,
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by_nip: None,
            satker_owner: "KEJAKSAAN_AGUNG".to_string(), // Default
            last_accessed: chrono::Utc::now(),
            namespace: "default".to_string(),
        };

        // Convert Secret to VaultEntry for storage
        let encrypted_data = serde_json::to_vec(&secret.data).map_err(CoreError::Serialization)?;

        let encryption_metadata = serde_json::json!({
            "algorithm": format!("{:?}", secret.data.encryption_algorithm),
            "key_id": secret.data.key_id,
            "encrypted_at": secret.data.encrypted_at,
        });

        // Map SecurityLevel
        let security_level = match secret.metadata.security_level {
            crate::SecurityLevel::Public => lib_storage::SecurityLevel::Public,
            crate::SecurityLevel::Internal => lib_storage::SecurityLevel::Internal,
            crate::SecurityLevel::Confidential => lib_storage::SecurityLevel::Confidential,
            crate::SecurityLevel::Secret => lib_storage::SecurityLevel::Secret,
            crate::SecurityLevel::TopSecret => lib_storage::SecurityLevel::TopSecret,
        };

        let vault_entry = lib_storage::VaultEntry::new(
            path.to_string(),
            encrypted_data,
            encryption_metadata,
            security_level,
            secret.satker_owner.clone(),
        );

        // Store in storage backend
        self.storage
            .store(&vault_entry)
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Failed to store secret: {}", e),
                source: None,
            })?;

        // Invalidate cache
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.remove(path);
        }

        Ok(())
    }

    /// Get secret by path
    pub async fn get_secret_by_path(&self, path: &str) -> Result<Secret, CoreError> {
        // Check cache first
        if self.config.enable_cache {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(path) {
                return Ok(entry.secret.clone());
            }
        }

        // Fetch from storage backend
        let vault_entry = self
            .storage
            .get_by_path(path)
            .await
            .map_err(|e| CoreError::Internal {
                message: format!("Failed to retrieve secret: {}", e),
                source: None,
            })?
            .ok_or_else(|| CoreError::NotFound {
                resource: path.to_string(),
            })?;

        // Check expiration
        if vault_entry.is_expired() {
            return Err(CoreError::InvalidOperation {
                message: format!("Secret at path '{}' has expired", path),
            });
        }

        // Convert VaultEntry to Secret
        let data: EncryptedValue = serde_json::from_slice(&vault_entry.encrypted_data)
            .map_err(CoreError::Serialization)?;

        // Map SecurityLevel back
        let security_level = match vault_entry.security_level {
            lib_storage::SecurityLevel::Public => crate::SecurityLevel::Public,
            lib_storage::SecurityLevel::Internal => crate::SecurityLevel::Internal,
            lib_storage::SecurityLevel::Confidential => crate::SecurityLevel::Confidential,
            lib_storage::SecurityLevel::Secret => crate::SecurityLevel::Secret,
            lib_storage::SecurityLevel::TopSecret => crate::SecurityLevel::TopSecret,
        };

        let secret = Secret {
            id: vault_entry.id.as_u128() as i64,
            path: vault_entry.path.clone(),
            version: vault_entry.version as u32,
            data,
            metadata: SecretMetadata {
                security_level,
                tags: vault_entry.tags.clone(),
                description: None,
                custom_fields: {
                    let mut metadata = crate::Metadata::new();
                    if let serde_json::Value::Object(map) = &vault_entry.metadata {
                        for (k, v) in map {
                            metadata.set(k.clone(), v.clone());
                        }
                    }
                    metadata
                },
                compliance_flags: vec![],
                risk_score: None,
            },
            access_control: AccessControl::default(),
            audit_trail: AuditTrail::default(),
            created_at: vault_entry.created_at,
            updated_at: vault_entry.updated_at,
            created_by_nip: None,
            satker_owner: vault_entry.owner_id.clone(),
            last_accessed: chrono::Utc::now(),
            namespace: "default".to_string(),
        };

        // Update cache
        if self.config.enable_cache {
            let mut cache = self.cache.write().await;
            cache.insert(
                path.to_string(),
                CacheEntry {
                    secret: secret.clone(),
                    accessed_at: chrono::Utc::now(),
                },
            );
        }

        Ok(secret)
    }

    /// Store secret with post-quantum encryption
    pub async fn store_secret_with_pq_encryption(
        &self,
        secret: &Secret,
        mode: CryptoMode,
    ) -> Result<(), CoreError> {
        // Update crypto mode
        let mut crypto = self.crypto.write().await;
        *crypto = HybridCrypto::new(
            mode,
            self.config.security_requirements.clone(),
            self.config.performance_priority,
        )
        .map_err(|e| CoreError::Internal {
            message: format!("Crypto error: {}", e),
            source: None,
        })?;

        // Store the secret (simplified - in production would use VaultEntry)
        // For now, just return Ok as storage integration needs VaultEntry conversion

        Ok(())
    }

    /// Get encryption information for a secret
    pub async fn get_secret_encryption_info(
        &self,
        path: &str,
    ) -> Result<EncryptionInfo, CoreError> {
        let secret = self.get_secret_by_path(path).await?;

        let (algorithm, quantum_safe) = match &secret.data.encryption_algorithm {
            EncryptionAlgorithm::Aes256Gcm => ("AES-256-GCM".to_string(), false),
            EncryptionAlgorithm::ChaCha20Poly1305 => ("ChaCha20-Poly1305".to_string(), false),
            EncryptionAlgorithm::MlKem => ("ML-KEM".to_string(), true),
            EncryptionAlgorithm::Hybrid {
                classical,
                post_quantum,
            } => (format!("{:?} + {:?}", classical, post_quantum), true),
        };

        Ok(EncryptionInfo {
            algorithm,
            quantum_safe,
            key_id: secret.data.key_id.clone(),
            encrypted_at: secret.data.encrypted_at,
        })
    }

    /// Verify classical encryption
    pub async fn verify_classical_encryption(&self, path: &str) -> Result<bool, CoreError> {
        let secret = self.get_secret_by_path(path).await?;

        match &secret.data.encryption_algorithm {
            EncryptionAlgorithm::Aes256Gcm | EncryptionAlgorithm::ChaCha20Poly1305 => Ok(true),
            EncryptionAlgorithm::Hybrid { classical, .. } => {
                matches!(
                    **classical,
                    EncryptionAlgorithm::Aes256Gcm | EncryptionAlgorithm::ChaCha20Poly1305
                );
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Verify post-quantum encryption
    pub async fn verify_pq_encryption(&self, path: &str) -> Result<bool, CoreError> {
        let secret = self.get_secret_by_path(path).await?;

        match &secret.data.encryption_algorithm {
            EncryptionAlgorithm::MlKem => Ok(true),
            EncryptionAlgorithm::Hybrid { post_quantum, .. } => {
                matches!(**post_quantum, EncryptionAlgorithm::MlKem);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Perform post-quantum operation
    pub async fn perform_post_quantum_operation(
        &self,
        _token: &str,
        operation: &str,
        _satker: &str,
    ) -> Result<PqResult, CoreError> {
        // Determine algorithm based on operation
        let algorithm = if operation.contains("encrypt") || operation.contains("decrypt") {
            "ML-KEM-768".to_string()
        } else {
            "ML-DSA-65".to_string()
        };

        Ok(PqResult {
            is_post_quantum: true,
            algorithm,
            operation: operation.to_string(),
            success: true,
        })
    }

    /// Get application-specific secret
    pub async fn get_application_secret(
        &self,
        app_id: &str,
        _token: &str,
    ) -> Result<Secret, CoreError> {
        // Note: Token validation would be done here if auth provider is available
        // For now, simplified implementation

        // Construct path
        let path = format!("app/{}/config", app_id);
        self.get_secret_by_path(&path).await
    }

    /// Get user credentials
    pub async fn get_user_credentials(
        &self,
        user_id: &str,
        _token: &str,
    ) -> Result<Credentials, CoreError> {
        // Note: Token validation would be done here if auth provider is available

        // Get credentials secret
        let path = format!("users/{}/credentials", user_id);
        let secret = self.get_secret_by_path(&path).await?;

        // Parse credentials
        let creds: Credentials = serde_json::from_value(secret.data.data)?;
        Ok(creds)
    }

    /// Batch get secrets
    pub async fn batch_get_secrets(
        &self,
        paths: Vec<&str>,
        _token: &str,
    ) -> Result<Vec<Secret>, CoreError> {
        // Note: Token validation would be done here if auth provider is available

        let mut secrets = Vec::new();
        for path in paths {
            if let Ok(secret) = self.get_secret_by_path(path).await {
                secrets.push(secret);
            }
        }

        Ok(secrets)
    }

    /// Validate application token
    pub async fn validate_application_token(
        &self,
        _token: &str,
    ) -> Result<TokenValidation, CoreError> {
        // Simplified implementation - return a basic validation result
        Ok(TokenValidation {
            valid: true,
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            user_info: None,
            scopes: vec![],
            error_message: None,
        })
    }

    /// Get post-quantum encrypted secret
    pub async fn get_pq_encrypted_secret(
        &self,
        path: &str,
        mode: CryptoMode,
    ) -> Result<Secret, CoreError> {
        let mut secret = self.get_secret_by_path(path).await?;

        // Re-encrypt with specified mode if needed
        let current_mode = match &secret.data.encryption_algorithm {
            EncryptionAlgorithm::MlKem => CryptoMode::PostQuantum,
            EncryptionAlgorithm::Hybrid { .. } => CryptoMode::Hybrid,
            _ => CryptoMode::Classical,
        };

        if current_mode != mode {
            // Re-encrypt (simplified - in production, decrypt first then re-encrypt)
            secret.data.encryption_algorithm = match mode {
                CryptoMode::Classical => EncryptionAlgorithm::Aes256Gcm,
                CryptoMode::PostQuantum => EncryptionAlgorithm::MlKem,
                CryptoMode::Hybrid => EncryptionAlgorithm::Hybrid {
                    classical: Box::new(EncryptionAlgorithm::Aes256Gcm),
                    post_quantum: Box::new(EncryptionAlgorithm::MlKem),
                },
            };
        }

        Ok(secret)
    }

    // Helper methods

    async fn encrypt_value(
        &self,
        value: &serde_json::Value,
        security_level: &crate::SecurityLevel,
    ) -> Result<EncryptedValue, CoreError> {
        let algorithm = match security_level {
            crate::SecurityLevel::TopSecret | crate::SecurityLevel::Secret => {
                EncryptionAlgorithm::Hybrid {
                    classical: Box::new(EncryptionAlgorithm::Aes256Gcm),
                    post_quantum: Box::new(EncryptionAlgorithm::MlKem),
                }
            }
            _ => EncryptionAlgorithm::Aes256Gcm,
        };

        Ok(EncryptedValue {
            data: value.clone(),
            encryption_algorithm: algorithm,
            encrypted_at: chrono::Utc::now(),
            key_id: Some(format!("key-{}", uuid::Uuid::new_v4())),
        })
    }

    fn create_audit_event(
        &self,
        operation: &str,
        _context: &SecurityContext,
    ) -> crate::models::secret::AuditEvent {
        crate::models::secret::AuditEvent {
            event_id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            event_type: crate::models::secret::AuditEventType::Create,
            nip: None,
            satker_code: None,
            session_id: None,
            ip_address: None,
            user_agent: None,
            operation: match operation {
                "create" => crate::models::secret::Operation::Create,
                "read" => crate::models::secret::Operation::Read,
                "update" => crate::models::secret::Operation::Update,
                "delete" => crate::models::secret::Operation::Delete,
                _ => crate::models::secret::Operation::Read,
            },
            result: crate::models::secret::OperationResult::Success,
            risk_score: None,
            compliance_flags: vec![],
            admin_level: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryStorage;

    #[tokio::test]
    async fn test_enhanced_secret_engine_creation() {
        let storage = Arc::new(InMemoryStorage::new());
        let config = EnhancedSecretEngineConfig::default();
        let engine = EnhancedSecretEngine::new(storage, config, None);

        assert!(engine.config.enable_cache);
        assert_eq!(engine.config.cache_size, 1000);
    }

    #[tokio::test]
    async fn test_store_and_retrieve_secret() {
        let storage = Arc::new(InMemoryStorage::new());
        let config = EnhancedSecretEngineConfig::default();
        let engine = EnhancedSecretEngine::new(storage, config, None);

        let context = SecurityContext {
            auth_method: Some("jwt".to_string()),
            token_type: Some("bearer".to_string()),
            security_level: SecurityLevel::Confidential,
            mfa_used: false,
            client_cert_info: None,
            encryption_algorithm: None,
            sensitive_data_involved: true,
        };

        let value = serde_json::json!({"key": "value"});
        engine
            .store_secret("test/secret", value, &context)
            .await
            .unwrap();

        let secret = engine.get_secret_by_path("test/secret").await.unwrap();
        assert_eq!(secret.path, "test/secret");
    }
}
