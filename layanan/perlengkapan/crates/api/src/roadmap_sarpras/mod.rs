//! # Predictive Analytics Module (formerly Roadmap Sarpras)
//!
//! Forecasting engine for BMN needs based on historical kebutuhan_bmn data.
//!
//! ## Features
//! - Forecast future BMN needs using SMA, WMA, or Exponential Smoothing
//! - Confidence interval calculation
//! - Historical data aggregation from kebutuhan_bmn table
//! - Forecast snapshots for comparison over time
//! - CSV export of historical + predicted data
//!
//! ## Endpoints (all GET / read-only)
//! - `GET /forecast`          — Generate forecast with configurable method & horizon
//! - `GET /forecast/summary`  — Quick summary statistics
//! - `GET /forecast/compare`  — Compare current forecast with previous snapshots
//! - `GET /forecast/export`   — Download CSV export
//!
//! ## Requirements
//! - REQ-K008: Predictive analytics dashboard for sarpras planning

pub mod handlers;
pub mod models;
pub mod repository;
pub mod services;

pub use handlers::*;
pub use models::*;
pub use repository::*;
pub use services::*;
