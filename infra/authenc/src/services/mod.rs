// Security and protection services
/// Advanced federation protocols and identity bridging
pub mod advanced_federation;
/// Advanced authentication protocols implementation
pub mod advanced_protocols;
/// Anomaly detection and threat intelligence
pub mod anomaly_detector;
/// Authentication flow management and orchestration
pub mod auth_flow;
/// Brute force attack prevention and detection
pub mod brute_force_protector;
/// Client policy enforcement and validation
pub mod client_policy;
/// OAuth 2.0 Dynamic Client Registration (RFC 7591/7592)
pub mod client_registration;
/// Service managers for coordinating complex operations
pub mod managers;
/// Pushed Authorization Requests (PAR) implementation
pub mod par;
/// Password policy enforcement and validation
pub mod password_policy;
/// Comprehensive security testing framework - Enterprise-grade implementation
pub mod security_testing;

// Storage abstraction layer
// pub mod storage;

// Storage services
/// Group data storage and management
pub mod group_store;
/// JWT validation service with caching support
pub mod jwt_validator;
/// OIDC client storage and management
pub mod oidc_client_store;
/// OIDC authorization code storage
pub mod oidc_code_store;
/// Permission ticket storage and management
pub mod permission_ticket_store;
/// Resource server storage and management
pub mod resource_server_store;
/// Resource data storage and management
pub mod resource_store;
/// Scope storage and management
pub mod scope_store;
/// Session storage and management
pub mod session_store;
/// Token management and validation (OAuth2/OIDC tokens)
pub mod token;
/// TOTP secret storage and management
pub mod totp_store;

/// MFA administration service for managing lockouts and settings
pub mod mfa_admin_service;
/// MFA audit logging with enhanced security context
pub mod mfa_audit_logger;
/// MFA client with automatic fallback and sync
pub mod mfa_fallback_client;
/// Local encrypted storage fallback for MFA secrets
pub mod mfa_local_storage;
/// MFA performance monitoring and metrics collection
pub mod mfa_performance_monitor;
/// MFA security monitoring and anomaly detection service
pub mod mfa_security_monitor;
/// Multi-Factor Authentication service wrapper
pub mod mfa_service;

/// Automatic key rotation service for enhanced security
pub mod key_rotation;

/// AI-resistant CAPTCHA service for bot detection and prevention
pub mod captcha;

// Caching services
/// Caching services for performance optimization
pub mod cache;
/// Cache invalidation event listener for automatic cache management
pub mod cache_invalidation_listener;
/// Database optimization for MFA operations
pub mod database_optimizer;

// Core entity stores from services/stores/
/// Core entity storage services
pub mod stores {
    /// Audit log storage service
    pub mod audit_log_store;
    /// Authentication flow storage service
    pub mod auth_flow_store;
    /// Consent storage service for GDPR compliance
    pub mod consent_store;
    /// Permission storage service
    pub mod permission_store;
    /// Realm storage service
    pub mod realm_store;
    /// Role storage service
    pub mod role_store;
    /// Social account storage service
    pub mod social_account_store;
    /// User storage service
    pub mod user_store;

    // Re-export commonly used types from sub-modules
    pub use consent_store::{ConsentStore, ConsentStoreTrait};
    pub use user_store::{UserStore, UserStoreTrait};
}

// Re-export for convenience
pub use stores::*;

// Audit and logging services
/// Comprehensive audit event helpers
pub mod audit_events;
/// Audit integrity checker for periodic verification
pub mod audit_integrity;
/// Audit log sink interface and implementations
pub mod audit_log_sink;
/// Audit signature service for tamper-proof logging
pub mod audit_signature;
/// Elasticsearch-based audit log streaming for SIEM integration
pub mod elasticsearch_audit_log_sink;
/// Enhanced audit logging with comprehensive context capture
pub mod enhanced_audit;
/// Event listener implementations
pub mod event_listeners;
/// Enhanced event publisher with reliability features
pub mod event_publisher;
/// Event retention and lifecycle management
pub mod event_retention;
#[cfg(test)]
mod event_retention_tests;
/// Event system and listener management
pub mod events;
/// Kafka-based audit log streaming
pub mod kafka_audit_log_sink;
/// Kafka-based event streaming
pub mod kafka_event_listener;
/// PostgreSQL audit log storage
pub mod pg_audit_log_store;
/// PostgreSQL event storage
pub mod pg_event_store;

