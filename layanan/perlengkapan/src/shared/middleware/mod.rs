//! # Authentication & authorization primitives
//!
//! Two pieces, deliberately separate:
//!
//! * [`require_authentication`] — a **deny-by-default** middleware mounted on the
//!   whole `/api/v1/perlengkapan` router. A request without a valid bearer token
//!   never reaches a handler, so forgetting the [`Claims`] extractor on a new
//!   handler no longer publishes it to the internet (OWASP API2:2023 Broken
//!   Authentication / API5 BFLA — the previous design authenticated *per
//!   handler*, so "public" was whatever nobody remembered to protect).
//! * [`Claims`] — the authenticated caller. It carries **every** realm role the
//!   token holds plus one deterministic primary role, and its guards match
//!   exactly; there is no implicit "admin passes everything" escape hatch.

pub mod cancel_safe;
pub mod metrics;
pub mod size_limit;

use axum::{
    extract::{ConnectInfo, FromRef, FromRequestParts, Request, State},
    http::{HeaderMap, header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

use crate::shared::error::AppError;
use crate::shared::grpc::clients::{AuthencClient, authenc::v1::ValidateTokenResponse};

/// Axum extractor for the originating client IP, used by audit log /
/// workflow transition records.
///
/// The address is resolved by [`lib_backend::client_ip`]: the forwarding
/// headers (`X-Forwarded-For`, `X-Real-IP`) are believed **only** when the TCP
/// peer is a proxy we operate (`TRUSTED_PROXY_CIDRS`), and `X-Forwarded-For` is
/// read from the right. The previous version returned the *leftmost* header
/// entry verbatim, which is whatever the caller typed — an audit row could name
/// any address, and any string, at all.
///
/// Falls back to `"unknown"` only when there is no peer information at all
/// (unit tests that drive the router without a socket).
pub struct ClientIp(pub String);

impl ClientIp {
    /// Resolve from an explicit peer + headers. Pure, so it is unit-testable.
    pub fn resolve(peer: Option<SocketAddr>, headers: &HeaderMap) -> Self {
        let Some(peer) = peer else {
            return ClientIp("unknown".to_string());
        };
        let header = |name: &str| headers.get(name).and_then(|v| v.to_str().ok());
        ClientIp(lib_backend::client_ip::client_ip_string(
            peer.ip(),
            header("x-forwarded-for"),
            header("x-real-ip"),
        ))
    }
}

impl<S> FromRequestParts<S> for ClientIp
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let peer = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| *addr);
        Ok(ClientIp::resolve(peer, &parts.headers))
    }
}

/// Role names a token is never allowed to assert.
///
/// `system` is the workflow engine's internal actor (scheduled expiry, audited
/// break-glass). Roles are database rows in authenc, so nothing but this list
/// stops someone from minting a role *named* `system` and walking through the
/// engine's internal-actor bypass.
const RESERVED_ROLES: &[&str] = &[crate::workflow::engine::INTERNAL_ACTOR_ROLE];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: Uuid,
    pub username: String,
    /// The caller's **primary** role: the first entry of
    /// [`lib_core::authz::PRIMARY_ROLE_PRIORITY`] the caller holds — never
    /// "whichever role the token listed first". Used as a label (audit, the
    /// workflow engine's actor role) and by code that has not been migrated to
    /// [`Claims::roles`]; authorization decisions should use the `require_*` /
    /// `holds_*` methods, which look at every role.
    pub role: String,
    /// Every realm role the token carries: lower-cased, de-duplicated, sorted.
    /// With `AUTHENC_ACTIVE_ROLE_ENFORCEMENT` on this is exactly the session's
    /// one active role (NIST INCITS 359 dynamic separation of duty).
    #[serde(default)]
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    // Extended fields for Pakaian Dinas & Kebutuhan BMN modules
    pub nip: Option<String>,
    pub name: Option<String>,
    pub nama: Option<String>,
    pub jabatan: Option<String>,
    /// Caller's MySIMKARI `kode_satker` (Kejaksaan-internal org code, dotted —
    /// e.g. "02.28"), sourced from authenc's `satker_code` claim / the `satker:`
    /// scope. NOTE: this is NOT the SIMAN finance code `kdsatker_keu` (the two
    /// systems are disjoint — see integrasi `003_satker_code_mapping.sql`). Used
    /// for satker-level authorization — handlers must reject writes to other
    /// satker's data unless the caller holds a cross-satker role. For SIMAN-asset
    /// (bank_aset) data scoping it is mapped to `kdsatker_keu` via
    /// `integrasi.v_satker_code_map` (see [`crate::bank_aset::AsetScope`]).
    pub satker_code: Option<String>,
}

