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

    /// Get primary role (Admin > Supervisor > User)
    pub fn get_primary_role(&self) -> crate::auth::UserRole {
        if self.has_role("admin") {
            crate::auth::UserRole::Admin
        } else if self.has_role("supervisor") {
            crate::auth::UserRole::Supervisor
        } else if self.has_role("guest") {
            crate::auth::UserRole::Guest
        } else {
            crate::auth::UserRole::User
        }
    }
}
