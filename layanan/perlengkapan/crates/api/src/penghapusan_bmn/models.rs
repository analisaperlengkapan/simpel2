// ============================================================================
// Penghapusan BMN Models
// Description: Data models for BMN disposal workflow
// Requirements: REQ-W001
// ============================================================================

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Penghapusan BMN entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenghapusanBmn {
    pub id: Uuid,
    pub satker_id: Uuid,
    pub asset_id: Uuid,
    pub kode_barang: String,
    pub nama_barang: String,
    pub nup: String,
    pub tanggal_penghapusan: NaiveDate,
    pub alasan: String,
    pub metode_penghapusan: String, // DIJUAL, DIHIBAHKAN, DIMUSNAHKAN
    pub nilai_residu: Option<f64>,
    pub status: String, // DRAFT, SUBMITTED, REVIEWED, APPROVED, REJECTED
    pub document_id: Option<Uuid>,
    pub document_url: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create penghapusan BMN request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePenghapusanBmnRequest {
    pub satker_id: Uuid,
    pub asset_id: Uuid,
    pub kode_barang: String,
    #[validate(length(min = 1, max = 255))]
    pub nama_barang: String,
    #[validate(length(min = 1, max = 50))]
    pub nup: String,
    pub tanggal_penghapusan: NaiveDate,
    #[validate(length(min = 10, max = 1000))]
    pub alasan: String,
    #[validate(length(min = 1, max = 100))]
    pub metode_penghapusan: String,
    pub nilai_residu: Option<f64>,
}

/// Update penghapusan BMN request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdatePenghapusanBmnRequest {
    pub tanggal_penghapusan: Option<NaiveDate>,
    #[validate(length(min = 10, max = 1000))]
    pub alasan: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub metode_penghapusan: Option<String>,
    pub nilai_residu: Option<f64>,
}

/// Penghapusan BMN list filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenghapusanBmnFilters {
    pub satker_id: Option<Uuid>,
    pub status: Option<String>,
    pub metode_penghapusan: Option<String>,
    pub tahun: Option<i32>,
}

impl PenghapusanBmn {
    /// Create from database row
    pub fn from_row(row: &tokio_postgres::Row) -> Self {
        Self {
            id: row.get("id"),
            satker_id: row.get("satker_id"),
            asset_id: row.get("asset_id"),
            kode_barang: row.get("kode_barang"),
            nama_barang: row.get("nama_barang"),
            nup: row.get("nup"),
            tanggal_penghapusan: row.get("tanggal_penghapusan"),
            alasan: row.get("alasan"),
            metode_penghapusan: row.get("metode_penghapusan"),
            nilai_residu: row.get("nilai_residu"),
            status: row.get("status"),
            document_id: row.get("document_id"),
            document_url: row.get("document_url"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
