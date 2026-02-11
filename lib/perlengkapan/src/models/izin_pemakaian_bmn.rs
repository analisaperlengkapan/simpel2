//! Izin Pemakaian BMN domain models

use chrono::{DateTime, NaiveDate, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[cfg(feature = "backend")]
use tokio_postgres::Row;

/// Izin Pemakaian BMN model - usage permits for vehicles, housing, laptops
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IzinPemakaianBmn {
    pub id: Uuid,
    pub nomor_izin: String,
    pub bmn_id: Uuid,
    pub bmn_type: String, // VEHICLE, HOUSING, LAPTOP
    pub kode_barang: String,
    pub nama_barang: String,
    pub pegawai_nip: String,
    pub pegawai_nama: String,
    pub satker_id: Uuid,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_akhir: NaiveDate,
    pub status: String, // ACTIVE, EXPIRED, REVOKED, RENEWED
    pub keperluan: Option<String>,
    pub dokumen_pendukung: Option<Vec<String>>, // Array of document URLs
    pub workflow_status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<Uuid>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoke_reason: Option<String>,
    pub parent_izin_id: Option<Uuid>, // For renewals
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[cfg(feature = "backend")]
impl IzinPemakaianBmn {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            nomor_izin: row.get("nomor_izin"),
            bmn_id: row.get("bmn_id"),
            bmn_type: row.get("bmn_type"),
            kode_barang: row.get("kode_barang"),
            nama_barang: row.get("nama_barang"),
            pegawai_nip: row.get("pegawai_nip"),
            pegawai_nama: row.get("pegawai_nama"),
            satker_id: row.get("satker_id"),
            tanggal_mulai: row.get("tanggal_mulai"),
            tanggal_akhir: row.get("tanggal_akhir"),
            status: row.get("status"),
            keperluan: row.get("keperluan"),
            dokumen_pendukung: row.get("dokumen_pendukung"),
            workflow_status: row.get("workflow_status"),
            approved_by: row.get("approved_by"),
            approved_at: row.get("approved_at"),
            revoked_by: row.get("revoked_by"),
            revoked_at: row.get("revoked_at"),
            revoke_reason: row.get("revoke_reason"),
            parent_izin_id: row.get("parent_izin_id"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        }
    }
}

/// Request to create Izin Pemakaian BMN
#[derive(Debug, Clone, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreateIzinPemakaianRequest {
    pub bmn_id: Uuid,
    #[validate(length(min = 1, max = 50))]
    pub bmn_type: String,
    #[validate(length(min = 1, max = 20))]
    pub pegawai_nip: String,
    #[validate(length(min = 1, max = 255))]
    pub pegawai_nama: String,
    pub satker_id: Uuid,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_akhir: NaiveDate,
    pub keperluan: Option<String>,
    pub dokumen_pendukung: Option<Vec<String>>,
}

impl CreateIzinPemakaianRequest {
    /// Validate that tanggal_akhir is after tanggal_mulai
    pub fn validate_dates(&self) -> Result<(), String> {
        if self.tanggal_akhir <= self.tanggal_mulai {
            return Err("Tanggal akhir must be after tanggal mulai".to_string());
        }
        Ok(())
    }
}

/// Request to renew Izin Pemakaian
#[derive(Debug, Clone, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RenewIzinPemakaianRequest {
    pub parent_izin_id: Uuid,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_akhir: NaiveDate,
    pub keperluan: Option<String>,
}

/// Request to revoke Izin Pemakaian
#[derive(Debug, Clone, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RevokeIzinPemakaianRequest {
    pub izin_id: Uuid,
    #[validate(length(min = 1))]
    pub revoke_reason: String,
}

/// Available BMN for usage (no active permit)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AvailableBmn {
    pub bmn_id: Uuid,
    pub bmn_type: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub satker_id: Uuid,
    pub kondisi: String,
    pub has_active_permit: bool,
}

#[cfg(feature = "backend")]
impl AvailableBmn {
    pub fn from_row(row: &Row) -> Self {
        Self {
            bmn_id: row.get("bmn_id"),
            bmn_type: row.get("bmn_type"),
            kode_barang: row.get("kode_barang"),
            nama_barang: row.get("nama_barang"),
            satker_id: row.get("satker_id"),
            kondisi: row.get("kondisi"),
            has_active_permit: row.get("has_active_permit"),
        }
    }
}

/// Usage history per BMN
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BmnUsageHistory {
    pub bmn_id: Uuid,
    pub kode_barang: String,
    pub nama_barang: String,
    pub total_permits: i64,
    pub current_user: Option<String>,
    pub current_permit_start: Option<NaiveDate>,
    pub current_permit_end: Option<NaiveDate>,
    pub history: Vec<IzinPemakaianBmn>,
}

/// Usage history per Pegawai
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PegawaiUsageHistory {
    pub pegawai_nip: String,
    pub pegawai_nama: String,
    pub total_permits: i64,
    pub active_permits: i64,
    pub history: Vec<IzinPemakaianBmn>,
}

/// BMN utilization report
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BmnUtilizationReport {
    pub satker_id: Uuid,
    pub bmn_type: String,
    pub total_bmn: i64,
    pub bmn_with_active_permit: i64,
    pub bmn_available: i64,
    pub utilization_rate: f64,
}

#[cfg(feature = "backend")]
impl BmnUtilizationReport {
    pub fn from_row(row: &Row) -> Self {
        let total_bmn: i64 = row.get("total_bmn");
        let bmn_with_active_permit: i64 = row.get("bmn_with_active_permit");
        let utilization_rate = if total_bmn > 0 {
            (bmn_with_active_permit as f64 / total_bmn as f64) * 100.0
        } else {
            0.0
        };

        Self {
            satker_id: row.get("satker_id"),
            bmn_type: row.get("bmn_type"),
            total_bmn,
            bmn_with_active_permit,
            bmn_available: total_bmn - bmn_with_active_permit,
            utilization_rate,
        }
    }
}
