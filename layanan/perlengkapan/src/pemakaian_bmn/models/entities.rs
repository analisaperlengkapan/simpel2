use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Main Entity: Izin Pemakaian BMN
// ============================================================================

/// Main entity for BMN usage permit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IzinPemakaianBmn {
    pub id: Uuid,
    pub nomor_izin: Option<String>,

    // Pegawai Information
    pub pegawai_nip: String,
    pub pegawai_nama: String,
    pub pegawai_satker_id: Uuid,
    pub pegawai_satker_nama: String,
    pub pegawai_jabatan: Option<String>,
    pub pegawai_golongan: Option<String>,
    pub pegawai_pangkat: Option<String>,
    pub pegawai_unit_kerja: Option<String>,
    pub foto_pegawai: Option<String>,

    // BMN Information (primary, kept for backward compat)
    pub jenis_bmn: String,
    pub bmn_nup: String,
    pub bmn_kode_barang: String,
    pub bmn_nama_barang: String,
    pub bmn_merk: Option<String>,
    pub bmn_tahun_perolehan: Option<i32>,

    // Vehicle-specific fields
    pub no_polisi: Option<String>,
    pub no_bpkb: Option<String>,
    pub no_stnk: Option<String>,
    pub no_rangka: Option<String>,
    pub no_mesin: Option<String>,

    // Housing-specific fields
    pub alamat: Option<String>,
    pub luas_tanah: Option<f64>,
    pub luas_bangunan: Option<f64>,

    // Laptop-specific fields
    pub serial_number: Option<String>,
    pub spesifikasi: Option<serde_json::Value>,

    // Permit Details
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
    pub keperluan: String,
    pub lokasi_pemakaian: Option<String>,

    // Renewal
    pub is_renewal: bool,
    pub previous_permit_id: Option<Uuid>,

    // Supporting Documents
    pub file_pendukung: Option<serde_json::Value>,

    // Document Generation & Upload
    pub document_id: Option<Uuid>,
    pub document_url: Option<String>,
    pub konsep_surat_url: Option<String>, // Generated DOCX concept (editable)
    pub konsep_surat_generated_at: Option<DateTime<Utc>>,
    pub konsep_surat_pdf_url: Option<String>, // Generated PDF concept (final, side-by-side with DOCX)
    pub konsep_surat_pdf_generated_at: Option<DateTime<Utc>>,
    pub signed_pdf_url: Option<String>, // Uploaded signed PDF
    pub signed_pdf_uploaded_at: Option<DateTime<Utc>>,
    pub is_completed: bool,

    // Workflow Status
    pub status: String,
    pub catatan_approval: Option<String>,
    pub catatan_revocation: Option<String>,

    // Approval Information (legacy field — Pimpinan langsung)
    pub approved_by: Option<Uuid>,
    pub approved_by_nama: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,

    // V035 (Fase 1.5): Internal-satker 3-step approval audit fields
    pub validator_satker_id: Option<Uuid>,
    pub validator_satker_nama: Option<String>,
    pub tanggal_validasi_satker: Option<DateTime<Utc>>,
    pub catatan_validator_satker: Option<String>,
    pub approver_satker_id: Option<Uuid>,
    pub approver_satker_nama: Option<String>,
    pub tanggal_approval_satker: Option<DateTime<Utc>>,
    pub catatan_approver_satker: Option<String>,
    pub version: i32,

    // Revocation Information
    pub revoked_by: Option<Uuid>,
    pub revoked_by_nama: Option<String>,
    pub revoked_at: Option<DateTime<Utc>>,

    // Audit Fields
    // Nullable in the schema (`izin_pemakaian_bmn.created_by/created_by_nama`),
    // so these must be Option: decoding a legally-NULL row into `Uuid`/`String`
    // panics, and the service builds with panic=abort — one such row killed the
    // whole process mid-suite.
    pub created_by: Option<Uuid>,
    pub created_by_nama: Option<String>,
    pub updated_by: Option<Uuid>,
    pub updated_by_nama: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Multi-BMN items (loaded separately)
    #[serde(default)]
    pub bmn_items: Vec<PemakaianBmnItem>,
}
// ============================================================================
// Entity: Pemakaian BMN Item (Multi-BMN per permit)
// ============================================================================

/// Individual BMN item in a usage permit (supports multiple BMN per pegawai)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PemakaianBmnItem {
    pub id: Uuid,
    pub izin_pemakaian_id: Uuid,
    pub bmn_nup: String,
    pub bmn_kode_barang: String,
    pub bmn_nama_barang: String,
    pub bmn_merk: Option<String>,
    pub bmn_tahun_perolehan: Option<i32>,
    pub bmn_kondisi: Option<String>,
    pub detail_bmn: serde_json::Value, // Type-specific details
    pub keterangan: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl PemakaianBmnItem {
    pub fn from_row(row: &tokio_postgres::Row) -> Self {
        Self {
            id: row.get("id"),
            izin_pemakaian_id: row.get("izin_pemakaian_id"),
            bmn_nup: row.get("bmn_nup"),
            bmn_kode_barang: row.get("bmn_kode_barang"),
            bmn_nama_barang: row.get("bmn_nama_barang"),
            bmn_merk: row.try_get("bmn_merk").ok().flatten(),
            bmn_tahun_perolehan: row.try_get("bmn_tahun_perolehan").ok().flatten(),
            bmn_kondisi: row.try_get("bmn_kondisi").ok().flatten(),
            detail_bmn: row.try_get("detail_bmn").unwrap_or(serde_json::json!({})),
            keterangan: row.try_get("keterangan").ok().flatten(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
