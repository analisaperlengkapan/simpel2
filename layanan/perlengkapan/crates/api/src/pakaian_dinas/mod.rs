//! # Pakaian Dinas Module
//!
//! Module for managing official uniforms (Pakaian Dinas) for government employees.
//! Migrated from simpel_laravel with full Rust implementation.
//!
//! ## Features
//! - Master data management (jenis, spesifikasi, subspesifikasi, ukuran)
//! - Request management (pengajuan) with multi-level approval workflow
//! - Employee size tracking integrated with MySIMKARI
//! - Report generation for uniform distribution

pub mod export;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod services;

pub use handlers::*;
pub use models::*;
pub use repository::*;
pub use services::*;
