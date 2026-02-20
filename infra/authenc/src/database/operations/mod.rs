// Database operations submodules
pub mod client_registration_ops;
pub mod client_scopes_ops;
pub mod dynamic_role_ops;
pub mod protocol_mappers_ops;
pub mod tokens;

// Re-export with shorter alias for backwards compatibility
pub use client_registration_ops as client_registration;

// Re-export sub-modules from client_scopes_ops for backward compatibility
pub use client_scopes_ops::{
    client_scope_assignments, client_scopes, scope_validation, user_consent_scopes,
};

// ⚠️ LEGACY OPERATIONS REMOVED
// All legacy operations have been migrated to crates/storage/src/operations/legacy/
// This directory (src/database/operations/legacy/) has been deleted.
//
// If you need legacy operations, import from:
//   use authenc_storage::operations::legacy::*;
//
// Or better yet, use the new store traits from authenc-storage:
//   use authenc_storage::stores::{UserStore, SessionStore, RealmStore, ClientStore, CredentialStore};
