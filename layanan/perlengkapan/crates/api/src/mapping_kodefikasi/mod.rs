//! # Mapping Kodefikasi Module
//!
//! Simplified read-only module for BMN code classification.
//! Lists standard and non-standard kode barang with CSV export.
//!
//! ## Features
//! - Read-only standard BMN code listing
//! - Non-standard code detection from SIMAN integration
//! - Fuzzy matching suggestions
//! - CSV export
//! - Progress statistics by satker

pub mod handlers;
pub mod models;
pub mod repository;
pub mod services;

// Re-export handlers for convenience
pub use handlers::*;
pub use models::*;
pub use repository::*;
pub use services::*;
