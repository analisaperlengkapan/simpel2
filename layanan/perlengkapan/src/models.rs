//! # Data Models for Perlengkapan Service

use serde::{Deserialize, Serialize};

// ============ Dashboard Models ============

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_aset: i64,
    pub total_nilai_aset: f64,
    pub total_satker: i64,
    pub aset_baik: i64,
    pub aset_rusak: i64,
    pub categories: Vec<CategoryStat>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryStat {
    pub category: String,
    pub count: i64,
    pub value: f64,
}

// ============ Response Models ============
//
// `ApiResponse<T>` + `PaginatedResponse<T>` live in `lib_perlengkapan` so
// the frontend (Leptos WASM) and the backend share one definition + one
// wire format. The constructors are inherent impls on the lib types, so
// callers keep using `ApiResponse::success(...)` / `PaginatedResponse::new(...)`
// unchanged after the re-export.

pub use lib_perlengkapan::response::{ApiResponse, PaginatedResponse};
