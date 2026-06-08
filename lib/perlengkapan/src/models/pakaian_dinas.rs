//! Pakaian Dinas domain models

use chrono::{DateTime, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

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
