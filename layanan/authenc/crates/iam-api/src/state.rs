//! IAM API state with admin service dependencies

use authenc_core::services::UserManagementServiceImpl;
use authenc_crypto::jwt::JwtService;
use authenc_storage::Database;
use std::sync::Arc;

/// IAM API state containing the admin service dependencies.
///
/// Trimmed with the #45 audit: the realm/group/satker services the old
/// state carried (plus a wall of "TODO: add service" placeholders for
/// handlers that were never implemented) served no mounted route. Roles,
/// clients, audit logs, and stats query the database directly — the same
/// pattern the api crate's session handlers use.
#[derive(Clone)]
pub struct IamApiState {
    /// User management service for CRUD operations
    pub user_service: Arc<UserManagementServiceImpl>,

    /// JWT service for admin-token validation (admin_auth middleware)
    pub jwt_service: Arc<JwtService>,

    /// Direct database access for roles / clients / audit-log queries
    pub database: Arc<Database>,
}

impl IamApiState {
    /// Create a new IAM API state with all dependencies
    pub fn new(
        user_service: Arc<UserManagementServiceImpl>,
        jwt_service: Arc<JwtService>,
        database: Arc<Database>,
    ) -> Self {
        Self {
            user_service,
            jwt_service,
            database,
        }
    }
}
