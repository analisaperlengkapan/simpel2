//! Store implementations for domain entities

pub mod user_store;
pub mod session_store;
pub mod realm_store;
pub mod client_store;
pub mod credential_store;

// Re-export store implementations
pub use user_store::PostgresUserStore;
pub use session_store::PostgresSessionStore;
pub use realm_store::PostgresRealmStore;
pub use client_store::PostgresClientStore;
pub use credential_store::PostgresCredentialStore;
