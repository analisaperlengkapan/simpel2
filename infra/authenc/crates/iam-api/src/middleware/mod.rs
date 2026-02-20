//! IAM API middleware

pub mod admin_auth;

pub use admin_auth::{admin_auth_middleware, require_permission, AdminUser};
