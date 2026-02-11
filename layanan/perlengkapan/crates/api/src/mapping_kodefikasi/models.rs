//! # Mapping Kodefikasi Models
//!
//! Data models for mapping kodefikasi functionality

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

/// Mapping proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingProposal {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub kode_barang_lama: String,
    pub nama_barang_lama: String,
    pub kode_barang_baru_id: Option<Uuid>,
    pub status_mapping: String,
    pub catatan_mapping: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a mapping proposal
#[derive(Debug, Clone, Deserialize)]
pub struct MappingProposalRequest {
    pub satker_id: Uuid,
    pub kode_lama: String,
    pub nama_lama: String,
    pub kode_baru_id: Uuid,
    pub catatan: Option<String>,
}

/// Request to verify a mapping proposal
#[derive(Debug, Clone, Deserialize)]
pub struct VerifyMappingRequest {
    pub approved: bool,
    pub catatan_verifikasi: Option<String>,
}

/// Mapping progress statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingProgress {
    pub total_non_standard: i64,
    pub total_mapped: i64,
    pub pending_verification: i64,
    pub mapping_percentage: f64,
    pub non_standard_codes: Vec<NonStandardCodeWithStatus>,
}

/// Non-standard code with mapping status
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

/// Mapping progress by satker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingProgressBySatker {
    pub satker_id: Uuid,
    pub satker_nama: String,
    pub total_non_standard: i64,
    pub total_mapped: i64,
    pub pending_verification: i64,
    pub mapping_percentage: f64,
}

/// Mapping progress by wilayah
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingProgressByWilayah {
    pub wilayah_id: Uuid,
    pub wilayah_nama: String,
    pub total_non_standard: i64,
    pub total_mapped: i64,
    pub pending_verification: i64,
    pub mapping_percentage: f64,
    pub satkers: Vec<MappingProgressBySatker>,
}
