//! Pakaian Dinas domain models

use chrono::{DateTime, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[cfg(feature = "backend")]
use tokio_postgres::Row;

/// Pakaian Dinas model - represents uniform requirements
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PakaianDinas {
    pub id: Uuid,
    pub pegawai_nip: String,
    pub pegawai_nama: String,
    pub satker_id: Uuid,
    pub jenis_pakaian: String, // PDH, PDL, etc.
    pub ukuran: String,
    pub jumlah: i32,
    pub tahun_anggaran: i32,
    pub status: String,
    pub tanggal_terakhir_terima: Option<DateTime<Utc>>,
    pub masa_pakai_bulan: i32, // Default: 24 months
    pub eligible_for_new: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[cfg(feature = "backend")]
impl PakaianDinas {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pegawai_nip: row.get("pegawai_nip"),
            pegawai_nama: row.get("pegawai_nama"),
            satker_id: row.get("satker_id"),
            jenis_pakaian: row.get("jenis_pakaian"),
            ukuran: row.get("ukuran"),
            jumlah: row.get("jumlah"),
            tahun_anggaran: row.get("tahun_anggaran"),
            status: row.get("status"),
            tanggal_terakhir_terima: row.get("tanggal_terakhir_terima"),
            masa_pakai_bulan: row.get("masa_pakai_bulan"),
            eligible_for_new: row.get("eligible_for_new"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        }
    }
}

/// Request to create Pakaian Dinas requirement
#[derive(Debug, Clone, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreatePakaianDinasRequest {
    #[validate(length(min = 1, max = 20))]
    pub pegawai_nip: String,
    #[validate(length(min = 1, max = 255))]
    pub pegawai_nama: String,
    pub satker_id: Uuid,
    #[validate(length(min = 1, max = 50))]
    pub jenis_pakaian: String,
    #[validate(length(min = 1, max = 10))]
    pub ukuran: String,
    #[validate(range(min = 1))]
    pub jumlah: i32,
    pub tahun_anggaran: i32,
}

/// Rekapitulasi Pakaian Dinas by type and size
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RekapitulasiPakaianDinas {
    pub jenis_pakaian: String,
    pub ukuran: String,
    pub total_pegawai: i64,
    pub total_jumlah: i64,
    pub satker_id: Option<Uuid>,
}

#[cfg(feature = "backend")]
impl RekapitulasiPakaianDinas {
    pub fn from_row(row: &Row) -> Self {
        Self {
            jenis_pakaian: row.get("jenis_pakaian"),
            ukuran: row.get("ukuran"),
            total_pegawai: row.get("total_pegawai"),
            total_jumlah: row.get("total_jumlah"),
            satker_id: row.get("satker_id"),
        }
    }
}
