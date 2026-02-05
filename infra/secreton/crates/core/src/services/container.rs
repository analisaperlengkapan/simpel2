//! Service container for dependency injection.
//!
//! Provides centralized access to all application services
//! including storage, crypto, authentication, and business logic.

use anyhow::Result;
use std::sync::Arc;

use crate::audit::AuditLogger;
use crate::config::api::ApiConfig;
use crate::namespace::NamespaceService;
use crate::services::admin_service::AdminService;
use crate::services::auth_service::AuthService;
use crate::services::identity::IdentityService;
use crate::services::lease::LeaseManager;
use crate::services::namespace_persistence;
use crate::services::policy::PolicySet;
use crate::services::policy_service::PolicyService;
use crate::services::rotation::AutoRotationEngine;
use crate::services::seal::{SealConfig, SealService};
use crate::services::seal_adapter::SealStorageAdapter;
use crate::services::secret_service::SecretService;
use crate::services::secrets::aws::AwsEngine;
use crate::services::secrets::azure::AzureEngine;
use crate::services::secrets::database::DatabaseSecretsEngine;
use crate::services::secrets::gcp::GcpEngine;
use crate::services::secrets::identity::IdentityEngine;
use crate::services::secrets::kafka::KafkaEngine;
use crate::services::secrets::kmip::KmipEngine;
use crate::services::secrets::ldap::LdapEngine;
use crate::services::secrets::rabbitmq::RabbitMqEngine;
use crate::services::secrets::ssh::SshEngine;
use crate::services::secrets::totp::TotpEngine;
use crate::services::secrets::transform::TransformEngine;
use crate::services::wrapping::WrappingService;

use secreton_crypto::CryptoEngine;
use secreton_hsm::HsmBackend;
use secreton_storage::StorageBackend;
use std::sync::RwLock;

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
    pub auth: Arc<AuthService>,

    /// Engine service
    pub engine: Arc<SecretService>,

    /// Admin service
    pub admin: Arc<AdminService>,

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

    /// Policy management service (CRUD)
    pub policy_service: Arc<PolicyService>,

    /// Response wrapping service
    pub wrapping_service: Arc<WrappingService>,

    /// HSM backend (optional)
    pub hsm: Option<Arc<HsmBackend>>,

    /// MFA service for multi-factor authentication
    pub mfa: Arc<crate::services::mfa::MfaService>,
}

impl ServiceContainer {
    /// Create new service container
    pub async fn new(config: &ApiConfig) -> Result<Self> {
        // Initialize core infrastructure
        let (storage, pool, crypto, audit) = Self::initialize_core_services(config).await?;

        // Initialize lease manager early (needed for admin service)
        let lease_manager = Arc::new(LeaseManager::new(pool.clone()));
        tracing::info!("✅ Lease manager initialized");

        // Initialize authentication services
        let (auth, admin) = Self::initialize_auth_services(
            config,
            storage.clone(),
            crypto.clone(),
            lease_manager.clone(),
        )
        .await?;

        // Initialize engine and seal services
        let (engine, seal, namespace) = Self::initialize_secret_services(
            config,
            storage.clone(),
            crypto.clone(),
            audit.clone(),
        )
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
            policy_service,
            wrapping_service,
        ) = Self::initialize_secrets_engines(pool.clone(), storage.clone());

        // Initialize optional services (HSM, MFA) - share TotpEngine
        let (hsm, mfa) = Self::initialize_optional_services(
            config,
            storage.clone(),
            crypto.clone(),
            totp_engine.clone(),
        )
        .await;

