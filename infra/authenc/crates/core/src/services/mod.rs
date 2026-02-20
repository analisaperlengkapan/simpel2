//! Core business services

pub mod authentication_service;
pub mod user_management_service;
pub mod realm_management_service;
pub mod oauth2_service;
pub mod brute_force_protector;
pub mod role_management_service;
pub mod audit_service;
pub mod anomaly_detector;
pub mod risk_engine;
pub mod auth_flow;
pub mod password_policy;
pub mod session_store;
pub mod sso_cookie;
pub mod realm;
pub mod organization;
pub mod satker_authorization;

// OAuth2/OIDC services
// Note: OidcClientStore, OidcCodeStore, ClientScopeService, and DeviceAuthorizationService
// are NOT migrated to crates/core. They remain in src/ for OLD architecture.
// NEW architecture uses OAuth2ServiceImpl (trait-based design) instead.
pub mod client_registration;
pub mod protocol_mapper_service;
pub mod scope_store;
pub mod service_account_store;
pub mod token_exchange;

// UMA 2.0 services
pub mod resource_store;
pub mod resource_server_store;
pub mod permission_ticket_store;
pub mod uma_policy_store;
pub mod uma;

// Audit services
pub mod audit_events;
pub mod audit_integrity;
pub mod audit_signature;
pub mod enhanced_audit;
pub mod pg_audit_log_store;
pub mod audit_log_sink;
pub mod elasticsearch_audit_log_sink;
#[cfg(feature = "kafka")]
pub mod kafka_audit_log_sink;

// Event services
#[cfg(feature = "kafka")]
pub mod event_publisher;
#[cfg(feature = "aws")]
pub mod event_retention;
pub mod event_listeners;
pub mod events;
pub mod pg_event_store;
#[cfg(feature = "kafka")]
pub mod kafka_event_listener;

// Cache services
pub mod cache;
pub mod cache_invalidation_listener;
pub mod cache_utils;

// Advanced services (Enterprise features)
pub mod client_policy;
pub mod authorization;
pub mod zero_trust;
pub mod compliance;
#[cfg(feature = "fips")]
pub mod fips;
pub mod clustering;
pub mod observability;
pub mod key_rotation;
pub mod database_optimizer;
pub mod config_manager;

// FAPI-2 services (OPTIONAL)
pub mod par;

pub use authentication_service::AuthenticationServiceImpl;
pub use user_management_service::UserManagementServiceImpl;
pub use realm_management_service::RealmManagementServiceImpl;
pub use oauth2_service::OAuth2ServiceImpl;
pub use brute_force_protector::{BruteForceProtectorImpl, BruteForceConfig};
pub use role_management_service::{RoleManagementServiceImpl, Role, Permission};
pub use audit_service::AuditService;
pub use anomaly_detector::{AnomalyDetector, AnomalyDetectorTrait};
pub use risk_engine::RiskEngine;
pub use auth_flow::{
    AuthenticationManager, AuthenticationContext, AuthenticationFlowModel,
    AuthenticationFlowType, AuthenticationSessionManager, DefaultAuthenticationFlowResolver,
};
pub use password_policy::{PasswordPolicyService, PasswordPolicyConfig, PasswordPolicyValidationResult};
pub use session_store::SessionStore;
pub use sso_cookie::{SsoCookieManager, SsoSession};
pub use realm::{RealmService, PostgresRealmService, RealmManager};
pub use organization::{OrganizationService, OrganizationRole, OrganizationInvitation, OrganizationSettings, OrganizationUpdate};
pub use satker_authorization::{SatkerAuthorizationService, SatkerHierarchyInfo};

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
pub use resource_store::{ResourceStore, ResourceStoreTrait};
pub use resource_server_store::{ResourceServerStore, ResourceServerStoreTrait};
pub use permission_ticket_store::{PermissionTicketStore, PermissionTicketStoreTrait};
pub use uma_policy_store::{UmaPolicyStore, UmaDelegationPolicyStore};
pub use uma::{
    ClaimsGatheringFlow, ClaimsGatheringRequest, ClaimsGatheringResponse, ClaimsGatheringService,
    PermissionEndpoint, PolicyEngine, PolicyEvaluationContext, PolicyEvaluationResult,
    UmaPolicy, PolicyType, PolicyDecision,
};

// Audit service exports
pub use audit_events::*;
pub use audit_integrity::*;
pub use audit_signature::*;
pub use enhanced_audit::{EnhancedAuditService, EnhancedAuditContext, create_audit_context, extract_audit_details};
pub use pg_audit_log_store::*;
pub use audit_log_sink::*;
pub use elasticsearch_audit_log_sink::*;
#[cfg(feature = "kafka")]
pub use kafka_audit_log_sink::*;

// Event service exports
#[cfg(feature = "kafka")]
pub use event_publisher::{EventPublisher, EventPublisherConfig, PublishableEvent, DlqEntry};
#[cfg(feature = "aws")]
pub use event_retention::*;
pub use event_listeners::*;
pub use events::*;
pub use pg_event_store::*;
#[cfg(feature = "kafka")]
pub use kafka_event_listener::*;

// Cache service exports
pub use cache::*;
pub use cache_invalidation_listener::*;
pub use cache_utils::*;

// Advanced service exports (Enterprise features)
pub use client_policy::*;
pub use authorization::*;
pub use zero_trust::*;
pub use compliance::*;
#[cfg(feature = "fips")]
pub use fips::*;
pub use clustering::*;
pub use observability::*;
pub use key_rotation::*;
pub use database_optimizer::*;
pub use config_manager::*;

// FAPI-2 service exports (OPTIONAL)
pub use par::*;
