//! Application state and initialization
//!
//! This module provides the core application state management and
//! initialization logic for the Authenc authentication service.

use crate::config::AppConfig;
use crate::error::{AuthencError, Result};
use std::sync::Arc;

/// Comprehensive application state with all services
#[derive(Clone)]
pub struct AppState {
    /// Application configuration
    pub config: Arc<AppConfig>,
    /// Database connection pool
    pub database: Arc<crate::database::Database>,
    /// User data store
    pub user_store: Arc<crate::services::stores::user_store::UserStore>,
    /// Session management store
    pub session_store: Arc<crate::services::session_store::SessionStore>,
    /// TOTP (Time-based One-Time Password) store
    pub totp_store: Arc<crate::services::totp_store::TotpStore>,
    /// Brute force attack protection service
    pub brute_force_protector: Arc<crate::services::brute_force_protector::BruteForceProtector>,
    /// Anomaly detection service
    pub anomaly_detector: Arc<crate::services::anomaly_detector::AnomalyDetector>,
    /// Federation provider registry
    pub federation_registry: Arc<crate::services::federation_provider::FederationRegistry>,
    /// Audit log storage
    pub audit_log_store: Arc<crate::services::pg_audit_log_store::PgAuditLogStore>,
    /// User consent management store for GDPR compliance
    pub consent_store: Arc<crate::services::stores::consent_store::ConsentStore>,
    /// Authentication flow store for pluggable authentication flows
    pub auth_flow_store: Arc<crate::services::stores::auth_flow_store::AuthFlowStore>,
    /// Realm configuration store
    pub realm_store: Arc<crate::services::stores::realm_store::RealmStore>,
    /// Realm management service
    pub realm_service: Arc<dyn crate::services::realm::RealmService>,
    /// Role management store
    pub role_store: Arc<crate::services::stores::role_store::RoleStore>,
    /// Permission management store
    pub permission_store: Arc<crate::services::stores::permission_store::PermissionStore>,
    /// Resource management store
    pub resource_store: Arc<crate::services::resource_store::ResourceStore>,
    /// Resource server management store
    pub resource_server_store: Arc<crate::services::resource_server_store::ResourceServerStore>,
    /// Permission ticket management store
    pub permission_ticket_store:
        Arc<crate::services::permission_ticket_store::PermissionTicketStore>,
    /// Scope management store
    pub scope_store: Arc<crate::services::scope_store::ScopeStore>,
    /// Client scope service for OAuth2/OIDC scope management with consent
    pub client_scope_service: Arc<crate::services::client_scope_service::ClientScopeService>,
    /// Protocol mapper service for claim transformation and token generation
    pub protocol_mapper_service:
        Arc<crate::services::protocol_mapper_service::ProtocolMapperService>,
    /// OIDC client store for OAuth2/OIDC client management
    pub oidc_client_store: Arc<crate::services::oidc_client_store::OidcClientStore>,
    /// Service account store for machine-to-machine authentication
    pub service_account_store: Arc<crate::services::service_account_store::ServiceAccountStore>,
    /// Social account store for social login account linking
    pub social_account_store:
        Arc<crate::services::stores::social_account_store::SocialAccountStore>,
    /// Identity broker registry for external authentication providers
    pub broker_registry: Arc<crate::services::broker::IdentityBrokerRegistry>,
    /// OID4VC service for verifiable credentials
    pub oid4vc_service: Arc<crate::services::oid4vc::EnhancedOid4VcManager>,
    /// SSO service for unified single sign-on
    pub sso_service: Arc<dyn crate::services::sso::SsoService>,
    /// SSO cookie manager for secure cookie operations
    pub sso_cookie_manager: Arc<crate::services::sso::SsoCookieManager>,
    /// Event manager for handling application events
    pub event_manager: Arc<tokio::sync::RwLock<crate::services::events::EventManager>>,
    /// Event retention service for managing event lifecycle
    pub event_retention_service: Arc<crate::services::event_retention::EventRetentionService>,
    /// Audit log sink for persistent audit logging
    pub audit_log_sink: Arc<dyn crate::services::audit_log_sink::AuditLogSink>,
    /// SPI manager for pluggable enterprise components
    pub spi_manager: Arc<crate::spi::SpiManager>,
    /// Cluster manager for high availability
    pub cluster_manager: Option<Arc<crate::services::clustering::ClusterManager>>,
    /// Observability service for monitoring and metrics
    pub observability_service: Arc<crate::services::observability::ObservabilityService>,
    /// Secreton client for secure secret management
    pub secreton_client: Arc<crate::vault::secreton_client::SecretonClient>,
    /// Database connection pool for direct access
    pub db_pool: deadpool_postgres::Pool,
    /// Redis cache for performance optimization (optional)
    pub redis_cache: Option<Arc<crate::services::cache::RedisCache>>,
    /// MFA cache service for MFA operations
    pub mfa_cache: Option<Arc<crate::services::cache::MfaCache>>,
    /// Key rotation service for automatic key rotation
    pub key_rotation_service: Option<Arc<crate::services::key_rotation::KeyRotationService>>,
    /// Federation manager for user federation orchestration
    pub federation_manager: Arc<crate::services::federation_manager::FederationManager>,
    /// User synchronization service for LDAP/AD sync
    pub user_sync_service: Arc<crate::services::user_sync_service::UserSyncService>,
    /// UMA 2.0 policy store for policy database operations
    pub uma_policy_store: Arc<crate::services::uma_policy_store::UmaPolicyStore>,
    /// UMA 2.0 delegation policy store
    pub uma_delegation_policy_store:
        Arc<crate::services::uma_policy_store::UmaDelegationPolicyStore>,
    /// UMA 2.0 policy engine for fine-grained authorization
    pub uma_policy_engine: Arc<crate::services::uma::PolicyEngine>,
    /// UMA 2.0 RPT service for token issuance
    pub uma_rpt_service: Arc<crate::services::uma::RptService>,
    /// UMA 2.0 permission endpoint for permission ticket and RPT issuance
    pub uma_permission_endpoint: Arc<crate::services::uma::PermissionEndpoint>,
    /// UMA 2.0 claims gathering service
    pub uma_claims_gathering: Arc<tokio::sync::Mutex<crate::services::uma::ClaimsGatheringService>>,
    /// UMA 2.0 resource owner authorization service
    pub uma_resource_owner_auth: Arc<crate::services::uma::ResourceOwnerAuthService>,
}

