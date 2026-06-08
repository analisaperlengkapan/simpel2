//! # lib-perlengkapan
//!
//! Shared library for the Perlengkapan domain, providing common types, models,
//! and validation logic for both backend services and the frontend microfrontend.
//!
//! ## Features
//!
//! - `serde` - Enables serialization (enabled by default).
//! - `backend` - Enables backend-specific functionality (tokio-postgres row conversion).
//! - `frontend` - Enables frontend-specific functionality.
//! - `wasm` - Enables WASM bindings for browser usage.
//!
//! The cross-module service trait contracts (`DocumentGenerator`,
//! `NotificationSender`, `AuditSink`, `DocumentStorage`) used to live here
//! behind a `contracts` feature; as of F0-C they live in the service crate
//! (`layanan/perlengkapan/src/contracts.rs`), their only implementor, so this
//! crate stays WASM-safe (no `bytes` / `async-trait` from contracts).
//!
//! ## Usage
//!
//! ```toml
//! # Backend service
//! lib-perlengkapan = { workspace = true, features = ["backend"] }
//!
//! # Frontend (WASM)
//! lib-perlengkapan = { workspace = true, features = ["frontend", "wasm"] }
//! ```

pub mod audit;
pub mod error;
pub mod gap_analysis;
pub mod kode_barang;
pub mod models;
pub mod pagination;
pub mod prioritization;
pub mod response;
pub mod search;
pub mod types;
pub mod utils;
pub mod validation;

// Re-export commonly used types at crate root
pub use audit::{AuditAction, AuditEvent};
pub use error::{ServiceError, ServiceResult};
pub use gap_analysis::*;
pub use kode_barang::*;
pub use models::*;
pub use pagination::{DEFAULT_PER_PAGE, MAX_PER_PAGE, PageParams, PageResult};
pub use prioritization::*;
pub use response::{ApiResponse, ErrorBody, ErrorDetail, PageMeta, PaginatedResponse};
pub use search::*;
pub use types::*;
