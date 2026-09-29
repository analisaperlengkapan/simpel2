//! Authorization vocabulary shared by backend and both microfrontends.
//!
//! # Why this module exists
//!
//! Before it, "is this caller an admin?" had **four** different answers in one
//! repository:
//!
//! | Location | Rule |
//! |---|---|
//! | `lib_core::auth::UserRole::is_admin` | `starts_with("admin")` |
//! | `authenc` `admin_auth_middleware` | `roles.contains("admin")` |
//! | perlengkapan `Claims::require_admin` | `admin \| admin_pusat \| superadmin` |
//! | perlengkapan `Claims::is_cross_satker_role` | the above **+** `validator_pusat`, `pusat`, `analis_pusat` |
//!
//! The frontend's answer was the loosest of the four, so the UI could claim
//! admin while the server refused — or, worse, treat a non-admin as an admin.
//! `starts_with("admin")` admits `admin_master_read_only`, `admin_audit_log`,
//! and `administrative`, all of which are names an operator may plausibly mint
//! because roles are database rows, not compile-time variants.
//!
//! Every predicate here is therefore an **exact match against a named role**.
//! That is the only rule that keeps the two sides in agreement when the role
//! table grows.
//!
//! # Layering
//!
//! [`RoleSet`] answers only "which realm roles does this caller hold?" — a
//! pure projection of the token, with no notion of what a role may *do*.
//! [`Capability`] is the action-oriented layer on top. Callers that gate a
//! surface should ask for a capability, not for a role name, so the mapping
//! from role to privilege lives in exactly one place.

use crate::jwt_claims::Claims;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Roles that grant administrative (cross-satker, system-wide) authority.
///
/// Mirrors perlengkapan's `Claims::require_admin` and `authenc`'s
/// `admin_auth_middleware`, which is the strictest definition in the repo.
/// `admin_pusat` and `superadmin` are included because the backend accepts
/// them; the frontend must not be stricter or looser than the API it calls.
pub const ADMIN_ROLES: &[&str] = &["admin", "superadmin", "admin_pusat"];

/// Is `role` an administrative role? Case-insensitive.
///
/// The single predicate both the frontends and the backend call, so
/// `require_admin` in the API and `Capability::Administer` in the UI can never
/// disagree about who is an admin.
pub fn is_admin_role(role: &str) -> bool {
    let normalized = role.trim().to_ascii_lowercase();
    ADMIN_ROLES.contains(&normalized.as_str())
}

/// Roles that may administer the **identity provider** (authenc IAM: users,
/// role assignment, MFA resets, OAuth clients, the audit trail).
///
/// Deliberately narrower than [`ADMIN_ROLES`]. Being the administrator of the
/// *perlengkapan application* (master data, templates, workflow config) does
/// not entitle anyone to create accounts, reset passwords, switch off another
/// user's MFA or grant roles — the IdP is the root of trust for every other
/// service, so its administrators are the smallest set that works. This is the
/// pre-#930 behaviour of `admin_auth_middleware` (exact `admin`); PR #930 had
/// widened it to the application-admin list, which let a role that does not
/// even exist in the seed grant itself `admin`.
pub const IAM_ADMIN_ROLES: &[&str] = &["admin"];

/// Is `role` an IAM (identity-provider) administrator? Case-insensitive.
pub fn is_iam_admin_role(role: &str) -> bool {
    let normalized = role.trim().to_ascii_lowercase();
    IAM_ADMIN_ROLES.contains(&normalized.as_str())
}

/// Roles that may read or act across satker boundaries.
///
/// Superset of [`ADMIN_ROLES`]: verification and analysis roles see the whole
/// estate without being able to administer it. Mirrors
/// `Claims::is_cross_satker_role`.
pub const CROSS_SATKER_ROLES: &[&str] = &[
    "admin",
    "superadmin",
    "admin_pusat",
    "validator_pusat",
    "pusat",
    "analis_pusat",
];

/// Verification roles, ordered from the narrowest scope to the widest.
///
/// `validator_satker` is satker-bound; `validator_wilayah` covers a Kejati's
/// wilayah; `validator_pusat` covers everything. The backend derives RBAC data
/// scoping from exactly this ordering (see perlengkapan `SatkerScope`).
pub const VALIDATOR_ROLES: &[&str] = &["validator_satker", "validator_wilayah", "validator_pusat"];

