//! Pages module — all page components
//!
//! Removes password_reset (admin-only in government context).

pub mod admin;
pub mod apps;
pub mod dashboard;
#[path = "portal_dashboard.rs"]
pub mod dashboard_page;
pub mod home;
pub mod not_found;
pub mod profile;
pub mod settings;

pub use admin::*;
pub use apps::*;
pub use dashboard::DashboardPage;
pub use dashboard_page::PortalDashboardPage;
pub use home::*;
pub use not_found::*;
pub use profile::*;
pub use settings::SettingsPage;
