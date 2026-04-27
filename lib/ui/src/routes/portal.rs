//! Typed route enum for `antarmuka/portal`.
//!
//! Mirrors the constants in `antarmuka/portal/src/routes.rs`. Once the
//! workspace build is healthy enough to verify a flag-day migration,
//! that file is deleted in favor of this enum.

use super::ToPath;

/// Every navigable route inside the Portal microfrontend.
///
/// Tuple variants carry path parameters. Pages and links should
/// produce URLs through [`Self::to_path`] rather than concatenating
/// strings, so a typo or a renamed segment is a compile error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PortalRoute {
    Home,
    Login,
    Callback,
    LoggedOut,

    Dashboard,
    Apps,
    Notifications,
    Settings,

    Profile,
    Passkeys,
    Password,
    Sessions,

    MfaSetup,
    MfaVerify,
    MfaBackupVerify,
    MfaBackupCodes,

    AdminOverview,
    AdminUsers,
    AdminUserDetail { id: String },
    AdminRealms,
    AdminClients,
    AdminClientDetail { id: String },
    AdminRoles,
    AdminFederation,
    AdminPermissions,
    AdminAudit,
    AdminGroups,
    AdminRealmSettings,
    AdminAuthFlows,
    AdminLinkedAccounts,
}

impl PortalRoute {
    /// Path segment as registered with `<Route path=...>` (relative to
    /// the `<Router base="/portal">`). Useful when wiring up the
    /// route tree without re-deriving the parent prefix.
    pub fn segment(&self) -> &'static str {
        use PortalRoute::*;
        match self {
            Home => "",
            Login => "login",
            Callback => "callback",
            LoggedOut => "logged-out",

            Dashboard => "dashboard",
            Apps => "apps",
            Notifications => "notifications",
            Settings => "settings",

            Profile => "profile",
            Passkeys => "passkeys",
            Password => "password",
            Sessions => "sessions",

            MfaSetup => "mfa/setup",
            MfaVerify => "mfa/verify",
            MfaBackupVerify => "mfa/backup-verify",
            MfaBackupCodes => "mfa/backup-codes",

            AdminOverview => "admin",
            AdminUsers => "admin/users",
            AdminUserDetail { .. } => "admin/users/:id",
            AdminRealms => "admin/realms",
            AdminClients => "admin/clients",
            AdminClientDetail { .. } => "admin/clients/:id",
            AdminRoles => "admin/roles",
            AdminFederation => "admin/federation",
            AdminPermissions => "admin/permissions",
            AdminAudit => "admin/audit",
            AdminGroups => "admin/groups",
            AdminRealmSettings => "admin/realm-settings",
            AdminAuthFlows => "admin/auth-flows",
            AdminLinkedAccounts => "admin/linked-accounts",
        }
    }
}

impl ToPath for PortalRoute {
    fn to_path(&self) -> String {
        use PortalRoute::*;
        match self {
            AdminUserDetail { id } => format!("/portal/admin/users/{id}"),
            AdminClientDetail { id } => format!("/portal/admin/clients/{id}"),
            other => format!("/portal/{}", other.segment()).trim_end_matches('/').to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_resolves_to_portal_root() {
        assert_eq!(PortalRoute::Home.to_path(), "/portal");
    }

    #[test]
    fn dashboard_resolves() {
        assert_eq!(PortalRoute::Dashboard.to_path(), "/portal/dashboard");
    }

    #[test]
    fn parameterized_user_detail() {
        let url = PortalRoute::AdminUserDetail {
            id: "user-123".into(),
        }
        .to_path();
        assert_eq!(url, "/portal/admin/users/user-123");
    }
}
