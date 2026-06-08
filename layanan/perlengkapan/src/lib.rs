// ── Domain modules (`semua setara`) ──────────────────────────────────
pub mod admin;
pub mod analisis;
pub mod audit;
pub mod bank_aset;
pub mod bantuan;
pub mod contracts;
pub mod dashboard;
pub mod dokumen;
pub mod export;
pub mod kebutuhan_bmn;
pub mod mapping_kodefikasi;
pub mod notifikasi;
pub mod pakaian_dinas;
pub mod pemakaian_bmn;
pub mod penghapusan_bmn;
pub mod roadmap_sarpras;
pub mod workflow;

// ── Cross-cutting infrastructure (per plan A.1 / A.3) ────────────────
pub mod shared;

// ── HTTP layer wiring + bootstrap helpers ────────────────────────────
pub mod migrations;
pub mod routes;
pub mod state;

pub use state::AppState;

#[cfg(test)]
pub mod tests;
