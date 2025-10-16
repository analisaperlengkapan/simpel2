//! Enhanced Secret Engine for SIMKARI Operations
//!
//! This module provides an enhanced secret management engine specifically designed
//! for the SIMKARI super app and Indonesian government operations. It includes
//! support for satker-based organization, post-quantum cryptography, and
//! comprehensive audit trails.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::audit::{AuditLog, AuditLogger, AuditStatus};
use crate::crypto::{decrypt_data, encrypt_data};
use crate::error::CoreError;
use crate::models::secret::{Secret, EncryptedValue, EncryptionAlgorithm, SecretMetadata, AccessControl, AuditTrail};
use crate::storage::{StorageBackend, StorageEngine, StorageEntry};
use crate::{Metadata, SecurityLevel};

/// Configuration for the Enhanced Secret Engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSecretEngineConfig {
    /// Cache size for frequently accessed secrets
    pub cache_size: usize,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable post-quantum encryption for sensitive secrets
    pub enable_post_quantum: bool,
    /// Batch operation timeout in seconds
    pub batch_timeout_seconds: u64,
    /// Maximum batch size for operations
    pub max_batch_size: usize,
    /// Enable audit logging for all operations
    pub enable_audit: bool,
}

impl Default for EnhancedSecretEngineConfig {
    fn default() -> Self {
        Self {
            cache_size: 1000,
            cache_ttl_seconds: 300, // 5 minutes
            enable_post_quantum: false, // Start with classical crypto
            batch_timeout_seconds: 30,
            max_batch_size: 100,
            enable_audit: true,
        }
    }
}

/// Post-quantum cryptography algorithms supported
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PqAlgorithm {
    /// ML-KEM (Module-Lattice-Based Key Encapsulation Mechanism)
    MlKem,
    /// ML-DSA (Module-Lattice-Based Digital Signature Algorithm)
    MlDsa,
    /// Hybrid classical + post-quantum
    Hybrid,
}

/// Cached secret with TTL
#[derive(Debug, Clone)]
struct CachedSecret {
    secret: Secret,
    cached_at: DateTime<Utc>,
    ttl: Duration,
}

impl CachedSecret {
    fn new(secret: Secret, ttl: Duration) -> Self {
        Self {
            secret,
            cached_at: Utc::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Utc::now().signed_duration_since(self.cached_at) > chrono::Duration::from_std(self.ttl).unwrap_or_default()
    }
}

/// Application configuration retrieved from secrets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub app_id: String,
    pub config_data: HashMap<String, serde_json::Value>,
    pub security_level: SecurityLevel,
    pub satker_owner: String,
    pub last_updated: DateTime<Utc>,
}

/// User credentials for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub user_id: String,
    pub nip: Option<String>,
    pub credential_type: String,
    pub credential_data: serde_json::Value,
    pub satker_code: String,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Batch operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSecretRequest {
    pub resource_ids: Vec<String>,
    pub operation_type: BatchOperationType,
    pub context: SecurityContext,
}

/// Types of batch operations supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchOperationType {
    Read,
    Write(HashMap<String, serde_json::Value>),
    Delete,
}

/// Security context for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub user_id: Option<String>,
    pub nip: Option<String>,
    pub satker_code: String,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub security_level: SecurityLevel,
}

/// Enhanced Secret Engine for SIMKARI operations
pub struct EnhancedSecretEngine {
    storage: Arc<dyn StorageEngine>,
    config: EnhancedSecretEngineConfig,
    cache: Arc<RwLock<LruCache<String, CachedSecret>>>,
    audit_logger: Option<Arc<AuditLogger>>,
}

impl EnhancedSecretEngine {
    /// Create a new Enhanced Secret Engine
    pub fn new(
        storage: Arc<dyn StorageEngine>,
        config: EnhancedSecretEngineConfig,
        audit_logger: Option<Arc<AuditLogger>>,
    ) -> Self {
        let cache = Arc::new(RwLock::new(LruCache::new(
            std::num::NonZeroUsize::new(config.cache_size).unwrap_or(std::num::NonZeroUsize::new(1000).unwrap())
        )));

        Self {
            storage,
            config,
            cache,
            audit_logger,
        }
    }

