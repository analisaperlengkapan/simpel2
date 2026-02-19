//! API client for Perlengkapan microfrontend.
//!
//! Setiap sub-modul berisi models + async fetch functions untuk satu domain.
//! Semua tipe dan fungsi di-re-export di sini agar kode lama (`crate::api::X`)
//! tetap bekerja tanpa perubahan pada 37+ file konsumen.

// ── Sub-modules ───────────────────────────────────────────────────────────
mod common;
mod kebutuhan_bmn_api;
mod kebutuhan_bmn_types;
mod pakaian_dinas;
mod pemakaian_bmn;
mod penghapusan_bmn;

// ── Re-exports: common (dashboard, legacy CRUD) ───────────────────────────
pub use common::*;

// ── Re-exports: Kebutuhan BMN ──────────────────────────────────────────────
pub use kebutuhan_bmn_types::*;
pub use kebutuhan_bmn_api::*;

// ── Re-exports: Pakaian Dinas ─────────────────────────────────────────────
pub use pakaian_dinas::*;

// ── Re-exports: Pemakaian BMN ─────────────────────────────────────────────
pub use pemakaian_bmn::*;

// ── Re-exports: Penghapusan BMN ───────────────────────────────────────────
pub use penghapusan_bmn::*;
