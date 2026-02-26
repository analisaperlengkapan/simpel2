//! Kebutuhan BMN domain models

use chrono::{DateTime, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[cfg(feature = "backend")]
use tokio_postgres::Row;

/// Kebutuhan BMN model - represents BMN requirements per satker per year
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KebutuhanBmn {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub kode_barang: String,
    pub nama_barang: String,
    pub jumlah_kebutuhan: i32,
    pub jumlah_existing_baik: i32,
    pub gap: i32,
    pub tahun_anggaran: i32,
    pub status: String,
    pub prioritas: Option<String>,
    pub justifikasi: Option<String>,
    pub estimasi_harga_satuan: Option<f64>,
    pub estimasi_total: Option<f64>,
    pub is_sbsk: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[cfg(feature = "backend")]
impl KebutuhanBmn {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            satker_id: row.get("satker_id"),
            kode_barang: row.get("kode_barang"),
            nama_barang: row.get("nama_barang"),
            jumlah_kebutuhan: row.get("jumlah_kebutuhan"),
            jumlah_existing_baik: row.get("jumlah_existing_baik"),
            gap: row.get("gap"),
            tahun_anggaran: row.get("tahun_anggaran"),
            status: row.get("status"),
            prioritas: row.get("prioritas"),
            justifikasi: row.get("justifikasi"),
            estimasi_harga_satuan: row.get("estimasi_harga_satuan"),
            estimasi_total: row.get("estimasi_total"),
            is_sbsk: row.get("is_sbsk"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        }
    }
}

/// Request to create a new Kebutuhan BMN
#[derive(Debug, Clone, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreateKebutuhanBmnRequest {
    pub satker_id: Uuid,
    #[validate(length(min = 1, max = 50))]
    pub kode_barang: String,
    #[validate(length(min = 1, max = 255))]
    pub nama_barang: String,
    #[validate(range(min = 1))]
    pub jumlah_kebutuhan: i32,
    pub tahun_anggaran: i32,
    pub justifikasi: Option<String>,
    pub estimasi_harga_satuan: Option<f64>,
}

/// Request to update Kebutuhan BMN
#[derive(Debug, Clone, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UpdateKebutuhanBmnRequest {
    #[validate(range(min = 1))]
    pub jumlah_kebutuhan: Option<i32>,
    pub justifikasi: Option<String>,
    pub estimasi_harga_satuan: Option<f64>,
    pub prioritas: Option<String>,
}
