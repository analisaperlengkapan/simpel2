//! Mapping Kodefikasi domain models

use chrono::{DateTime, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[cfg(feature = "backend")]
use tokio_postgres::Row;

/// Mapping Kodefikasi model - maps non-standard kode barang to standard
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MappingKodefikasi {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub kode_barang_lama: String,
    pub nama_barang_lama: String,
    pub kode_barang_baru_id: Option<Uuid>,
    pub kode_barang_baru: Option<String>,
    pub nama_barang_baru: Option<String>,
    pub status_mapping: String, // PROPOSED, VERIFIED, REJECTED
    pub catatan_mapping: Option<String>,
    pub proposed_by: Option<Uuid>,
    pub verified_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(feature = "backend")]
impl MappingKodefikasi {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            satker_id: row.get("satker_id"),
            kode_barang_lama: row.get("kode_barang_lama"),
            nama_barang_lama: row.get("nama_barang_lama"),
            kode_barang_baru_id: row.get("kode_barang_baru_id"),
            kode_barang_baru: row.get("kode_barang_baru"),
            nama_barang_baru: row.get("nama_barang_baru"),
            status_mapping: row.get("status_mapping"),
            catatan_mapping: row.get("catatan_mapping"),
            proposed_by: row.get("proposed_by"),
            verified_by: row.get("verified_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

/// Request to propose a kode barang mapping
#[derive(Debug, Clone, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProposeMappingRequest {
    pub satker_id: Uuid,
    #[validate(length(min = 1, max = 50))]
    pub kode_barang_lama: String,
    #[validate(length(min = 1, max = 255))]
    pub nama_barang_lama: String,
    pub kode_barang_baru_id: Uuid,
    pub catatan_mapping: Option<String>,
}

/// Request to verify a mapping
#[derive(Debug, Clone, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VerifyMappingRequest {
    pub mapping_id: Uuid,
    #[validate(length(min = 1, max = 50))]
    pub status_mapping: String, // VERIFIED or REJECTED
    pub catatan_mapping: Option<String>,
}

/// Mapping progress statistics
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MappingProgress {
    pub satker_id: Option<Uuid>,
    pub total_non_standard: i64,
    pub total_proposed: i64,
    pub total_verified: i64,
    pub total_rejected: i64,
    pub persentase_mapped: f64,
}

#[cfg(feature = "backend")]
impl MappingProgress {
    pub fn from_row(row: &Row) -> Self {
        let total_non_standard: i64 = row.get("total_non_standard");
        let total_verified: i64 = row.get("total_verified");
        let persentase_mapped = if total_non_standard > 0 {
            (total_verified as f64 / total_non_standard as f64) * 100.0
        } else {
            0.0
        };

        Self {
            satker_id: row.get("satker_id"),
            total_non_standard,
            total_proposed: row.get("total_proposed"),
            total_verified,
            total_rejected: row.get("total_rejected"),
            persentase_mapped,
        }
    }
}

/// Auto-detected non-standard kode barang
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NonStandardKodeBarang {
    pub kode_barang: String,
    pub nama_barang: String,
    pub satker_id: Uuid,
    pub source: String, // SIMAN, MANUAL_INPUT
    pub detected_at: DateTime<Utc>,
    pub is_mapped: bool,
}

#[cfg(feature = "backend")]
impl NonStandardKodeBarang {
    pub fn from_row(row: &Row) -> Self {
        Self {
            kode_barang: row.get("kode_barang"),
            nama_barang: row.get("nama_barang"),
            satker_id: row.get("satker_id"),
            source: row.get("source"),
            detected_at: row.get("detected_at"),
            is_mapped: row.get("is_mapped"),
        }
    }
}
