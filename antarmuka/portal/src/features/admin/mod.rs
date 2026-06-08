//! IAM Admin Pages Module
//!
//! Administrative pages for Identity & Access Management.
//! All pages protected by AdminRoute guard.

pub mod audit;
pub mod auth_flows;
pub mod client_detail;
pub mod clients;
pub mod federation;
pub mod groups;
pub mod linked_accounts;
pub mod overview;
pub mod permissions;
pub mod realm_settings;
pub mod realms;
pub mod roles;
pub mod user_detail;
pub mod users;

pub use audit::AuditLogsPage;
pub use auth_flows::AuthFlowsPage;
pub use client_detail::ClientDetailPage;
pub use clients::ClientsManagementPage;
pub use federation::FederationManagementPage;
pub use groups::GroupsManagementPage;
pub use linked_accounts::LinkedAccountsPage;
pub use overview::AdminOverviewPage;
pub use permissions::PermissionsManagementPage;
pub use realm_settings::RealmSettingsPage;
pub use realms::RealmsManagementPage;
pub use roles::RolesManagementPage;
pub use user_detail::UserDetailPage;
pub use users::UsersManagementPage;