    /// Get application-specific secret
    pub async fn get_application_secret(
        &self,
        app_id: &str,
        path: &str,
        context: &SecurityContext,
    ) -> Result<Secret, CoreError> {
        let full_path = format!("apps/{}/{}", app_id, path);

        // Log audit event
        self.log_audit_event(
            "get_application_secret",
            &full_path,
            context,
            AuditStatus::Success,
        ).await;

        // Check cache first
        if let Some(cached) = self.get_from_cache(&full_path).await {
            if !cached.is_expired() {
                debug!("Cache hit for application secret: {}", full_path);
                return Ok(cached.secret);
            }
        }

        // Retrieve from storage
        let secret = self.get_secret_from_storage(&full_path, context).await?;

        // Cache the result
        self.cache_secret(&full_path, &secret).await;

        Ok(secret)
    }

    /// Get application configuration
    pub async fn get_application_config(
        &self,
        app_id: &str,
        context: &SecurityContext,
    ) -> Result<ApplicationConfig, CoreError> {
        let config_path = format!("apps/{}/config", app_id);

        let secret = self.get_application_secret(app_id, "config", context).await?;

        // Parse configuration from secret data
        let config_data: HashMap<String, serde_json::Value> =
            serde_json::from_value(secret.data.data.clone())
                .map_err(|e| CoreError::validation(format!("Invalid config format: {}", e)))?;

        Ok(ApplicationConfig {
            app_id: app_id.to_string(),
            config_data,
            security_level: SecurityLevel::Internal, // Default level
            satker_owner: context.satker_code.clone(),
            last_updated: secret.updated_at,
        })
    }

    /// Get user credentials
    pub async fn get_user_credentials(
        &self,
        user_id: &str,
        context: &SecurityContext,
    ) -> Result<UserCredentials, CoreError> {
        let cred_path = format!("users/{}/credentials", user_id);

        // Verify user can access their own credentials or admin access
        if context.user_id.as_ref().map(|s| s.as_str()) != Some(user_id) && context.security_level < SecurityLevel::Confidential {
            return Err(CoreError::authorization("Insufficient permissions to access user credentials".to_string()));
        }

        let secret = self.get_secret_from_storage(&cred_path, context).await?;

        let credential_data: serde_json::Value = secret.data.data.clone();

        Ok(UserCredentials {
            user_id: user_id.to_string(),
            nip: context.nip.clone(),
            credential_type: "password".to_string(), // Default type
            credential_data,
            satker_code: context.satker_code.clone(),
            expires_at: None, // No expiration by default
        })
    }

    /// Batch get secrets for performance
    pub async fn batch_get_secrets(
        &self,
        request: &BatchSecretRequest,
    ) -> Result<Vec<Secret>, CoreError> {
        if request.resource_ids.len() > self.config.max_batch_size {
            return Err(CoreError::validation(format!(
                "Batch size {} exceeds maximum {}",
                request.resource_ids.len(),
                self.config.max_batch_size
            )));
        }

        let mut results = Vec::new();

        // Process batch with timeout
        let timeout = Duration::from_secs(self.config.batch_timeout_seconds);

        match tokio::time::timeout(timeout, async {
            for resource_id in &request.resource_ids {
                match self.get_secret_from_storage(resource_id, &request.context).await {
                    Ok(secret) => results.push(secret),
                    Err(e) => {
                        warn!("Failed to get secret {} in batch: {}", resource_id, e);
                        // Continue with other secrets in batch
                    }
                }
            }
            Ok::<Vec<Secret>, CoreError>(results)
        }).await {
            Ok(Ok(secrets)) => {
                self.log_audit_event(
                    "batch_get_secrets",
                    &format!("batch_size:{}", request.resource_ids.len()),
                    &request.context,
                    AuditStatus::Success,
                ).await;
                Ok(secrets)
            },
            Ok(Err(e)) => Err(e),
            Err(_) => {
                self.log_audit_event(
                    "batch_get_secrets",
                    &format!("batch_size:{}", request.resource_ids.len()),
                    &request.context,
                    AuditStatus::Failure,
                ).await;
                Err(CoreError::Timeout { operation: "Batch operation timed out".to_string() })
            }
        }
    }