impl Claims {
    /// Build a caller with the given roles. The primary role is derived, so a
    /// fixture cannot disagree with the production constructor about it.
    pub fn with_roles<I, S>(user_id: Uuid, username: impl Into<String>, roles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let set = lib_core::authz::RoleSet::new(roles);
        let roles: Vec<String> = set
            .iter()
            .filter(|r| !RESERVED_ROLES.contains(r))
            .map(str::to_string)
            .collect();
        let role = lib_core::authz::RoleSet::new(roles.clone())
            .primary()
            .unwrap_or("user")
            .to_string();
        Self {
            user_id,
            username: username.into(),
            role,
            roles,
            permissions: Vec::new(),
            nip: None,
            name: None,
            nama: None,
            jabatan: None,
            satker_code: None,
        }
    }

    /// Set the caller's MySIMKARI satker (builder style, for fixtures).
    pub fn in_satker(mut self, satker_code: impl Into<String>) -> Self {
        self.satker_code = Some(satker_code.into());
        self
    }

    /// The caller's roles as a [`lib_core::authz::RoleSet`].
    ///
    /// Falls back to the single primary `role` when `roles` is empty, so a
    /// `Claims` deserialized from before the field existed still answers.
    pub fn role_set(&self) -> lib_core::authz::RoleSet {
        if self.roles.is_empty() {
            lib_core::authz::RoleSet::new([self.role.clone()])
        } else {
            lib_core::authz::RoleSet::new(self.roles.clone())
        }
    }

    /// Does the caller hold `role` (exact, case-insensitive)?
    pub fn holds_role(&self, role: &str) -> bool {
        self.role_set().has(role)
    }

    /// Does the caller hold any of `roles`?
    pub fn holds_any_role(&self, roles: &[&str]) -> bool {
        self.role_set().has_any(roles)
    }

    /// Does the caller hold an application-administrator role?
    pub fn is_admin(&self) -> bool {
        self.role_set().is_admin()
    }

    /// Roles allowed to read data across satker boundaries. Anything else is
    /// treated as satker-scoped.
    ///
    /// Delegates to [`lib_core::authz::CROSS_SATKER_ROLES`] rather than
    /// restating the list. This predicate decides whether the satker boundary
    /// is enforced at all ([`can_access_satker`]), so a second copy here is a
    /// boundary that widens or narrows the moment the shared list moves —
    /// silently, because both sides still compile.
    ///
    /// Reading across satkers is not writing across them: the write paths gate
    /// on the workflow policy, which no longer has an admin bypass.
    pub fn is_cross_satker_role(&self) -> bool {
        self.role_set().is_cross_satker()
    }

    /// Returns true when the caller may access data belonging to the given
    /// satker code. Cross-satker roles always pass; everyone else must match
    /// their own `satker_code`.
    pub fn can_access_satker(&self, target: &str) -> bool {
        if self.is_cross_satker_role() {
            return true;
        }
        match &self.satker_code {
            Some(code) => code == target,
            None => false,
        }
    }

    /// Assert that the caller may act on data scoped to `target_satker`.
    /// Returns `AppError::Authorization` when the satker boundary is violated.
    pub fn require_satker(&self, target_satker: &str) -> Result<(), AppError> {
        if self.can_access_satker(target_satker) {
            Ok(())
        } else {
            Err(AppError::Authorization(format!(
                "Akses ditolak: data milik satker lain ({})",
                target_satker
            )))
        }
    }

