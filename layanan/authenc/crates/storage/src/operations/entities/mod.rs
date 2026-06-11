//! Database operations modules
//!
//! SQL-based database operations for all authenc entities.

// Re-export common types that all submodules need
pub use crate::Database;
pub use authenc_types::{AuthencError, DateTime, Result, Utc};
pub use chrono::Duration;
pub use serde_json::Value as JsonValue;
pub use tracing::{debug, error, info, warn};
pub use uuid::Uuid;

pub mod admin_console;
pub mod audit;
pub mod auth_flows;
pub mod authenticators;
pub mod devices;
// pub mod event_functions; // Disabled: uses authenc_core::spi types (circular dep)
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