/// Operator (data-entry) roles, bound to a single satker.
///
/// Only the role that exists in the IAM seed. The list used to include a bare
/// `operator` that nothing issues — a phantom that would have granted
/// `Create` to whoever eventually minted the name.
pub const OPERATOR_ROLES: &[&str] = &["operator_satker"];

/// Satker-internal approval chain of the Pemakaian BMN workflow.
///
/// `validator_satker` forwards/returns a submission; `approver_satker` (the
/// Kuasa Pengguna Barang's seat) approves, returns or revokes. Both are seeded
/// by authenc migration 004 and are gated by `PemakaianBmnPolicy`, but had no
/// place in this vocabulary — so they held **zero** capabilities and no
/// display metadata.
pub const SATKER_VALIDATOR_ROLES: &[&str] = &["validator_satker"];
/// See [`SATKER_VALIDATOR_ROLES`].
pub const SATKER_APPROVER_ROLES: &[&str] = &["approver_satker"];

/// Roles in display-priority order: when a caller holds several, the first of
/// these is the one identity surfaces show.
///
/// Widest authority first. It lives here, next to the allowlists, because the
/// perlengkapan frontend used to keep its own copy that listed `super_admin` —
/// a spelling no issuer mints — while omitting the real `superadmin`. A
/// superadmin therefore rendered as an ordinary operator in the profile badge
/// and the dashboard scope wording.
///
/// This order is also what makes the backend deterministic for a caller who
/// holds several roles: the primary role is the *first entry of this list that
/// the caller holds*, never "whichever the database returned first".
pub const PRIMARY_ROLE_PRIORITY: &[&str] = &[
    "admin",
    "superadmin",
    "admin_pusat",
    "validator_pusat",
    "validator_wilayah",
    "approver_satker",
    "validator_satker",
    "operator_satker",
];

/// A human-readable description of one role, for identity surfaces.
///
/// Kept here rather than in a microfrontend so the portal and perlengkapan
/// describe the same role the same way — previously each app had its own
/// hardcoded label list, and they disagreed (`super_admin` vs `superadmin`,
/// `admin_wilayah` present in one and absent in the other).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoleInfo {
    /// The exact realm role string, as it appears in the token.
    pub key: &'static str,
    /// Indonesian display label.
    pub label: &'static str,
    /// One-line explanation of the role's scope, shown in identity surfaces.
    pub description: &'static str,
}

/// Every role the user interface knows how to describe.
///
/// This is a **display** catalog, not an authorization allowlist: a caller
/// holding a role absent from this list is still authenticated and still
/// authorized by whatever capability that role grants through
/// [`Capability::roles`]. It exists so the UI can say something better than
/// the raw string.
pub const ROLE_CATALOG: &[RoleInfo] = &[
    RoleInfo {
        key: "admin",
        label: "Administrator",
        description: "Administrator sistem: data master, template, audit, pengguna dan role (tanpa wewenang persetujuan bisnis)",
    },
    RoleInfo {
        key: "superadmin",
        label: "Super Administrator",
        description: "Administrator aplikasi (tanpa wewenang persetujuan bisnis)",
    },
    RoleInfo {
        key: "admin_pusat",
        label: "Admin Pusat",
        description: "Administrator tingkat Kejaksaan Agung",
    },
    RoleInfo {
        key: "validator_pusat",
        label: "Validator Pusat",
        description: "Verifikator akhir di Kejaksaan Agung",
    },
    RoleInfo {
        key: "validator_wilayah",
        label: "Validator Wilayah",
        description: "Verifikator di tingkat Kejaksaan Tinggi",
    },
    RoleInfo {
        key: "approver_satker",
        label: "Approver Satker",
        description: "Kuasa Pengguna Barang: menyetujui, mengembalikan atau mencabut izin pemakaian BMN di Satuan Kerja",
    },
    RoleInfo {
        key: "validator_satker",
        label: "Validator Satker",
        description: "Memverifikasi usulan pemakaian BMN di dalam Satuan Kerja sebelum diteruskan ke Approver Satker",
    },
    RoleInfo {
        key: "operator_satker",
        label: "Operator Satker",
        description: "Pengelola perlengkapan di tingkat Satuan Kerja",
    },
    RoleInfo {
        key: "pusat",
        label: "Pusat",
        description: "Akses lintas satker tingkat pusat",
    },
    RoleInfo {
        key: "analis_pusat",
        label: "Analis Pusat",
        description: "Analis lintas satker tingkat pusat",
    },
];

/// Look up the display metadata for a role, case-insensitively.
pub fn role_info(role: &str) -> Option<&'static RoleInfo> {
    let needle = role.trim().to_lowercase();
    ROLE_CATALOG.iter().find(|r| r.key == needle)
}

