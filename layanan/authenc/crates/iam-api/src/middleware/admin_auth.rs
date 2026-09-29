//! Admin authentication middleware

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use crate::state::IamApiState;
use authenc_types::domain::audit_log::AuthenticatedActor;

/// Admin user extracted from JWT token
#[derive(Debug, Clone)]
pub struct AdminUser {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub roles: Vec<String>,
}

/// The address the admin request is attributed to, resolved by the guard so the
/// handlers that write audit rows do not each re-derive it (and cannot get it
/// wrong). `None` only when the server was not built with connect info.
#[derive(Debug, Clone, Copy)]
pub struct ClientAddr(pub Option<IpAddr>);

/// Claim marking a **half-authenticated** token: issued after the password but
/// before the second factor. Same key as `authenc-api`'s `MFA_PENDING_CLAIM`.
const MFA_PENDING_CLAIM: &str = "mfa_pending";

/// Whether the token's custom claims mark it MFA-pending.
pub(crate) fn is_mfa_pending(custom: &HashMap<String, serde_json::Value>) -> bool {
    custom
        .get(MFA_PENDING_CLAIM)
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Whether any of `roles` may administer the identity provider.
///
/// Delegates to [`lib_core::authz::is_iam_admin_role`] — exact `admin`, not the
/// wider application-admin list (see `lib_core::authz::IAM_ADMIN_ROLES`).
fn holds_iam_admin_role(roles: &[String]) -> bool {
    roles.iter().any(|r| lib_core::authz::is_iam_admin_role(r))
}

/// Why a token was refused the IAM console. Separate from the HTTP mapping so
/// the decision is testable without a database.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AdminDenial {
    /// Password-only token: the second factor has not been completed.
    MfaPending,
    /// Authenticated, but not an IAM administrator.
    NotIamAdmin,
    /// The account must change its password before doing anything else.
    PasswordChangeRequired,
}

impl AdminDenial {
    fn status(self) -> StatusCode {
        match self {
            // The credential itself is not acceptable here: 401, not 403.
            AdminDenial::MfaPending => StatusCode::UNAUTHORIZED,
            AdminDenial::NotIamAdmin | AdminDenial::PasswordChangeRequired => StatusCode::FORBIDDEN,
        }
    }

    fn message(self) -> &'static str {
        match self {
            AdminDenial::MfaPending => {
                "Token verifikasi MFA tidak dapat dipakai untuk mengakses fitur admin."
            }
            AdminDenial::NotIamAdmin => "Admin role required",
            AdminDenial::PasswordChangeRequired => {
                "Anda harus mengubah password sebelum mengakses fitur admin."
            }
        }
    }
}

/// The admin decision, pure over the token's claims and roles.
///
/// Order matters: a half-authenticated token is refused **before** roles are
/// even considered. Roles come from the token only — this used to fall back to
/// a database lookup when the token carried none, which (a) let a password-only
/// `mfa_pending` token, which never carries roles, resolve to the user's real
/// roles and walk into the whole IAM API without a second factor, and (b)
/// defeated session-scoped role activation, where a token holding no active
/// role must hold no authority.
pub(crate) fn evaluate_admin(
    custom: &HashMap<String, serde_json::Value>,
    roles: &[String],
) -> Result<(), AdminDenial> {
    if is_mfa_pending(custom) {
        return Err(AdminDenial::MfaPending);
    }
    if !holds_iam_admin_role(roles) {
        return Err(AdminDenial::NotIamAdmin);
    }
    let must_change = custom
        .get("require_password_change")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if must_change {
        return Err(AdminDenial::PasswordChangeRequired);
    }
    Ok(())
}

fn client_addr(request: &Request) -> ClientAddr {
    let Some(peer) = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0)
    else {
        return ClientAddr(None);
    };
    let header = |name: &str| request.headers().get(name).and_then(|v| v.to_str().ok());
    ClientAddr(Some(lib_backend::client_ip::resolve_client_ip(
        peer.ip(),
        header("x-forwarded-for"),
        header("x-real-ip"),
        lib_backend::client_ip::TrustedProxies::global(),
    )))
}

