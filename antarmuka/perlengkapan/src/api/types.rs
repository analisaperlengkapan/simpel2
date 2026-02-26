//! Shared API response types.

use serde::{Deserialize, Serialize};

/// Standard API response envelope.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

/// Paginated API response envelope.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
    pub message: String,
}

/// Dashboard statistics from `GET /dashboard/stats`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_aset: i64,
    pub total_nilai_aset: f64,
    pub total_satker: i64,
    pub aset_baik: i64,
    pub aset_rusak: i64,
    pub categories: Vec<CategoryStat>,
}

/// Per-category statistic.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryStat {
    pub category: String,
    pub count: i64,
    pub value: f64,
}

/// Asset entity from the backend.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Asset {
    pub id: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub nup: String,
    pub kondisi: String,
    pub satker_id: String,
    pub satker_nama: String,
    pub nilai_perolehan: Option<f64>,
    pub tahun_perolehan: Option<i32>,
    pub lokasi: Option<String>,
    pub status_penggunaan: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
