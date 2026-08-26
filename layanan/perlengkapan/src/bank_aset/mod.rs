//! # Bank Aset
//!
//! Unified Bank Aset module. Read-only façade over `integrasi.siman_aset`
//! that exposes list, detail, dashboard (KPI), sebaran (per-satker count),
//! and last-sync timestamp for the frontend. Does not own write data — the
//! sync is performed asynchronously by `layanan/integrasi`.

pub mod handlers;
pub mod models;
pub mod repository;
pub mod scope;

pub use handlers::*;
pub use models::AsetIdentity;
pub use scope::AsetScope;