    /// Assert bahwa caller memegang salah satu dari `allowed`.
    /// Mengembalikan `AppError::Authorization` (mapped ke 403) jika tidak.
    ///
    /// Match **persis** (case-insensitive) terhadap SEMUA role caller — bukan
    /// hanya role pertama. Tidak ada jalan pintas admin: administrator sistem
    /// tidak otomatis boleh menyetujui/menolak/mencabut apa yang oleh proses
    /// bisnis dialamatkan ke pejabat (segregation of duties; OWASP
    /// Authorization: least privilege). Endpoint yang memang boleh dilayani
    /// admin harus menyebutkannya — lihat [`Claims::require_any_role_or_admin`].
    ///
    /// # Contoh
    /// ```ignore
    /// // Endpoint yg hanya boleh dipicu Validator Pusat:
    /// claims.require_any_role(&["validator_pusat"])?;
    /// ```
    pub fn require_any_role(&self, allowed: &[&str]) -> Result<(), AppError> {
        if self.holds_any_role(allowed) {
            Ok(())
        } else {
            Err(AppError::Authorization(format!(
                "Akses ditolak: role '{}' tidak diizinkan utk aksi ini (perlu salah satu dari: {})",
                self.role,
                allowed.join(", ")
            )))
        }
    }

    /// Seperti [`Claims::require_any_role`], tetapi role administrator
    /// aplikasi ([`lib_core::authz::ADMIN_ROLES`]) juga diterima.
    ///
    /// Untuk permukaan administratif/operasional (data master, template,
    /// pemantauan) — BUKAN untuk keputusan bisnis.
    pub fn require_any_role_or_admin(&self, allowed: &[&str]) -> Result<(), AppError> {
        if self.is_admin() {
            return Ok(());
        }
        self.require_any_role(allowed)
    }

    /// Alias untuk satu role tunggal.
    pub fn require_role(&self, role: &str) -> Result<(), AppError> {
        self.require_any_role(&[role])
    }

    /// Versi predikat dari [`require_any_role`]: menjawab "boleh?" tanpa
    /// menghasilkan error. Dipakai saat MENYUSUN respons (mis. memfilter
    /// daftar aksi yg ditawarkan ke FE), bukan saat menegakkan akses —
    /// penegakan tetap `require_*` di handler aksinya.
    ///
    /// Sengaja mendelegasikan ke `require_any_role` agar aturannya hanya hidup
    /// di SATU tempat; menyalin aturannya ke sini akan membuat keduanya bisa
    /// hanyut.
    ///
    /// [`require_any_role`]: Self::require_any_role
    pub fn has_any_role(&self, allowed: &[&str]) -> bool {
        self.require_any_role(allowed).is_ok()
    }

    /// May the caller exercise `capability` (see [`lib_core::authz::Capability`])?
    ///
    /// The same table the two frontends gate their UI on, so a button and the
    /// endpoint behind it cannot disagree about who is allowed.
    pub fn can(&self, capability: lib_core::authz::Capability) -> bool {
        lib_core::authz::Authorization::new(self.role_set()).can(capability)
    }

    /// Assert the caller holds `capability`; `403` otherwise.
    pub fn require_capability(
        &self,
        capability: lib_core::authz::Capability,
    ) -> Result<(), AppError> {
        if self.can(capability) {
            Ok(())
        } else {
            Err(AppError::Authorization(format!(
                "Akses ditolak: role '{}' tidak memiliki kewenangan '{}'",
                self.role,
                capability.key()
            )))
        }
    }

    /// Assert caller adalah administrator aplikasi. Dipakai utk endpoint
    /// master/referensi yg hanya boleh diubah admin.
    pub fn require_admin(&self) -> Result<(), AppError> {
        if self.is_admin() {
            Ok(())
        } else {
            Err(AppError::Authorization(format!(
                "Akses ditolak: aksi ini memerlukan role admin (role caller: '{}')",
                self.role
            )))
        }
    }

    /// The role the caller should be recorded as when acting under a rule that
    /// allows `allowed`: the first of `allowed` (in that order) that the caller
    /// holds, otherwise the primary role.
    ///
    /// A user holding several roles must act as the one that *authorizes* the
    /// action — not as whichever happens to rank highest. Otherwise the policy
    /// check passes on `operator_satker` while the workflow engine, handed the
    /// primary `validator_satker`, refuses the same move.
    pub fn acting_role(&self, allowed: &[&str]) -> String {
        let set = self.role_set();
        allowed
            .iter()
            .find(|r| set.has(r))
            .map(|r| r.to_ascii_lowercase())
            .unwrap_or_else(|| self.role.clone())
    }

