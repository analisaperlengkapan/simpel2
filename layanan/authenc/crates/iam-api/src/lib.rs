//! # authenc-iam-api
//!
//! IAM Administration REST API for Authenc identity provider.
//!
//! This crate provides admin HTTP endpoints for:
//! - User management (CRUD, password reset, MFA setup)
//! - Realm management
//! - OAuth2 client management
//! - Role and permission management
//! - Federation/SSO configuration
//! - Audit log access
//! - System configuration

// Re-export types from authenc-types
pub use authenc_types::*;

pub mod error;
pub mod handlers;
pub mod middleware;
pub mod router;
pub mod routes;
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