impl AppState {
    /// Initialize application state with all services
    pub async fn new(config: AppConfig) -> Result<Self> {
        let config = Arc::new(config);

        // Initialize database connection pool
        let database = crate::app_init::database::initialize_database(&config).await?;

        // Run database migrations before initializing other services
        tracing::info!("🔄 Running database migrations...");
        let db_pool = database.get_pool();
        match crate::database::migrations::run_migrations_with_schema(db_pool.clone()).await {
            Ok(result) => {
                if result.is_success() {
                    tracing::info!(
                        "✅ Database migrations completed: {} applied, {} skipped",
                        result.applied,
                        result.skipped
                    );
                } else {
                    tracing::warn!(
                        "⚠️ Database migrations completed with {} errors: {:?}",
                        result.errors.len(),
                        result.errors
                    );
                }
            }
            Err(e) => {
                tracing::error!("❌ Failed to run database migrations: {}", e);
                return Err(AuthencError::database(format!(
                    "Database migration failed: {}",
                    e
                )));
            }
        }

        // Initialize UMA 2.0 tables
        crate::app_init::database::init_uma_tables(&database).await?;

        // Initialize audit log store
        let audit_log_store = crate::app_init::database::initialize_audit_store(&config).await?;

        // Initialize consent store
        let consent_store = crate::app_init::database::initialize_consent_store(database.clone());

        // Initialize authentication flow store
        let auth_flow_store =
            crate::app_init::database::initialize_auth_flow_store(database.clone());

        // Initialize other services
        let user_store = Arc::new(crate::services::stores::user_store::UserStore::new(
            database.clone(),
        ));
        let session_store = Arc::new(crate::services::session_store::SessionStore::new(
            database.clone(),
        ));
        let totp_store = Arc::new(crate::services::totp_store::TotpStore::new());

        let brute_force_protector = Arc::new(
            crate::services::brute_force_protector::BruteForceProtector::new(
                config.security.brute_force_max_attempts as usize,
                config.security.brute_force_window_seconds,
            ),
        );

        let anomaly_detector = Arc::new(crate::services::anomaly_detector::AnomalyDetector::new());
        let federation_registry =
            Arc::new(crate::services::federation_provider::FederationRegistry::new());
        let realm_store = Arc::new(crate::services::stores::realm_store::RealmStore::new());
        let realm_service = Arc::new(crate::services::realm::PostgresRealmService::new(
            database.clone(),
        ));
        let role_store = Arc::new(crate::services::stores::role_store::RoleStore::new());
        let permission_store =
            Arc::new(crate::services::stores::permission_store::PermissionStore::new());
        let resource_store = Arc::new(crate::services::resource_store::ResourceStore::new(
            database.clone(),
        ));
        let resource_server_store = Arc::new(
            crate::services::resource_server_store::ResourceServerStore::new(database.clone()),
        );
        let permission_ticket_store = Arc::new(
            crate::services::permission_ticket_store::PermissionTicketStore::new(database.clone()),
        );
        let scope_store = Arc::new(crate::services::scope_store::ScopeStore::new(
            database.clone(),
        ));
        let client_scope_service = Arc::new(
            crate::services::client_scope_service::ClientScopeService::new(database.clone()),
        );
        let protocol_mapper_service = Arc::new(
            crate::services::protocol_mapper_service::ProtocolMapperService::new(database.clone()),
        );
        let oidc_client_store = Arc::new(
            crate::services::oidc_client_store::OidcClientStore::with_database(database.clone()),
        );

        // Initialize service account store for machine-to-machine authentication
        let service_account_store = Arc::new(
            crate::services::service_account_store::ServiceAccountStore::with_database(
                database.clone(),
            ),
        );

        // Initialize social account store
        let social_account_store = Arc::new(
            crate::services::stores::social_account_store::SocialAccountStore::new(
                database.clone(),
            ),
        );

        // Initialize identity broker registry
        let broker_registry = Arc::new(crate::services::broker::IdentityBrokerRegistry::new());

        // Initialize OID4VC service
        let oid4vc_service = Arc::new(crate::services::oid4vc::EnhancedOid4VcManager::new(
            "https://authenc.example.com".to_string(),
        ));

        // Initialize SSO cookie manager
        let sso_secret: &[u8] = if config.security.jwt_secret.is_empty() {
            b"default-sso-secret-key-change-in-production!!"
        } else {
            config.security.jwt_secret.as_bytes()
        };
        let sso_cookie_manager = Arc::new(crate::services::sso::SsoCookieManager::new(
            sso_secret,
            "AUTHENC_SSO",
            None, // cookie_domain from config
            true, // secure = true (use HTTPS in production)
        ));

        // Initialize SSO session manager
        let sso_session_manager =
            Arc::new(crate::services::sso::session::DefaultSsoSessionManager::new());

        // Initialize SSO service
        let sso_service: Arc<dyn crate::services::sso::SsoService> =
            Arc::new(crate::services::sso::DefaultSsoService::new(
                sso_session_manager,
                sso_cookie_manager.clone(),
                database.clone(),
            ));

        // Initialize audit log sink
        let audit_log_sink: Arc<dyn crate::services::audit_log_sink::AuditLogSink> = if let Some(
            kafka_config,
        ) =
            &config.kafka
        {
            if kafka_config.enabled {
                match crate::services::kafka_audit_log_sink::KafkaAuditLogSink::new(
                    &kafka_config.brokers,
                    &kafka_config.audit_topic,
                ) {
                    Ok(sink) => Arc::new(sink),
                    Err(e) => {
                        tracing::warn!(
                            "Failed to initialize Kafka audit log sink: {}. Falling back to PostgreSQL sink.",
                            e
                        );
                        Arc::new(crate::services::audit_log_sink::PgAuditLogSink::new(
                            (*audit_log_store).clone(),
                        ))
                    }
                }
            } else {
                Arc::new(crate::services::audit_log_sink::PgAuditLogSink::new(
                    (*audit_log_store).clone(),
                ))
            }
        } else {
            Arc::new(crate::services::audit_log_sink::PgAuditLogSink::new(
                (*audit_log_store).clone(),
            ))
        };

        // Initialize event manager
        let event_manager = crate::services::events::create_shared_event_manager();

        // Initialize event store provider
        let event_store = Arc::new(crate::services::pg_event_store::PgEventStoreProvider::new(
            database.clone(),
        ));
        event_store.init_tables().await.map_err(|e| {
            AuthencError::database(format!("Failed to initialize event store tables: {}", e))
        })?;

        // Set event store provider
        {
            let mut manager = event_manager.write().await;
            manager.set_store_provider(event_store.clone());

            // Register default event listeners
            for listener in crate::services::event_listeners::create_default_listeners() {
                manager.register_listener(listener);
            }

            // Register Kafka event listener if configured
            if let Some(kafka_config) = &config.kafka {
                if kafka_config.enabled
                    && !kafka_config.user_events_topic.is_empty()
                    && !kafka_config.admin_events_topic.is_empty()
                {
                    match crate::services::kafka_event_listener::KafkaEventListener::new(
                        &kafka_config.brokers,
                        &kafka_config.user_events_topic,
                        &kafka_config.admin_events_topic,
                    ) {
                        Ok(kafka_listener) => {
                            manager.register_listener(Arc::new(kafka_listener));
                            tracing::info!(
                                "Kafka event listener registered for topics: {} and {}",
                                kafka_config.user_events_topic,
                                kafka_config.admin_events_topic
                            );
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to initialize Kafka event listener: {}. Event streaming disabled.",
                                e
                            );
                        }
                    }
                }
            }
        }

