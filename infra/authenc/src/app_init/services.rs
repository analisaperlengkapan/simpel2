//! Core services initialization
//!
//! Extracted from app.rs. Handles initialization of core services:
//! user store, session store, TOTP, realms, roles, permissions, resources, scopes.

use crate::config::AppConfig;
use std::sync::Arc;

/// Initialize user store
pub fn initialize_user_store(
    database: Arc<crate::database::Database>,
) -> Arc<crate::services::stores::user_store::UserStore> {
    Arc::new(crate::services::stores::user_store::UserStore::new(
        database,
    ))
}

/// Initialize session store
pub fn initialize_session_store(
    database: Arc<crate::database::Database>,
) -> Arc<crate::services::session_store::SessionStore> {
    Arc::new(crate::services::session_store::SessionStore::new(database))
}

/// Initialize TOTP store
pub fn initialize_totp_store() -> Arc<crate::services::totp_store::TotpStore> {
    Arc::new(crate::services::totp_store::TotpStore::new())
}

/// Initialize realm store
pub fn initialize_realm_store() -> Arc<crate::services::stores::realm_store::RealmStore> {
    Arc::new(crate::services::stores::realm_store::RealmStore::new())
}

/// Initialize realm service
pub fn initialize_realm_service(
    database: Arc<crate::database::Database>,
) -> Arc<dyn crate::services::realm::RealmService> {
    Arc::new(crate::services::realm::PostgresRealmService::new(database))
}

/// Initialize role store
pub fn initialize_role_store() -> Arc<crate::services::stores::role_store::RoleStore> {
    Arc::new(crate::services::stores::role_store::RoleStore::new())
}

/// Initialize permission store
pub fn initialize_permission_store()
-> Arc<crate::services::stores::permission_store::PermissionStore> {
    Arc::new(crate::services::stores::permission_store::PermissionStore::new())
}

/// Initialize resource store
pub fn initialize_resource_store(
    database: Arc<crate::database::Database>,
) -> Arc<crate::services::resource_store::ResourceStore> {
    Arc::new(crate::services::resource_store::ResourceStore::new(
        database,
    ))
}

/// Initialize resource server store
pub fn initialize_resource_server_store(
    database: Arc<crate::database::Database>,
) -> Arc<crate::services::resource_server_store::ResourceServerStore> {
    Arc::new(crate::services::resource_server_store::ResourceServerStore::new(database))
}

/// Initialize permission ticket store
pub fn initialize_permission_ticket_store(
    database: Arc<crate::database::Database>,
) -> Arc<crate::services::permission_ticket_store::PermissionTicketStore> {
    Arc::new(crate::services::permission_ticket_store::PermissionTicketStore::new(database))
}

/// Initialize scope store
pub fn initialize_scope_store(
    database: Arc<crate::database::Database>,
) -> Arc<crate::services::scope_store::ScopeStore> {
    Arc::new(crate::services::scope_store::ScopeStore::new(database))
}

/// Initialize client scope service
pub fn initialize_client_scope_service(
    database: Arc<crate::database::Database>,
) -> Arc<crate::services::client_scope_service::ClientScopeService> {
    Arc::new(crate::services::client_scope_service::ClientScopeService::new(database))
}

/// Initialize protocol mapper service
pub fn initialize_protocol_mapper_service(
    database: Arc<crate::database::Database>,
) -> Arc<crate::services::protocol_mapper_service::ProtocolMapperService> {
    Arc::new(crate::services::protocol_mapper_service::ProtocolMapperService::new(database))
}

/// Initialize OIDC client store
pub fn initialize_oidc_client_store(
    database: Arc<crate::database::Database>,
) -> Arc<crate::services::oidc_client_store::OidcClientStore> {
    Arc::new(crate::services::oidc_client_store::OidcClientStore::with_database(database))
}

/// Initialize service account store
pub fn initialize_service_account_store(
    database: Arc<crate::database::Database>,
) -> Arc<crate::services::service_account_store::ServiceAccountStore> {
    Arc::new(crate::services::service_account_store::ServiceAccountStore::with_database(database))
}

/// Initialize social account store
pub fn initialize_social_account_store(
    database: Arc<crate::database::Database>,
) -> Arc<crate::services::stores::social_account_store::SocialAccountStore> {
    Arc::new(crate::services::stores::social_account_store::SocialAccountStore::new(database))
}

/// Initialize identity broker registry
pub fn initialize_broker_registry() -> Arc<crate::services::broker::IdentityBrokerRegistry> {
    Arc::new(crate::services::broker::IdentityBrokerRegistry::new())
}

/// Initialize OID4VC service
pub fn initialize_oid4vc_service() -> Arc<crate::services::oid4vc::EnhancedOid4VcManager> {
    Arc::new(crate::services::oid4vc::EnhancedOid4VcManager::new(
        "https://authenc.example.com".to_string(),
    ))
}
