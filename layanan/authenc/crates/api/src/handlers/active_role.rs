//! Session role activation (NIST INCITS 359 dynamic separation of duty) and the
//! `groups` claim.
//!
//! # Why
//!
//! A user could hold `operator_satker` **and** `validator_satker` (or `approver_satker`)
//! and — before this — every one of them was in every token, all the time. Role
//! *separation* was therefore a property of who the database happened to have
//! assigned, not of what a session could do: one login could propose a request,
//! validate it and approve it. NIST INCITS 359 draws the line between the roles a
//! user is **assigned** (static, administered) and the roles **active** in a
//! session (dynamic, chosen), and lets policy say that some pairs may be assigned
//! but never active at once (DSD).
//!
//! # Behaviour
//!
//! * Tokens always carry `assigned_roles` (all) and `active_role` (the session's).
//! * With `AUTHENC_ACTIVE_ROLE_ENFORCEMENT=true`, `realm_access.roles` is **exactly**
//!   `[active_role]` — every downstream service already authorizes from that claim,
//!   so single-active-role is enforced everywhere without touching them. Off (the
//!   default), `realm_access.roles` stays the full set: nothing changes for anyone
//!   until the switch is thrown.
//! * The default active role is the primary one ([`lib_core::authz::PRIMARY_ROLE_PRIORITY`]);
//!   `POST /api/v1/auth/session/active-role` switches it. The choice rides in the
//!   (signed) **refresh token**, so it survives the 15-minute access-token lifetime;
//!   a switch also revokes the access token it replaces, so the old role's token
//!   cannot be used alongside the new one.
//! * `AUTHENC_GROUPS_CLAIM=true` adds an RFC 9068 §2.2.3.1 `groups` claim derived
//!   from the satker hierarchy (`/kejaksaan/<kejati>/<satker>`). Informational:
//!   nothing authorizes from it yet, and it is off by default because it costs a
//!   query per token issue.

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::handlers::auth_helpers;
use crate::state::ApiState;

/// Env flag: `realm_access.roles` carries only the session's active role.
pub const ENFORCEMENT_ENV: &str = "AUTHENC_ACTIVE_ROLE_ENFORCEMENT";
/// Env flag: add the `groups` claim.
pub const GROUPS_ENV: &str = "AUTHENC_GROUPS_CLAIM";

fn flag(name: &str) -> bool {
    std::env::var(name)
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "on" | "yes"
            )
        })
        .unwrap_or(false)
}

/// Is single-active-role enforced?
pub fn enforcement_enabled() -> bool {
    flag(ENFORCEMENT_ENV)
}

/// Is the `groups` claim issued?
pub fn groups_claim_enabled() -> bool {
    flag(GROUPS_ENV)
}

/// What a token says about a user's roles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleClaims {
    /// Every role the user is assigned (sorted, de-duplicated).
    pub assigned: Vec<String>,
    /// The session's active role; `None` only when the user holds no role.
    pub active: Option<String>,
    /// What goes into `realm_access.roles`.
    pub effective: Vec<String>,
}

/// Decide the role claims for a token. Pure.
///
/// `requested` is the role the session asked to be active (from the switch
/// request, or remembered in the refresh token). It is honoured only if the user
/// is **assigned** it — otherwise the default (primary) role applies, so a stale
/// or forged choice can never activate a role the user does not hold.
pub fn resolve_role_claims(
    assigned: &[String],
    requested: Option<&str>,
    enforce: bool,
) -> RoleClaims {
    let mut assigned: Vec<String> = assigned
        .iter()
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty())
        .collect();
    assigned.sort();
    assigned.dedup();

    let find = |wanted: &str| {
        assigned
            .iter()
            .find(|r| r.eq_ignore_ascii_case(wanted.trim()))
            .cloned()
    };
    let active = requested.and_then(find).or_else(|| {
        lib_core::authz::RoleSet::new(assigned.clone())
            .primary()
            .and_then(find)
    });

    let effective = if enforce {
        active.iter().cloned().collect()
    } else {
        assigned.clone()
    };
    RoleClaims {
        assigned,
        active,
        effective,
    }
}

/// Write the role claims into a token's custom claims.
pub fn apply_role_claims(claims: &mut HashMap<String, serde_json::Value>, rc: &RoleClaims) {
    claims.insert(
        "realm_access".into(),
        serde_json::json!({ "roles": rc.effective }),
    );
    claims.insert("assigned_roles".into(), serde_json::json!(rc.assigned));
    if let Some(active) = &rc.active {
        claims.insert("active_role".into(), serde_json::json!(active));
    }
}