        Ok(Self {
            config: config.clone(),
            storage,
            pool,
            crypto,
            auth,
            engine,
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
            policy_service,
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
        lease_manager: Arc<LeaseManager>,
    ) -> Result<(Arc<AuthService>, Arc<AdminService>)> {
        let auth: Arc<AuthService> =
            Arc::new(AuthService::new(storage.clone(), crypto.clone(), &config.auth).await?);

        let api_audit = Arc::new(crate::audit::api_audit::AuditLogger::new(10000));
        let admin = Arc::new(
            AdminService::new(storage.clone(), auth.clone(), api_audit, lease_manager).await?,
        );

        Ok((auth, admin))
    }

    /// Initialize engine and seal services
    async fn initialize_secret_services(
        config: &ApiConfig,
        storage: Arc<dyn StorageBackend + Send + Sync>,
        crypto: Arc<CryptoEngine>,
        audit: Arc<AuditLogger>,
    ) -> Result<(Arc<SecretService>, Arc<SealService>, Arc<NamespaceService>)> {
        // Initialize engine service
        let engine =
            Arc::new(SecretService::new(storage.clone(), crypto.clone(), audit.clone()).await?);

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

        // Load engine state from storage
        match seal.load_from_storage().await {
            Ok(true) => {
                tracing::info!("✅ Engine state loaded from storage. Engine is SEALED.");
                tracing::info!(
                    "   Operators must unseal with threshold shares before engine can be used."
                );
            }
            Ok(false) => {
                tracing::warn!("⚠️  Engine not initialized. Use /v1/sys/init to initialize.");
            }
            Err(e) => {
                tracing::error!("❌ Failed to load engine state: {:?}", e);
                tracing::warn!("   Continuing with uninitialized engine.");
            }
        }

        // Check seal status
        if seal.is_sealed().await {
            tracing::warn!(
                "🔒 Engine is SEALED. All secret operations will be blocked until unsealed."
            );
        } else {
            tracing::info!("🔓 Engine is UNSEALED. Secret operations are allowed.");
        }

        // Initialize namespace service
        let namespace = Arc::new(NamespaceService::new(
            "Kejaksaan Agung RI".to_string(),
            "system".to_string(),
        ));

        // Load namespace hierarchy from storage
        match namespace_persistence::load_hierarchy(&storage, &crypto, &config.auth.jwt.secret)
            .await
        {
            Ok(Some(hierarchy)) => {
                namespace.update_hierarchy(hierarchy);
                tracing::info!("✅ Namespace hierarchy loaded from storage");
            }
            Ok(None) => {
                tracing::info!("✅ Namespace service initialized with default root namespace");
            }
            Err(e) => {
                tracing::error!("❌ Failed to load namespace hierarchy: {:?}", e);
                panic!("Failed to load namespace hierarchy from storage: {:?}", e);
            }
        }

        Ok((engine, seal, namespace))
    }

    /// Initialize secrets engines and policy services
    fn initialize_secrets_engines(
        pool: deadpool_postgres::Pool,
        storage: Arc<dyn StorageBackend + Send + Sync>,
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
        Arc<PolicyService>,
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

        let policy_service = Arc::new(PolicyService::new(storage));
        tracing::info!("✅ Policy management service initialized");

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
            policy_service,
            wrapping_service,
        )
    }

    /// Initialize optional services (HSM, MFA)
    async fn initialize_optional_services(
        config: &ApiConfig,
        _storage: Arc<dyn StorageBackend + Send + Sync>,
        _crypto: Arc<CryptoEngine>,
        totp_engine: Arc<TotpEngine>,
    ) -> (
        Option<Arc<HsmBackend>>,
        Arc<crate::services::mfa::MfaService>,
    ) {
        // Initialize HSM backend (optional)
        let hsm = if config.hsm.enabled {
            tracing::info!("Initializing HSM backend...");

            let hsm_config = secreton_hsm::HsmConfig {
                enabled: true,
                provider: secreton_hsm::config::HsmProvider::Pkcs11,
                pkcs11_library_path: config
                    .hsm
                    .library_path
                    .clone()
                    .map(std::path::PathBuf::from),
                slot_id: config.hsm.slot_id,
                pin: config.hsm.pin.clone(),
                key_label_prefix: config
                    .hsm
                    .key_label
                    .clone()
                    .unwrap_or_else(|| "secreton-".to_string()),
                ..secreton_hsm::HsmConfig::default()
            };

            match HsmBackend::new(hsm_config) {
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
        let mfa = Arc::new(crate::services::mfa::MfaService::with_totp_engine(
            totp_engine,
        ));
        tracing::info!("✅ MFA service initialized with shared TotpEngine");

        (hsm, mfa)
    }

    /// Create storage backend based on configuration
    async fn create_storage_backend(
        _config: &ApiConfig,
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
                #[cfg(feature = "raft-consensus")]
                {
                    tracing::info!("Using Raft storage backend (integrated mode)");

                    // Node ID for this Raft node (default: 1)
                    let node_id = std::env::var("SECRETON_RAFT_NODE_ID")
                        .ok()
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(1);

                    let mut raft_config = RaftClusterConfig::default();
                    raft_config.node_id = node_id;

                    // Optional peer list from environment: "2=http://node2:7000,3=http://node3:7000"
                    if let Ok(peers_str) = std::env::var("SECRETON_RAFT_PEERS") {
                        let mut peers = std::collections::HashMap::new();
                        for part in peers_str
                            .split(',')
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                        {
                            if let Some((id_str, addr)) = part.split_once('=') {
                                match id_str.trim().parse::<u64>() {
                                    Ok(id) => {
                                        peers.insert(id, addr.trim().to_string());
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "Invalid node id '{}' in SECRETON_RAFT_PEERS: {}",
                                            id_str,
                                            e
                                        );
                                    }
                                }
                            }
                        }

                        if !peers.is_empty() {
                            tracing::info!("Configured Raft peers from SECRETON_RAFT_PEERS");
                            raft_config.peers = peers;
                        }
                    }

                    let cluster = RaftCluster::new(raft_config).await.map_err(|e| {
                        anyhow::anyhow!("Failed to create Raft storage backend: {}", e)
                    })?;

                    Ok(Arc::new(cluster) as Arc<dyn StorageBackend + Send + Sync>)
                }

                #[cfg(not(feature = "raft-consensus"))]
                {
                    tracing::warn!(
                        "Raft storage backend requested but 'raft-consensus' feature is disabled; falling back to memory",
                    );
                    Ok(Arc::new(MemoryBackend::new()))
                }
            }

            "memory" | "mock" => {
                tracing::info!("Using in-memory mock storage backend");
                Ok(Arc::new(MemoryBackend::new()))
            }

            // TODO: Add other backends (PostgreSQL, Redis, File)
            "postgres" => {
                #[cfg(feature = "postgres")]
                {
                    let url = std::env::var("SECRETON_STORAGE_URL")
                        .or_else(|_| std::env::var("DATABASE_URL"))
                        .map_err(|_| {
                            anyhow::anyhow!(
                                "SECRETON_STORAGE_URL or DATABASE_URL must be set for postgres storage backend"
                            )
                        })?;
                    tracing::info!("Using PostgreSQL storage backend");
                    let backend = secreton_storage::StorageFactory::create_postgres(&url, None)
                        .await
                        .map_err(|e| {
                            anyhow::anyhow!("Failed to create PostgreSQL storage backend: {}", e)
                        })?;
                    Ok(backend)
                }
                #[cfg(not(feature = "postgres"))]
                {
                    tracing::warn!(
                        "PostgreSQL backend feature not enabled, falling back to memory"
                    );
                    Ok(Arc::new(MemoryBackend::new()))
                }
            }

            "redis" => {
                tracing::warn!("Redis backend not yet implemented, falling back to memory");
                Ok(Arc::new(MemoryBackend::new()))
            }

            "file" => {
                let path = std::env::var("Secreton_STORAGE_FILE_PATH")
                    .unwrap_or_else(|_| "/var/lib/secreton/data".to_string());
                tracing::info!("Using file storage backend at {}", path);
                let backend = secreton_storage::StorageFactory::create_file(path)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to create file storage backend: {}", e))?;
                Ok(backend)
            }

            "consul" => {
                #[cfg(feature = "consul")]
                {
                    let address = std::env::var("Secreton_CONSUL_ADDRESS")
                        .unwrap_or_else(|_| "127.0.0.1:8500".to_string());
                    let path = std::env::var("Secreton_CONSUL_PATH")
                        .unwrap_or_else(|_| "secreton/".to_string());
                    tracing::info!(
                        "Using Consul storage backend at {} with path {}",
                        address,
                        path
                    );
                    let backend = secreton_storage::StorageFactory::create_consul(&address, &path)
                        .await
                        .map_err(|e| {
                            anyhow::anyhow!("Failed to create Consul storage backend: {}", e)
                        })?;
                    Ok(backend)
                }
                #[cfg(not(feature = "consul"))]
                {
                    tracing::warn!("Consul backend feature not enabled, falling back to memory");
                    Ok(Arc::new(MemoryBackend::new()))
                }
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

        let mut pool_cfg = deadpool_postgres::PoolConfig::default();
        pool_cfg.max_size = config.database.max_connections as usize;
        pool_cfg.timeouts.wait = Some(std::time::Duration::from_secs(
            config.database.connection_timeout,
        ));
        pool_cfg.timeouts.create = Some(std::time::Duration::from_secs(
            config.database.connection_timeout,
        ));
        pool_cfg.timeouts.recycle = Some(std::time::Duration::from_secs(5));
        cfg.pool = Some(pool_cfg);

        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Verified,
        });

