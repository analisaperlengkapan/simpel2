//! Service container for dependency injection.
//!
//! Provides centralized access to all application services
//! including storage, crypto, authentication, and business logic.

pub mod admin;
pub mod auth;
pub mod seal_adapter;
pub mod vault;

use anyhow::Result;
use std::sync::Arc;

use crate::config::ApiConfig;
use secreton_core::audit::AuditLogger;
use secreton_core::namespace::NamespaceService;
use secreton_core::services::identity::IdentityService;
use secreton_core::services::lease::LeaseManager;
use secreton_core::services::policy::PolicySet;
use secreton_core::services::rotation::AutoRotationEngine;
use secreton_core::services::seal::{SealConfig, SealService};
use secreton_core::services::secrets::aws::AwsEngine;
use secreton_core::services::secrets::azure::AzureEngine;
use secreton_core::services::secrets::database::DatabaseSecretsEngine;
use secreton_core::services::secrets::gcp::GcpEngine;
use secreton_core::services::secrets::identity::IdentityEngine;
use secreton_core::services::secrets::kafka::KafkaEngine;
use secreton_core::services::secrets::kmip::KmipEngine;
use secreton_core::services::secrets::ldap::LdapEngine;
use secreton_core::services::secrets::rabbitmq::RabbitMqEngine;
use secreton_core::services::secrets::ssh::SshEngine;
use secreton_core::services::secrets::totp::TotpEngine;
use secreton_core::services::secrets::transform::TransformEngine;
use secreton_core::services::wrapping::WrappingService;
use secreton_crypto::CryptoEngine;
use secreton_hsm::HsmBackend;
use secreton_storage::StorageBackend;
use std::sync::RwLock;

// Re-export SealStorageAdapter for backward compatibility
pub use seal_adapter::SealStorageAdapter;

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

    /// TOTP secrets engine
    pub totp_engine: Arc<TotpEngine>,

    /// Transform secrets engine
    pub transform_engine: Arc<TransformEngine>,

    /// SSH secrets engine
    pub ssh_engine: Arc<SshEngine>,

    /// AWS secrets engine
    pub aws_engine: Arc<AwsEngine>,

    /// GCP secrets engine
    pub gcp_engine: Arc<GcpEngine>,

    /// Azure secrets engine
    pub azure_engine: Arc<AzureEngine>,

    /// Identity service (entity/group management)
    pub identity_service: Arc<IdentityService>,

    /// Identity secrets engine (OIDC Provider)
    pub identity_engine: Arc<IdentityEngine>,

    /// KMIP secrets engine
    pub kmip_engine: Arc<KmipEngine>,

    /// LDAP secrets engine
    pub ldap_engine: Arc<LdapEngine>,

    /// RabbitMQ secrets engine
    pub rabbitmq_engine: Arc<RabbitMqEngine>,

    /// Kafka secrets engine
    pub kafka_engine: Arc<KafkaEngine>,

    /// Auto-rotation engine
    pub rotation_engine: Arc<AutoRotationEngine>,

    /// Lease manager
    pub lease_manager: Arc<LeaseManager>,

    /// Policy service for authorization
    pub policy: Arc<RwLock<PolicySet>>,

    /// Response wrapping service
    pub wrapping_service: Arc<WrappingService>,

    /// HSM backend (optional)
    pub hsm: Option<Arc<HsmBackend>>,

    /// MFA service for multi-factor authentication
    pub mfa: Arc<secreton_core::services::mfa::MfaService>,
}