    /// The role to act as for a move INTO `to_state` under `config`: the role
    /// the workflow names for that state if the caller holds it, else the
    /// primary role.
    ///
    /// For the coarse transition endpoints, which know the target state but not
    /// (statically) which of the caller's roles authorizes it.
    pub fn role_for_transition(
        &self,
        config: &crate::workflow::config::WorkflowConfig,
        to_state: &str,
    ) -> String {
        match config.get_required_role(to_state) {
            Some(required) if self.holds_role(required) => required.to_ascii_lowercase(),
            _ => self.role.clone(),
        }
    }

    /// Build the caller from authenc's `ValidateToken` verdict.
    ///
    /// Pure (no I/O) so the role/satker mapping can be unit-tested. Returns an
    /// authentication error for a verdict that is not an authenticated user.
    pub fn from_validation(resp: &ValidateTokenResponse) -> Result<Self, AppError> {
        if !resp.valid {
            return Err(AppError::Authentication(
                resp.error
                    .clone()
                    .unwrap_or_else(|| "Invalid token".to_string()),
            ));
        }

        let user_id = resp
            .user_id
            .as_deref()
            .ok_or_else(|| AppError::Authentication("Token missing user_id".to_string()))?
            .parse::<Uuid>()
            .map_err(|_| AppError::Authentication("Invalid user_id format".to_string()))?;

        // Prefer first-class `realm_roles` from the ValidateTokenResponse; fall
        // back to the legacy `role:` scope convention when the issuer has not
        // been updated yet. ALL of them are kept — the previous code kept only
        // `realm_roles.first()`, so a user holding [validator_wilayah,
        // operator_satker] was authorized (or refused) by whichever role the
        // issuer happened to list first.
        let mut raw_roles: Vec<String> = resp.realm_roles.clone();
        if raw_roles.is_empty() {
            raw_roles.extend(
                resp.scopes
                    .iter()
                    .filter_map(|s| s.strip_prefix("role:"))
                    .map(str::to_string),
            );
        }
        let mut claims = Claims::with_roles(user_id, String::new(), raw_roles);
        if claims.roles.is_empty() {
            // Authenticated but holding no (permitted) role: keep the historical
            // `user` label. It matches no allowlist, so every role guard fails
            // closed while read paths fall back to the caller's own satker.
            claims.role = "user".to_string();
        }

        // First-class fields land as of commit 20; scope-prefix fallbacks keep
        // us compatible with tokens minted before the authenc upgrade.
        let scope_lookup = |prefix: &str| -> Option<String> {
            resp.scopes
                .iter()
                .find(|s| s.starts_with(prefix))
                .map(|s| s.trim_start_matches(prefix).to_string())
        };
        claims.username = resp
            .username
            .clone()
            .or_else(|| scope_lookup("username:"))
            .unwrap_or_else(|| user_id.to_string());
        claims.permissions = resp.scopes.clone();
        claims.nip = resp.nip.clone().or_else(|| scope_lookup("nip:"));
        claims.name = resp.name.clone().or_else(|| scope_lookup("name:"));
        claims.nama = resp.name.clone().or_else(|| scope_lookup("nama:"));
        claims.jabatan = resp.jabatan.clone().or_else(|| scope_lookup("jabatan:"));
        claims.satker_code = resp.satker_code.clone().or_else(|| scope_lookup("satker:"));
        Ok(claims)
    }

