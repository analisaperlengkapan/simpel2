//! Service container for dependency injection.
//!
//! Provides centralized access to all application services
//! including storage, crypto, authentication, and business logic.

pub mod auth;
pub mod vault;
pub mod admin;

use std::sync::Arc;
use anyhow::Result;

use crate::config::ApiConfig;
use secreton_core::audit::AuditLogger;
use secreton_core::namespace::NamespaceService;
use secreton_core::services::seal::{SealService, SealConfig, VaultState, VaultStateStorage};
use secreton_core::services::secrets::database::DatabaseSecretsEngine;
use secreton_core::services::lease::LeaseManager;
use secreton_core::services::policy::PolicySet;
use secreton_core::services::wrapping::WrappingService;
use secreton_core::hsm::HsmBackend;
use secreton_crypto::{CryptoEngine, SecurityParams};
use secreton_storage::StorageBackend;
use std::sync::RwLock;

/// Storage adapter for SealService to use StorageBackend
/// This bridges the VaultStateStorage trait with the StorageBackend trait
pub struct SealStorageAdapter {
    storage: Arc<dyn StorageBackend + Send + Sync>,
}

impl SealStorageAdapter {
    pub fn new(storage: Arc<dyn StorageBackend + Send + Sync>) -> Self {
        Self { storage }
    }
}

#[async_trait::async_trait]
impl VaultStateStorage for SealStorageAdapter {
    async fn store_vault_state(&self, state: &VaultState) -> Result<(), String> {
        // Serialize vault state to JSON
        let json_data = serde_json::to_vec(state)
            .map_err(|e| format!("Failed to serialize vault state: {}", e))?;

        // Store in storage backend with a special key
        // For now, we'll use a simple in-memory approach
        // TODO: Implement proper persistence using storage backend
        tracing::info!("Storing vault state (size: {} bytes)", json_data.len());

        // Store as a special entry in the storage backend
        // This is a simplified implementation - in production, you'd want a dedicated table
        Ok(())
    }

    async fn load_vault_state(&self) -> Result<Option<VaultState>, String> {
        // Load vault state from storage backend
        // TODO: Implement proper loading using storage backend
        tracing::debug!("Loading vault state from storage");

        // For now, return None (vault not initialized)
        // In production, this would query the storage backend
        Ok(None)
    }
}

/// Service container holding all application services
pub struct ServiceContainer {
    /// Configuration
    pub config: ApiConfig,

    /// Storage service
    pub storage: Arc<dyn StorageBackend + Send + Sync>,

    /// Database connection pool (for direct SQL access)
    pub pool: deadpool_postgres::Pool,

    /// Cryptographic service
    pub crypto: Arc<CryptoEngine>,

    /// Authentication service
    pub auth: Arc<auth::AuthService>,

    /// Vault service
    pub vault: Arc<vault::VaultService>,

    /// Admin service
    pub admin: Arc<admin::AdminService>,

    /// Audit logger
    pub audit: Arc<AuditLogger>,

    /// Seal/Unseal service
    pub seal: Arc<SealService>,

    /// Namespace service
    pub namespace: Arc<NamespaceService>,

    /// Database secrets engine
    pub database_engine: Arc<DatabaseSecretsEngine>,

    /// Lease manager
    pub lease_manager: Arc<LeaseManager>,

    /// Policy service for authorization
    pub policy: Arc<RwLock<PolicySet>>,

    /// Response wrapping service
    pub wrapping_service: Arc<WrappingService>,

    /// HSM backend (optional)
    pub hsm: Option<Arc<HsmBackend>>,
}

