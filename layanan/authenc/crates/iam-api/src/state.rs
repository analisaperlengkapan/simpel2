//! IAM API state with admin service dependencies

use authenc_core::services::UserManagementServiceImpl;
use authenc_crypto::jwt::JwtService;
use authenc_storage::{Database, PostgresRevocationStore};
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

    /// Token revocation list. The admin guard consults it: a revoked admin
    /// token must not keep administering the IdP until it expires. Built from
    /// `database` in [`IamApiState::new`], so the constructor signature is
    /// unchanged.
    pub revocation_store: Arc<PostgresRevocationStore>,
}

impl IamApiState {
    /// Create a new IAM API state with all dependencies
    pub fn new(
        user_service: Arc<UserManagementServiceImpl>,
        jwt_service: Arc<JwtService>,
        database: Arc<Database>,
    ) -> Self {
        let revocation_store = Arc::new(PostgresRevocationStore::new(database.clone()));
        Self {
            user_service,
            jwt_service,
            database,
            revocation_store,
        }
    }

    /// Invalidate every token the user was issued before now.
    ///
    /// A role revoked, an account disabled/deleted or a password reset by an
    /// administrator must take effect *now*, not when the access token happens
    /// to expire (up to 15 minutes) or, for a refresh token, days later. The
    /// revocation cutoff rejects every token with `iat` before this moment;
    /// tokens minted by a fresh login stay valid. The row is kept for longer
    /// than the longest-lived token (refresh, 7 days) and then purged.
    ///
    /// Failure is logged, not propagated: the administrative change itself has
    /// already been committed and must not be reported as failed.
    pub async fn revoke_user_sessions(&self, user_id: uuid::Uuid, reason: &str) {
        let expires_at = chrono::Utc::now() + chrono::Duration::days(8);
        if let Err(e) = self
            .revocation_store
            .revoke_user(&user_id.to_string(), expires_at, Some(reason))
            .await
        {
            tracing::error!(%user_id, reason, "could not revoke the user's sessions: {e}");
        }
    }
}