    /// Validate the `Authorization: Bearer` header against authenc and build
    /// the caller. Shared by [`require_authentication`] and the extractor.
    pub async fn authenticate(
        authenc: &AuthencClient,
        headers: &HeaderMap,
    ) -> Result<Self, AppError> {
        let auth_header = headers
            .get(AUTHORIZATION)
            .ok_or_else(|| AppError::Authentication("Missing Authorization header".to_string()))?;

        let auth_str = auth_header
            .to_str()
            .map_err(|_| AppError::Authentication("Invalid Authorization header".to_string()))?;

        let token = auth_str.strip_prefix("Bearer ").ok_or_else(|| {
            AppError::Authentication("Authorization header must start with 'Bearer '".to_string())
        })?;

        // Validate token via Authenc gRPC (signature, expiry, revocation and the
        // half-authenticated `mfa_pending` refusal all live on that side).
        match authenc.validate_token(token).await {
            Ok(resp) => Claims::from_validation(&resp),
            Err(e) => {
                tracing::error!("Authenc validation failed: {}", e);
                Err(AppError::Internal(
                    "Authentication service unavailable".to_string(),
                ))
            }
        }
    }
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
    AuthencClient: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // The normal path: `require_authentication` already validated the token
        // once for this request. Re-validating here would be a second gRPC round
        // trip per request for the same answer.
        if let Some(claims) = parts.extensions.get::<Claims>() {
            return Ok(claims.clone());
        }
        // Not mounted behind the middleware (tests, an internal sub-router):
        // authenticate directly, as this extractor always did.
        let authenc = AuthencClient::from_ref(state);
        Claims::authenticate(&authenc, &parts.headers).await
    }
}

/// Routes (relative to the `/api/v1/perlengkapan` nest) that authenticate
/// **themselves** because the browser cannot attach an `Authorization` header to
/// them. Everything else is refused here without a valid bearer token.
///
/// Keep this list to what genuinely cannot carry the header. A WebSocket
/// handshake is the one such case (see `dashboard::websocket`, which validates
/// the query-string token itself).
const SELF_AUTHENTICATING_ROUTES: &[&str] = &["/dashboard/ws"];

const API_PREFIX: &str = "/api/v1/perlengkapan";

/// Is `path` one of [`SELF_AUTHENTICATING_ROUTES`]? Accepts the path with or
/// without the nest prefix, because a layer on the inner router sees the
/// stripped path while one on the outer router sees the full one.
fn is_self_authenticating(path: &str) -> bool {
    let relative = path.strip_prefix(API_PREFIX).unwrap_or(path);
    SELF_AUTHENTICATING_ROUTES.contains(&relative)
}

/// Deny-by-default authentication for the API router.
///
/// On success the [`Claims`] are placed in the request extensions, where the
/// [`Claims`] extractor and the per-user rate limiter read them. On failure the
/// request is answered here — a handler is never reached.
///
/// Answers `401` for **every** unauthenticated request, including ones for
/// paths that do not exist: telling an anonymous caller "404 vs 401" would let
/// them enumerate the API surface.
pub async fn require_authentication(
    State(authenc): State<AuthencClient>,
    mut request: Request,
    next: Next,
) -> Response {
    if is_self_authenticating(request.uri().path()) {
        return next.run(request).await;
    }
    match Claims::authenticate(&authenc, request.headers()).await {
        Ok(claims) => {
            request.extensions_mut().insert(claims);
            next.run(request).await
        }
        Err(e) => e.into_response(),
    }
}

#[cfg(test)]
mod policy_tests {
    use super::*;

    fn claims_with_role(role: &str) -> Claims {
        Claims::with_roles(Uuid::nil(), "u", [role])
    }

    fn response(roles: &[&str], satker: Option<&str>) -> ValidateTokenResponse {
        ValidateTokenResponse {
            valid: true,
            user_id: Some(Uuid::nil().to_string()),
            scopes: vec![],
            expires_at: None,
            error: None,
            username: Some("budi".into()),
            name: None,
            nip: None,
            jabatan: None,
            satker_code: satker.map(str::to_string),
            realm_roles: roles.iter().map(|r| r.to_string()).collect(),
        }
    }

    #[test]
    fn require_role_accepts_exact_match() {
        let c = claims_with_role("validator_pusat");
        assert!(c.require_role("validator_pusat").is_ok());
        assert!(c.require_any_role(&["validator_pusat", "admin"]).is_ok());
    }

    #[test]
    fn require_role_rejects_wrong_role() {
        let c = claims_with_role("operator_satker");
        let err = c.require_role("validator_pusat").unwrap_err();
        assert!(matches!(err, AppError::Authorization(_)));
    }