impl ServiceContainer {
    /// Create new service container
    pub async fn new(config: &ApiConfig) -> Result<Self> {
        // Initialize core infrastructure
        let (storage, pool, crypto, audit) = Self::initialize_core_services(config).await?;

        // Initialize authentication services
        let (auth, admin) =
            Self::initialize_auth_services(config, storage.clone(), crypto.clone()).await?;

        // Initialize vault and seal services
        let (vault, seal, namespace) =
            Self::initialize_vault_services(config, storage.clone(), crypto.clone(), audit.clone())
                .await?;

        // Initialize secrets engines and policies
        let (
            database_engine,
            totp_engine,
            transform_engine,
            ssh_engine,
            aws_engine,
            gcp_engine,
            azure_engine,
            identity_service,
            identity_engine,
            kmip_engine,
            ldap_engine,
            rabbitmq_engine,
            kafka_engine,
            rotation_engine,
            lease_manager,
            policy,
            wrapping_service,
        ) = Self::initialize_secrets_engines(pool.clone());

        // Initialize optional services (HSM, MFA) - share TotpEngine
        let (hsm, mfa) = Self::initialize_optional_services(config, totp_engine.clone()).await;

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
            totp_engine,
            transform_engine,
            ssh_engine,
            aws_engine,
            gcp_engine,
            azure_engine,
            identity_service,
            identity_engine,
            kmip_engine,
            ldap_engine,
            rabbitmq_engine,
            kafka_engine,
            rotation_engine,
            lease_manager,
            policy,
            wrapping_service,
            hsm,
            mfa,
        })
    }

    /// Initialize core infrastructure services
    async fn initialize_core_services(
        config: &ApiConfig,
    ) -> Result<(
        Arc<dyn StorageBackend + Send + Sync>,
        deadpool_postgres::Pool,
        Arc<CryptoEngine>,
        Arc<AuditLogger>,
    )> {
        let storage = Self::create_storage_backend(config).await?;
        let pool = Self::create_database_pool(config).await?;
        let crypto = Arc::new(CryptoEngine::new());
        let audit = Arc::new(AuditLogger::new(vec![]));

        Ok((storage, pool, crypto, audit))
    }

    /// Initialize authentication services
    async fn initialize_auth_services(
        config: &ApiConfig,
        storage: Arc<dyn StorageBackend + Send + Sync>,
        crypto: Arc<CryptoEngine>,
    ) -> Result<(Arc<auth::AuthService>, Arc<admin::AdminService>)> {
        let auth =
            Arc::new(auth::AuthService::new(storage.clone(), crypto.clone(), &config.auth).await?);

        let api_audit = Arc::new(crate::audit::AuditLogger::new(10000));
        let admin =
            Arc::new(admin::AdminService::new(storage.clone(), auth.clone(), api_audit).await?);

        Ok((auth, admin))
    }

    /// Initialize vault and seal services
    async fn initialize_vault_services(
        config: &ApiConfig,
        storage: Arc<dyn StorageBackend + Send + Sync>,
        crypto: Arc<CryptoEngine>,
        audit: Arc<AuditLogger>,
    ) -> Result<(
        Arc<vault::VaultService>,
        Arc<SealService>,
        Arc<NamespaceService>,
    )> {
        // Initialize vault service
        let vault = Arc::new(
            vault::VaultService::new(storage.clone(), crypto.clone(), audit.clone()).await?,
        );

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

        let seal_storage = Arc::new(SealStorageAdapter::new(storage.clone()));
        let seal = Arc::new(SealService::with_storage(seal_config, seal_storage));

        // Load vault state from storage
        match seal.load_from_storage().await {
            Ok(true) => {
                tracing::info!("✅ Vault state loaded from storage. Vault is SEALED.");
                tracing::info!(
                    "   Operators must unseal with threshold shares before vault can be used."
                );
            }
            Ok(false) => {
                tracing::warn!("⚠️  Vault not initialized. Use /v1/sys/init to initialize.");
            }
            Err(e) => {
                tracing::error!("❌ Failed to load vault state: {:?}", e);
                tracing::warn!("   Continuing with uninitialized vault.");
            }
        }

        // Check seal status
        if seal.is_sealed().await {
            tracing::warn!(
                "🔒 Vault is SEALED. All secret operations will be blocked until unsealed."
            );
        } else {
            tracing::info!("🔓 Vault is UNSEALED. Secret operations are allowed.");
        }

        // Initialize namespace service
        let namespace = Arc::new(NamespaceService::new(
            "Kejaksaan Agung RI".to_string(),
            "system".to_string(),
        ));
        tracing::info!("✅ Namespace service initialized with root namespace");

        Ok((vault, seal, namespace))
    }

    /// Initialize secrets engines and policy services
    fn initialize_secrets_engines(
        pool: deadpool_postgres::Pool,
    ) -> (
        Arc<DatabaseSecretsEngine>,
        Arc<TotpEngine>,
        Arc<TransformEngine>,
        Arc<SshEngine>,
        Arc<AwsEngine>,
        Arc<GcpEngine>,
        Arc<AzureEngine>,
        Arc<IdentityService>,
        Arc<IdentityEngine>,
        Arc<KmipEngine>,
        Arc<LdapEngine>,
        Arc<RabbitMqEngine>,
        Arc<KafkaEngine>,
        Arc<AutoRotationEngine>,
        Arc<LeaseManager>,
        Arc<RwLock<PolicySet>>,
        Arc<WrappingService>,
    ) {
        let database_engine = Arc::new(DatabaseSecretsEngine::new());
        tracing::info!("✅ Database secrets engine initialized");

        let totp_engine = Arc::new(TotpEngine::new());
        tracing::info!("✅ TOTP secrets engine initialized");

        let transform_engine = Arc::new(TransformEngine::with_storage(pool.clone()));
        tracing::info!("✅ Transform secrets engine initialized");

        let ssh_engine = Arc::new(SshEngine::with_storage(pool.clone()));
        tracing::info!("✅ SSH secrets engine initialized");

        let aws_engine = Arc::new(AwsEngine::new());
        tracing::info!("✅ AWS secrets engine initialized");

        let gcp_engine = Arc::new(GcpEngine::new());
        tracing::info!("✅ GCP secrets engine initialized");

        let azure_engine = Arc::new(AzureEngine::new());
        tracing::info!("✅ Azure secrets engine initialized");

        let identity_service = Arc::new(IdentityService::new());
        tracing::info!("✅ Identity service initialized");

        let identity_engine = Arc::new(IdentityEngine::new(identity_service.clone()));
        tracing::info!("✅ Identity secrets engine (OIDC Provider) initialized");

        let kmip_engine = Arc::new(KmipEngine::with_storage(pool.clone()));
        tracing::info!("✅ KMIP secrets engine initialized");

        let ldap_engine = Arc::new(LdapEngine::with_storage(pool.clone()));
        tracing::info!("✅ LDAP secrets engine initialized");

        let rabbitmq_engine = Arc::new(RabbitMqEngine::with_storage(pool.clone()));
        tracing::info!("✅ RabbitMQ secrets engine initialized");

        let kafka_engine = Arc::new(KafkaEngine::with_storage(pool.clone()));
        tracing::info!("✅ Kafka secrets engine initialized");

        let rotation_engine = Arc::new(AutoRotationEngine::new());
        tracing::info!("✅ Auto-rotation engine initialized");

        let lease_manager = Arc::new(LeaseManager::new(pool.clone()));
        tracing::info!("✅ Lease manager initialized");

        let default_policies = vec![];
        let policy = Arc::new(RwLock::new(PolicySet::new(default_policies)));
        tracing::info!("✅ Policy service initialized");

        let wrapping_service = Arc::new(WrappingService::new(pool));
        tracing::info!("✅ Response wrapping service initialized");

        (
            database_engine,
            totp_engine,
            transform_engine,
            ssh_engine,
            aws_engine,
            gcp_engine,
            azure_engine,
            identity_service,
            identity_engine,
            kmip_engine,
            ldap_engine,
            rabbitmq_engine,
            kafka_engine,
            rotation_engine,
            lease_manager,
            policy,
            wrapping_service,
        )
    }

    /// Initialize optional services (HSM, MFA)
    async fn initialize_optional_services(
        config: &ApiConfig,
        totp_engine: Arc<TotpEngine>,
    ) -> (
        Option<Arc<HsmBackend>>,
        Arc<secreton_core::services::mfa::MfaService>,
    ) {
        // Initialize HSM backend (optional)
        let hsm = if config.hsm.enabled {
            tracing::info!("Initializing HSM backend...");
            match HsmBackend::new(config.hsm.clone()) {
                Ok(hsm_backend) => match hsm_backend.initialize().await {
                    Ok(()) => {
                        tracing::info!("✅ HSM backend initialized successfully");
                        Some(Arc::new(hsm_backend))
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to initialize HSM backend: {:?}", e);
                        tracing::warn!("   Continuing without HSM support");
                        None
                    }
                },
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

        // Initialize MFA service with shared TotpEngine
        let mfa = Arc::new(secreton_core::services::mfa::MfaService::with_totp_engine(
            totp_engine,
        ));
        tracing::info!("✅ MFA service initialized with shared TotpEngine");

        (hsm, mfa)
    }

    /// Create storage backend based on configuration
    async fn create_storage_backend(
        config: &ApiConfig,
    ) -> Result<Arc<dyn StorageBackend + Send + Sync>> {
        use secreton_storage::MemoryBackend;
        #[cfg(feature = "raft-consensus")]
        use secreton_storage::{RaftCluster, RaftClusterConfig};

        // Get storage backend type from config or environment
        let backend_type =
            std::env::var("Secreton_STORAGE_BACKEND").unwrap_or_else(|_| "memory".to_string());

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
                tracing::warn!(
                    "Unknown storage backend '{}', falling back to memory",
                    backend_type
                );
                Ok(Arc::new(MemoryBackend::new()))
            }
        }
    }

    /// Create database connection pool
    async fn create_database_pool(config: &ApiConfig) -> Result<deadpool_postgres::Pool> {
        use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Runtime};
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
    pub fn new_mock(
        storage: Arc<dyn StorageBackend + Send + Sync>,
        pool: deadpool_postgres::Pool,
    ) -> Self {
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
        let totp_engine = Arc::new(TotpEngine::new());
        let transform_engine = Arc::new(TransformEngine::with_storage(pool.clone()));
        let ssh_engine = Arc::new(SshEngine::with_storage(pool.clone()));
        let aws_engine = Arc::new(AwsEngine::new());
        let gcp_engine = Arc::new(GcpEngine::new());
        let azure_engine = Arc::new(AzureEngine::new());

        let identity_service = Arc::new(IdentityService::new());
        let identity_engine = Arc::new(IdentityEngine::new(identity_service.clone()));

        let kmip_engine = Arc::new(KmipEngine::with_storage(pool.clone()));
        let ldap_engine = Arc::new(LdapEngine::with_storage(pool.clone()));
        let rabbitmq_engine = Arc::new(RabbitMqEngine::with_storage(pool.clone()));
        let kafka_engine = Arc::new(KafkaEngine::with_storage(pool.clone()));
        let rotation_engine = Arc::new(AutoRotationEngine::new());

        let lease_manager = Arc::new(LeaseManager::new(pool.clone()));

        // Initialize policy service for mock
        let default_policies = vec![];
        let policy = Arc::new(RwLock::new(PolicySet::new(default_policies)));

        // Initialize wrapping service for mock
        let wrapping_service = Arc::new(WrappingService::new(pool.clone()));

        // Initialize MFA service with shared TotpEngine
        let mfa = Arc::new(secreton_core::services::mfa::MfaService::with_totp_engine(
            totp_engine.clone(),
        ));
        tracing::info!("✅ MFA service initialized with shared TotpEngine (mock mode)");

        Self {
            config: ApiConfig::default(),
            storage: storage.clone(),
            pool,
            crypto: crypto.clone(),
            auth: Arc::new(auth::AuthService::new_mock(storage.clone(), crypto.clone())),
            vault: Arc::new(vault::VaultService::new_mock(
                storage.clone(),
                crypto.clone(),
            )),
            admin: Arc::new(admin::AdminService::new_mock(storage.clone())),
            audit,
            seal,
            namespace,
            database_engine,
            totp_engine,
            transform_engine,
            ssh_engine,
            aws_engine,
            gcp_engine,
            azure_engine,
            identity_service,
            identity_engine,
            kmip_engine,
            ldap_engine,
            rabbitmq_engine,
            kafka_engine,
            rotation_engine,
            lease_manager,
            policy,
            wrapping_service,
            hsm: None, // No HSM in mock
            mfa,
        }
    }
}

#[cfg(all(test, feature = "enable-inline-tests"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_container_creation() {
        let config = ApiConfig::default();
        let container = ServiceContainer::new(&config).await;
        assert!(container.is_ok());
    }
}