/// Admin authentication middleware
///
/// Verifies the JWT, refuses half-authenticated (`mfa_pending`) and revoked
/// tokens, and requires an IAM administrator role.
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

    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID in token").into_response())?;

    // A signature-valid, unexpired token may still have been revoked (logout,
    // role change, deactivation). The gRPC `ValidateToken` path checks this; the
    // admin API did not, so a revoked ADMIN token kept working until expiry —
    // on the one surface where that matters most. Fail closed on a store error.
    match state
        .revocation_store
        .is_revoked(&claims.jti, claims.sid.as_deref(), &claims.sub, claims.iat)
        .await
    {
        Ok(false) => {}
        Ok(true) => {
            return Err((StatusCode::UNAUTHORIZED, "Token has been revoked").into_response());
        }
        Err(e) => {
            tracing::error!("admin auth: revocation check failed: {e}");
            return Err((StatusCode::UNAUTHORIZED, "Token validation failed").into_response());
        }
    }

    // Roles from the JWT `realm_access.roles` claim — and ONLY from there.
    let roles: Vec<String> = claims
        .custom
        .get("realm_access")
        .and_then(|ra| ra.get("roles"))
        .and_then(|r| serde_json::from_value::<Vec<String>>(r.clone()).ok())
        .unwrap_or_default();

    // The authenticated subject is known from here on; hand it to the audit
    // writer on every exit, including refusals — an AUTHZ_FAILURE row that names
    // nobody is close to useless for the incident it exists to record.
    let actor = AuthenticatedActor {
        user_id: user_id.to_string(),
        client_id: None,
    };

    if let Err(denial) = evaluate_admin(&claims.custom, &roles) {
        let mut response = (denial.status(), denial.message()).into_response();
        response.extensions_mut().insert(actor);
        return Err(response);
    }

    // Extract username from JWT preferred_username claim
    let username = claims
        .custom
        .get("preferred_username")
        .and_then(|v| v.as_str())
        .unwrap_or(&claims.sub)
        .to_string();

    let addr = client_addr(&request);
    request.extensions_mut().insert(AdminUser {
        user_id,
        username,
        roles,
    });
    request.extensions_mut().insert(addr);

    let mut response = next.run(request).await;
    response.extensions_mut().insert(actor);
    Ok(response)
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
///   `require_permission("view_audit")` looked like a check that never ran. An
///   unmounted guard that looks mounted is how a missing check stays missing.
/// * **Fail open.** An unrecognised permission is a denial, not a grant.
///   Answering 500 for one surfaces the typo in the route definition instead
///   of blaming the caller (403) or, worse, letting the request through.
///
/// **This is not currently mounted on any route** — the IAM router gates
/// routes through [`admin_auth_middleware`] instead. It is kept because the
/// capability table, not this function, is the thing routes should call; when
/// a route needs finer granularity than "IAM admin", mount this and it will
/// honour the permission it is given.
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
    use serde_json::json;

    fn roles(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    fn custom(pairs: &[(&str, serde_json::Value)]) -> HashMap<String, serde_json::Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    /// The exact shape `generate_mfa_temp_token` mints: scope `mfa`, claim
    /// `mfa_pending=true`, and — the detail that made this exploitable — NO
    /// `realm_access`, so the guard used to fall through to a DB role lookup and
    /// resolve the user's real roles. A password alone must never open the IAM
    /// console, whatever roles the account holds.
    #[test]
    fn password_only_mfa_pending_token_is_refused_even_for_an_admin_account() {
        let pending = custom(&[("mfa_pending", json!(true))]);
        assert_eq!(
            evaluate_admin(&pending, &roles(&["admin"])),
            Err(AdminDenial::MfaPending)
        );
        // ...and with no roles at all (the real temp-token shape).
        assert_eq!(evaluate_admin(&pending, &[]), Err(AdminDenial::MfaPending));
        assert_eq!(AdminDenial::MfaPending.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn mfa_pending_false_or_absent_is_a_fully_authenticated_token() {
        assert!(evaluate_admin(&custom(&[]), &roles(&["admin"])).is_ok());
        assert!(
            evaluate_admin(
                &custom(&[("mfa_pending", json!(false))]),
                &roles(&["admin"])
            )
            .is_ok()
        );
        // Only a boolean `true` marks it; a string must not, or a stray claim
        // shape could either lock everyone out or slip through unnoticed.
        assert!(!is_mfa_pending(&custom(&[("mfa_pending", json!("true"))])));
    }

    /// The negative case is the one that matters: a non-admin must be refused.
    #[test]
    fn non_admin_roles_are_refused() {
        for r in [
            &["operator_satker"][..],
            &["validator_wilayah"],
            &["validator_pusat"],
            &["approver_satker"],
            &[],
        ] {
            assert_eq!(
                evaluate_admin(&custom(&[]), &roles(r)),
                Err(AdminDenial::NotIamAdmin),
                "{r:?} must be refused"
            );
        }
        assert_eq!(AdminDenial::NotIamAdmin.status(), StatusCode::FORBIDDEN);
    }

    /// PR #930 widened this guard to the application-admin list, which made
    /// `admin_pusat`/`superadmin` — roles the seed does not even define — able to
    /// create users, reset passwords, switch off MFA and grant `admin`. The IdP is
    /// the root of trust; its console is exact `admin`.
    #[test]
    fn application_admins_are_not_identity_provider_admins() {
        for r in ["superadmin", "admin_pusat", "Admin_Pusat"] {
            assert_eq!(
                evaluate_admin(&custom(&[]), &roles(&[r])),
                Err(AdminDenial::NotIamAdmin),
                "'{r}' must not administer the IdP"
            );
        }
        assert!(evaluate_admin(&custom(&[]), &roles(&["ADMIN"])).is_ok());
    }

    /// Prefix look-alikes must not pass either — roles are database rows.
    #[test]
    fn prefix_lookalikes_are_not_admin() {
        for r in [
            "admin_readonly",
            "administrative",
            "super_admin",
            "admin_audit_log",
        ] {
            assert_eq!(
                evaluate_admin(&custom(&[]), &roles(&[r])),
                Err(AdminDenial::NotIamAdmin)
            );
        }
    }

    /// Holding an admin role alongside others still passes, and the guard does
    /// not stop at the first non-admin entry.
    #[test]
    fn admin_among_other_roles_passes() {
        assert!(evaluate_admin(&custom(&[]), &roles(&["operator_satker", "admin"])).is_ok());
    }

    #[test]
    fn password_change_blocks_admin_access_after_the_role_check() {
        let must_change = custom(&[("require_password_change", json!(true))]);
        assert_eq!(
            evaluate_admin(&must_change, &roles(&["admin"])),
            Err(AdminDenial::PasswordChangeRequired)
        );
        // A non-admin who must change password is told they are not an admin,
        // not that they must change a password to become one.
        assert_eq!(
            evaluate_admin(&must_change, &roles(&["operator_satker"])),
            Err(AdminDenial::NotIamAdmin)
        );
    }
}