    #[test]
    fn require_role_is_case_insensitive() {
        let c = claims_with_role("Validator_Pusat");
        assert!(c.require_role("validator_pusat").is_ok());
    }

    /// The pre-#930 behaviour this replaces: `admin`/`superadmin` passed EVERY
    /// role guard, so the person who administers the system could also approve
    /// and reject the very requests it routes to business authorities.
    #[test]
    fn admin_no_longer_bypasses_business_role_guards() {
        for role in lib_core::authz::ADMIN_ROLES {
            let c = claims_with_role(role);
            assert!(
                c.require_role("validator_pusat").is_err(),
                "{role} must not pass a validator_pusat guard"
            );
            assert!(c.require_any_role(&["operator_satker"]).is_err());
            // ...but is still an administrator where admin is what is asked for.
            assert!(c.require_admin().is_ok());
            assert!(c.require_any_role_or_admin(&["validator_pusat"]).is_ok());
        }
    }

    #[test]
    fn require_admin_rejects_non_admin() {
        let c = claims_with_role("validator_pusat");
        assert!(c.require_admin().is_err());
        assert!(c.require_any_role_or_admin(&["validator_wilayah"]).is_err());
    }

    /// A user holding several roles is authorized by ANY of them. Before this,
    /// only `realm_roles.first()` counted, so the outcome depended on the order
    /// the issuer listed the roles in.
    #[test]
    fn every_held_role_counts_not_just_the_first() {
        let c = Claims::with_roles(Uuid::nil(), "u", ["operator_satker", "validator_wilayah"]);
        assert!(c.require_role("operator_satker").is_ok());
        assert!(c.require_role("validator_wilayah").is_ok());
        assert!(c.require_role("validator_pusat").is_err());

        let reversed =
            Claims::with_roles(Uuid::nil(), "u", ["validator_wilayah", "operator_satker"]);
        assert_eq!(
            c.role, reversed.role,
            "primary role must not depend on token order"
        );
        assert_eq!(c.roles, reversed.roles);
    }

    #[test]
    fn transition_role_is_the_one_the_workflow_names_for_the_target_state() {
        use crate::workflow::config::WorkflowConfig;
        let config = WorkflowConfig::default_kebutuhan_bmn();
        let c = Claims::with_roles(Uuid::nil(), "u", ["operator_satker", "validator_wilayah"]);
        // SUBMIT_WILAYAH is the operator's move even though the primary is wilayah.
        assert_eq!(
            c.role_for_transition(&config, "SUBMIT_WILAYAH"),
            "operator_satker"
        );
        assert_eq!(
            c.role_for_transition(&config, "SUBMIT_PUSAT"),
            "validator_wilayah"
        );
        // A state whose role the caller does not hold falls back to the primary,
        // and the engine then refuses it.
        assert_eq!(
            c.role_for_transition(&config, "APPROVED"),
            "validator_wilayah"
        );
    }

    #[test]
    fn primary_role_is_the_widest_by_priority() {
        let c = Claims::with_roles(Uuid::nil(), "u", ["operator_satker", "validator_pusat"]);
        assert_eq!(c.role, "validator_pusat");
    }

    #[test]
    fn acting_role_is_the_one_that_authorizes_the_action() {
        let c = Claims::with_roles(Uuid::nil(), "u", ["operator_satker", "validator_wilayah"]);
        assert_eq!(c.role, "validator_wilayah");
        // Submitting as operator must be recorded/checked as operator...
        assert_eq!(c.acting_role(&["operator_satker"]), "operator_satker");
        // ...and a rule the caller does not satisfy falls back to the primary.
        assert_eq!(c.acting_role(&["validator_pusat"]), "validator_wilayah");
    }

    /// The satker boundary is enforced by `is_cross_satker_role`, so this pins
    /// that it answers from the shared allowlist and not a private copy. Every
    /// role in `CROSS_SATKER_ROLES` must unlock cross-satker access, and a
    /// satker-bound role must not.
    #[test]
    fn cross_satker_roles_follow_the_shared_allowlist() {
        for role in lib_core::authz::CROSS_SATKER_ROLES {
            let c = claims_with_role(role);
            assert!(
                c.is_cross_satker_role(),
                "{role} is cross-satker but the boundary was enforced"
            );
            assert!(c.can_access_satker("99.99"));
        }

        for role in ["operator_satker", "validator_satker", "validator_wilayah"] {
            let c = claims_with_role(role);
            assert!(!c.is_cross_satker_role(), "{role} must stay satker-bound");
        }
    }