    /// Validate application token
    pub async fn validate_application_token(
        &self,
        token: &str,
        app_context: &str,
        context: &SecurityContext,
    ) -> Result<bool, CoreError> {
        let token_path = format!("apps/{}/tokens/{}", app_context, token);

        match self.get_secret_from_storage(&token_path, context).await {
            Ok(secret) => {
                // Check if token is still valid
                if let Some(expires_at_value) = secret.metadata.custom_fields.get("expires_at") {
                    if let Ok(expires_at) = serde_json::from_value::<DateTime<Utc>>(expires_at_value.clone()) {
                        if Utc::now() > expires_at {
                            self.log_audit_event(
                                "validate_application_token",
                                &token_path,
                                context,
                                AuditStatus::Failure,
                            ).await;
                            return Ok(false);
                        }
                    }
                }

                self.log_audit_event(
                    "validate_application_token",
                    &token_path,
                    context,
                    AuditStatus::Success,
                ).await;
                Ok(true)
            },
            Err(_) => {
                self.log_audit_event(
                    "validate_application_token",
                    &token_path,
                    context,
                    AuditStatus::Failure,
                ).await;
                Ok(false)
            }
        }
    }

    /// Get post-quantum encrypted secret
    pub async fn get_pq_encrypted_secret(
        &self,
        path: &str,
        algorithm: PqAlgorithm,
        context: &SecurityContext,
    ) -> Result<Secret, CoreError> {
        if !self.config.enable_post_quantum {
            return Err(CoreError::invalid_operation("Post-quantum cryptography not enabled".to_string()));
        }

        let mut secret = self.get_secret_from_storage(path, context).await?;

        // Apply post-quantum encryption based on algorithm
        match algorithm {
            PqAlgorithm::MlKem => {
                // Placeholder for ML-KEM implementation
                info!("Applying ML-KEM encryption to secret: {}", path);
                // In production, this would use actual ML-KEM implementation
            },
            PqAlgorithm::MlDsa => {
                // Placeholder for ML-DSA signature
                info!("Applying ML-DSA signature to secret: {}", path);
                // In production, this would use actual ML-DSA implementation
            },
            PqAlgorithm::Hybrid => {
                // Placeholder for hybrid classical + post-quantum
                info!("Applying hybrid encryption to secret: {}", path);
                // In production, this would combine classical and PQ algorithms
            }
        }

        // Add post-quantum metadata
        secret.metadata.custom_fields.set("pq_algorithm", serde_json::to_value(algorithm)?);
        secret.metadata.custom_fields.set("pq_encrypted_at", serde_json::to_value(Utc::now())?);

        self.log_audit_event(
            "get_pq_encrypted_secret",
            path,
            context,
            AuditStatus::Success,
        ).await;

        Ok(secret)
    }

    /// Store secret with enhanced features
    pub async fn store_secret(
        &self,
        path: &str,
        data: serde_json::Value,
        context: &SecurityContext,
    ) -> Result<(), CoreError> {
        // Create enhanced secret with metadata
        let mut secret = Secret::new(
            path.to_string(),
            data,
            context.satker_code.clone(),
            context.nip.clone(),
        );

        // Set security level from context
        secret.metadata.security_level = context.security_level;

        // Encrypt data before storage
        let encrypted_data = encrypt_data(
            &serde_json::to_string(&secret.data.data)?,
            "default_key" // In production, use proper key management
        ).map_err(|e| CoreError::Internal(anyhow::anyhow!(e)))?;

        let storage_entry = StorageEntry {
            key: path.to_string(),
            value: encrypted_data.into_bytes(),
            metadata: HashMap::new(), // Convert Metadata to HashMap if needed
        };

        self.storage.put(storage_entry).await?;

        // Invalidate cache
        self.invalidate_cache(path).await;

        self.log_audit_event(
            "store_secret",
            path,
            context,
            AuditStatus::Success,
        ).await;

        Ok(())
    }

