//! # Mapping Kodefikasi Models
//!
//! Data models for simplified mapping kodefikasi — read-only standard/non-standard
//! BMN code listing with export capabilities (no proposal/verification workflow)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Non-standard code detected from MonSAKTI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonStandardCode {
    pub kode_lama: String,
    pub nama_lama: String,
    pub satker_id: Uuid,
    pub jumlah_aset: i64,
    pub suggested_mapping: Option<MappingSuggestion>,
}

/// Mapping suggestion based on fuzzy matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingSuggestion {
    pub barang_id: Uuid,
    pub kode_baru: String,
    pub nama_baru: String,
    pub similarity_score: f64,
}

/// Standard BMN code from master table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardBmnCode {
    pub id: Uuid,
    pub kode: String,
    pub nama: String,
    pub kategori: Option<String>,
    pub jumlah_aset: i64,
}

/// Non-standard code with mapping status (read-only view)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonStandardCodeWithStatus {
    pub kode_lama: String,
    pub nama_lama: String,
    pub satker_id: Uuid,
    pub satker_nama: String,
    pub jumlah_aset: i64,
    pub status_mapping: Option<String>,
    pub kode_baru: Option<String>,
    pub nama_baru: Option<String>,
}

/// Mapping progress statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingProgress {
    pub total_non_standard: i64,
    pub total_mapped: i64,
    pub total_standard: i64,
    pub mapping_percentage: f64,
    pub non_standard_codes: Vec<NonStandardCodeWithStatus>,
}

/// Mapping progress by satker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingProgressBySatker {
    pub satker_id: Uuid,
    pub satker_nama: String,
    pub total_non_standard: i64,
    pub total_mapped: i64,
    pub mapping_percentage: f64,
}

/// Query parameters for listing
#[derive(Debug, Deserialize)]
pub struct MappingListQuery {
    pub search: Option<String>,
    pub satker_id: Option<Uuid>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

/// Export format
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>, // "xlsx" or "csv"
    pub satker_id: Option<Uuid>,
}

/// Paginated response for mapping list
#[derive(Debug, Serialize)]
pub struct MappingListResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}