        // Initialize event retention service
        let event_retention_service = Arc::new(
            crate::services::event_retention::EventRetentionService::new(
                config.events.clone(),
                database.clone(),
                event_store,
            )
            .await?,
        );

        // Start the retention cleanup task if enabled
        event_retention_service.clone().start_cleanup_task();

        // Initialize SPI manager with default providers
        let mut spi_manager = crate::spi::SpiManager::new();

        // Register SPIs
        spi_manager.register_spi(Box::new(crate::spi::admin_console::AdminConsoleSpi));
        spi_manager.register_spi(Box::new(crate::spi::credential::CredentialSpi));
        spi_manager.register_spi(Box::new(crate::spi::theme::ThemeSpi));
        spi_manager.register_spi(Box::new(crate::spi::userprofile::UserProfileSpi));
        spi_manager.register_spi(Box::new(crate::spi::validation::ValidationSpi));
        spi_manager.register_spi(Box::new(crate::spi::locale::LocaleSpi));
        spi_manager.register_spi(Box::new(crate::spi::events::EventsSpi));
        spi_manager.register_spi(Box::new(crate::spi::ldap_federation::LdapFederationSpi));
        spi_manager.register_spi(Box::new(crate::spi::social::SocialProviderSpi));
        spi_manager.register_spi(Box::new(crate::spi::storage::StorageSpi));
        spi_manager.register_spi(Box::new(crate::spi::sessions::SessionSpi));
        spi_manager.register_spi(Box::new(crate::spi::protocol_mappers::ProtocolMapperSpi));
        spi_manager.register_spi(Box::new(crate::spi::authenticator::AuthenticatorSpi));
        spi_manager.register_spi(Box::new(crate::spi::required_actions::RequiredActionSpi));
        spi_manager.register_spi(Box::new(crate::spi::organization::OrganizationSpi));
        spi_manager.register_spi(Box::new(
            crate::spi::rich_authorization::RichAuthorizationSpi,
        ));
        spi_manager.register_spi(Box::new(crate::spi::migration::MigrationSpi));
        spi_manager.register_spi(Box::new(crate::spi::hostname::HostnameSpi));

