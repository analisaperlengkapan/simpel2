// Database operations submodules
// NOTE: Some modules temporarily disabled until models are migrated to authenc-types
// TODO: Re-enable after Phase 3 (model migration)
pub mod client_registration_ops;
// pub mod client_scopes_ops;
// pub mod dynamic_role_ops;
pub mod protocol_mappers_ops;
pub mod tokens;

// Re-export with shorter alias for backwards compatibility
pub use client_registration_ops as client_registration;
pub use protocol_mappers_ops as protocol_mappers;

// Re-export sub-modules from client_scopes_ops for backward compatibility
// pub use client_scopes_ops::{
//     client_scope_assignments, client_scopes, scope_validation, user_consent_scopes,
// };

// Legacy operations now modularized into separate files
// Previously in operations_legacy.rs (11,263 lines), now split into 29 modules
// for better maintainability and navigation
pub mod legacy;

// Re-export all legacy modules for backward compatibility
// This allows existing code to use `operations::groups` instead of `operations::legacy::groups`
// NOTE: Many modules disabled until models are migrated
pub use legacy::{
    admin_console,
    // audit, // Disabled - needs models
    auth_flows, authenticators,
    // devices, // Disabled - needs models
    events,
    // federated_identities, // Disabled - needs models
    federated_identity,
    // groups, // Disabled - needs models
    identity_providers,
    // oauth2, // Disabled - needs models
    oauth2_providers,
    // organizations, // Disabled - needs models
    // permission_tickets, // Disabled - needs models
    // protocol_mappers, // Disabled - conflicts with new protocol_mappers_ops
    // realms, // Disabled - needs models
    // resource_servers, resources, // Disabled - needs models
    // roles, // Disabled - needs models
    // saml, // Disabled - needs models
    // scopes, // Disabled - needs models
    // service_accounts, // Disabled - needs models
    sessions,
    // social_accounts, // Disabled - needs models
    themes,
    // user_consents, users, webauthn, // Disabled - needs models
};

// Re-export top-level functions from legacy for backward compatibility
// Disabled until models are migrated
// pub use legacy::{
//     clear_old_admin_events, clear_old_events, query_admin_events, query_events, store_admin_event,
//     store_event,
// };
