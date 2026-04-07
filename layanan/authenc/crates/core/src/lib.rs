// Many services and SPI modules contain placeholder/stub implementations
// with unused variables, dead code, and deprecated type references that
// will be cleaned up incrementally.  Suppress these crate-wide for now.
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_assignments)]
#![allow(unreachable_code)]
#![allow(deprecated)]

//! # authenc-core
//!
//! Core business logic for Authenc identity provider.
//!
//! This crate contains the core business logic and service implementations for the Authenc
//! enterprise-grade Identity Provider (IdP) and Authorization Server.
//!
//! ## Architecture
//!
//! The core crate is organized into several key modules:
//!
//! - **[`config`]**: Configuration management (dynamic config, security settings, MFA fallback)
//! - **[`init`]**: Initialization helpers (database setup, service initialization)
//! - **[`services`]**: Business logic services (authentication, OAuth2/OIDC, audit, events, etc.)
//! - **[`stores`]**: High-level store abstractions (user, realm, role, permission, consent, etc.)
//! - **[`spi`]**: Service Provider Interface for pluggable components
//!
//! ## Core Services
//!
//! ### Authentication & Authorization
//! - [`AuthenticationServiceImpl`]: Core authentication logic with MFA support
//! - [`BruteForceProtectorImpl`]: Attack prevention and rate limiting
//! - [`AnomalyDetector`]: Threat detection and risk scoring
//! - [`RiskEngine`]: Dynamic risk assessment
//! - [`AuthenticationManager`]: Authentication flow orchestration
//! - [`PasswordPolicyService`]: Password policy enforcement
//!
//! ### User & Realm Management
//! - [`UserManagementServiceImpl`]: User CRUD operations
//! - [`RealmManagementServiceImpl`]: Multi-tenant realm management
//! - [`RoleManagementServiceImpl`]: Role-based access control (RBAC)
//! - [`OrganizationService`]: Organization hierarchy management
//! - [`SatkerAuthorizationService`]: Government hierarchy authorization (Kejaksaan RI)
//!
//! ### OAuth2/OIDC
//! - [`OAuth2ServiceImpl`]: OAuth2 authorization server
//! - [`OidcClientStore`]: OIDC client management
//! - [`OidcCodeStore`]: Authorization code storage
//! - [`ClientRegistrationService`]: Dynamic Client Registration (RFC 7591/7592)
//! - [`TokenExchangeService`]: Token Exchange (RFC 8693)
//! - [`DeviceAuthorizationService`]: Device Authorization Grant (RFC 8628)
//! - [`ScopeStore`]: OAuth2 scope management
//! - [`ServiceAccountStore`]: Service account management
//!
//! ### UMA 2.0 (User-Managed Access)
//! - [`PolicyEngine`]: Policy evaluation engine
//! - [`ResourceStore`]: Protected resource management
//! - [`ResourceServerStore`]: Resource server registration
//! - [`PermissionTicketStore`]: Permission ticket management
//! - [`UmaPolicyStore`]: UMA policy storage
//! - [`ClaimsGatheringService`]: Claims gathering flow
//!
//! ### Audit & Events
//! - [`AuditService`]: Audit trail management
//! - [`EnhancedAuditService`]: Enhanced audit with integrity verification
//! - [`EventPublisher`]: Event publishing with DLQ support
//! - [`PgAuditLogStore`]: PostgreSQL audit log storage
//! - [`PgEventStore`]: PostgreSQL event storage
//! - [`KafkaAuditLogSink`]: Kafka audit log sink
//! - [`ElasticsearchAuditLogSink`]: Elasticsearch audit log sink
//!
//! ### Session Management
//! - [`SessionStore`]: Session storage and management
//! - [`SsoCookieManager`]: SSO cookie management
//!
//! ### Cache
//! - Cache services with Redis integration
//! - Cache invalidation listeners
//!
//! ### Enterprise Features
//! - **Client Policy**: Dynamic client policies
//! - **Authorization**: Advanced authorization logic
//! - **Zero Trust**: Zero-trust security policies
//! - **Compliance**: Compliance reporting (GDPR, FIPS)
//! - **FIPS**: FIPS 140-2 mode
//! - **Clustering**: High availability with Raft consensus
//! - **Observability**: Metrics and tracing
//! - **Key Rotation**: Automatic key rotation
//!
//! ### FAPI-2 (Optional)
//! - **PAR**: Pushed Authorization Requests (RFC 9126)
//!
//! ## Stores
//!
//! High-level store abstractions following the repository pattern:
//!
//! - [`UserStore`]: User management and authentication
//! - [`RealmStore`]: Multi-tenant realm management
//! - [`RoleStore`]: Role-based access control
//! - [`PermissionStore`]: Permission management
//! - [`ConsentStore`]: User consent tracking for OAuth2/OIDC
//! - [`AuthFlowStore`]: Authentication flow and session management
//! - [`SocialAccountStore`]: Social login account linking
//!
//! ## Service Provider Interface (SPI)
//!
//! The SPI module provides pluggable components for extensibility:
//!
//! - **Authenticator SPI**: Custom authentication mechanisms
//! - **Storage SPI**: Custom storage backends
//! - **Events SPI**: Custom event listeners
//! - **Protocol Mappers**: Custom OIDC/SAML claim mappers
//! - **Social Providers**: Custom social login providers
//! - **Theme SPI**: Custom UI themes
//! - **Validation SPI**: Custom validation logic
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use authenc_core::{
//!     services::{AuthenticationServiceImpl, UserManagementServiceImpl},
//!     stores::{UserStore, UserStoreTrait},
//!     config::AppConfig,
//! };
//! use authenc_storage::Database;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize database
//! let db = Database::new("postgres://...", Default::default()).await?;
//!
//! // Create stores
//! let user_store = UserStore::new(Arc::new(db.clone()));
//!
//! // Create services
//! let auth_service = AuthenticationServiceImpl::new(
//!     Arc::new(user_store),
//!     // ... other dependencies
//! );
//!
//! // Use services
//! let user = auth_service.authenticate("username", "password").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Feature Flags
//!
//! This crate supports the following feature flags:
//!
//! - `default`: All core features enabled
//! - `kafka`: Kafka event streaming support
//! - `elasticsearch`: Elasticsearch audit log sink
//! - `redis`: Redis caching support
//! - `fips`: FIPS 140-2 compliance mode
//! - `quantum`: Post-quantum cryptography support
//!
//! ## Dependencies
//!
//! This crate depends on:
//!
//! - [`authenc-types`]: Core traits and domain types
//! - [`authenc-storage`]: Database layer and storage backends
//! - [`authenc-crypto`]: Cryptographic operations
//!
//! ## Related Crates
//!
//! - [`authenc-api`]: Public REST API (Axum)
//! - [`authenc-iam-api`]: Admin REST API (Axum)
//! - [`authenc-grpc`]: Service-to-service gRPC API (Tonic)
//! - [`authenc-mfa`]: Multi-factor authentication
//! - [`authenc-federation`]: SSO and federation
//! - [`authenc-webauthn`]: WebAuthn/Passkeys support

// Re-export types from authenc-types for convenience
pub use authenc_types::*;

// Error module - re-exports AuthencError and Result for crate::error:: paths
pub mod error;

// Core modules
pub mod config;
pub mod init;
pub mod services;
pub mod spi;
pub mod stores;
/// Database module - re-exports `authenc_storage::Database`.
/// Services that use `crate::database::Database` will get the real storage Database.
pub mod database {
    pub use authenc_storage::Database;
}

// Re-export key types and services for convenience
pub use config::{AppConfig, MfaFallbackConfig};
// TODO: Add initialize_database and initialize_services when implemented
// pub use init::{initialize_database, initialize_services};

// Re-export all service types from services module
pub use services::{
    AnomalyDetector,
    AnomalyDetectorTrait,
    // Audit & Events
    AuditService,
    AuthenticationContext,
    AuthenticationFlowModel,
    AuthenticationFlowType,
    AuthenticationManager,
    // Authentication & Authorization
    AuthenticationServiceImpl,
    AuthenticationSessionManager,
    BruteForceConfig,
    BruteForceProtectorImpl,
    ClaimsGatheringFlow,
    ClaimsGatheringRequest,
    ClaimsGatheringResponse,
    ClaimsGatheringService,
    ClientRegistrationService,
    DefaultAuthenticationFlowResolver,
    EnhancedAuditContext,
    EnhancedAuditService,
    // OAuth2/OIDC
    // Note: OidcClientStore, OidcCodeStore, ClientScopeService are NOT in core
    // (they remain in src/ for old architecture; new arch uses OAuth2ServiceImpl)
    OAuth2ServiceImpl,
    OrganizationInvitation,
    OrganizationRole,
    OrganizationService,
    OrganizationSettings,
    OrganizationUpdate,
    PasswordPolicyConfig,
    PasswordPolicyService,
    PasswordPolicyValidationResult,

    Permission,
    PermissionEndpoint,
    PermissionTicketStore,
    PermissionTicketStoreTrait,
    PolicyDecision,

    PolicyEngine,
    PolicyEvaluationContext,
    PolicyEvaluationResult,
    PolicyType,
    PostgresRealmService,
    ProductionClientRegistrationService,
    ProtocolMapperService,
    RealmManagementServiceImpl,
    RealmManager,
    RealmService,
    ResourceServerStore,
    ResourceServerStoreTrait,
    // TODO: DeviceAuthorizationService not yet implemented
    // DeviceAuthorizationService,

    // UMA 2.0
    ResourceStore,
    ResourceStoreTrait,
    RiskEngine,
    Role,
    RoleManagementServiceImpl,
    SatkerAuthorizationService,
    SatkerHierarchyInfo,

    ScopeStore,
    ServiceAccountStore,
    // Session Management
    SessionStore,
    SsoCookieManager,
    SsoSession,

    TokenExchangeService,
    UmaDelegationPolicyStore,
    UmaPolicy,
    UmaPolicyStore,
    // User & Realm Management
    UserManagementServiceImpl,
    create_audit_context,
    extract_audit_details,
    // Note: EventPublisher, EventPublisherConfig, PublishableEvent, DlqEntry
    // require the "kafka" feature - use services::event_publisher directly when needed
};

// Feature-gated re-exports
#[cfg(feature = "kafka")]
pub use services::{DlqEntry, EventPublisher, EventPublisherConfig, PublishableEvent};

// Re-export all store types from stores module
pub use stores::{
    AuthFlowStore, AuthFlowStoreTrait, ConsentStore, ConsentStoreTrait, PermissionStore,
    RealmStore, RoleStore, SocialAccountStore, SocialAccountStoreTrait, UserStore, UserStoreTrait,
};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