        // Register default providers
        spi_manager.registry_mut().register_factory(
            "admin-console",
            crate::spi::admin_console::DefaultAdminConsoleProviderFactory::new(),
        );
        spi_manager.registry_mut().register_factory(
            "credential",
            crate::spi::credential::DefaultCredentialProviderFactory,
        );
        spi_manager.registry_mut().register_factory(
            "credential",
            crate::spi::credential::PasswordCredentialProviderFactory::new(),
        );
        spi_manager.registry_mut().register_factory(
            "credential",
            crate::spi::credential::OTPCredentialProviderFactory::new(),
        );
        spi_manager.registry_mut().register_factory(
            "theme",
            crate::spi::theme::DefaultThemeProviderFactory::new("simpelv2".to_string()),
        );
        spi_manager.registry_mut().register_factory(
            "userprofile",
            crate::spi::userprofile::DefaultUserProfileProviderFactory::new(),
        );
        spi_manager.registry_mut().register_factory(
            "validation",
            crate::spi::validation::DefaultValidationProviderFactory::new(),
        );
        spi_manager.registry_mut().register_factory(
            "locale",
            crate::spi::locale::DefaultLocaleProviderFactory::new(),
        );
        spi_manager.registry_mut().register_factory(
            "events",
            crate::spi::events::DefaultEventProviderFactory::new(),
        );
        spi_manager.registry_mut().register_factory(
            "ldap-federation",
            crate::spi::ldap_federation::DefaultLdapFederationProviderFactory::new(),
        );
        spi_manager.registry_mut().register_factory(
            "social",
            crate::spi::social::DefaultSocialProviderFactory::new(),
        );
        spi_manager.registry_mut().register_factory(
            "storage",
            crate::spi::storage::DefaultStorageProviderFactory::new(
                user_store.clone(),
                oidc_client_store.clone(),
                role_store.clone(),
                Arc::new(crate::services::group_store::GroupStore::new()),
            ),
        );
        spi_manager.registry_mut().register_factory(
            "session",
            crate::spi::sessions::DefaultSessionProviderFactory::new(session_store.clone()),
        );
        spi_manager.registry_mut().register_factory(
            "protocol-mapper",
            crate::spi::protocol_mappers::DefaultProtocolMapperProviderFactory::new(),
        );
        spi_manager.registry_mut().register_factory(
            "authenticator",
            crate::spi::authenticator::DefaultAuthenticatorProviderFactory::new(),
        );
        spi_manager.registry_mut().register_factory(
            "required-action",
            crate::spi::required_actions::DefaultRequiredActionProviderFactory::new(),
        );
        spi_manager.registry_mut().register_factory(
            "organization",
            crate::spi::organization::DefaultOrganizationProviderFactory::new(),
        );
        spi_manager.registry_mut().register_factory(
            "rich-authorization",
            crate::spi::rich_authorization::DefaultRichAuthorizationProviderFactory::new(),
        );
        spi_manager.registry_mut().register_factory(
            "migration",
            crate::spi::migration::DefaultMigrationProviderFactory::new(),
        );
        spi_manager.registry_mut().register_factory(
            "hostname",
            crate::spi::hostname::DefaultHostnameProviderFactory::new(),
        );
        let spi_manager = Arc::new(spi_manager);

