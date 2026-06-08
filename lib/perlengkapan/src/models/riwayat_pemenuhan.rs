//! Riwayat Pemenuhan domain models

use chrono::{DateTime, NaiveDate, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

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
