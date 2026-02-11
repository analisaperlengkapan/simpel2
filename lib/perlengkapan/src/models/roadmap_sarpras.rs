//! Roadmap Sarpras domain models

use chrono::{DateTime, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[cfg(feature = "backend")]
use tokio_postgres::Row;

/// Roadmap Sarpras model - 5-year infrastructure roadmap
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RoadmapSarpras {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub periode_mulai: i32,
    pub periode_akhir: i32,
    pub kode_barang: String,
    pub nama_barang: String,
    pub tahun_rencana: i32,
    pub jumlah_kebutuhan: i32,
    pub jumlah_terpenuhi: i32,
    pub estimasi_anggaran: Option<f64>,
    pub realisasi_anggaran: Option<f64>,
    pub status_pemenuhan: String,
    pub keterangan: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

#[cfg(feature = "backend")]
impl RoadmapSarpras {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            satker_id: row.get("satker_id"),
            periode_mulai: row.get("periode_mulai"),
            periode_akhir: row.get("periode_akhir"),
            kode_barang: row.get("kode_barang"),
            nama_barang: row.get("nama_barang"),
            tahun_rencana: row.get("tahun_rencana"),
            jumlah_kebutuhan: row.get("jumlah_kebutuhan"),
            jumlah_terpenuhi: row.get("jumlah_terpenuhi"),
            estimasi_anggaran: row.get("estimasi_anggaran"),
            realisasi_anggaran: row.get("realisasi_anggaran"),
            status_pemenuhan: row.get("status_pemenuhan"),
            keterangan: row.get("keterangan"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
        }
    }
}

/// Request to create Roadmap Sarpras
#[derive(Debug, Clone, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CreateRoadmapSarprasRequest {
    pub satker_id: Uuid,
    pub periode_mulai: i32,
    pub periode_akhir: i32,
    #[validate(length(min = 1, max = 50))]
    pub kode_barang: String,
    #[validate(length(min = 1, max = 255))]
    pub nama_barang: String,
    pub tahun_rencana: i32,
    #[validate(range(min = 1))]
    pub jumlah_kebutuhan: i32,
    pub estimasi_anggaran: Option<f64>,
    pub keterangan: Option<String>,
}

impl CreateRoadmapSarprasRequest {
    /// Validate that tahun_rencana is within periode
    pub fn validate_tahun_in_periode(&self) -> Result<(), String> {
        if self.tahun_rencana < self.periode_mulai || self.tahun_rencana > self.periode_akhir {
            return Err(format!(
                "Tahun rencana {} must be between {} and {}",
                self.tahun_rencana, self.periode_mulai, self.periode_akhir
            ));
        }
        Ok(())
    }

    /// Validate that periode is exactly 5 years
    pub fn validate_periode_duration(&self) -> Result<(), String> {
        let duration = self.periode_akhir - self.periode_mulai + 1;
        if duration != 5 {
            return Err(format!(
                "Periode must be exactly 5 years, got {} years",
                duration
            ));
        }
        Ok(())
    }
}

/// Request to update Roadmap Sarpras realization
#[derive(Debug, Clone, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UpdateRoadmapRealizationRequest {
    #[validate(range(min = 0))]
    pub jumlah_terpenuhi: i32,
    pub realisasi_anggaran: Option<f64>,
    #[validate(length(min = 1, max = 50))]
    pub status_pemenuhan: String,
}

/// Roadmap vs Realization comparison
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RoadmapRealizationComparison {
    pub roadmap_id: Uuid,
    pub satker_id: Uuid,
    pub kode_barang: String,
    pub nama_barang: String,
    pub tahun_rencana: i32,
    pub jumlah_kebutuhan: i32,
    pub jumlah_terpenuhi: i32,
    pub persentase_pemenuhan: f64,
    pub estimasi_anggaran: Option<f64>,
    pub realisasi_anggaran: Option<f64>,
    pub status_pemenuhan: String,
}

#[cfg(feature = "backend")]
impl RoadmapRealizationComparison {
    pub fn from_row(row: &Row) -> Self {
        let jumlah_kebutuhan: i32 = row.get("jumlah_kebutuhan");
        let jumlah_terpenuhi: i32 = row.get("jumlah_terpenuhi");
        let persentase_pemenuhan = if jumlah_kebutuhan > 0 {
            (jumlah_terpenuhi as f64 / jumlah_kebutuhan as f64) * 100.0
        } else {
            0.0
        };

        Self {
            roadmap_id: row.get("id"),
            satker_id: row.get("satker_id"),
            kode_barang: row.get("kode_barang"),
            nama_barang: row.get("nama_barang"),
            tahun_rencana: row.get("tahun_rencana"),
            jumlah_kebutuhan,
            jumlah_terpenuhi,
            persentase_pemenuhan,
            estimasi_anggaran: row.get("estimasi_anggaran"),
            realisasi_anggaran: row.get("realisasi_anggaran"),
            status_pemenuhan: row.get("status_pemenuhan"),
        }
    }
}
