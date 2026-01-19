//! Legacy database operations modules
//!
//! This directory contains the modularized legacy operations that were
//! previously in a single operations_legacy.rs file (11,263 lines).
//! Each module has been extracted to its own file for better maintainability.
//!
//! Refactored: November 11, 2024

// Re-export common types that all submodules need
pub use crate::{
    database::Database,
    error::{AuthencError, Result},
};
pub use chrono::{DateTime, Duration, Utc};
pub use serde_json::Value as JsonValue;
pub use tracing::{debug, error, info, warn};
pub use uuid::Uuid;

/// Modul `admin_console`.
pub mod admin_console;
/// Modul `auth_flows`.
pub mod audit;
/// Modul `devices`.
pub mod auth_flows;
/// Modul `events`.
pub mod authenticators;
/// Modul `federated_identity`.
pub mod devices;
/// Modul `identity_providers`.
pub mod event_functions; // Top-level event functions (not in events module)
/// Modul `oauth2_providers`.
pub mod events;
/// Modul `permission_tickets`.
pub mod federated_identities;
/// Modul `realms`.
pub mod federated_identity;
/// Modul `resources`.
pub mod groups;
/// Modul `saml`.
pub mod identity_providers;
/// Modul `service_accounts`.
pub mod oauth2;
/// Modul `social_accounts`.
pub mod oauth2_providers;
/// Modul `tokens`.
pub mod organizations;
/// Modul `users`.
pub mod permission_tickets;
/// Modul `protocol_mappers`.
pub mod protocol_mappers;
/// Modul `resource_servers`.
pub mod realms;
/// Modul `roles`.
pub mod resource_servers;
/// Modul `scopes`.
pub mod resources;
/// Modul `sessions`.
pub mod roles;
/// Modul `themes`.
pub mod saml;
/// Modul `user_consents`.
pub mod scopes;
/// Modul `webauthn`.
pub mod service_accounts;
/// Modul `sessions`.
pub mod sessions;
/// Modul `themes`.
pub mod social_accounts;
/// Modul `user_consents`.
pub mod themes;
/// Modul `webauthn`.
pub mod tokens;
/// Modul `user_consents`.
pub mod user_consents;
/// Modul `webauthn`.
pub mod users;
pub mod webauthn;

// Re-export top-level event functions for backward compatibility
pub use event_functions::{
    clear_old_admin_events, clear_old_events, query_admin_events, query_events, store_admin_event,
    store_event,
};