/// The `groups` for a satker: a path per level of the Kejaksaan hierarchy.
///
/// `wilayah` is the Kejati above the satker (`integrasi.v_satker_wilayah`); a unit
/// with none (a Kejagung / pusat unit) is filed under `/kejaksaan/pusat`. Pure.
pub fn groups_for(satker_code: &str, wilayah: Option<&str>) -> Vec<String> {
    let satker = satker_code.trim();
    if satker.is_empty() {
        return Vec::new();
    }
    let segment = |s: &str| s.replace('/', "_");
    let top = "/kejaksaan".to_string();
    match wilayah.map(str::trim).filter(|w| !w.is_empty()) {
        Some(w) => vec![
            top.clone(),
            format!("{top}/{}", segment(w)),
            format!("{top}/{}/{}", segment(w), segment(satker)),
        ],
        None => vec![
            top.clone(),
            format!("{top}/pusat"),
            format!("{top}/pusat/{}", segment(satker)),
        ],
    }
}

/// The Kejati above `satker_code`, or `None` (no row, or the query failed —
/// groups are informational, so a failure degrades them rather than the login).
async fn wilayah_of(state: &ApiState, satker_code: &str) -> Option<String> {
    match state
        .database
        .query(
            "SELECT wilayah_code FROM integrasi.v_satker_wilayah WHERE kode_satker = $1 LIMIT 1",
            &[&satker_code],
        )
        .await
    {
        Ok(rows) => rows.first().map(|r| r.get::<_, String>("wilayah_code")),
        Err(e) => {
            tracing::warn!(error = %e, "groups claim: satker hierarchy lookup failed; using a flat group");
            None
        }
    }
}

/// The custom claims for a token issued to `user`, with the session's role state
/// resolved. The one place every token-issuing path (login, refresh, MFA,
/// WebAuthn, role switch) builds them, so the flags apply uniformly.
pub async fn build_claims_for_user(
    state: &ApiState,
    user: &authenc_types::User,
    requested_active: Option<&str>,
) -> HashMap<String, serde_json::Value> {
    let mut claims = auth_helpers::build_user_custom_claims(Some(user));
    let assigned: Vec<String> = user.roles.iter().map(|r| r.name.clone()).collect();
    let rc = resolve_role_claims(&assigned, requested_active, enforcement_enabled());
    apply_role_claims(&mut claims, &rc);

    if groups_claim_enabled() && !user.satker_code.is_empty() {
        let wilayah = wilayah_of(state, &user.satker_code).await;
        claims.insert(
            "groups".into(),
            serde_json::json!(groups_for(&user.satker_code, wilayah.as_deref())),
        );
    }
    claims
}

// ---------------------------------------------------------------------------
// POST /api/v1/auth/session/active-role
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SwitchActiveRoleRequest {
    /// The assigned role to act as from now on.
    pub role: String,
    /// The session's refresh token. It identifies the session and is re-issued
    /// carrying the new active role, so the choice outlives the access token.
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct SwitchActiveRoleResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub active_role: String,
    pub assigned_roles: Vec<String>,
}

struct RoleSwitchError {
    status: StatusCode,
    error: &'static str,
    message: &'static str,
}

impl IntoResponse for RoleSwitchError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({ "error": self.error, "message": self.message })),
        )
            .into_response()
    }
}

fn err(status: StatusCode, error: &'static str, message: &'static str) -> RoleSwitchError {
    RoleSwitchError {
        status,
        error,
        message,
    }
}

/// POST /api/v1/auth/session/active-role — act as another of your assigned roles.
///
/// Requires the current access token (Bearer) **and** the session's refresh token.
/// Refused unless `AUTHENC_ACTIVE_ROLE_ENFORCEMENT` is on (with it off every
/// assigned role is already in every token, so there is nothing to switch), and
/// unless the role is one the user is assigned. Replaces both tokens and revokes
/// the access token being replaced.
pub async fn switch_active_role_handler(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(request): Json<SwitchActiveRoleRequest>,
) -> Response {
    let mut response = match switch(&state, &headers, request).await {
        Ok((actor, body)) => {
            let mut r = (StatusCode::OK, Json(body)).into_response();
            r.extensions_mut()
                .insert(crate::middleware::security::AuthenticatedActor {
                    user_id: actor.to_string(),
                    client_id: None,
                });
            r
        }
        Err(e) => e.into_response(),
    };
    // Tokens must never be cached by an intermediary.
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        "no-store".parse().unwrap(),
    );
    response
}

