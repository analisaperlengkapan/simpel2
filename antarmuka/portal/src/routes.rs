//! Centralized route constants for portal frontend.
//!
//! Keep all route paths in one place to avoid string duplication.

pub mod segment {
    pub const HOME: &str = "";
    pub const LOGIN: &str = "login";
    pub const CALLBACK: &str = "callback";
    pub const LOGGED_OUT: &str = "logged-out";

    pub const DASHBOARD: &str = "dashboard";
    pub const APPS: &str = "apps";
    pub const NOTIFICATIONS: &str = "notifications";
    pub const SETTINGS: &str = "settings";

    pub const PROFILE: &str = "profile";
    pub const PASSKEYS: &str = "passkeys";
    pub const PASSWORD: &str = "password";
    pub const SESSIONS: &str = "sessions";

    pub const MFA_SETUP: &str = "mfa/setup";
    pub const MFA_VERIFY: &str = "mfa/verify";
    pub const MFA_BACKUP_VERIFY: &str = "mfa/backup-verify";
    pub const MFA_BACKUP_CODES: &str = "mfa/backup-codes";

    pub const ADMIN: &str = "admin";
    pub const ADMIN_USERS: &str = "admin/users";
    pub const ADMIN_REALMS: &str = "admin/realms";
    pub const ADMIN_CLIENTS: &str = "admin/clients";
    pub const ADMIN_ROLES: &str = "admin/roles";
    pub const ADMIN_FEDERATION: &str = "admin/federation";
    pub const ADMIN_PERMISSIONS: &str = "admin/permissions";
    pub const ADMIN_AUDIT: &str = "admin/audit";
    pub const ADMIN_GROUPS: &str = "admin/groups";
    pub const ADMIN_REALM_SETTINGS: &str = "admin/realm-settings";
    pub const ADMIN_AUTH_FLOWS: &str = "admin/auth-flows";
    pub const ADMIN_LINKED_ACCOUNTS: &str = "admin/linked-accounts";
}

pub mod path {
    pub const DASHBOARD: &str = "/portal/dashboard";
    pub const APPS: &str = "/portal/apps";
    pub const NOTIFICATIONS: &str = "/portal/notifications";
    pub const SETTINGS: &str = "/portal/settings";

    pub const PROFILE: &str = "/portal/profile";
    pub const PASSKEYS: &str = "/portal/passkeys";
    pub const PASSWORD: &str = "/portal/password";
    pub const SESSIONS: &str = "/portal/sessions";

    pub const ADMIN: &str = "/portal/admin";
    pub const ADMIN_USERS: &str = "/portal/admin/users";
    pub const ADMIN_REALMS: &str = "/portal/admin/realms";
    pub const ADMIN_CLIENTS: &str = "/portal/admin/clients";
    pub const ADMIN_ROLES: &str = "/portal/admin/roles";
    pub const ADMIN_FEDERATION: &str = "/portal/admin/federation";
    pub const ADMIN_PERMISSIONS: &str = "/portal/admin/permissions";
    pub const ADMIN_AUDIT: &str = "/portal/admin/audit";
    pub const ADMIN_GROUPS: &str = "/portal/admin/groups";
    pub const ADMIN_REALM_SETTINGS: &str = "/portal/admin/realm-settings";
    pub const ADMIN_AUTH_FLOWS: &str = "/portal/admin/auth-flows";
    pub const ADMIN_LINKED_ACCOUNTS: &str = "/portal/admin/linked-accounts";
}
