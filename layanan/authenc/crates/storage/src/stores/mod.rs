//! Store implementations for domain entities

pub mod client_store;
pub mod credential_store;
pub mod realm_store;
pub mod revocation_store;
pub mod session_store;
pub mod user_store;

// Re-export store implementations
pub use client_store::PostgresClientStore;
pub use credential_store::PostgresCredentialStore;
pub use realm_store::PostgresRealmStore;
pub use revocation_store::PostgresRevocationStore;
pub use session_store::PostgresSessionStore;
pub use user_store::PostgresUserStore;