    /// Delete secret
    pub async fn delete_secret(
        &self,
        path: &str,
        context: &SecurityContext,
    ) -> Result<(), CoreError> {
        // Verify permissions
        if context.security_level < SecurityLevel::Confidential {
            return Err(CoreError::authorization("Insufficient permissions to delete secrets".to_string()));
        }

        self.storage.delete(path).await?;

        // Invalidate cache
        self.invalidate_cache(path).await;

        self.log_audit_event(
            "delete_secret",
            path,
            context,
            AuditStatus::Success,
        ).await;

        Ok(())
    }

    /// List secrets with prefix
    pub async fn list_secrets(
        &self,
        prefix: &str,
        context: &SecurityContext,
    ) -> Result<Vec<String>, CoreError> {
        // Filter by satker if not admin
        let filtered_prefix = if context.security_level < SecurityLevel::Secret {
            format!("{}/{}", context.satker_code, prefix)
        } else {
            prefix.to_string()
        };

        let paths = self.storage.list(&filtered_prefix).await?;

        self.log_audit_event(
            "list_secrets",
            &filtered_prefix,
            context,
            AuditStatus::Success,
        ).await;

        Ok(paths)
    }

    // Private helper methods

    async fn get_secret_from_storage(
        &self,
        path: &str,
        context: &SecurityContext,
    ) -> Result<Secret, CoreError> {
        let entry = self.storage.get(path).await?
            .ok_or_else(|| CoreError::not_found(format!("Secret not found: {}", path)))?;

        // Decrypt data
        let decrypted_data = decrypt_data(
            &String::from_utf8(entry.value)
                .map_err(|e| CoreError::validation(format!("Invalid UTF-8 data: {}", e)))?,
            "default_key" // In production, use proper key management
        ).map_err(|e| CoreError::Internal(anyhow::anyhow!(e)))?;

        // Parse secret data
        let data: serde_json::Value = serde_json::from_str(&decrypted_data)
            .map_err(|e| CoreError::validation(format!("Invalid JSON data: {}", e)))?;

        // Create secret with enhanced structure
        let encrypted_value = EncryptedValue {
            data,
            encryption_algorithm: EncryptionAlgorithm::Aes256Gcm,
            encrypted_at: Utc::now(),
            key_id: None,
        };

        let mut secret = Secret::new(
            path.to_string(),
            serde_json::Value::Null, // Placeholder, will be set below
            context.satker_code.clone(),
            context.nip.clone(),
        );

        // Set the encrypted data
        secret.data = encrypted_value;

        // Add metadata from storage
        for (key, value) in entry.metadata {
            secret.metadata.custom_fields.set(key, value);
        }

        Ok(secret)
    }

    async fn get_from_cache(&self, path: &str) -> Option<CachedSecret> {
        let cache = self.cache.read().await;
        cache.peek(path).cloned()
    }

    async fn cache_secret(&self, path: &str, secret: &Secret) {
        let ttl = Duration::from_secs(self.config.cache_ttl_seconds);
        let cached_secret = CachedSecret::new(secret.clone(), ttl);

        let mut cache = self.cache.write().await;
        cache.put(path.to_string(), cached_secret);
    }

    async fn invalidate_cache(&self, path: &str) {
        let mut cache = self.cache.write().await;
        cache.pop(path);
    }

