//! # Kebutuhan BMN HTTP Handlers
//!
//! Request handlers for BMN needs analysis REST API.
//! All endpoints require authentication via JWT middleware.

pub mod analisis;
pub mod barang;
pub mod batch;
pub mod dashboard;
pub mod laporan;
pub mod params;
pub mod pengajuan;
pub mod satker;
pub mod search;
pub mod siman;
pub mod workflow;

pub use analisis::*;
pub use barang::*;
pub use batch::*;
pub use dashboard::*;
pub use laporan::*;
pub use params::*;
pub use pengajuan::*;
pub use satker::*;
pub use search::*;
pub use siman::*;
pub use workflow::*;

#[cfg(test)]
mod tests;
