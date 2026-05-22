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
    /// Returns the highest-priority role found, or User if no recognized roles.
    /// Custom roles (like admin_pusat, admin_wilayah, etc.) are supported flexibly
    /// without hardcoding - any role starting with "admin" gets admin privileges.
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

        // Check for any admin-like role in realm_access (flexible)
        if let Some(ref ra) = self.realm_access {
            for role in &ra.roles {
                let lower = role.to_lowercase();
                // Any role starting with "admin" (admin_pusat, admin_wilayah, etc.)
                if lower.starts_with("admin") {
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