/// A display label for a role: the catalog label when known, otherwise the
/// role string with underscores turned into spaces and each word capitalised
/// (so a newly minted role renders readably instead of as raw snake_case).
pub fn role_label(role: &str) -> String {
    if let Some(info) = role_info(role) {
        return info.label.to_string();
    }
    role.split(['_', '-'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Something a caller is allowed to do, independent of which role grants it.
///
/// Surfaces should gate on these rather than on role names. Adding a role then
/// means editing [`Capability::roles`], not hunting every `has_role("admin")`
/// across two WASM apps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Read data, including one's own satker's records. Held by **any**
    /// authenticated caller that carries at least one role (see
    /// [`Authorization::can`]); the row-level boundary is the backend's
    /// `SatkerScope`, not this capability.
    View,
    /// Create and edit draft records (kebutuhan, pemakaian, penghapusan, …).
    Create,
    /// Verify (forward/return) a Pemakaian BMN submission inside the satker.
    ValidateSatker,
    /// Approve, return or revoke a Pemakaian BMN permit as the satker's
    /// Kuasa Pengguna Barang.
    ApproveSatker,
    /// Verify or reject a submission as validator wilayah.
    ValidateWilayah,
    /// Verify or reject a submission as validator pusat, and issue final SK.
    ValidatePusat,
    /// See records across satker boundaries.
    ViewAllSatker,
    /// Administer the **perlengkapan application**: master data, templates,
    /// workflow configuration. Does *not* imply any business approval and does
    /// *not* imply [`Capability::AdministerIam`].
    Administer,
    /// Administer the **identity provider**: users, role assignment, MFA
    /// resets, OAuth clients (authenc IAM). See [`IAM_ADMIN_ROLES`].
    AdministerIam,
    /// View audit logs and operational monitoring surfaces.
    ViewAudit,
    /// Triage helpdesk tickets on behalf of other users.
    ManageTickets,
}

impl Capability {
    /// Every capability, in declaration order. Used by [`Authorization::capabilities`].
    pub const ALL: &'static [Capability] = &[
        Capability::View,
        Capability::Create,
        Capability::ValidateSatker,
        Capability::ApproveSatker,
        Capability::ValidateWilayah,
        Capability::ValidatePusat,
        Capability::ViewAllSatker,
        Capability::Administer,
        Capability::AdministerIam,
        Capability::ViewAudit,
        Capability::ManageTickets,
    ];

    /// The roles that grant this capability.
    ///
    /// This table is the single mapping between roles and privileges. It is
    /// **exact and additive**: a role holds precisely what is listed for it
    /// here — an administrator is not silently a superset of the business
    /// roles. That is a deliberate change from the pre-#930 "admin bypasses
    /// every check" behaviour: the IT administrator of a system must not also
    /// be able to approve the very requests the system routes to business
    /// authorities (segregation of duties; NIST INCITS 359 SSD/DSD, OWASP
    /// Authorization: least privilege). Emergency intervention goes through the
    /// audited break-glass path, not through an implicit role.
    ///
    /// [`Capability::View`] is empty here on purpose: it is granted to any
    /// caller holding at least one role by [`Authorization::can`], so a role
    /// that appears in no list still reads its own satker's data.
    pub fn roles(self) -> &'static [&'static str] {
        match self {
            Capability::View => &[],
            Capability::Create => OPERATOR_ROLES,
            Capability::ValidateSatker => SATKER_VALIDATOR_ROLES,
            Capability::ApproveSatker => SATKER_APPROVER_ROLES,
            Capability::ValidateWilayah => &["validator_wilayah"],
            Capability::ValidatePusat => &["validator_pusat"],
            Capability::ViewAllSatker => CROSS_SATKER_ROLES,
            // Application administration; also monitoring/audit surfaces and
            // helpdesk triage, which are operational rather than business
            // approvals. There is no dedicated auditor role yet, so these three
            // share ADMIN_ROLES — see the audit report (A9) before adding a
            // capability that claims to be separate but is not.
            Capability::Administer => ADMIN_ROLES,
            Capability::ViewAudit => ADMIN_ROLES,
            Capability::ManageTickets => ADMIN_ROLES,
            Capability::AdministerIam => IAM_ADMIN_ROLES,
        }
    }

    /// The stable wire key for this capability.
    ///
    /// Matches the serde representation, so a permission string sent over an
    /// API and one parsed here are the same vocabulary.
    pub fn key(self) -> &'static str {
        match self {
            Capability::View => "view",
            Capability::Create => "create",
            Capability::ValidateSatker => "validate_satker",
            Capability::ApproveSatker => "approve_satker",
            Capability::ValidateWilayah => "validate_wilayah",
            Capability::ValidatePusat => "validate_pusat",
            Capability::ViewAllSatker => "view_all_satker",
            Capability::Administer => "administer",
            Capability::AdministerIam => "administer_iam",
            Capability::ViewAudit => "view_audit",
            Capability::ManageTickets => "manage_tickets",
        }
    }

    /// Parse a capability from its [`key`](Self::key), case-insensitively.
    ///
    /// Returns `None` for an unknown permission. Callers **must** treat `None`
    /// as a denial: an unrecognised permission is not a permission, and
    /// defaulting it to `true` would make every typo in a route guard silently
    /// grant access.
    pub fn parse(key: &str) -> Option<Capability> {
        let needle = key.trim().to_ascii_lowercase();
        Capability::ALL.iter().copied().find(|c| c.key() == needle)
    }
}