    #[test]
    fn a_cross_satker_role_among_several_unlocks_the_boundary() {
        let c = Claims::with_roles(Uuid::nil(), "u", ["operator_satker", "validator_pusat"])
            .in_satker("02.28");
        assert!(c.can_access_satker("99.99"));
    }

    #[test]
    fn validation_keeps_every_role_and_the_satker() {
        let c = Claims::from_validation(&response(
            &["operator_satker", "validator_wilayah"],
            Some("02.28"),
        ))
        .unwrap();
        assert_eq!(c.roles, vec!["operator_satker", "validator_wilayah"]);
        assert_eq!(c.role, "validator_wilayah");
        assert_eq!(c.satker_code.as_deref(), Some("02.28"));
        assert_eq!(c.username, "budi");
    }

    #[test]
    fn validation_without_a_role_is_a_roleless_user_that_fails_every_guard() {
        let c = Claims::from_validation(&response(&[], Some("02.28"))).unwrap();
        assert_eq!(c.role, "user");
        assert!(c.roles.is_empty());
        assert!(c.require_any_role(&["operator_satker"]).is_err());
        assert!(c.require_admin().is_err());
    }

    #[test]
    fn legacy_role_scope_is_still_honoured() {
        let mut resp = response(&[], None);
        resp.scopes = vec!["role:operator_satker".into(), "satker:02.28".into()];
        let c = Claims::from_validation(&resp).unwrap();
        assert_eq!(c.roles, vec!["operator_satker"]);
        assert_eq!(c.satker_code.as_deref(), Some("02.28"));
    }

    /// `system` is the workflow engine's internal actor. A token must not be
    /// able to assert it just because someone created a role with that name.
    #[test]
    fn a_token_cannot_assert_the_reserved_system_role() {
        let c = Claims::from_validation(&response(&["system", "operator_satker"], None)).unwrap();
        assert_eq!(c.roles, vec!["operator_satker"]);
        assert!(!c.holds_role("system"));

        let only_system = Claims::from_validation(&response(&["system"], None)).unwrap();
        assert!(only_system.roles.is_empty());
        assert_eq!(only_system.role, "user");
    }

    #[test]
    fn an_invalid_verdict_is_an_authentication_error() {
        let mut resp = response(&["admin"], None);
        resp.valid = false;
        resp.error = Some("expired".into());
        assert!(matches!(
            Claims::from_validation(&resp).unwrap_err(),
            AppError::Authentication(m) if m == "expired"
        ));

        let mut no_user = response(&["admin"], None);
        no_user.user_id = None;
        assert!(Claims::from_validation(&no_user).is_err());
    }

    #[test]
    fn only_the_websocket_handshake_skips_the_bearer_requirement() {
        assert!(is_self_authenticating("/dashboard/ws"));
        assert!(is_self_authenticating("/api/v1/perlengkapan/dashboard/ws"));
        for guarded in [
            "/dashboard/perlengkapan",
            "/dashboard/ws/extra",
            "/admin/master",
            "/pemakaian-bmn",
            "/",
        ] {
            assert!(
                !is_self_authenticating(guarded),
                "{guarded} must need a token"
            );
        }
    }

    /// The forwarding headers are believed only from a trusted proxy, and the
    /// value that comes back is always a real address.
    #[test]
    fn client_ip_ignores_forwarding_headers_from_an_untrusted_peer() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "6.6.6.6".parse().unwrap());
        headers.insert("x-real-ip", "7.7.7.7".parse().unwrap());
        let peer: SocketAddr = "203.0.113.9:4444".parse().unwrap();
        assert_eq!(ClientIp::resolve(Some(peer), &headers).0, "203.0.113.9");
    }

    #[test]
    fn client_ip_without_a_peer_is_unknown_not_a_header_value() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "6.6.6.6".parse().unwrap());
        assert_eq!(ClientIp::resolve(None, &headers).0, "unknown");
    }
}
