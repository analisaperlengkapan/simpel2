//! lib-core: WASM-safe core types and utilities for SIMPEL
//!
//! This crate provides foundational types shared between frontend (WASM)
//! and backend services. All types in this crate are WASM-compatible.

pub mod audit;
pub mod auth;
pub mod config;
pub mod context;
pub mod correlation;
pub mod encoding;
pub mod error;
pub mod health;
pub mod jwt_claims;
pub mod models;
pub mod sanitizer;
pub mod validation;

pub use error::CommonError;