        // Initialize cluster manager if clustering is enabled
        let cluster_manager = if config.clustering.enabled {
            let node_id = config
                .clustering
                .node_id
                .clone()
                .unwrap_or_else(|| format!("node-{}", uuid::Uuid::new_v4().simple()));
            let (manager, _broadcast_tx) =
                crate::services::clustering::ClusterManager::new_in_memory(
                    node_id,
                    config.clustering.cluster_name.clone(),
                );
            Some(Arc::new(manager))
        } else {
            None
        };

        // Initialize observability service
        let mut observability_service =
            crate::services::observability::ObservabilityService::default();

        // Register default health checks
        observability_service.register_health_check(Box::new(
            crate::services::observability::DatabaseHealthCheck::new(10, 5), // TODO: Get from config
        ));

        // Register default metrics collectors
        observability_service.register_metrics_collector(Box::new(
            crate::services::observability::PrometheusMetricsCollector::new(),
        ));

        let observability_service = Arc::new(observability_service);

        // Initialize Secreton client for MFA secret management
        let secreton_endpoint = std::env::var("SECRETON_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8200".to_string());
        let secreton_token =
            std::env::var("SECRETON_TOKEN").unwrap_or_else(|_| "dev-token".to_string());

        let secreton_client = Arc::new(crate::vault::secreton_client::SecretonClient::new(
            secreton_endpoint,
            secreton_token,
        ));

        // Get database pool for direct access
        let db_pool = database.get_pool();

