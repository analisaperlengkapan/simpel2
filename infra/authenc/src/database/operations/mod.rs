// Database operations submodules
/// Modul `client_registration_ops`.
pub mod client_registration_ops;
/// Modul `protocol_mappers_ops`.
pub mod client_scopes_ops;
/// Modul `protocol_mappers_ops`.
pub mod protocol_mappers_ops;
/// Modul `tokens`.
pub mod tokens;

// Re-export with shorter alias for backwards compatibility
pub use client_registration_ops as client_registration;

// Re-export sub-modules from client_scopes_ops for backward compatibility
pub use client_scopes_ops::{
    client_scope_assignments, client_scopes, scope_validation, user_consent_scopes,
};

// Legacy operations now modularized into separate files
/// Modul `legacy`.
// Previously in operations_legacy.rs (11,263 lines), now split into 29 modules
/// Modul `legacy`.
// for better maintainability and navigation
pub mod legacy;

// Re-export all legacy modules for backward compatibility
// This allows existing code to use `operations::groups` instead of `operations::legacy::groups`
pub use legacy::{
    admin_console, audit, auth_flows, authenticators, devices, events, federated_identities,
    federated_identity, groups, identity_providers, oauth2, oauth2_providers, organizations,
    permission_tickets, protocol_mappers, realms, resource_servers, resources, roles, saml, scopes,
    service_accounts, sessions, social_accounts, themes, user_consents, users, webauthn,
};

// Re-export top-level functions from legacy for backward compatibility
pub use legacy::{
    clear_old_admin_events, clear_old_events, query_admin_events, query_events, store_admin_event,
    store_event,
};
