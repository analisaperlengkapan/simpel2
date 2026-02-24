//! # Store Layer
//!
//! High-level store implementations for domain entities in Authenc.
//!
//! This module provides store abstractions that encapsulate database operations
//! for various domain entities. Stores follow the repository pattern and provide
//! trait-based interfaces for dependency injection and testing.
//!
//! ## Available Stores
//!
//! - [`UserStore`]: User management and authentication
//! - [`RealmStore`]: Multi-tenant realm management
//! - [`RoleStore`]: Role-based access control
//! - [`PermissionStore`]: Permission management
//! - [`ConsentStore`]: User consent tracking for OAuth2/OIDC
//! - [`AuthFlowStore`]: Authentication flow and session management
//! - [`SocialAccountStore`]: Social login account linking
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use authenc_core::stores::{UserStore, UserStoreTrait};
//! use authenc_storage::Database;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let db = Database::new("postgres://...", Default::default()).await?;
//! let user_store = UserStore::new(Arc::new(db));
//!
//! // Use the store
//! let user = user_store.get_user_by_username("admin").await?;
//! # Ok(())
//! # }
//! ```

// Module declarations
pub mod auth_flow_store;
pub mod consent_store;
pub mod permission_store;
pub mod realm_store;
pub mod role_store;
pub mod social_account_store;
pub mod user_store;

// Re-export store types and traits for convenience
pub use auth_flow_store::{AuthFlowStore, AuthFlowStoreTrait};
pub use consent_store::{ConsentStore, ConsentStoreTrait};
pub use permission_store::PermissionStore;
pub use realm_store::RealmStore;
pub use role_store::RoleStore;
pub use social_account_store::{SocialAccountStore, SocialAccountStoreTrait};
pub use user_store::{UserStore, UserStoreTrait};