        // Initialize Redis cache if configured
        let (redis_cache, mfa_cache) = if let Some(redis_config) = &config.redis {
            if redis_config.enabled {
                match crate::services::cache::RedisCache::new(redis_config).await {
                    Ok(cache) => {
                        let redis_cache = Arc::new(cache);
                        let mfa_cache = Arc::new(crate::services::cache::MfaCache::new(
                            redis_cache.clone(),
                            std::time::Duration::from_secs(redis_config.mfa_cache_ttl),
                            std::time::Duration::from_secs(redis_config.otp_verification_ttl),
                        ));
                        tracing::info!("Redis cache initialized successfully");
                        (Some(redis_cache), Some(mfa_cache))
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to initialize Redis cache: {}. Continuing without cache.",
                            e
                        );
                        (None, None)
                    }
                }
            } else {
                tracing::info!("Redis cache disabled in configuration");
                (None, None)
            }
        } else {
            tracing::info!("No Redis configuration found, running without cache");
            (None, None)
        };

        // Initialize key rotation service if configured
        let key_rotation_service = if let Some(key_rotation_config) = &config.key_rotation {
            if key_rotation_config.enabled {
                let service = Arc::new(crate::services::key_rotation::KeyRotationService::new(
                    secreton_client.clone(),
                    database.clone(),
                    key_rotation_config.clone(),
                ));

                // Start the rotation scheduler
                if let Err(e) = service.start() {
                    tracing::warn!("Failed to start key rotation scheduler: {}", e);
                    None
                } else {
                    tracing::info!("Key rotation service initialized and started");
                    Some(service)
                }
            } else {
                tracing::info!("Key rotation disabled in configuration");
                None
            }
        } else {
            tracing::info!("No key rotation configuration found");
            None
        };

        // Initialize federation manager for user federation
        let federation_manager = Arc::new(
            crate::services::federation_manager::FederationManager::new(database.clone()),
        );

        // Initialize federation manager (load providers from database)
        if let Err(e) = federation_manager.initialize().await {
            tracing::warn!("Failed to initialize federation manager: {}", e);
        } else {
            tracing::info!("Federation manager initialized successfully");
        }

        // Initialize user sync service for LDAP/AD synchronization
        let user_sync_service = Arc::new(crate::services::user_sync_service::UserSyncService::new(
            database.clone(),
            federation_manager.clone(),
        ));

        // Start sync scheduler if configured
        if let Some(sync_interval) = config
            .federation
            .as_ref()
            .and_then(|f| f.sync_interval_minutes)
        {
            user_sync_service.start_scheduler(sync_interval).await;
            tracing::info!(
                "User sync scheduler started with interval: {} minutes",
                sync_interval
            );
        } else {
            tracing::info!("User sync scheduler not configured");
        }

        Ok(Self {
            config: config.clone(),
            database: database.clone(),
            user_store,
            session_store,
            totp_store,
            brute_force_protector,
            anomaly_detector,
            federation_registry,
            audit_log_store,
            consent_store,
            auth_flow_store,
            realm_store,
            realm_service,
            role_store,
            permission_store,
            resource_store: resource_store.clone(),
            resource_server_store,
            permission_ticket_store: permission_ticket_store.clone(),
            scope_store,
            client_scope_service,
            protocol_mapper_service,
            oidc_client_store,
            service_account_store,
            social_account_store,
            broker_registry,
            oid4vc_service,
            sso_service,
            sso_cookie_manager,
            event_manager,
            event_retention_service,
            audit_log_sink,
            spi_manager,
            cluster_manager,
            observability_service,
            secreton_client,
            db_pool,
            redis_cache,
            mfa_cache,
            key_rotation_service,
            federation_manager,
            user_sync_service: user_sync_service.clone(),
            uma_policy_store: Arc::new(crate::services::uma_policy_store::UmaPolicyStore::new(
                database.clone(),
            )),
            uma_delegation_policy_store: Arc::new(
                crate::services::uma_policy_store::UmaDelegationPolicyStore::new(database.clone()),
            ),
            uma_policy_engine: Arc::new(crate::services::uma::PolicyEngine::new(
                database.clone(),
                resource_store.clone(),
            )),
            uma_rpt_service: Arc::new(
                crate::services::uma::RptService::new(
                    "authenc".to_string(), // issuer
                    // In production, load signing keys from Secreton
                    jsonwebtoken::EncodingKey::from_secret(config.security.jwt_secret.as_bytes()),
                    jsonwebtoken::DecodingKey::from_secret(config.security.jwt_secret.as_bytes()),
                )
                .with_lifetime(3600),
            ), // token lifetime: 1 hour
            uma_permission_endpoint: {
                let uma_policy_store = Arc::new(
                    crate::services::uma_policy_store::UmaPolicyStore::new(database.clone()),
                );
                let uma_rpt_service = Arc::new(
                    crate::services::uma::RptService::new(
                        "authenc".to_string(),
                        jsonwebtoken::EncodingKey::from_secret(
                            config.security.jwt_secret.as_bytes(),
                        ),
                        jsonwebtoken::DecodingKey::from_secret(
                            config.security.jwt_secret.as_bytes(),
                        ),
                    )
                    .with_lifetime(3600),
                );
                let uma_policy_engine = Arc::new(crate::services::uma::PolicyEngine::new(
                    database.clone(),
                    resource_store.clone(),
                ));

                Arc::new(crate::services::uma::PermissionEndpoint::new(
                    database.clone(),
                    resource_store.clone(),
                    permission_ticket_store.clone(),
                    uma_policy_engine,
                    uma_rpt_service,
                    uma_policy_store,
                    3600, // ticket lifetime: 1 hour
                ))
            },
            uma_claims_gathering: Arc::new(tokio::sync::Mutex::new(
                crate::services::uma::ClaimsGatheringService::new(
                    format!("https://{}:{}", config.server.host, config.server.port), // base_url
                ),
            )),
            uma_resource_owner_auth: Arc::new(crate::services::uma::ResourceOwnerAuthService::new(
                resource_store.clone(),
                permission_ticket_store.clone(),
            )),
        })
    }

    /// Initialize social identity brokers from environment variables
    pub fn initialize_social_brokers(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use crate::services::broker::{
            IdentityProviderConfig, IdentityProviderType, SocialConfig, SocialIdentityBroker,
        };
        use uuid::Uuid;

        // Initialize Google broker if configured
        if let (Ok(client_id), Ok(client_secret)) = (
            std::env::var("GOOGLE_CLIENT_ID"),
            std::env::var("GOOGLE_CLIENT_SECRET"),
        ) {
            let google_config = SocialConfig {
                client_id,
                client_secret,
                redirect_uri: std::env::var("GOOGLE_REDIRECT_URI")
                    .unwrap_or_else(|_| "http://localhost:3000/auth/social/callback".to_string()),
                scopes: vec![
                    "openid".to_string(),
                    "email".to_string(),
                    "profile".to_string(),
                ],
            };
            let _google_broker =
                SocialIdentityBroker::new(google_config, IdentityProviderType::SocialGoogle);

            let _provider_config = IdentityProviderConfig {
                id: Uuid::new_v4(),
                name: "Google".to_string(),
                provider_type: IdentityProviderType::SocialGoogle,
                enabled: true,
                config: serde_json::json!({
                    "client_id": std::env::var("GOOGLE_CLIENT_ID").unwrap(),
                    "client_secret": std::env::var("GOOGLE_CLIENT_SECRET").unwrap(),
                    "redirect_uri": std::env::var("GOOGLE_REDIRECT_URI").unwrap_or_else(|_| "http://localhost:3000/auth/social/callback".to_string())
                }),
                realm_id: Uuid::new_v4(), // Default realm
            };

            // We need to make broker_registry mutable, but it's in Arc. Let's skip this for now.
            // self.broker_registry.register_broker(provider_config, Box::new(google_broker));
        }

        // Initialize GitHub broker if configured
        if let (Ok(client_id), Ok(client_secret)) = (
            std::env::var("GITHUB_CLIENT_ID"),
            std::env::var("GITHUB_CLIENT_SECRET"),
        ) {
            let github_config = SocialConfig {
                client_id,
                client_secret,
                redirect_uri: std::env::var("GITHUB_REDIRECT_URI")
                    .unwrap_or_else(|_| "http://localhost:3000/auth/social/callback".to_string()),
                scopes: vec!["user:email".to_string()],
            };
            let _github_broker =
                SocialIdentityBroker::new(github_config, IdentityProviderType::SocialGitHub);

            let _provider_config = IdentityProviderConfig {
                id: Uuid::new_v4(),
                name: "GitHub".to_string(),
                provider_type: IdentityProviderType::SocialGitHub,
                enabled: true,
                config: serde_json::json!({
                    "client_id": std::env::var("GITHUB_CLIENT_ID").unwrap(),
                    "client_secret": std::env::var("GITHUB_CLIENT_SECRET").unwrap(),
                    "redirect_uri": std::env::var("GITHUB_REDIRECT_URI").unwrap_or_else(|_| "http://localhost:3000/auth/social/callback".to_string())
                }),
                realm_id: Uuid::new_v4(), // Default realm
            };

            // self.broker_registry.register_broker(provider_config, Box::new(github_broker));
        }

        // Initialize Facebook broker if configured
        if let (Ok(client_id), Ok(client_secret)) = (
            std::env::var("FACEBOOK_CLIENT_ID"),
            std::env::var("FACEBOOK_CLIENT_SECRET"),
        ) {
            let facebook_config = SocialConfig {
                client_id,
                client_secret,
                redirect_uri: std::env::var("FACEBOOK_REDIRECT_URI")
                    .unwrap_or_else(|_| "http://localhost:3000/auth/social/callback".to_string()),
                scopes: vec!["email".to_string(), "public_profile".to_string()],
            };
            let _facebook_broker =
                SocialIdentityBroker::new(facebook_config, IdentityProviderType::SocialFacebook);

            let _provider_config = IdentityProviderConfig {
                id: Uuid::new_v4(),
                name: "Facebook".to_string(),
                provider_type: IdentityProviderType::SocialFacebook,
                enabled: true,
                config: serde_json::json!({
                    "client_id": std::env::var("FACEBOOK_CLIENT_ID").unwrap(),
                    "client_secret": std::env::var("FACEBOOK_CLIENT_SECRET").unwrap(),
                    "redirect_uri": std::env::var("FACEBOOK_REDIRECT_URI").unwrap_or_else(|_| "http://localhost:3000/auth/social/callback".to_string())
                }),
                realm_id: Uuid::new_v4(), // Default realm
            };

            // self.broker_registry.register_broker(provider_config, Box::new(facebook_broker));
        }

        Ok(())
    }
}

