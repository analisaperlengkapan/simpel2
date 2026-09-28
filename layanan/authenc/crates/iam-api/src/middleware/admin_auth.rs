//! Admin authentication middleware

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::state::IamApiState;

/// Admin user extracted from JWT token
#[derive(Debug, Clone)]
pub struct AdminUser {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub roles: Vec<String>,
}

/// Whether any of `roles` is an administrative role.
///
/// The single predicate both guards in this module call, delegating to
/// [`lib_core::authz::is_admin_role`] so this service cannot disagree with the
/// perlengkapan backend or either microfrontend about who is an admin.
fn holds_admin_role(roles: &[String]) -> bool {
    roles.iter().any(|r| lib_core::authz::is_admin_role(r))
}

/// Admin authentication middleware
///
/// Verifies JWT token and checks for admin role
pub async fn admin_auth_middleware(
    State(state): State<Arc<IamApiState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            (StatusCode::UNAUTHORIZED, "Missing Authorization header").into_response()
        })?;

    // Extract Bearer token
    let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            "Invalid Authorization header format",
        )
            .into_response()
    })?;

    // Validate JWT token
    let claims = state
        .jwt_service
        .verify_token(token)
        .map_err(|e| (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", e)).into_response())?;

    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid user ID in token",
        )
            .into_response()
    })?;

    // Extract roles from JWT realm_access.roles claim
    let mut roles: Vec<String> = claims
        .custom
        .get("realm_access")
        .and_then(|ra| ra.get("roles"))
        .and_then(|r| serde_json::from_value::<Vec<String>>(r.clone()).ok())
        .unwrap_or_default();

    // If JWT has no roles, look up from DB via user_service
    if roles.is_empty()
        && let Ok(user) = state
            .user_service
            .get_user(authenc_types::UserId::from_uuid(user_id))
            .await
    {
        roles = user.roles.iter().map(|r| r.name.clone()).collect();
    }

    // Check for admin role.
    //
    // The allowlist is `lib_core::authz::ADMIN_ROLES` — the same constant the
    // perlengkapan backend and both microfrontends gate their admin surfaces
    // on. This middleware previously accepted only the exact string "admin",
    // so `admin_pusat` and `superadmin` were administrators everywhere except
    // here: the profile badge said "Admin Pusat", the FE rendered the admin
    // console, and this guard answered 403 on every request behind it.
    if !holds_admin_role(&roles) {
        return Err((StatusCode::FORBIDDEN, "Admin role required").into_response());
    }

    // Block admin access when user must change password first
    let must_change = claims
        .custom
        .get("require_password_change")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if must_change {
        return Err((
            StatusCode::FORBIDDEN,
            "Anda harus mengubah password sebelum mengakses fitur admin.",
        )
            .into_response());
    }

    // Extract username from JWT preferred_username claim
    let username = claims
        .custom
        .get("preferred_username")
        .and_then(|v| v.as_str())
        .unwrap_or(&claims.sub)
        .to_string();

    // Create AdminUser and insert into request extensions
    let admin_user = AdminUser {
        user_id,
        username,
        roles,
    };

    request.extensions_mut().insert(admin_user);

    Ok(next.run(request).await)
}

/// Permission-based authorization middleware
///
/// Resolves the caller's roles into a [`lib_core::authz::Authorization`] and
/// asks whether it holds `permission`, parsed as a
/// [`lib_core::authz::Capability`]. The permission vocabulary and the
/// role→capability table therefore live in `lib-core`, shared with the
/// perlengkapan backend and both microfrontends.
///
/// Two things this must not do, both of which it used to:
///
/// * **Collapse to "is admin".** Before this it ignored `permission` entirely
///   and accepted any admin role, so a route mounted with
///   `require_permission("view_audit")` would have admitted `superadmin` — who
///   holds `ViewAudit` — but the name implied a check that never ran. An
///   unmounted guard that looks mounted is how a missing check stays missing.
/// * **Fail open.** An unrecognised permission is a denial, not a grant.
///   Answering 500 for one surfaces the typo in the route definition instead
///   of blaming the caller (403) or, worse, letting the request through.
///
/// **This is not currently mounted on any route** — the IAM router gates
/// routes through [`admin_auth_middleware`] instead. It is kept because the
/// capability table, not this function, is the thing routes should call; when
/// a route needs finer granularity than "admin", mount this and it will honour
/// the permission it is given.
pub async fn require_permission(
    permission: &'static str,
) -> impl Fn(
    Request,
    Next,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, Response>> + Send>>
+ Clone {
    let capability = lib_core::authz::Capability::parse(permission);
    move |request: Request, next: Next| {
        Box::pin(async move {
            // Extract AdminUser from extensions
            let admin_user = request.extensions().get::<AdminUser>().ok_or_else(|| {
                (StatusCode::UNAUTHORIZED, "Admin authentication required").into_response()
            })?;

            let Some(capability) = capability else {
                // An unknown permission string cannot be satisfied. Answering
                // 500 (rather than 403) surfaces the coding error in the route
                // definition instead of blaming the caller for it.
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unknown permission '{permission}'"),
                )
                    .into_response());
            };

            let authz = lib_core::authz::Authorization::from_slice(&admin_user.roles);
            if !authz.can(capability) {
                return Err((
                    StatusCode::FORBIDDEN,
                    format!("Permission '{}' required", capability.key()),
                )
                    .into_response());
            }

            Ok(next.run(request).await)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roles(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    /// The negative case is the one that matters: a non-admin must be refused.
    /// The prior version of this module had no test here at all — the only
    /// admin test in the crate was a `TODO` stub — so nothing pinned that a
    /// satker-bound role could not pass the guard.
    #[test]
    fn non_admin_roles_are_refused() {
        assert!(!holds_admin_role(&roles(&["operator_satker"])));
        assert!(!holds_admin_role(&roles(&["validator_wilayah"])));
        assert!(!holds_admin_role(&roles(&["validator_pusat"])));
        assert!(!holds_admin_role(&roles(&[])));
    }

    /// A prefix look-alike must not pass. `admin_readonly` is the shape an
    /// operator can mint, because roles are database rows.
    #[test]
    fn prefix_lookalikes_are_not_admin() {
        assert!(!holds_admin_role(&roles(&["admin_readonly"])));
        assert!(!holds_admin_role(&roles(&["administrative"])));
        assert!(!holds_admin_role(&roles(&["super_admin"])));
    }

    /// Every role the shared allowlist blesses must pass this guard, in any
    /// casing. `admin_pusat` and `superadmin` failing here was the defect: the
    /// frontends rendered their admin console and this guard returned 403.
    #[test]
    fn every_shared_admin_role_passes() {
        for role in lib_core::authz::ADMIN_ROLES {
            assert!(
                holds_admin_role(&roles(&[role])),
                "{role} must pass the authenc admin guard"
            );
        }
        assert!(holds_admin_role(&roles(&["Admin_Pusat"])));
    }

    /// Holding an admin role alongside others still passes, and the guard does
    /// not stop at the first non-admin entry.
    #[test]
    fn admin_among_other_roles_passes() {
        assert!(holds_admin_role(&roles(&["operator_satker", "admin"])));
        assert!(holds_admin_role(&roles(&[
            "validator_wilayah",
            "superadmin"
        ])));
    }
}
