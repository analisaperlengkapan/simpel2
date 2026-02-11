//! # Roadmap Sarpras Module
//!
//! Module for managing 5-year infrastructure roadmap (Roadmap Sarana Prasarana).
//!
//! ## Features
//! - 5-year period planning with validation
//! - Multi-year infrastructure planning per satker
//! - Realization tracking against planned targets
//! - Roadmap vs realization comparison and analysis
//! - Progress monitoring and reporting
//!
//! ## Workflow
//! 1. Create Roadmap (Admin Pusat/Wilayah) → Define 5-year plan with yearly targets
//! 2. Track Realization (System) → Sync fulfillment data from MonSAKTI
//! 3. Compare Progress (Admin/Pimpinan) → Analyze roadmap vs actual realization
//! 4. Generate Reports (Authorized users) → Export roadmap analysis
//!
//! ## Requirements
//! - REQ-K008: 5-year roadmap sarpras feature
//! - REQ-DB003: Roadmap vs realization visualization

pub mod handlers;
pub mod models;
pub mod repository;
pub mod services;

pub use handlers::*;
pub use models::*;
pub use repository::*;
pub use services::*;
