//! Static page-view factories used by route declarations.
//!
//! Keeping these as function pointers avoids repeated inline closures in app routes.

use leptos::prelude::*;

pub fn profile_page() -> AnyView {
    view! { <crate::pages::profile::ProfilePage /> }.into_any()
}

pub fn passkeys_page() -> AnyView {
    view! { <crate::pages::passkeys::PasskeysPage /> }.into_any()
}

pub fn password_change_page() -> AnyView {
    view! { <crate::pages::password_change::PasswordChangePage /> }.into_any()
}

pub fn sessions_page() -> AnyView {
    view! { <crate::pages::sessions::SessionsPage /> }.into_any()
}

pub fn admin_overview_page() -> AnyView {
    view! { <crate::pages::admin::AdminOverviewPage /> }.into_any()
}

pub fn admin_users_page() -> AnyView {
    view! { <crate::pages::admin::UsersManagementPage /> }.into_any()
}

pub fn admin_user_detail_page() -> AnyView {
    view! { <crate::pages::admin::UserDetailPage /> }.into_any()
}

pub fn admin_realms_page() -> AnyView {
    view! { <crate::pages::admin::RealmsManagementPage /> }.into_any()
}

pub fn admin_clients_page() -> AnyView {
    view! { <crate::pages::admin::ClientsManagementPage /> }.into_any()
}

pub fn admin_client_detail_page() -> AnyView {
    view! { <crate::pages::admin::ClientDetailPage /> }.into_any()
}

pub fn admin_roles_page() -> AnyView {
    view! { <crate::pages::admin::RolesManagementPage /> }.into_any()
}

pub fn admin_federation_page() -> AnyView {
    view! { <crate::pages::admin::FederationManagementPage /> }.into_any()
}

pub fn admin_permissions_page() -> AnyView {
    view! { <crate::pages::admin::PermissionsManagementPage /> }.into_any()
}

pub fn admin_audit_page() -> AnyView {
    view! { <crate::pages::admin::AuditLogsPage /> }.into_any()
}

pub fn admin_groups_page() -> AnyView {
    view! { <crate::pages::admin::GroupsManagementPage /> }.into_any()
}

pub fn admin_realm_settings_page() -> AnyView {
    view! { <crate::pages::admin::RealmSettingsPage /> }.into_any()
}

pub fn admin_auth_flows_page() -> AnyView {
    view! { <crate::pages::admin::AuthFlowsPage /> }.into_any()
}

pub fn admin_linked_accounts_page() -> AnyView {
    view! { <crate::pages::admin::LinkedAccountsPage /> }.into_any()
}
