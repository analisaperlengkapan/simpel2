//! # lib-perlengkapan
//!
//! Shared library for the Perlengkapan domain, providing common types, models,
//! and validation logic for both backend services and the frontend microfrontend.
//!
//! ## Features
//!
//! - `backend` - Enables backend-specific functionality (tokio-postgres row conversion)
//! - `frontend` - Enables frontend-specific functionality
//! - `wasm` - Enables WASM bindings for browser usage
//! - `serde` - Enables serialization (enabled by default)
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

pub mod gap_analysis;
pub mod kode_barang;
pub mod models;
pub mod prioritization;
pub mod search;
pub mod traits;
pub mod types;
pub mod utils;
pub mod validation;

// Re-export commonly used types at crate root
pub use gap_analysis::*;
pub use kode_barang::*;
pub use models::*;
pub use prioritization::*;
pub use search::*;
pub use types::*;
