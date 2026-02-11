//! Riwayat Pemenuhan domain models

use chrono::{DateTime, NaiveDate, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[cfg(feature = "backend")]
use tokio_postgres::Row;

/// Riwayat Pemenuhan model - tracks fulfillment history
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RiwayatPemenuhan {
    pub id: Uuid,
    pub kebutuhan_bmn_id: Option<Uuid>,
    pub roadmap_id: Option<Uuid>,
    pub satker_id: Uuid,
    pub tahun_anggaran: i32,
    pub kode_barang: String,
    pub nama_barang: String,
    pub jumlah_terpenuhi: i32,
    pub sumber_data: String, // SIMAN, HIBAH, PNBP
    pub tanggal_pemenuhan: NaiveDate,
    pub nilai_perolehan: Option<f64>,
    pub keterangan: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

#[cfg(feature = "backend")]
impl RiwayatPemenuhan {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            kebutuhan_bmn_id: row.get("kebutuhan_bmn_id"),
            roadmap_id: row.get("roadmap_id"),
            satker_id: row.get("satker_id"),
            tahun_anggaran: row.get("tahun_anggaran"),
            kode_barang: row.get("kode_barang"),
            nama_barang: row.get("nama_barang"),
            jumlah_terpenuhi: row.get("jumlah_terpenuhi"),
            sumber_data: row.get("sumber_data"),
            tanggal_pemenuhan: row.get("tanggal_pemenuhan"),
            nilai_perolehan: row.get("nilai_perolehan"),
            keterangan: row.get("keterangan"),
            created_at: row.get("created_at"),
            created_by: row.get("created_by"),
        }
    }
}

/// Request to create Riwayat Pemenuhan
#[derive(Debug, Clone, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreateRiwayatPemenuhanRequest {
    pub kebutuhan_bmn_id: Option<Uuid>,
    pub roadmap_id: Option<Uuid>,
    pub satker_id: Uuid,
    pub tahun_anggaran: i32,
    #[validate(length(min = 1, max = 50))]
    pub kode_barang: String,
    #[validate(length(min = 1, max = 255))]
    pub nama_barang: String,
    #[validate(range(min = 1))]
    pub jumlah_terpenuhi: i32,
    #[validate(length(min = 1, max = 50))]
    pub sumber_data: String,
    pub tanggal_pemenuhan: NaiveDate,
    pub nilai_perolehan: Option<f64>,
    pub keterangan: Option<String>,
}

/// Aggregated fulfillment statistics
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FulfillmentStatistics {
    pub satker_id: Uuid,
    pub tahun_anggaran: i32,
    pub total_kebutuhan: i32,
    pub total_terpenuhi: i32,
    pub persentase_pemenuhan: f64,
    pub total_nilai: f64,
    pub by_sumber: Vec<FulfillmentBySumber>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FulfillmentBySumber {
    pub sumber_data: String,
    pub jumlah: i32,
    pub nilai: f64,
}

#[cfg(feature = "backend")]
impl FulfillmentBySumber {
    pub fn from_row(row: &Row) -> Self {
        Self {
            sumber_data: row.get("sumber_data"),
            jumlah: row.get("jumlah"),
            nilai: row.get("nilai"),
        }
    }
}
