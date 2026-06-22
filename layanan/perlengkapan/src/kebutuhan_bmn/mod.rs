//! # Kebutuhan BMN Module
//!
//! Module for managing BMN (Barang Milik Negara) needs analysis system.
//! Migrated from simpel_laravel with improved architecture and Rust implementation.
//!
//! ## Features
//! - Request management (pengajuan) with multi-level approval workflow
//! - Per-satker goods tracking
//! - Feasibility analysis with inventory comparison (SIMAN integration)
//! - Priority ranking and scoring
//! - Report generation
//!
//! ## Workflow
//! 1. Create Request (Validator Pusat) → Select Satkers + Asset Types
//! 2. Input Goods (Pelaksana Satker) → Add goods + attachments
//! 3. Feasibility Analysis (Validator Pusat) → Compare inventory, decide
//! 4. Priority Ranking (Validator Pusat) → Rank + Score
//! 5. Print Documents (Authorized users) → Generate reports
//!
//! ## SIMAN Integration
//! Uses layanan-integrasi SIMAN API to fetch existing BMN inventory data
//! for comparing requested goods against current stock levels.

pub mod handlers;
pub mod models;
pub mod pdf_laporan;
pub mod repository;
pub mod scope;
pub mod services;
pub mod siman_integration;

#[cfg(test)]
mod tests;

pub use handlers::*;
pub use repository::*;
pub use services::*;
