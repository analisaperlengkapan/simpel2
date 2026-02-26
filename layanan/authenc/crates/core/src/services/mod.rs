//! Core business services

pub mod anomaly_detector;
pub mod audit_service;
pub mod auth_flow;
pub mod authentication_service;
pub mod brute_force_protector;
pub mod oauth2_service;
pub mod organization;
pub mod password_policy;
pub mod realm;
pub mod realm_management_service;
pub mod risk_engine;
pub mod role_management_service;
pub mod satker_authorization;
pub mod session_store;
pub mod sso_cookie;
pub mod user_management_service;

// OAuth2/OIDC services
// Note: OidcClientStore, OidcCodeStore, ClientScopeService, and DeviceAuthorizationService
// are NOT migrated to crates/core. They remain in src/ for OLD architecture.
// NEW architecture uses OAuth2ServiceImpl (trait-based design) instead.
pub mod client_registration;
pub mod protocol_mapper_service;
pub mod scope_store;
pub mod service_account_store;
pub mod token_exchange;
pub mod token_exchange_helpers;

// UMA 2.0 services
pub mod permission_ticket_store;
pub mod resource_server_store;
pub mod resource_store;
pub mod uma;
pub mod uma_policy_store;

// Audit services
pub mod audit_events;
pub mod audit_integrity;
pub mod audit_log_sink;
pub mod audit_signature;
#[cfg(feature = "elasticsearch")]
pub mod elasticsearch_audit_log_sink;
pub mod enhanced_audit;
#[cfg(feature = "kafka")]
pub mod kafka_audit_log_sink;
pub mod pg_audit_log_store;

// Event services
pub mod event_listeners;
#[cfg(feature = "kafka")]
pub mod event_publisher;
#[cfg(feature = "aws")]
pub mod event_retention;
pub mod events;
#[cfg(feature = "kafka")]
pub mod kafka_event_listener;
pub mod pg_event_store;

// Cache services
pub mod cache;
#[cfg(feature = "kafka")]
pub mod cache_invalidation_listener;
pub mod cache_utils;

// Identity Broker services
pub mod broker;

// Federation services
#[cfg(feature = "oidc-federation")]
pub mod advanced_federation;
pub mod federation;
pub mod federation_manager;
pub mod federation_provider;

// Advanced services (Enterprise features)
pub mod authorization;
pub mod client_policy;
pub mod clustering;
pub mod compliance;
pub mod config_manager;
pub mod database_optimizer;
#[cfg(feature = "fips")]
pub mod fips;
pub mod key_rotation;
pub mod observability;
pub mod zero_trust;

// FAPI-2 services (OPTIONAL)
pub mod par;

// Store implementations
pub mod group_store;
pub mod oidc_client_store;
pub mod software_statement_validator;

pub use anomaly_detector::{AnomalyDetector, AnomalyDetectorTrait};
pub use audit_service::AuditService;
pub use auth_flow::{
    AuthenticationContext, AuthenticationFlowModel, AuthenticationFlowType, AuthenticationManager,
    AuthenticationSessionManager, DefaultAuthenticationFlowResolver,
};
pub use authentication_service::AuthenticationServiceImpl;
pub use brute_force_protector::{BruteForceConfig, BruteForceProtectorImpl};
pub use oauth2_service::OAuth2ServiceImpl;
pub use organization::{
    OrganizationInvitation, OrganizationRole, OrganizationService, OrganizationSettings,
    OrganizationUpdate,
};
pub use password_policy::{
    PasswordPolicyConfig, PasswordPolicyService, PasswordPolicyValidationResult,
};
pub use realm::{PostgresRealmService, RealmManager, RealmService};
pub use realm_management_service::RealmManagementServiceImpl;
pub use risk_engine::RiskEngine;
pub use role_management_service::{Permission, Role, RoleManagementServiceImpl};
pub use satker_authorization::{SatkerAuthorizationService, SatkerHierarchyInfo};
pub use session_store::SessionStore;
pub use sso_cookie::{SsoCookieManager, SsoSession};
pub use user_management_service::UserManagementServiceImpl;

// OAuth2/OIDC service exports
// Note: OidcClientStore, OidcCodeStore, ClientScopeService, and DeviceAuthorizationService
// are NOT migrated to crates/core. They remain in src/ for OLD architecture.
// NEW architecture uses OAuth2ServiceImpl (trait-based design) instead.
pub use client_registration::{ClientRegistrationService, ProductionClientRegistrationService};
pub use protocol_mapper_service::ProtocolMapperService;
pub use scope_store::ScopeStore;
pub use service_account_store::ServiceAccountStore;
pub use token_exchange::TokenExchangeService;

// UMA 2.0 service exports
pub use permission_ticket_store::{PermissionTicketStore, PermissionTicketStoreTrait};
pub use resource_server_store::{ResourceServerStore, ResourceServerStoreTrait};
pub use resource_store::{ResourceStore, ResourceStoreTrait};
pub use uma::{
    ClaimsGatheringFlow, ClaimsGatheringRequest, ClaimsGatheringResponse, ClaimsGatheringService,
    PermissionEndpoint, PolicyDecision, PolicyEngine, PolicyEvaluationContext,
    PolicyEvaluationResult, PolicyType, UmaPolicy,
};
pub use uma_policy_store::{UmaDelegationPolicyStore, UmaPolicyStore};

// Audit service exports
pub use audit_events::*;
pub use audit_integrity::*;
pub use audit_log_sink::*;
pub use audit_signature::*;
#[cfg(feature = "elasticsearch")]
pub use elasticsearch_audit_log_sink::*;
pub use enhanced_audit::{
    EnhancedAuditContext, EnhancedAuditService, create_audit_context, extract_audit_details,
};
#[cfg(feature = "kafka")]
pub use kafka_audit_log_sink::*;
pub use pg_audit_log_store::*;

// Event service exports
pub use event_listeners::*;
#[cfg(feature = "kafka")]
pub use event_publisher::{DlqEntry, EventPublisher, EventPublisherConfig, PublishableEvent};
#[cfg(feature = "aws")]
pub use event_retention::*;
pub use events::*;
#[cfg(feature = "kafka")]
pub use kafka_event_listener::*;
pub use pg_event_store::*;

// Cache service exports
pub use cache::*;
#[cfg(feature = "kafka")]
pub use cache_invalidation_listener::*;
pub use cache_utils::*;

// Identity Broker service exports
pub use broker::*;

// Federation service exports
#[cfg(feature = "oidc-federation")]
pub use advanced_federation::*;
pub use federation::*;
pub use federation_manager::*;
pub use federation_provider::*;

// Advanced service exports (Enterprise features)
pub use authorization::*;
pub use client_policy::*;
pub use clustering::*;
pub use compliance::*;
pub use config_manager::*;
pub use database_optimizer::*;
#[cfg(feature = "fips")]
pub use fips::*;
pub use key_rotation::*;
pub use observability::*;
pub use zero_trust::*;

// FAPI-2 service exports (OPTIONAL)
pub use par::*;