// Federation services
/// Identity broker and federation management
pub mod broker;
/// Federation protocol implementations
pub mod federation;
/// Federation provider integrations
pub mod federation_provider;

// Authorization services
/// Authorization policy engine and enforcement
pub mod authorization;
/// Satker-aware authorization with hierarchy support
pub mod satker_authorization;

// Zero Trust services
/// Zero Trust security model implementation
pub mod zero_trust;

// Social login services
/// Social identity provider integrations
pub mod social;

// Admin services
/// Administrative operations and management
pub mod admin;
/// Delegated administration for realm-specific admin access
pub mod delegated_admin;

// WebAuthn services
/// WebAuthn/FIDO2 authentication services
pub mod webauthn;

// Organization services
/// Organization management and multi-tenancy
pub mod organization;

/// Realm services
/// Multi-tenant realm management and isolation
pub mod realm;

// SAML services
/// SAML 2.0 protocol implementation
pub mod saml;
/// SAML signature and message storage
pub mod saml_signature;

// OID4VC services
/// OpenID for Verifiable Credentials
pub mod oid4vc;

// SSO services
/// Single Sign-On (SSO) orchestration across protocols
pub mod sso;

// Kubernetes operator services
/// Kubernetes operator for Authenc
pub mod kubernetes;

// Device management services
/// Device trust and management
pub mod device;

// Enterprise-grade services
/// Clustering and high availability
pub mod clustering;
/// Compliance checking and validation services
pub mod compliance;
/// Compliance mode management for different regulatory frameworks
pub mod compliance_mode;
/// FIPS compliance and cryptographic modules
pub mod fips;
/// Forever unknown secrets service (never persisted, auto-rotating)
pub mod forever_unknown_secrets;
/// Observability and monitoring
pub mod observability;
/// Secret management service (legacy vault providers)
/// Note: Main secret management is through crate::secreton_client module
pub mod vault;

// Re-exports for convenience
pub use anomaly_detector::AnomalyDetector;
pub use auth_flow::AuthenticationManager;
pub use brute_force_protector::BruteForceProtector;
pub use client_registration::{ClientRegistrationService, DefaultClientRegistrationService};
pub use group_store::GroupStore;
pub use session_store::SessionStore;
pub use totp_store::TotpStore;

// MFA services
pub use crate::secreton_client::secreton_client::MfaSetupData;
pub use mfa_admin_service::{
    AccountLockoutInfo, AutoUnlockService, MfaAdminResult, MfaAdminService,
};
pub use mfa_security_monitor::{
    AlertSeverity, MfaSecurityMonitor, MfaSecurityMonitorConfig, SecurityAlert,
};
pub use mfa_service::{MfaService, MfaSetupResponse, MfaStatus};

// CAPTCHA services
pub use captcha::{
    BehavioralAnalyzer, BehavioralAnalyzerTrait, BehavioralMetrics, CaptchaError, CaptchaService,
    CaptchaServiceTrait, Challenge, ChallengeGenerator, ChallengeGeneratorTrait,
    ChallengeValidatorTrait, ValidationResult as CaptchaValidationResult,
};

// Event publishing services
pub use event_publisher::{DlqEntry, EventPublisher, EventPublisherConfig, PublishableEvent};

// JWT validation services
pub use jwt_validator::{JwtValidator, ValidationResult as JwtValidationResult};

// Audit event helpers
pub use audit_events::{
    create_mfa_disabled_admin_event, create_mfa_disabled_event, create_mfa_enabled_admin_event,
    create_mfa_enabled_event, create_password_changed_event, create_permission_granted_admin_event,
    create_permission_revoked_admin_event, create_user_login_event, create_user_logout_event,
};