/// The set of realm roles a caller holds.
///
/// A thin, allocation-light wrapper over the `realm_access.roles` claim. It
/// exists so that "does this caller hold role X" is answered in one place, by
/// exact match, instead of by the four divergent predicates described in the
/// module docs.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleSet(BTreeSet<String>);

impl RoleSet {
    /// Build from an explicit role list.
    pub fn new<I, S>(roles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self(
            roles
                .into_iter()
                .map(Into::into)
                .map(|r: String| r.trim().to_lowercase())
                .filter(|r| !r.is_empty())
                .collect(),
        )
    }

    /// Build from decoded JWT claims, reading `realm_access.roles`.
    pub fn from_claims(claims: &Claims) -> Self {
        claims
            .realm_access
            .as_ref()
            .map(|ra| Self::new(ra.roles.clone()))
            .unwrap_or_default()
    }

    /// Build from a raw role list, as it appears in a session projection.
    pub fn from_slice(roles: &[String]) -> Self {
        Self::new(roles.to_vec())
    }

    /// Every role held, lowercased and de-duplicated.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(String::as_str)
    }

    /// Number of distinct roles.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// True when no roles are present at all.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Exact-match role test. Case-insensitive; **never** a prefix match.
    pub fn has(&self, role: &str) -> bool {
        self.0.contains(&role.trim().to_lowercase())
    }

    /// True when the caller holds any role in `roles`.
    pub fn has_any(&self, roles: &[&str]) -> bool {
        roles.iter().any(|r| self.has(r))
    }

    /// Administrative authority (see [`ADMIN_ROLES`]).
    pub fn is_admin(&self) -> bool {
        self.has_any(ADMIN_ROLES)
    }

    /// May act across satker boundaries (see [`CROSS_SATKER_ROLES`]).
    pub fn is_cross_satker(&self) -> bool {
        self.has_any(CROSS_SATKER_ROLES)
    }

    /// Holds any verification role.
    pub fn is_validator(&self) -> bool {
        self.has_any(VALIDATOR_ROLES)
    }

    /// Holds any operator role.
    pub fn is_operator(&self) -> bool {
        self.has_any(OPERATOR_ROLES)
    }

    /// The role to show when a caller holds several, in
    /// [`PRIMARY_ROLE_PRIORITY`] order. Falls back to the lexicographically
    /// first role the token carries, so an unrecognised role still yields a
    /// stable label instead of an empty one.
    pub fn primary(&self) -> Option<&str> {
        PRIMARY_ROLE_PRIORITY
            .iter()
            .find(|candidate| self.has(candidate))
            .copied()
            .or_else(|| self.iter().next())
    }
}

/// A resolved permission decision for one caller.
///
/// Construct it once per render from the session's roles, then ask it
/// questions. Keeping this a value (rather than free functions over a session)
/// means the same logic runs in `lib-ui`, in the portal, and in perlengkapan
/// without any of them re-implementing a role predicate.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Authorization {
    roles: RoleSet,
}

impl Authorization {
    /// Resolve from explicit roles.
    pub fn new(roles: RoleSet) -> Self {
        Self { roles }
    }

    /// Resolve from decoded JWT claims.
    pub fn from_claims(claims: &Claims) -> Self {
        Self::new(RoleSet::from_claims(claims))
    }

    /// Resolve from a session's role projection (`UserSession::roles`).
    pub fn from_slice(roles: &[String]) -> Self {
        Self::new(RoleSet::from_slice(roles))
    }

