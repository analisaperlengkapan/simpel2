//! IAM API unified router with all admin endpoints
//!
//! This module provides a comprehensive router for the IAM administration API,
//! including all endpoints for user management, realm management, client management,
//! federation, audit logs, and advanced features.

use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

use crate::{
    handlers::{
        admin, audit, client_policy, client_registration, clients, dcr_admin, federation,
        federation_admin, groups, jit_admin, oid4vc, organizations, realms, roles, satker,
        spi_federation, spi_management, uma, users, zero_trust,
    },
    middleware::admin_auth_middleware,
    state::IamApiState,
};

/// Create the unified IAM API router with all endpoints
///
/// This router includes:
/// - Admin dashboard and system statistics
/// - User management (CRUD, password reset, MFA)
/// - Realm management
/// - OAuth2 client management
/// - Role and permission management
/// - Group management
/// - Organization management
/// - Satker (government hierarchy) management
/// - JIT provisioning configuration
/// - Dynamic Client Registration (DCR)
/// - Client policy management
/// - Federation and SSO configuration
/// - SPI (Service Provider Interface) management
/// - UMA 2.0 (User-Managed Access)
/// - Zero Trust policies
/// - OID4VC (Verifiable Credentials)
/// - Audit log access
///
/// All routes are protected by admin authentication middleware.
pub fn create_iam_router(state: Arc<IamApiState>) -> Router {
    Router::new()
        // ============================================================
        // Admin Dashboard and System Statistics
        // ============================================================
        .route("/api/v1/iam/admin/stats", get(admin::get_system_stats))
        .route(
            "/api/v1/iam/admin/dashboard",
            get(admin::get_dashboard_data),
        )
        .route(
            "/api/v1/iam/admin/security-events",
            get(admin::get_security_events),
        )
        .route(
            "/api/v1/iam/admin/risk-analytics",
            get(admin::get_risk_analytics),
        )
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
        // Realm Management
        // ============================================================
        .route(
            "/api/v1/iam/realms",
            get(realms::list_realms).post(realms::create_realm),
        )
        .route(
            "/api/v1/iam/realms/{id}",
            get(realms::get_realm)
                .put(realms::update_realm)
                .delete(realms::delete_realm),
        )
        // ============================================================
        // OAuth2 Client Management
        // ============================================================
        .route(
            "/api/v1/iam/clients",
            get(clients::list_clients).post(clients::create_client),
        )
        .route(
            "/api/v1/iam/clients/{id}",
            get(clients::get_client)
                .put(clients::update_client)
                .delete(clients::delete_client),
        )
        .route(
            "/api/v1/iam/clients/{id}/secret/regenerate",
            post(clients::regenerate_client_secret),
        )
        // ============================================================
        // Role Management
        // ============================================================
        .route(
            "/api/v1/iam/roles",
            get(roles::list_roles).post(roles::create_role),
        )
        .route(
            "/api/v1/iam/roles/{id}",
            get(roles::get_role)
                .put(roles::update_role)
                .delete(roles::delete_role),
        )
        .route(
            "/api/v1/iam/users/{user_id}/roles/{role_id}",
            post(roles::assign_role_to_user).delete(roles::remove_role_from_user),
        )
        // ============================================================
        // Group Management
        // ============================================================
        .route(
            "/api/v1/iam/groups",
            get(groups::list_groups).post(groups::create_group),
        )
        .route(
            "/api/v1/iam/groups/{id}",
            get(groups::get_group_by_id)
                .put(groups::update_group)
                .delete(groups::delete_group),
        )
        .route(
            "/api/v1/iam/groups/{id}/members",
            get(groups::get_group_members).post(groups::add_group_member),
        )
        .route(
            "/api/v1/iam/groups/{id}/members/{user_id}",
            delete(groups::remove_group_member),
        )
        .route(
            "/api/v1/iam/groups/{id}/subgroups",
            get(groups::get_subgroups),
        )
        // ============================================================
        // Organization Management
        // ============================================================
        .route(
            "/api/v1/iam/organizations",
            get(organizations::list_organizations).post(organizations::create_organization),
        )
        .route(
            "/api/v1/iam/organizations/{id}",
            get(organizations::get_organization)
                .put(organizations::update_organization)
                .delete(organizations::delete_organization),
        )
        .route(
            "/api/v1/iam/organizations/{id}/members",
            get(organizations::get_members).post(organizations::add_member),
        )
        .route(
            "/api/v1/iam/organizations/{id}/members/{user_id}",
            delete(organizations::remove_member),
        )
        .route(
            "/api/v1/iam/organizations/{id}/invitations",
            post(organizations::create_invitation),
        )
        .route(
            "/api/v1/iam/organizations/{id}/settings",
            get(organizations::get_settings).put(organizations::update_settings),
        )
        // ============================================================
        // Satker (Government Hierarchy) Management
        // ============================================================
        // Satker identity is read-only here (SoT = integrasi/MySIMKARI, #42);
        // no POST/PUT — authenc does not master satker data.
        .route("/api/v1/iam/satker", get(satker::list_satkers))
        .route("/api/v1/iam/satker/{id}", get(satker::get_satker))
        .route(
            "/api/v1/iam/satker/{id}/hierarchy",
            get(satker::get_satker_hierarchy),
        )
        .route(
            "/api/v1/iam/satker/{id}/authorization",
            post(satker::check_satker_access),
        )
        // ============================================================
        // JIT (Just-In-Time) Provisioning Configuration
        // ============================================================
        .route(
            "/api/v1/iam/jit/config",
            get(jit_admin::get_jit_config).put(jit_admin::update_jit_config),
        )
        .route("/api/v1/iam/jit/stats", get(jit_admin::get_jit_stats))
        // ============================================================
        // Dynamic Client Registration (DCR) - RFC 7591/7592
        // ============================================================
        .route(
            "/api/v1/iam/dcr/register",
            post(client_registration::register_client),
        )
        .route(
            "/api/v1/iam/dcr/register/{client_id}",
            get(client_registration::get_client_configuration)
                .put(client_registration::update_client_configuration)
                .delete(client_registration::delete_client_registration),
        )
        // ============================================================
        // DCR Admin - Initial Access Tokens and Policies
        // ============================================================
        .route(
            "/api/v1/iam/dcr/initial-access-tokens",
            get(dcr_admin::list_initial_access_tokens).post(dcr_admin::create_initial_access_token),
        )
        .route(
            "/api/v1/iam/dcr/initial-access-tokens/{id}",
            delete(dcr_admin::revoke_initial_access_token),
        )
        .route(
            "/api/v1/iam/dcr/policies",
            get(dcr_admin::list_dcr_policies).post(dcr_admin::create_dcr_policy),
        )
        .route(
            "/api/v1/iam/dcr/policies/{id}",
            get(dcr_admin::get_dcr_policy)
                .put(dcr_admin::update_dcr_policy)
                .delete(dcr_admin::delete_dcr_policy),
        )
        // ============================================================
        // Client Policy Management
        // ============================================================
        .route(
            "/api/v1/iam/client-policies",
            get(client_policy::list_client_policies).post(client_policy::create_client_policy),
        )
        .route(
            "/api/v1/iam/client-policies/{id}",
            get(client_policy::get_client_policy)
                .put(client_policy::update_client_policy)
                .delete(client_policy::delete_client_policy),
        )
        .route(
            "/api/v1/iam/client-policy-profiles",
            get(client_policy::list_client_policies),
        )
        .route(
            "/api/v1/iam/clients/{client_id}/policies",
            get(client_policy::get_client_policy).post(client_policy::assign_client_policy),
        )
        .route(
            "/api/v1/iam/clients/{client_id}/policies/{policy_id}",
            delete(client_policy::unassign_client_policy),
        )
        // ============================================================
        // Federation and SSO Configuration
        // ============================================================
        .route(
            "/api/v1/iam/identity-providers",
            get(federation::list_identity_providers).post(federation::create_identity_provider),
        )
        .route(
            "/api/v1/iam/identity-providers/{id}",
            put(federation::update_identity_provider).delete(federation::delete_identity_provider),
        )
        // ============================================================
        // Federation Admin - Sync and Statistics
        // ============================================================
        .route(
            "/api/v1/iam/federation/sync",
            post(federation_admin::trigger_federation_sync),
        )
        .route(
            "/api/v1/iam/federation/stats",
            get(federation_admin::get_federation_stats),
        )
        // ============================================================
        // SPI (Service Provider Interface) Management
        // ============================================================
        .route(
            "/api/v1/iam/spi/plugins",
            get(spi_management::list_spi_plugins).post(spi_management::register_spi_plugin),
        )
        .route(
            "/api/v1/iam/spi/plugins/{id}",
            get(spi_management::get_spi_plugin)
                .put(spi_management::update_spi_plugin)
                .delete(spi_management::unregister_spi_plugin),
        )
        .route(
            "/api/v1/iam/spi/plugins/{id}/enable",
            post(spi_management::enable_plugin),
        )
        .route(
            "/api/v1/iam/spi/plugins/{id}/disable",
            post(spi_management::disable_plugin),
        )
        // ============================================================
        // SPI Federation - Custom Federation Providers
        // ============================================================
        .route(
            "/api/v1/iam/spi/federation-providers",
            get(spi_federation::list_spi_federation_configs)
                .post(spi_federation::create_spi_federation_config),
        )
        .route(
            "/api/v1/iam/spi/federation-providers/{id}",
            get(spi_federation::get_spi_federation_config)
                .put(spi_federation::update_spi_federation_config)
                .delete(spi_federation::delete_spi_federation_config),
        )
        // ============================================================
        // UMA 2.0 (User-Managed Access)
        // ============================================================
        .route(
            "/api/v1/iam/uma/resources",
            get(uma::list_uma_resources).post(uma::create_uma_resource),
        )
        .route(
            "/api/v1/iam/uma/resources/{id}",
            get(uma::get_uma_resource)
                .put(uma::update_uma_resource)
                .delete(uma::delete_uma_resource),
        )
        .route(
            "/api/v1/iam/uma/policies",
            get(uma::list_all_uma_policies).post(uma::create_uma_policy_standalone),
        )
        .route(
            "/api/v1/iam/uma/policies/{id}",
            put(uma::update_uma_policy).delete(uma::delete_uma_policy),
        )
        .route(
            "/api/v1/iam/uma/permissions",
            get(uma::list_uma_permissions).post(uma::create_uma_permission),
        )
        .route(
            "/api/v1/iam/uma/permissions/{id}",
            delete(uma::delete_uma_permission),
        )
        // ============================================================
        // Zero Trust Policies
        // ============================================================
        .route(
            "/api/v1/iam/zero-trust/policies",
            get(zero_trust::list_zero_trust_policies).post(zero_trust::create_zero_trust_policy),
        )
        .route(
            "/api/v1/iam/zero-trust/policies/{id}",
            get(zero_trust::get_zero_trust_policy)
                .put(zero_trust::update_zero_trust_policy)
                .delete(zero_trust::delete_zero_trust_policy),
        )
        .route(
            "/api/v1/iam/zero-trust/device-trust",
            get(zero_trust::get_zero_trust_dashboard).post(zero_trust::register_device_trust),
        )
        .route(
            "/api/v1/iam/zero-trust/device-trust/{id}",
            get(zero_trust::get_device_trust_status)
                .put(zero_trust::update_device_trust)
                .delete(zero_trust::revoke_device_trust),
        )
        // ============================================================
        // OID4VC (OpenID for Verifiable Credentials)
        // ============================================================
        .route(
            "/api/v1/iam/oid4vc/credentials",
            get(oid4vc::list_credentials).post(oid4vc::issue_credential),
        )
        .route(
            "/api/v1/iam/oid4vc/credentials/{id}",
            get(oid4vc::get_credential).delete(oid4vc::revoke_credential),
        )
        .route(
            "/api/v1/iam/oid4vc/credentials/{id}/verify",
            post(oid4vc::verify_credential),
        )
        // ============================================================
        // Audit Logs
        // ============================================================
        .route("/api/v1/iam/audit-logs", get(audit::list_audit_logs))
        .route(
            "/api/v1/iam/audit-logs/export",
            get(audit::export_audit_logs),
        )
        // ============================================================
        // Sessions Management (from admin.rs)
        // ============================================================
        .route("/api/v1/iam/sessions", get(admin::list_sessions))
        .route(
            "/api/v1/iam/sessions/{id}",
            delete(admin::terminate_session),
        )
        .route(
            "/api/v1/iam/sessions/terminate-all",
            post(admin::terminate_all_sessions),
        )
        // ============================================================
        // Authorization Policies (from admin.rs)
        // ============================================================
        .route(
            "/api/v1/iam/policies",
            get(admin::list_policies).post(admin::create_policy),
        )
        // ============================================================
        // Identity Providers (from admin.rs - additional endpoints)
        // ============================================================
        .route(
            "/api/v1/iam/admin/identity-providers",
            get(admin::list_identity_providers).post(admin::create_identity_provider),
        )
        .route(
            "/api/v1/iam/admin/identity-providers/{id}",
            get(admin::get_identity_provider)
                .put(admin::update_identity_provider)
                .delete(admin::delete_identity_provider),
        )
        .route(
            "/api/v1/iam/admin/identity-providers/{id}/test",
            post(admin::test_identity_provider),
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
