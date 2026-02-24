// Database operations submodules
pub mod client_registration_ops;
pub mod protocol_mappers_ops;
pub mod tokens;

// Re-export with shorter aliases
pub use client_registration_ops as client_registration;
pub use protocol_mappers_ops as protocol_mappers;

// All database operations (organized by entity)
pub mod legacy;

pub use legacy::{
    admin_console,
    audit,
    auth_flows,
    authenticators,
    devices,
    // event_functions, // Disabled: circular dep on authenc_core::spi
    events,
    federated_identities,
    federated_identity,
    groups,
    identity_providers,
    oauth2,
    oauth2_providers,
    organizations,
    permission_tickets,
    realms,
    resource_servers,
    resources,
    roles,
    saml,
    scopes,
    service_accounts,
    sessions,
    social_accounts,
    themes,
    user_consents,
    users,
    webauthn,
};