    /// Resolve from comma/space separated roles, for config or test fixtures.
    pub fn from_csv(roles: &str) -> Self {
        Self::new(RoleSet::new(roles.split([',', ' '])))
    }

    /// The underlying role set.
    pub fn roles(&self) -> &RoleSet {
        &self.roles
    }

    /// Administrative authority.
    pub fn is_admin(&self) -> bool {
        self.roles.is_admin()
    }

    /// May act across satker boundaries.
    pub fn is_cross_satker(&self) -> bool {
        self.roles.is_cross_satker()
    }

    /// The single role identity surfaces should display, per
    /// [`PRIMARY_ROLE_PRIORITY`].
    pub fn primary_role(&self) -> Option<&str> {
        self.roles.primary()
    }

    /// Administer the identity provider (authenc IAM). Exact `admin` only.
    pub fn is_iam_admin(&self) -> bool {
        self.roles.has_any(IAM_ADMIN_ROLES)
    }

    /// True when the caller may exercise `capability`.
    ///
    /// Exact and additive (see [`Capability::roles`]): there is no implicit
    /// "admin can do everything" here, and the backend no longer has one
    /// either, so the UI and the API cannot disagree about it. The single
    /// exception is [`Capability::View`], granted to any caller holding at
    /// least one role — before this, `can(View)` was `false` for every
    /// non-admin because its role list is empty, the opposite of what the
    /// variant's own documentation promises.
    pub fn can(&self, capability: Capability) -> bool {
        if capability == Capability::View {
            return !self.roles.is_empty();
        }
        self.roles.has_any(capability.roles())
    }

