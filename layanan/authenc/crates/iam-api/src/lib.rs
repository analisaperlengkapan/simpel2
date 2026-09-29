//! # authenc-iam-api
//!
//! IAM Administration REST API for Authenc identity provider.
//!
//! This crate provides admin HTTP endpoints for:
//! - User management (CRUD, password reset, MFA enable/disable)
//! - Role listing and user-role assignment
//! - OAuth2 client inspection (read-only; clients are seeded config)
//! - Audit log access and export
//! - System statistics

// Re-export types from authenc-types
pub use authenc_types::*;

pub mod audit_trail;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod router;
pub mod state;

// Re-export the main router creation function for convenience
pub use router::create_iam_router;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
