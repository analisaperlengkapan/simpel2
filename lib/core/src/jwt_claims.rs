use serde::{Deserialize, Serialize};

/// Standard JWT Claims shared across services
/// This module contains only the claim structures without decoding logic
/// so it can be used in WASM frontends without the jsonwebtoken dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (usually User ID)
    pub sub: String,

    /// Preferred username
    #[serde(default)]
    pub preferred_username: Option<String>,

    /// Full name
    #[serde(default)]
    pub name: Option<String>,

    /// Email address
    #[serde(default)]
    pub email: Option<String>,

    /// NIP (Nomor Induk Pegawai)
    #[serde(default)]
    pub nip: Option<String>,

    /// Jabatan (position/title)
    #[serde(default)]
    pub jabatan: Option<String>,

    /// Kode Satker (work unit code)
    #[serde(default)]
    pub satker_code: Option<String>,

    /// Satker UUID — paired with `satker_code` so frontends that need a
    /// stable foreign key (request payloads, BMN ownership checks) can
    /// project it directly without a round-trip lookup. Optional because
    /// some legacy users in authenc still lack the mapping; callers must
    /// handle `None` (typically by surfacing an "Anda belum terdaftar di
    /// satker manapun" error before submission).
    #[serde(default)]
    pub satker_id: Option<String>,

    /// Human-readable satker name. Convenience field for surfaces like
    /// `pemakaian_bmn_form` that want to display the satker name without
    /// fetching it again.
    #[serde(default)]
    pub satker_nama: Option<String>,

    /// Realm access (roles)
    #[serde(default)]
    pub realm_access: Option<RealmAccess>,

    /// Resource access (client specific roles)
    #[serde(default)]
    pub resource_access: Option<serde_json::Value>,

    /// MFA enabled status
    #[serde(default)]
    pub mfa_enabled: bool,

    /// MFA setup required
    #[serde(default)]
    pub mfa_setup_required: bool,

    /// Whether user must change password before using the system
    #[serde(default)]
    pub require_password_change: bool,

    /// Token expiration
    pub exp: usize,

    /// Issued at
    pub iat: usize,

    /// Issuer
    pub iss: String,
}

/// Keycloak-style Realm Access
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealmAccess {
    /// List of roles
    pub roles: Vec<String>,
}

impl Claims {
    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        if let Some(ra) = &self.realm_access {
            return ra.roles.iter().any(|r| r == role);
        }
        false
    }

    /// Get username (from preferred_username or sub)
    pub fn username(&self) -> String {
        self.preferred_username
            .clone()
            .unwrap_or_else(|| self.sub.clone())
    }

    /// Get primary role from JWT claims.
    /// Returns the highest-priority role found, or `User` if no recognized
    /// roles are present.
    ///
    /// Administrative roles are matched **exactly** against
    /// [`crate::authz::ADMIN_ROLES`]. This used to accept any role whose name
    /// merely began with `admin`, which classified read-only roles such as
    /// `admin_master_read_only` — a name this very repository uses as its
    /// example of a false positive — as administrative principals.
    pub fn get_primary_role(&self) -> crate::auth::UserRole {
        // Check for standard roles first (highest priority)
        if self.has_role("admin") {
            return crate::auth::UserRole::Admin;
        }
        if self.has_role("supervisor") {
            return crate::auth::UserRole::Supervisor;
        }
        if self.has_role("guest") {
            return crate::auth::UserRole::Guest;
        }

        // Any *exact* administrative role from the shared allowlist.
        if let Some(ref ra) = self.realm_access {
            for role in &ra.roles {
                if crate::authz::ADMIN_ROLES.contains(&role.to_lowercase().as_str()) {
                    return crate::auth::UserRole::Custom(role.clone());
                }
            }
        }

        // Default to User
        crate::auth::UserRole::User
    }

    /// Get all roles as UserRole enums
    pub fn get_all_roles(&self) -> Vec<crate::auth::UserRole> {
        if let Some(ref ra) = self.realm_access {
            ra.roles
                .iter()
                .map(|r| crate::auth::UserRole::from_string(r))
                .collect()
        } else {
            vec![crate::auth::UserRole::User]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::UserRole;

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
            realm_access: Some(RealmAccess {
                roles: roles.iter().map(|r| r.to_string()).collect(),
            }),
            resource_access: None,
            mfa_enabled: false,
            mfa_setup_required: false,
            require_password_change: false,
            exp: 9_999_999_999,
            iat: 0,
            iss: "authenc".into(),
        }
    }

    /// A role whose name merely begins with `admin` must not be promoted to an
    /// administrative primary role. This is the prefix bug that lived in
    /// `get_primary_role` and in `UserRole::is_admin`.
    #[test]
    fn admin_prefixed_roles_are_not_primary_admins() {
        for bogus in [
            "admin_audit_log",
            "administrative",
            "admin_master_read_only",
            "administrator",
        ] {
            let primary = claims_with(&[bogus]).get_primary_role();
            assert_eq!(
                primary,
                UserRole::User,
                "'{bogus}' must not become the administrative primary role"
            );
            assert!(!primary.is_admin(), "'{bogus}' must not be an admin");
        }
    }

    #[test]
    fn allowlisted_admin_roles_do_become_primary() {
        assert_eq!(claims_with(&["admin"]).get_primary_role(), UserRole::Admin);
        assert!(claims_with(&["admin_pusat"]).get_primary_role().is_admin());
        assert!(claims_with(&["superadmin"]).get_primary_role().is_admin());
    }

    #[test]
    fn satker_roles_do_not_become_admins() {
        for role in [
            "operator_satker",
            "validator_wilayah",
            "validator_pusat",
            "validator_satker",
        ] {
            let primary = claims_with(&[role]).get_primary_role();
            assert!(!primary.is_admin(), "'{role}' must not be an admin");
        }
    }

    #[test]
    fn standard_roles_keep_priority() {
        assert_eq!(
            claims_with(&["supervisor", "operator_satker"]).get_primary_role(),
            UserRole::Supervisor
        );
        assert_eq!(claims_with(&["guest"]).get_primary_role(), UserRole::Guest);
    }

    #[test]
    fn role_checks_are_exact() {
        let c = claims_with(&["operator_satker"]);
        assert!(c.has_role("operator_satker"));
        assert!(!c.has_role("operator"));
        assert!(!c.has_role("operator_satker_extra"));
    }
}
