pub mod error;
pub use error::AppError;
/// API module — types and fetch functions for all domains.
pub mod admin;
pub mod bank_aset;
pub mod client;
pub mod common;
pub mod dashboard;
pub mod dokumen;
pub mod integrasi;
pub mod kebutuhan_bmn_api;
pub mod notifikasi;
pub mod kebutuhan_bmn_types;
pub mod pakaian_dinas;
pub mod pemakaian_bmn;
pub mod penghapusan_bmn;
pub mod types;
pub mod workflow;

// Re-exports for backward compat (old components use `crate::api::TypeName`)
pub use common::*;
pub use kebutuhan_bmn_api::*;
pub use kebutuhan_bmn_types::*;
pub use pakaian_dinas::*;
pub use pemakaian_bmn::*;
pub use penghapusan_bmn::*;