/// Application builder for configuring and running the server
pub struct ApplicationBuilder {
    config: AppConfig,
}

impl ApplicationBuilder {
    /// Create a new application builder
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Build the application state
    pub async fn build_state(self) -> Result<AppState> {
        let state = AppState::new(self.config).await?;

        // Initialize social identity brokers
        if let Err(e) = state.initialize_social_brokers() {
            tracing::warn!("Failed to initialize social brokers: {}", e);
        }

        Ok(state)
    }

    /// Run the application server
    pub async fn run(self) -> Result<()> {
        let state = self.build_state().await?;

        #[cfg(feature = "axum")]
        {
            // Check if gRPC is enabled
            if state.config.server.grpc_enabled {
                // Run both HTTP and gRPC servers
                use crate::server::DualServer;
                let server = DualServer::new(state);
                server.run().await?;
            } else {
                // Run HTTP server only
                use crate::axum_app::AxumApp;
                let app = AxumApp::new(state);
                app.run().await?;
            }
        }

        #[cfg(not(feature = "axum"))]
        {
            return Err(AuthencError::ConfigurationError {
                message: "No web framework feature enabled. Enable 'axum' feature.".to_string(),
            });
        }

        Ok(())
    }
}

// Re-export initialize_logging from app_logging module
// The implementation has been extracted for better code organization
pub use crate::app_logging::initialize_logging;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_initialization() {
        // Use a config that will definitely fail to connect
        let mut config = AppConfig::default();
        config.database.host = "nonexistent.host.invalid".to_string();
        config.database.port = 12345; // Invalid port
        let result = AppState::new(config).await;
        // Should fail with invalid database configuration
        assert!(result.is_err());
    }

    #[test]
    fn test_application_builder_creation() {
        let config = AppConfig::default();
        let builder = ApplicationBuilder::new(config);
        assert!(builder.config.server.port > 0);
    }
}
