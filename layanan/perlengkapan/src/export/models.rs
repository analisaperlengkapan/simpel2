//! # Export Models
//!
//! DTOs for the generic Excel export feature (kebutuhan_bmn, pakaian_dinas,
//! roadmap_sarpras, riwayat_pemenuhan). Small datasets stream synchronously;
//! large ones queue an async job tracked in `perlengkapan.export_jobs`.

use serde::Deserialize;
use uuid::Uuid;

/// Export query parameters
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    /// Entity type to export (kebutuhan_bmn, pakaian_dinas, etc.)
    pub entity_type: String,
    /// JSON-encoded filters
    pub filters: Option<String>,
    /// Maximum number of rows to export
    pub limit: Option<u32>,
    /// Tahun anggaran filter
    pub tahun_anggaran: Option<i32>,
    /// Satker filter, as a MySIMKARI `kode_satker`.
    ///
    /// This is a `String`, not a `Uuid`: the only fetcher that applies it
    /// (`fetch_kebutuhan_bmn_for_export`) filters
    /// `pengajuan_kebutuhan_bmn_satker.satker_id`, which is `varchar(20)`
    /// holding a kode — a uuid could never have matched it (same key-type
    /// mismatch as #94).
    pub satker_id: Option<String>,
    /// Status filter
    pub status: Option<String>,
}

/// Export job response
#[derive(Debug, serde::Serialize)]
pub struct ExportJobResponse {
    pub job_id: Uuid,
    pub status: String,
    pub message: String,
}

/// Export job status response
#[derive(Debug, serde::Serialize)]
pub struct ExportJobStatusResponse {
    pub job_id: Uuid,
    pub status: String,
    pub progress: Option<f32>,
    pub document_id: Option<Uuid>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}
