//! # Mapping Kodefikasi Module
//!
//! This module handles the mapping of non-standard kode barang from MonSAKTI
//! to standard codes in SIMPEL.
//!
//! ## Features
//! - Auto-detection of non-standard codes
//! - Mapping proposal submission
//! - Mapping verification workflow
//! - Progress dashboard

pub mod handlers;
pub mod models;
pub mod repository;
pub mod services;

// Re-export handlers for convenience
pub use handlers::*;
pub use models::*;
pub use repository::*;
pub use services::*;
