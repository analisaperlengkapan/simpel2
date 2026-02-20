//! Legacy database operations modules
//!
//! This directory contains the modularized legacy operations that were
//! previously in a single operations_legacy.rs file (11,263 lines).
//! Each module has been extracted to its own file for better maintainability.
//!
//! Refactored: November 11, 2024

// Re-export common types that all submodules need
pub use crate::Database;
pub use authenc_types::{AuthencError, Result, DateTime, Utc};
pub use chrono::Duration;
pub use serde_json::Value as JsonValue;
pub use tracing::{debug, error, info, warn};
pub use uuid::Uuid;

pub mod admin_console;
// pub mod audit; // Disabled - needs models
pub mod auth_flows;
pub mod authenticators;
// pub mod devices; // Disabled - needs models
// pub mod event_functions; // Disabled - needs models
pub mod events;
// pub mod federated_identities; // Disabled - needs models
pub mod federated_identity;
// pub mod groups; // Disabled - needs models
pub mod identity_providers;
// pub mod oauth2; // Disabled - needs models
pub mod oauth2_providers;
// pub mod organizations; // Disabled - needs models
// pub mod permission_tickets; // Disabled - needs models
pub mod protocol_mappers;
// pub mod realms; // Disabled - needs models
// pub mod resource_servers; // Disabled - needs models
// pub mod resources; // Disabled - needs models
// pub mod roles; // Disabled - needs models
// pub mod saml; // Disabled - needs models
// pub mod scopes; // Disabled - needs models
// pub mod service_accounts; // Disabled - needs models
pub mod sessions;
// pub mod social_accounts; // Disabled - needs models
pub mod themes;
pub mod tokens;
// pub mod user_consents; // Disabled - needs models
// pub mod users; // Disabled - needs models
// pub mod webauthn; // Disabled - needs models

// Re-export top-level event functions for backward compatibility
// Disabled until models are migrated
// pub use event_functions::{
//     clear_old_admin_events, clear_old_events, query_admin_events, query_events, store_admin_event,
//     store_event,
// };
