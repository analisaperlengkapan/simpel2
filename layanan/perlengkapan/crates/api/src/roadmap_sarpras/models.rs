//! Roadmap Sarpras API models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// Re-export from lib-perlengkapan
pub use lib_perlengkapan::models::{
    CreateRoadmapSarprasRequest, RoadmapRealizationComparison, RoadmapSarpras,
    UpdateRoadmapRealizationRequest,
};

/// Request to create multiple roadmap items for a 5-year period
#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct CreateRoadmapBatchRequest {
    pub satker_id: Uuid,
    #[validate(range(min = 2020, max = 2100))]
    pub periode_mulai: i32,
    #[validate(range(min = 2020, max = 2100))]
    pub periode_akhir: i32,
    #[validate(length(min = 1))]
    pub items: Vec<RoadmapItemRequest>,
}

impl CreateRoadmapBatchRequest {
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

/// Individual roadmap item in batch request
#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct RoadmapItemRequest {
    #[validate(length(min = 1, max = 50))]
    pub kode_barang: String,
    #[validate(length(min = 1, max = 255))]
    pub nama_barang: String,
    #[validate(range(min = 2020, max = 2100))]
    pub tahun_rencana: i32,
    #[validate(range(min = 1))]
    pub jumlah_kebutuhan: i32,
    pub estimasi_anggaran: Option<f64>,
    pub keterangan: Option<String>,
}

/// Response for roadmap creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoadmapResponse {
    pub roadmap_ids: Vec<Uuid>,
    pub message: String,
}

/// Query parameters for listing roadmaps
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct ListRoadmapQuery {
    pub satker_id: Option<Uuid>,
    pub periode_mulai: Option<i32>,
    pub periode_akhir: Option<i32>,
    pub tahun_rencana: Option<i32>,
    pub kode_barang: Option<String>,
    #[validate(range(min = 1, max = 1000))]
    pub limit: Option<i64>,
    #[validate(range(min = 0))]
    pub offset: Option<i64>,
}

/// Response for roadmap list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRoadmapResponse {
    pub roadmaps: Vec<RoadmapSarpras>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Query parameters for roadmap vs realization comparison
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct RoadmapComparisonQuery {
    pub satker_id: Uuid,
    #[validate(range(min = 2020, max = 2100))]
    pub periode_mulai: i32,
    #[validate(range(min = 2020, max = 2100))]
    pub periode_akhir: i32,
}

/// Response for roadmap vs realization comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapComparisonResponse {
    pub comparisons: Vec<RoadmapRealizationComparison>,
    pub summary: RoadmapSummary,
}

/// Summary statistics for roadmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapSummary {
    pub total_items: i64,
    pub total_kebutuhan: i64,
    pub total_terpenuhi: i64,
    pub persentase_pemenuhan_rata_rata: f64,
    pub total_estimasi_anggaran: f64,
    pub total_realisasi_anggaran: f64,
}

/// Request to update roadmap realization (from MonSAKTI sync)
#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct SyncRealizationRequest {
    pub roadmap_id: Uuid,
    #[validate(range(min = 0))]
    pub jumlah_terpenuhi_increment: i32,
    pub realisasi_anggaran_increment: Option<f64>,
}
