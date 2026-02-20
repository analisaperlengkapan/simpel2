//! IAM API route definitions

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

use crate::{
    handlers::{audit, clients, federation, realms, roles, users},
    middleware::admin_auth_middleware,
    state::IamApiState,
};

/// Create IAM API router with all endpoints
pub fn create_router(state: Arc<IamApiState>) -> Router {
    Router::new()
        // User management endpoints
        .route("/api/v1/iam/users", get(users::list_users).post(users::create_user))
        .route(
            "/api/v1/iam/users/:id",
            get(users::get_user)
                .put(users::update_user)
                .delete(users::delete_user),
        )
        .route("/api/v1/iam/users/:id/password/reset", post(users::reset_user_password))
        .route("/api/v1/iam/users/:id/mfa/enable", post(users::enable_user_mfa))

        // Realm management endpoints
        .route("/api/v1/iam/realms", get(realms::list_realms).post(realms::create_realm))
        .route(
            "/api/v1/iam/realms/:id",
            get(realms::get_realm)
                .put(realms::update_realm)
                .delete(realms::delete_realm),
        )

        // OAuth2 client management endpoints
        .route("/api/v1/iam/clients", get(clients::list_clients).post(clients::create_client))
        .route(
            "/api/v1/iam/clients/:id",
            get(clients::get_client)
                .put(clients::update_client)
                .delete(clients::delete_client),
        )
        .route("/api/v1/iam/clients/:id/secret/regenerate", post(clients::regenerate_client_secret))

        // Role management endpoints
        .route("/api/v1/iam/roles", get(roles::list_roles).post(roles::create_role))
        .route(
            "/api/v1/iam/roles/:id",
            put(roles::update_role)
                .delete(roles::delete_role),
        )
        .route(
            "/api/v1/iam/users/:user_id/roles/:role_id",
            post(roles::assign_role_to_user)
                .delete(roles::remove_role_from_user),
        )

        // Federation management endpoints
        .route(
            "/api/v1/iam/identity-providers",
            get(federation::list_identity_providers)
                .post(federation::create_identity_provider),
        )
        .route(
            "/api/v1/iam/identity-providers/:id",
            put(federation::update_identity_provider)
                .delete(federation::delete_identity_provider),
        )

        // Audit log endpoints
        .route("/api/v1/iam/audit-logs", get(audit::list_audit_logs))
        .route("/api/v1/iam/audit-logs/export", get(audit::export_audit_logs))

        // Apply admin authentication middleware to all routes
        .layer(middleware::from_fn_with_state(
            state.clone(),
            admin_auth_middleware,
        ))
        .with_state(state)
}
