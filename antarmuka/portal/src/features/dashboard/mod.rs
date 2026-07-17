//! Dashboard feature (F0-B): landing dashboard after login.
//!
//! `DashboardPage` renders real data only: profile from `/api/v1/auth/me` and
//! (admin) system stats from `/api/v1/iam/admin/stats`. The former
//! `PortalDashboardPage` fetched `/api/v1/dashboard/portal` — an endpoint no
//! backend ever served — and was deleted in the 2026-07 portal audit.

pub mod page;

pub use page::DashboardPage;
