//! # Export Feature
//!
//! Generic Excel export for several read models (kebutuhan_bmn, pakaian_dinas,
//! roadmap_sarpras, riwayat_pemenuhan). Small datasets stream synchronously;
//! large ones queue an async job tracked in `perlengkapan.export_jobs`.

pub mod handlers;
pub mod models;
pub mod repository;
pub mod services;

pub use repository::ExportRepository;
pub use services::ExportService;

#[cfg(test)]
mod tests;