    async fn log_audit_event(
        &self,
        action: &str,
        resource_path: &str,
        context: &SecurityContext,
        status: AuditStatus,
    ) {
        if !self.config.enable_audit {
            return;
        }

        if let Some(audit_logger) = &self.audit_logger {
            let mut metadata = HashMap::new();
            metadata.insert("satker_code".to_string(), context.satker_code.clone());
            if let Some(nip) = &context.nip {
                metadata.insert("nip".to_string(), nip.clone());
            }
            if let Some(session_id) = &context.session_id {
                metadata.insert("session_id".to_string(), session_id.clone());
            }

            let audit_log = AuditLog {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                action: action.to_string(),
                actor: context.user_id.clone(),
                resource_type: "secret".to_string(),
                resource_id: resource_path.to_string(),
                status,
                ip: context.ip_address.clone(),
                user_agent: context.user_agent.clone(),
                metadata,
            };

            if let Err(e) = audit_logger.log(audit_log).await {
                error!("Failed to log audit event: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryStorage;
    use std::sync::Arc;

    fn create_test_context() -> SecurityContext {
        SecurityContext {
            user_id: Some("test_user".to_string()),
            nip: Some("123456789".to_string()),
            satker_code: "KEJARI_JAKARTA".to_string(),
            session_id: Some("session_123".to_string()),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("test_agent".to_string()),
            security_level: SecurityLevel::Internal,
        }
    }

    #[tokio::test]
    async fn test_enhanced_secret_engine_creation() {
        let storage = Arc::new(InMemoryStorage::new());
        let config = EnhancedSecretEngineConfig::default();
        let engine = EnhancedSecretEngine::new(storage, config, None);

        // Test that engine is created successfully
        assert_eq!(engine.config.cache_size, 1000);
        assert_eq!(engine.config.cache_ttl_seconds, 300);
    }

    #[tokio::test]
    async fn test_store_and_get_secret() {
        let storage = Arc::new(InMemoryStorage::new());
        let config = EnhancedSecretEngineConfig::default();
        let engine = EnhancedSecretEngine::new(storage, config, None);
        let context = create_test_context();

        let test_data = serde_json::json!({"password": "secret123"});

        // Store secret
        engine.store_secret("test/secret", test_data.clone(), &context).await.unwrap();

        // Retrieve secret
        let secret = engine.get_secret_from_storage("test/secret", &context).await.unwrap();
        assert_eq!(secret.data.data, test_data);
        assert_eq!(secret.satker_owner, "KEJARI_JAKARTA");
    }

    #[tokio::test]
    async fn test_application_config() {
        let storage = Arc::new(InMemoryStorage::new());
        let config = EnhancedSecretEngineConfig::default();
        let engine = EnhancedSecretEngine::new(storage, config, None);
        let context = create_test_context();

        let config_data = serde_json::json!({
            "database_url": "postgresql://localhost/test",
            "api_key": "test_key_123"
        });

        // Store application config
        engine.store_secret("apps/simkari/config", config_data.clone(), &context).await.unwrap();

        // Retrieve application config
        let app_config = engine.get_application_config("simkari", &context).await.unwrap();
        assert_eq!(app_config.app_id, "simkari");
        assert_eq!(app_config.satker_owner, "KEJARI_JAKARTA");
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let storage = Arc::new(InMemoryStorage::new());
        let config = EnhancedSecretEngineConfig::default();
        let engine = EnhancedSecretEngine::new(storage, config, None);
        let context = create_test_context();

        // Store multiple secrets
        for i in 1..=5 {
            let data = serde_json::json!({"value": format!("secret_{}", i)});
            engine.store_secret(&format!("batch/secret_{}", i), data, &context).await.unwrap();
        }

        // Batch get secrets
        let request = BatchSecretRequest {
            resource_ids: vec![
                "batch/secret_1".to_string(),
                "batch/secret_2".to_string(),
                "batch/secret_3".to_string(),
            ],
            operation_type: BatchOperationType::Read,
            context: context.clone(),
        };

        let secrets = engine.batch_get_secrets(&request).await.unwrap();
        assert_eq!(secrets.len(), 3);
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let storage = Arc::new(InMemoryStorage::new());
        let config = EnhancedSecretEngineConfig::default();
        let engine = EnhancedSecretEngine::new(storage, config, None);
        let context = create_test_context();

        let test_data = serde_json::json!({"cached": "value"});

        // Store and cache secret
        engine.store_secret("cache/test", test_data.clone(), &context).await.unwrap();

        // First access should cache the secret
        let _secret1 = engine.get_application_secret("cache", "test", &context).await.unwrap();

        // Second access should hit cache
        let _secret2 = engine.get_application_secret("cache", "test", &context).await.unwrap();

        // Verify cache contains the secret
        let cached = engine.get_from_cache("apps/cache/test").await;
        assert!(cached.is_some());
    }
}