impl ServiceContainer {
    /// Create new service container
    pub async fn new(config: &ApiConfig) -> Result<Self> {
        // Initialize storage backend
        let storage = Self::create_storage_backend(config).await?;

        // Initialize database connection pool
        let pool = Self::create_database_pool(config).await?;

        // Initialize crypto service
        let crypto = Arc::new(CryptoEngine::new());

        // Initialize audit logger (memory/raft backends currently); Postgres wiring removed due to trait divergence
        // TODO: Create proper audit backends
        let audit = Arc::new(AuditLogger::new(vec![]));

        // Initialize authentication service
        let auth = Arc::new(auth::AuthService::new(
            storage.clone(),
            crypto.clone(),
            &config.auth,
        ).await?);

        // Initialize vault service
        let vault = Arc::new(vault::VaultService::new(
            storage.clone(),
            crypto.clone(),
            audit.clone(),
        ).await?);

        // Initialize admin service
        let admin = Arc::new(admin::AdminService::new(
            storage.clone(),
            auth.clone(),
            audit.clone(),
        ).await?);

        // Initialize seal/unseal service
        let seal_config = SealConfig {
            seal_type: "shamir".to_string(),
            secret_shares: std::env::var("SECRETON_SEAL_SHARES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            secret_threshold: std::env::var("SECRETON_SEAL_THRESHOLD")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3),
            created_at: chrono::Utc::now(),
        };

        // Create storage adapter for SealService
        let seal_storage = Arc::new(SealStorageAdapter::new(storage.clone()));
        let seal = Arc::new(SealService::with_storage(seal_config, seal_storage));

        // CRITICAL: Load vault state from storage on startup
        // This checks if vault is initialized and loads seal configuration
        match seal.load_from_storage().await {
            Ok(true) => {
                tracing::info!("✅ Vault state loaded from storage. Vault is SEALED.");
                tracing::info!("   Operators must unseal with threshold shares before vault can be used.");
            }
            Ok(false) => {
                tracing::warn!("⚠️  Vault not initialized. Use /v1/sys/init to initialize.");
            }
            Err(e) => {
                tracing::error!("❌ Failed to load vault state: {:?}", e);
                tracing::warn!("   Continuing with uninitialized vault.");
            }
        }

        // Check seal status and log
        if seal.is_sealed().await {
            tracing::warn!("🔒 Vault is SEALED. All secret operations will be blocked until unsealed.");
        } else {
            tracing::info!("🔓 Vault is UNSEALED. Secret operations are allowed.");
        }

        // Initialize namespace service
        let namespace = Arc::new(NamespaceService::new(
            "Kejaksaan Agung RI".to_string(),
            "system".to_string(),
        ));

        // TODO: Load namespace hierarchy from storage on startup
        tracing::info!("✅ Namespace service initialized with root namespace");

        // Initialize database secrets engine
        let database_engine = Arc::new(DatabaseSecretsEngine::new());
        tracing::info!("✅ Database secrets engine initialized");

        // Initialize lease manager
        let lease_manager = Arc::new(LeaseManager::new(pool.clone()));
        tracing::info!("✅ Lease manager initialized");

        // Initialize policy service with default policies
        // TODO: Load policies from storage on startup
        let default_policies = vec![];
        let policy = Arc::new(RwLock::new(PolicySet::new(default_policies)));
        tracing::info!("✅ Policy service initialized");

        // Initialize wrapping service
        let wrapping_service = Arc::new(WrappingService::new(pool.clone()));
        tracing::info!("✅ Response wrapping service initialized");

        // Initialize HSM backend (optional)
        let hsm = if config.hsm.enabled {
            tracing::info!("Initializing HSM backend...");
            match HsmBackend::new(config.hsm.clone()) {
                Ok(hsm_backend) => {
                    match hsm_backend.initialize().await {
                        Ok(()) => {
                            tracing::info!("✅ HSM backend initialized successfully");
                            Some(Arc::new(hsm_backend))
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to initialize HSM backend: {:?}", e);
                            tracing::warn!("   Continuing without HSM support");
                            None
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("❌ Failed to create HSM backend: {:?}", e);
                    tracing::warn!("   Continuing without HSM support");
                    None
                }
            }
        } else {
            tracing::info!("HSM integration is disabled");
            None
        };

        Ok(Self {
            config: config.clone(),
            storage,
            pool,
            crypto,
            auth,
            vault,
            admin,
            audit,
            seal,
            namespace,
            database_engine,
            lease_manager,
            policy,
            wrapping_service,
            hsm,
        })
    }

    /// Create storage backend based on configuration
    async fn create_storage_backend(
        config: &ApiConfig,
    ) -> Result<Arc<dyn StorageBackend + Send + Sync>> {
        use secreton_storage::MemoryBackend;
        #[cfg(feature = "raft-consensus")]
        use secreton_storage::{RaftCluster, RaftClusterConfig};

        // Get storage backend type from config or environment
        let backend_type = std::env::var("Secreton_STORAGE_BACKEND")
            .unwrap_or_else(|_| "memory".to_string());

        tracing::info!("Initializing storage backend: {}", backend_type);

        match backend_type.as_str() {
            "raft" | "integrated" => {
                // TODO: Implement Raft storage backend
                tracing::warn!("Raft backend not yet fully implemented, falling back to memory");
                Ok(Arc::new(MemoryBackend::new()))
            }

            "memory" | "mock" => {
                tracing::info!("Using in-memory mock storage backend");
                Ok(Arc::new(MemoryBackend::new()))
            }

            // TODO: Add other backends (PostgreSQL, Redis, File)
            "postgres" => {
                tracing::warn!("PostgreSQL backend not yet implemented, falling back to memory");
                Ok(Arc::new(MemoryBackend::new()))
            }

            "redis" => {
                tracing::warn!("Redis backend not yet implemented, falling back to memory");
                Ok(Arc::new(MemoryBackend::new()))
            }

            "file" => {
                tracing::warn!("File backend not yet implemented, falling back to memory");
                Ok(Arc::new(MemoryBackend::new()))
            }

            _ => {
                tracing::warn!("Unknown storage backend '{}', falling back to memory", backend_type);
                Ok(Arc::new(MemoryBackend::new()))
            }
        }
    }

    /// Create database connection pool
    async fn create_database_pool(config: &ApiConfig) -> Result<deadpool_postgres::Pool> {
        use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
        use tokio_postgres::NoTls;

        let mut cfg = Config::new();
        cfg.host = Some(config.database.host.clone());
        cfg.port = Some(config.database.port);
        cfg.dbname = Some(config.database.database.clone());
        cfg.user = Some(config.database.username.clone());
        cfg.password = Some(config.database.password.clone());
        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });

        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
        tracing::info!("✅ Database connection pool created");
        Ok(pool)
    }

    /// Create a mock service container for testing/gRPC
    pub fn new_mock(storage: Arc<dyn StorageBackend + Send + Sync>, pool: deadpool_postgres::Pool) -> Self {
        let crypto = Arc::new(CryptoEngine::new());
        let audit = Arc::new(AuditLogger::new(vec![]));

        let seal_config = SealConfig {
            seal_type: "shamir".to_string(),
            secret_shares: 5,
            secret_threshold: 3,
            created_at: chrono::Utc::now(),
        };

        let seal_storage = Arc::new(SealStorageAdapter::new(storage.clone()));
        let seal = Arc::new(SealService::with_storage(seal_config, seal_storage));

        let namespace = Arc::new(NamespaceService::new(
            "Kejaksaan Agung RI".to_string(),
            "system".to_string(),
        ));

        let database_engine = Arc::new(DatabaseSecretsEngine::new());
        let lease_manager = Arc::new(LeaseManager::new(pool.clone()));

        // Initialize policy service for mock
        let default_policies = vec![];
        let policy = Arc::new(RwLock::new(PolicySet::new(default_policies)));

        // Initialize wrapping service for mock
        let wrapping_service = Arc::new(WrappingService::new(pool.clone()));

        Self {
            config: ApiConfig::default(),
            storage: storage.clone(),
            pool,
            crypto: crypto.clone(),
            auth: Arc::new(auth::AuthService::new_mock(storage.clone(), crypto.clone())),
            vault: Arc::new(vault::VaultService::new_mock(storage.clone(), crypto.clone())),
            admin: Arc::new(admin::AdminService::new_mock(storage.clone())),
            audit,
            seal,
            namespace,
            database_engine,
            lease_manager,
            policy,
            wrapping_service,
            hsm: None, // No HSM in mock
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_container_creation() {
        let config = ApiConfig::default();
        let container = ServiceContainer::new(&config).await;
        assert!(container.is_ok());
    }
}
