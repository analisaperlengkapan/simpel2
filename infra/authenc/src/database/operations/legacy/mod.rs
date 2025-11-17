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

pub mod admin_console;
pub mod audit;
pub mod auth_flows;
pub mod authenticators;
pub mod devices;
pub mod event_functions; // Top-level event functions (not in events module)
pub mod events;
pub mod federated_identities;
pub mod federated_identity;
pub mod groups;
pub mod identity_providers;
pub mod oauth2;
pub mod oauth2_providers;
pub mod organizations;
pub mod permission_tickets;
pub mod protocol_mappers;
pub mod realms;
pub mod resource_servers;
pub mod resources;
pub mod roles;
pub mod saml;
pub mod scopes;
pub mod service_accounts;
pub mod sessions;
pub mod social_accounts;
pub mod themes;
pub mod tokens;
pub mod user_consents;
pub mod users;
pub mod webauthn;

// Re-export top-level event functions for backward compatibility
pub use event_functions::{
    clear_old_admin_events, clear_old_events, query_admin_events, query_events, store_admin_event,
    store_event,
};
