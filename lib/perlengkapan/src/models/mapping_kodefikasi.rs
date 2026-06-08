//! Mapping Kodefikasi domain models
//!
//! Simplified read-only models for standard/non-standard BMN code classification.
//! No proposal/verification workflow — admin inputs reference codes, system detects mismatches.

use chrono::{DateTime, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub status_mapping: String,
    pub catatan_mapping: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Result of checking whether a BMN code is standard or non-standard
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BmnStandardCheckResult {
    /// NUP (Nomor Urut Pendaftaran) of the asset
    pub nup: Option<String>,
    /// The kode barang currently assigned to this asset
    pub kode_barang: String,
    /// Nama barang from asset data
    pub nama_barang: String,
    /// Whether this code exists in the ms_barang reference table
    pub is_standard: bool,
    /// If non-standard, the suggested standard code (from fuzzy match)
    pub kode_standar_rujukan: Option<String>,
    /// Nama of the suggested standard code
    pub nama_standar_rujukan: Option<String>,
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