        let tls_mode =
            std::env::var("SECRETON_DB_TLS_MODE").unwrap_or_else(|_| "disable".to_string());

        let pool = if tls_mode.eq_ignore_ascii_case("disable") {
            cfg.create_pool(Some(Runtime::Tokio1), NoTls)?
        } else {
            let mut root_store = rustls::RootCertStore::empty();
            let cert_result = rustls_native_certs::load_native_certs();

            // Log any errors but continue with whatever certs we got
            for err in &cert_result.errors {
                tracing::warn!("Error loading native TLS root certificate: {}", err);
            }

            // Add successfully loaded certificates
            if !cert_result.certs.is_empty() {
                let _ = root_store.add_parsable_certificates(cert_result.certs);
            } else {
                tracing::warn!("No native TLS root certificates found");
            }

            cfg.ssl_mode = Some(deadpool_postgres::SslMode::Require);

            let tls_config = rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();
            let tls = tokio_postgres_rustls::MakeRustlsConnect::new(tls_config);

            cfg.create_pool(Some(Runtime::Tokio1), tls)?
        };
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
        let policy_service = Arc::new(PolicyService::new(storage.clone()));

        // Initialize wrapping service for mock
        let wrapping_service = Arc::new(WrappingService::new(pool.clone()));

        // Initialize MFA service with shared TotpEngine
        let mfa = Arc::new(crate::services::mfa::MfaService::with_totp_engine(
            totp_engine.clone(),
        ));
        tracing::info!("✅ MFA service initialized with shared TotpEngine (mock mode)");

        Self {
            config: ApiConfig::default(),
            storage: storage.clone(),
            pool,
            crypto: crypto.clone(),
            auth: Arc::new(AuthService::new_mock(storage.clone(), crypto.clone())),
            engine: Arc::new(SecretService::new_mock(
                storage.clone(),
                crypto.clone(),
                audit.clone(),
            )),
            admin: Arc::new(AdminService::new_mock(storage.clone())),
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
            policy_service,
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
