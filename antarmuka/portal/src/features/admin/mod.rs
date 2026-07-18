//! IAM Admin Pages Module
//!
//! Administrative pages for Identity & Access Management.
//! All pages protected by the `PortalAdminLayout` parent-route guard.
//!
//! Trimmed with the #45 IAM audit: the Keycloak-mirror pages (realms,
//! realm-settings, federation, groups, auth-flows, permissions,
//! linked-accounts) were deleted — they rendered static tab shells or
//! lists whose backend endpoints were NotImplemented stubs.

pub mod audit;
pub mod client_detail;
pub mod clients;
pub mod overview;
pub mod roles;
pub mod user_detail;
pub mod users;

pub use audit::AuditLogsPage;
pub use client_detail::ClientDetailPage;
pub use clients::ClientsManagementPage;
pub use overview::AdminOverviewPage;
pub use roles::RolesManagementPage;
pub use user_detail::UserDetailPage;
pub use users::UsersManagementPage;
