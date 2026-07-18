//! IAM API router
//!
//! Routes only the admin surfaces the platform actually uses (portal admin
//! console + e2e): system stats, user management, role list/assignment,
//! OAuth2 client inspection, and audit logs. The Keycloak-parity route
//! blocks (realms, groups, organizations, federation/SSO, SPI, JIT, DCR,
//! client policies, UMA, zero-trust, OID4VC, satker admin) were removed in
//! the #45 trim — all were NotImplemented stubs with no consumer.

use axum::{
    Router, middleware,
    routing::{get, post},
};
use std::sync::Arc;

use crate::{
    handlers::{admin, audit, clients, roles, users},
    middleware::admin_auth_middleware,
    state::IamApiState,
};

/// Create the IAM API router
///
/// All routes are protected by the admin authentication middleware.
pub fn create_iam_router(state: Arc<IamApiState>) -> Router {
    Router::new()
        // ============================================================
        // Admin Dashboard Statistics
        // ============================================================
        .route("/api/v1/iam/admin/stats", get(admin::get_system_stats))
        // ============================================================
        // User Management
        // ============================================================
        .route(
            "/api/v1/iam/users",
            get(users::list_users).post(users::create_user),
        )
        .route(
            "/api/v1/iam/users/{id}",
            get(users::get_user)
                .put(users::update_user)
                .delete(users::delete_user),
        )
        .route(
            "/api/v1/iam/users/{id}/password/reset",
            post(users::reset_user_password),
        )
        .route(
            "/api/v1/iam/users/{id}/mfa/enable",
            post(users::enable_user_mfa),
        )
        .route(
            "/api/v1/iam/users/{id}/mfa/disable",
            post(users::disable_user_mfa),
        )
        // ============================================================
        // Role Listing and Assignment
        // ============================================================
        .route("/api/v1/iam/roles", get(roles::list_roles))
        .route(
            "/api/v1/iam/users/{user_id}/roles/{role_id}",
            post(roles::assign_role_to_user).delete(roles::remove_role_from_user),
        )
        // ============================================================
        // OAuth2 Client Inspection (read-only; clients are seeded config)
        // ============================================================
        .route("/api/v1/iam/clients", get(clients::list_clients))
        .route("/api/v1/iam/clients/{id}", get(clients::get_client))
        // ============================================================
        // Audit Logs
        // ============================================================
        .route("/api/v1/iam/audit-logs", get(audit::list_audit_logs))
        .route(
            "/api/v1/iam/audit-logs/export",
            get(audit::export_audit_logs),
        )
        // ============================================================
        // Apply admin authentication middleware to MATCHED routes only.
        //
        // `route_layer` (vs `layer`) scopes the middleware to routes that
        // exist in this router. Without this, the layer wraps the entire
        // Router service and runs on UNMATCHED paths after `.merge()`:
        // hitting `/api/v1/auth/health` or any non-existent path would
        // return 401 "Missing Authorization header" instead of 404,
        // making the API appear universally auth-gated.
        // ============================================================
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            admin_auth_middleware,
        ))
        .with_state(state)
}