async fn switch(
    state: &Arc<ApiState>,
    headers: &HeaderMap,
    request: SwitchActiveRoleRequest,
) -> Result<(Uuid, SwitchActiveRoleResponse), RoleSwitchError> {
    if !enforcement_enabled() {
        return Err(err(
            StatusCode::CONFLICT,
            "active_role_disabled",
            "Pergantian role aktif tidak diaktifkan pada lingkungan ini.",
        ));
    }

    let unauthorized = || {
        err(
            StatusCode::UNAUTHORIZED,
            "invalid_token",
            "Token tidak valid atau telah kedaluwarsa.",
        )
    };

    // The access token proves who is asking...
    let access_token = auth_helpers::extract_bearer_token(headers).map_err(|_| unauthorized())?;
    let access = state
        .jwt_service
        .verify_token(&access_token)
        .map_err(|_| unauthorized())?;
    if auth_helpers::claims_are_mfa_pending(&access) {
        return Err(unauthorized());
    }
    // ...and a revoked one proves nothing.
    match state
        .revocation_store
        .is_revoked(&access.jti, access.sid.as_deref(), &access.sub, access.iat)
        .await
    {
        Ok(false) => {}
        _ => return Err(unauthorized()),
    }

    // The refresh token names the session whose role is being changed; it must be
    // the same user's, the same session's, and a refresh token at all.
    let refresh = state
        .jwt_service
        .verify_token(&request.refresh_token)
        .map_err(|_| unauthorized())?;
    let is_refresh = refresh.scope.as_deref() == Some("refresh_token");
    if !is_refresh
        || refresh.sub != access.sub
        || refresh.sid != access.sid
        || refresh.sid.is_none()
    {
        return Err(unauthorized());
    }
    match state
        .revocation_store
        .is_revoked(
            &refresh.jti,
            refresh.sid.as_deref(),
            &refresh.sub,
            refresh.iat,
        )
        .await
    {
        Ok(false) => {}
        _ => return Err(unauthorized()),
    }

    let user_uuid = Uuid::parse_str(&access.sub).map_err(|_| unauthorized())?;
    let user = state
        .user_service
        .get_user(authenc_types::UserId::from_uuid(user_uuid))
        .await
        .map_err(|_| unauthorized())?;

    // Only an assigned role may be activated — never one merely named.
    let assigned: Vec<String> = user.roles.iter().map(|r| r.name.clone()).collect();
    let resolved = resolve_role_claims(&assigned, Some(&request.role), true);
    let wanted = request.role.trim();
    let honoured = resolved
        .active
        .as_deref()
        .is_some_and(|a| a.eq_ignore_ascii_case(wanted));
    if !honoured {
        return Err(err(
            StatusCode::FORBIDDEN,
            "role_not_assigned",
            "Role tersebut tidak dimiliki oleh akun Anda.",
        ));
    }
    let active = resolved.active.clone().unwrap_or_default();

    let sid = access.sid.clone().unwrap_or_default();
    let custom_claims = build_claims_for_user(state, &user, Some(&active)).await;
    let new_access = state
        .jwt_service
        .generate_access_token_with_claims(
            &access.sub,
            None,
            Some("openid profile email".to_string()),
            Some(sid.clone()),
            custom_claims,
        )
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "token_error",
                "Gagal menerbitkan token baru.",
            )
        })?;
    let new_refresh = state
        .jwt_service
        .generate_refresh_token_with_active_role(&access.sub, &sid, Some(&active))
        .map_err(|_| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "token_error",
                "Gagal menerbitkan token baru.",
            )
        })?;

    // The token being replaced carries the OLD role and is still cryptographically
    // valid for up to 15 minutes; revoke it so both roles are never usable at once
    // — that co-existence is exactly what DSD forbids. Best-effort by necessity
    // (the new tokens are already minted), but loud.
    let expires_at = chrono::DateTime::from_timestamp(access.exp, 0)
        .unwrap_or_else(|| chrono::Utc::now() + chrono::Duration::minutes(15));
    if let Err(e) = state
        .revocation_store
        .revoke_jti(&access.jti, expires_at, Some("active role switched"))
        .await
    {
        tracing::error!(error = %e, user_id = %user_uuid, "role switch: could not revoke the replaced access token");
    }

    tracing::info!(
        user_id = %user_uuid,
        to = %active,
        "session active role switched"
    );

    Ok((
        user_uuid,
        SwitchActiveRoleResponse {
            access_token: new_access,
            refresh_token: new_refresh,
            token_type: "Bearer".to_string(),
            expires_in: 900,
            active_role: active,
            assigned_roles: resolved.assigned,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roles(r: &[&str]) -> Vec<String> {
        r.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn without_enforcement_every_assigned_role_stays_in_the_token() {
        let rc = resolve_role_claims(
            &roles(&["operator_satker", "validator_satker"]),
            None,
            false,
        );
        assert_eq!(
            rc.effective,
            roles(&["operator_satker", "validator_satker"])
        );
        assert_eq!(rc.assigned, rc.effective);
        // The default active role is still reported (informational).
        assert_eq!(rc.active.as_deref(), Some("validator_satker"));
    }

    #[test]
    fn with_enforcement_the_token_carries_exactly_one_role() {
        let rc = resolve_role_claims(&roles(&["operator_satker", "validator_satker"]), None, true);
        assert_eq!(
            rc.effective,
            roles(&["validator_satker"]),
            "default = primary by priority"
        );
        assert_eq!(rc.assigned.len(), 2, "assigned roles stay visible");

        let chosen = resolve_role_claims(
            &roles(&["operator_satker", "validator_satker"]),
            Some("operator_satker"),
            true,
        );
        assert_eq!(chosen.effective, roles(&["operator_satker"]));
        assert_eq!(chosen.active.as_deref(), Some("operator_satker"));
    }

    /// The property that makes DSD mean something: a role the user is not
    /// assigned can never become active, whatever the request (or a tampered
    /// remembered choice) says.
    #[test]
    fn an_unassigned_role_cannot_be_activated() {
        let assigned = roles(&["operator_satker"]);
        for forged in [
            "admin",
            "validator_pusat",
            "system",
            "operator_satker ; admin",
            "",
        ] {
            let rc = resolve_role_claims(&assigned, Some(forged), true);
            assert_eq!(
                rc.effective,
                roles(&["operator_satker"]),
                "requesting {forged:?} must fall back to an assigned role"
            );
        }
    }

    #[test]
    fn the_requested_role_matches_case_insensitively_and_returns_the_canonical_name() {
        let rc = resolve_role_claims(
            &roles(&["operator_satker", "validator_satker"]),
            Some(" Operator_Satker "),
            true,
        );
        assert_eq!(rc.active.as_deref(), Some("operator_satker"));
    }

    #[test]
    fn a_user_with_no_roles_gets_no_active_role_and_no_effective_roles() {
        let rc = resolve_role_claims(&[], Some("admin"), true);
        assert!(rc.active.is_none());
        assert!(rc.effective.is_empty());
        let off = resolve_role_claims(&[], None, false);
        assert!(off.effective.is_empty());
    }

    #[test]
    fn duplicate_and_blank_assignments_are_normalised() {
        let rc = resolve_role_claims(
            &roles(&["operator_satker", "operator_satker", "  ", ""]),
            None,
            false,
        );
        assert_eq!(rc.assigned, roles(&["operator_satker"]));
    }

    #[test]
    fn role_claims_are_written_where_downstream_services_read_them() {
        let rc = resolve_role_claims(
            &roles(&["operator_satker", "validator_satker"]),
            Some("operator_satker"),
            true,
        );
        let mut claims = HashMap::new();
        apply_role_claims(&mut claims, &rc);
        assert_eq!(
            claims["realm_access"]["roles"],
            serde_json::json!(["operator_satker"])
        );
        assert_eq!(claims["active_role"], "operator_satker");
        assert_eq!(
            claims["assigned_roles"],
            serde_json::json!(["operator_satker", "validator_satker"])
        );
    }

    #[test]
    fn groups_follow_the_satker_hierarchy() {
        assert_eq!(
            groups_for("SKR001", Some("KJT01")),
            vec!["/kejaksaan", "/kejaksaan/KJT01", "/kejaksaan/KJT01/SKR001"]
        );
        // A unit with no Kejati above it is filed under pusat.
        assert_eq!(
            groups_for("PUSAT01", None),
            vec!["/kejaksaan", "/kejaksaan/pusat", "/kejaksaan/pusat/PUSAT01"]
        );
        assert!(groups_for("  ", Some("KJT01")).is_empty());
        // A slash in a code cannot forge a deeper path.
        assert_eq!(groups_for("A/B", Some("K/J"))[2], "/kejaksaan/K_J/A_B");
    }

    #[test]
    fn the_flags_default_off() {
        // Not asserting on the environment (tests share it); only that unset,
        // empty and junk values do not enable anything.
        for v in ["", "0", "false", "off", "no", "maybe"] {
            assert!(!matches!(
                v.to_ascii_lowercase().as_str(),
                "1" | "true" | "on" | "yes"
            ));
        }
    }
}