    /// Every capability the caller holds. Drives admin surfaces that render a
    /// capability list rather than branching on individual predicates.
    pub fn capabilities(&self) -> Vec<Capability> {
        Capability::ALL
            .iter()
            .copied()
            .filter(|c| self.can(*c))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claims_with(roles: &[&str]) -> Claims {
        Claims {
            sub: "00000000-0000-0000-0000-000000000001".into(),
            preferred_username: Some("tester".into()),
            name: None,
            email: None,
            nip: None,
            jabatan: None,
            satker_code: Some("02.28".into()),
            satker_id: None,
            satker_nama: None,
            realm_access: Some(crate::jwt_claims::RealmAccess {
                roles: roles.iter().map(|r| r.to_string()).collect(),
            }),
            assigned_roles: Vec::new(),
            active_role: None,
            groups: Vec::new(),
            resource_access: None,
            mfa_enabled: false,
            mfa_setup_required: false,
            require_password_change: false,
            exp: 9_999_999_999,
            iat: 0,
            iss: "authenc".into(),
        }
    }

    #[test]
    fn admin_is_exact_match_not_prefix() {
        // These four names all begin with "admin"; a prefix test admits them.
        // The backend's `admin_auth_middleware` accepts only the exact string
        // "admin", so treating any of these as an admin makes the UI grant a
        // surface the API will then refuse.
        for bogus in [
            "admin_audit_log",
            "administrative",
            "admin_master_read_only",
            "administrator",
            "admin_read_only",
        ] {
            let authz = Authorization::from_csv(bogus);
            assert!(
                !authz.is_admin(),
                "'{bogus}' must NOT be treated as an admin role"
            );
            assert!(
                !authz.can(Capability::Administer),
                "'{bogus}' must NOT be able to administer"
            );
        }
    }

    #[test]
    fn real_admin_roles_are_admins() {
        for role in ADMIN_ROLES {
            let authz = Authorization::from_csv(role);
            assert!(authz.is_admin(), "'{role}' should be an admin role");
            assert!(authz.can(Capability::Administer));
            assert!(authz.can(Capability::ViewAudit));
            assert!(authz.can(Capability::ManageTickets));
        }
    }

    #[test]
    fn satker_roles_are_not_admins_and_cannot_administer() {
        for role in [
            "operator_satker",
            "validator_wilayah",
            "validator_pusat",
            "validator_satker",
        ] {
            let authz = Authorization::from_csv(role);
            assert!(!authz.is_admin(), "'{role}' is not an admin");
            assert!(
                !authz.can(Capability::Administer),
                "'{role}' must not administer"
            );
        }
    }

    #[test]
    fn validator_scopes_are_distinct() {
        // The whole point of fine-grained RBAC: wilayah cannot do pusat's job.
        let wilayah = Authorization::from_csv("validator_wilayah");
        assert!(wilayah.can(Capability::ValidateWilayah));
        assert!(!wilayah.can(Capability::ValidatePusat));

        let pusat = Authorization::from_csv("validator_pusat");
        assert!(pusat.can(Capability::ValidatePusat));
        assert!(!pusat.can(Capability::ValidateWilayah));

        // operator_satker can create but cannot validate at any level.
        let operator = Authorization::from_csv("operator_satker");
        assert!(operator.can(Capability::Create));
        assert!(!operator.can(Capability::ValidateWilayah));
        assert!(!operator.can(Capability::ValidatePusat));
    }

    #[test]
    fn validator_pusat_sees_all_satker_without_being_admin() {
        let pusat = Authorization::from_csv("validator_pusat");
        assert!(pusat.is_cross_satker());
        assert!(pusat.can(Capability::ViewAllSatker));
        assert!(!pusat.is_admin());
        assert!(!pusat.can(Capability::Administer));
    }

    /// Segregation of duties, pinned. The pre-#930 model let an administrator
    /// stand in for every business role (`is_admin_implied`), which is how an IT
    /// admin could approve, reject and finalise a BMN disposal. An administrator
    /// now holds administration — and *reads* — but none of the business
    /// approval capabilities.
    #[test]
    fn admin_does_not_imply_business_capabilities() {
        let admin = Authorization::from_csv("admin");
        for cap in [
            Capability::Create,
            Capability::ValidateSatker,
            Capability::ApproveSatker,
            Capability::ValidateWilayah,
            Capability::ValidatePusat,
        ] {
            assert!(
                !admin.can(cap),
                "admin must NOT hold {cap:?} implicitly (SoD); use break-glass"
            );
        }
        assert!(admin.can(Capability::View));
        assert!(admin.can(Capability::Administer));
    }

    /// `can(View)` used to be `false` for every non-admin: `View`'s role list is
    /// empty and the only fallback was an admin-implied clause. Any caller with
    /// at least one role reads; a caller with none reads nothing.
    #[test]
    fn view_is_held_by_any_caller_with_a_role() {
        for role in [
            "operator_satker",
            "validator_satker",
            "approver_satker",
            "validator_wilayah",
            "validator_pusat",
            "some_future_role",
        ] {
            assert!(
                Authorization::from_csv(role).can(Capability::View),
                "'{role}' holds a role and must be able to View"
            );
        }
        assert!(!Authorization::from_csv("").can(Capability::View));
        assert!(!Authorization::default().can(Capability::View));
    }

    /// The Pemakaian BMN chain has two satker-internal seats that used to hold
    /// zero capabilities. Each holds exactly its own step.
    #[test]
    fn satker_chain_roles_hold_their_own_step_only() {
        let validator = Authorization::from_csv("validator_satker");
        assert!(validator.can(Capability::ValidateSatker));
        assert!(!validator.can(Capability::ApproveSatker));
        assert!(!validator.can(Capability::Create));

        let approver = Authorization::from_csv("approver_satker");
        assert!(approver.can(Capability::ApproveSatker));
        assert!(!approver.can(Capability::ValidateSatker));
        assert!(!approver.can(Capability::Create));
    }

    /// IAM administration is a strictly smaller set than application
    /// administration: the roles PR #930 admitted to the IdP console do not
    /// administer the IdP.
    #[test]
    fn iam_admin_is_exact_admin_only() {
        assert!(Authorization::from_csv("admin").can(Capability::AdministerIam));
        assert!(Authorization::from_csv("admin").is_iam_admin());
        for role in [
            "superadmin",
            "admin_pusat",
            "admin_readonly",
            "operator_satker",
        ] {
            let a = Authorization::from_csv(role);
            assert!(
                !a.can(Capability::AdministerIam),
                "'{role}' must not administer the IdP"
            );
            assert!(!a.is_iam_admin());
        }
        // Still an application administrator, just not an IdP one.
        assert!(Authorization::from_csv("admin_pusat").can(Capability::Administer));
        assert!(is_iam_admin_role("ADMIN"));
        assert!(!is_iam_admin_role("superadmin"));
        assert!(IAM_ADMIN_ROLES.iter().all(|r| ADMIN_ROLES.contains(r)));
    }

    /// Every role that any capability names must be describable — a privileged
    /// role must never render as a raw string on an identity surface — and no
    /// capability may name a role the seed does not issue.
    #[test]
    fn every_role_named_by_a_capability_is_in_the_catalog() {
        for cap in Capability::ALL {
            for role in cap.roles() {
                assert!(
                    role_info(role).is_some(),
                    "{cap:?} names role '{role}' which has no display metadata"
                );
            }
        }
    }

    #[test]
    fn admin_does_not_imply_everything() {
        // `capabilities()` must not be a constant function; if it were, the
        // carve-outs for audit/tickets would be decorative.
        let admin = Authorization::from_csv("admin").capabilities();
        let operator = Authorization::from_csv("operator_satker").capabilities();
        assert!(
            admin.len() > operator.len(),
            "admin should hold strictly more capabilities than an operator"
        );
        assert!(operator.contains(&Capability::Create));
        assert!(!operator.contains(&Capability::Administer));
        assert!(!operator.contains(&Capability::ViewAudit));
    }

    #[test]
    fn multi_role_user_accumulates_capabilities() {
        let authz = Authorization::from_csv("operator_satker,validator_wilayah");
        assert!(authz.can(Capability::Create));
        assert!(authz.can(Capability::ValidateWilayah));
        assert!(!authz.can(Capability::ValidatePusat));
    }

    #[test]
    fn unknown_or_empty_role_grants_nothing_privileged() {
        let empty = Authorization::from_csv("");
        assert!(!empty.is_admin());
        assert!(!empty.is_cross_satker());
        assert!(!empty.can(Capability::Administer));
        assert!(!empty.can(Capability::Create));

        let unknown = Authorization::from_csv("some_future_role");
        assert!(!unknown.can(Capability::Administer));
        assert!(!unknown.can(Capability::ViewAudit));
        assert!(!unknown.can(Capability::ManageTickets));
    }

    #[test]
    fn claims_projection_reads_realm_access() {
        let authz = Authorization::from_claims(&claims_with(&["validator_wilayah"]));
        assert!(authz.can(Capability::ValidateWilayah));
        assert!(!authz.is_admin());

        // No realm_access at all → no roles → no privileges.
        let mut bare = claims_with(&[]);
        bare.realm_access = None;
        let authz = Authorization::from_claims(&bare);
        assert!(!authz.is_admin());
        assert!(authz.roles().is_empty());
    }

    #[test]
    fn role_matching_is_case_and_whitespace_insensitive() {
        let authz = Authorization::from_csv("  ADMIN , Validator_Wilayah ");
        assert!(authz.is_admin());
        assert!(authz.can(Capability::ValidateWilayah));
    }

    #[test]
    fn role_catalog_labels_known_roles_and_passes_through_unknown() {
        assert_eq!(role_label("validator_pusat"), "Validator Pusat");
        assert_eq!(role_label("admin"), "Administrator");
        // Unknown roles still render readably rather than as raw snake_case.
        assert_eq!(role_label("some_new_role"), "Some New Role");
        assert_eq!(role_label("already"), "Already");
        assert!(role_info("operator_satker").is_some());
        assert!(role_info("OPERATOR_SATKER").is_some());
        assert!(role_info("nope_not_a_role").is_none());
    }

    #[test]
    fn role_catalog_is_consistent_with_the_authorization_allowlists() {
        // Every role that grants or scopes authority must be describable, or
        // an identity surface would show a raw string for a privileged role.
        for role in ADMIN_ROLES.iter().chain(CROSS_SATKER_ROLES.iter()) {
            assert!(
                role_info(role).is_some(),
                "role '{role}' grants authority but has no display metadata"
            );
        }
    }

    #[test]
    fn admin_roles_are_mirrored_by_the_backend_list() {
        // This pins the CONTRACT, not a copy. The perlengkapan backend's
        // `require_admin`/`require_any_role` and both frontends' guards all
        // call `is_admin_role`, so they cannot disagree by construction; what
        // this test protects is the value itself changing without intent,
        // which would silently widen or narrow every one of those call sites
        // at once. authenc's `admin_auth_middleware` still accepts the exact
        // string "admin", so the list must not lose that entry either.
        assert_eq!(ADMIN_ROLES, &["admin", "superadmin", "admin_pusat"]);
        assert!(CROSS_SATKER_ROLES.starts_with(ADMIN_ROLES));
    }

    /// The API's `require_admin` and the UI's `Capability::Administer` now both
    /// resolve through [`ADMIN_ROLES`]. This is the invariant that makes that
    /// wiring worth anything: no role may be an API admin while the UI refuses
    /// to render its controls, and vice versa.
    #[test]
    fn every_admin_role_holds_administer() {
        for role in ADMIN_ROLES {
            assert!(is_admin_role(role), "{role} must satisfy is_admin_role");
            let authz = Authorization::from_slice(&[role.to_string()]);
            assert!(
                authz.can(Capability::Administer),
                "{role} is an API admin but lacks Capability::Administer"
            );
        }
    }

    /// The issuer mints `superadmin`; `super_admin` (underscore) is not a realm
    /// role. Pinned so a future "helpful" normalization cannot quietly widen
    /// the allowlist to a name nothing issues.
    #[test]
    fn super_admin_with_underscore_is_not_an_admin() {
        assert!(!is_admin_role("super_admin"));
        assert!(!is_admin_role("admin_readonly"));
        assert!(!is_admin_role(""));
    }

    /// A holder of several roles shows the widest one. The regression this
    /// pins: perlengkapan's private copy of this list named `super_admin`, so
    /// a `superadmin` fell through every entry and the badge showed the
    /// fallback (often an operator role) instead.
    #[test]
    fn primary_role_prefers_next_widest_authority() {
        let authz = Authorization::from_csv("superadmin operator_satker");
        assert_eq!(authz.primary_role(), Some("superadmin"));

        let authz = Authorization::from_csv("validator_wilayah validator_pusat");
        assert_eq!(authz.primary_role(), Some("validator_pusat"));

        let authz = Authorization::from_csv("operator_satker");
        assert_eq!(authz.primary_role(), Some("operator_satker"));
    }

    /// An unrecognised role still yields a stable label rather than `None`, so
    /// a newly minted role renders instead of leaving the badge blank.
    #[test]
    fn primary_role_falls_back_to_an_unknown_role() {
        let authz = Authorization::from_csv("auditor_eksternal");
        assert_eq!(authz.primary_role(), Some("auditor_eksternal"));

        assert_eq!(Authorization::from_csv("").primary_role(), None);
    }

    #[test]
    fn capability_key_round_trips_through_parse() {
        for cap in Capability::ALL {
            assert_eq!(
                Capability::parse(cap.key()),
                Some(*cap),
                "{:?} does not round-trip through its key '{}'",
                cap,
                cap.key()
            );
        }
    }

    #[test]
    fn capability_keys_are_unique() {
        let mut keys: Vec<&str> = Capability::ALL.iter().map(|c| c.key()).collect();
        keys.sort_unstable();
        let before = keys.len();
        keys.dedup();
        assert_eq!(before, keys.len(), "two capabilities share a wire key");
    }

    #[test]
    fn capability_key_matches_the_serde_representation() {
        // A capability sent over an API (serde) and one parsed from a route
        // guard must be the same vocabulary, or the two drift silently.
        for cap in Capability::ALL {
            let json = serde_json::to_string(cap).unwrap();
            assert_eq!(json, format!("\"{}\"", cap.key()));
            assert_eq!(Capability::parse(json.trim_matches('"')), Some(*cap));
        }
    }

    #[test]
    fn unknown_permission_does_not_parse() {
        // `parse` returning `Some` for anything would let a typo in a route
        // guard resolve to a real capability.
        for bogus in [
            "administrator",
            "admin",
            "view_audit_logs",
            "creates",
            "validate",
            "VALIDATE_WILAYAH_EXTRA",
            "",
        ] {
            assert_eq!(
                Capability::parse(bogus),
                None,
                "'{bogus}' must not parse as a capability"
            );
        }
    }

    #[test]
    fn capability_parse_is_case_insensitive() {
        assert_eq!(
            Capability::parse("  View_Audit "),
            Some(Capability::ViewAudit)
        );
        assert_eq!(
            Capability::parse("ADMINISTER"),
            Some(Capability::Administer)
        );
    }

    /// The permission names must line up with what the guards actually check.
    /// `view_audit` and `administer` are deliberately distinct: an audit-log
    /// reader must not acquire user-administration rights by holding the
    /// former, and this pins that the parse yields the narrower capability.
    #[test]
    fn audit_and_administer_are_separate_capabilities() {
        let audit = Capability::parse("view_audit").unwrap();
        let administer = Capability::parse("administer").unwrap();
        assert_ne!(audit, administer);

        // An admin holds both; that is the superset, not a collapse of the two.
        let admin = Authorization::from_csv("admin");
        assert!(admin.can(audit));
        assert!(admin.can(administer));

        // An operator holds neither.
        let operator = Authorization::from_csv("operator_satker");
        assert!(!operator.can(audit));
        assert!(!operator.can(administer));
    }
}
