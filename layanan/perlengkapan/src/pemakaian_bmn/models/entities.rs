use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::status_tone::StatusTone;

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
    /// MySIMKARI `kode_satker` derived from the creating operator's claims
    /// (#66) — the authoritative satker for RBAC scoping, and half of the
    /// asset identity the booking check keys on. On the row since V003, never
    /// surfaced on the entity until the draft-edit path needed it.
    #[serde(default)]
    pub satker_code: Option<String>,
    pub pegawai_jabatan: Option<String>,
    pub pegawai_golongan: Option<String>,
    pub pegawai_pangkat: Option<String>,
    pub pegawai_unit_kerja: Option<String>,
    pub foto_pegawai: Option<String>,

    // No asset fields here any more (V014). A permit is one pegawai holding a
    // LIST of BMN, and every one of them lives in `bmn_items`. While the first
    // asset sat in these columns and the rest sat in the item table, everything
    // written against the permit only ever learned the first: the SK printed
    // one row, the conflict check could not see the others, and renewal
    // dropped them.

    // Permit Details
    /// Overall window of the permit = MIN over `bmn_items`. DERIVED, not the
    /// authority — each asset states its own period. Kept on the header
    /// because the expiry scheduler, two dozen indexes and every monitoring
    /// query read it; the single writer is `sync_permit_window` (V014).
    pub tanggal_mulai: NaiveDate,
    /// Overall window of the permit = MAX over `bmn_items`. See `tanggal_mulai`.
    pub tanggal_selesai: NaiveDate,
    /// Nullable in the schema (`keperluan text`) and `Option<String>` in the
    /// shared `lib-perlengkapan` model — this entity was the only place
    /// declaring it required, so any row with a NULL `keperluan` (seed and
    /// pre-`V003` rows have them) panicked `row_to_permit` instead of
    /// deserialising.
    pub keperluan: Option<String>,

    // Renewal
    pub is_renewal: bool,
    pub previous_permit_id: Option<Uuid>,
    /// The permit that REPLACED this one. A signed SK is never edited: a
    /// renewal is signed on a different date, so it is issued as a new SK and
    /// this one is revoked as replaced. `previous_permit_id` points backwards,
    /// this points forwards, so "what is in force today" is one hop (V014).
    pub superseded_by_id: Option<Uuid>,
    pub superseded_at: Option<DateTime<Utc>>,

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
    /// Label bahasa Indonesia untuk `status`, diturunkan dari
    /// [`PemakaianBmnStatus::label`](super::PemakaianBmnStatus::label).
    /// Diisi saat baris dibaca; frontend merendernya apa adanya.
    #[serde(default)]
    pub status_label: String,
    /// Kelas semantik badge (`neutral`/`info`/`success`/`warning`/`danger`).
    #[serde(default)]
    pub status_tone: StatusTone,
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
    /// Position within the permit, 1-based. This is what orders the table on
    /// page 2 of the SK, so it is stored rather than derived from `created_at`
    /// — re-saving a permit must not shuffle the letter.
    pub urutan: i32,

    // Identity. (kode satker, kode barang, NUP) is the asset identity; the
    // satker half lives on the header — 44 017 assets carry NUP `1`.
    pub jenis_bmn: String,
    pub bmn_nup: String,
    pub bmn_kode_barang: String,
    pub bmn_nama_barang: String,
    pub bmn_merk: Option<String>,
    pub bmn_tipe: Option<String>,
    pub bmn_tahun_perolehan: Option<i32>,
    pub bmn_kondisi: Option<String>,

    // Vehicle-specific
    pub no_polisi: Option<String>,
    pub no_bpkb: Option<String>,
    pub no_stnk: Option<String>,
    pub no_rangka: Option<String>,
    pub no_mesin: Option<String>,

    // Housing-specific
    pub alamat: Option<String>,
    pub luas_tanah: Option<f64>,
    pub luas_bangunan: Option<f64>,

    // Laptop-specific
    pub serial_number: Option<String>,
    pub spesifikasi: Option<serde_json::Value>,

    pub lokasi_pemakaian: Option<String>,
    /// Per-asset period — the authority. The header's dates are MIN/MAX of
    /// these.
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,

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
            urutan: row.get("urutan"),
            jenis_bmn: row.get("jenis_bmn"),
            bmn_nup: row.get("bmn_nup"),
            bmn_kode_barang: row.get("bmn_kode_barang"),
            bmn_nama_barang: row.get("bmn_nama_barang"),
            bmn_merk: row.try_get("bmn_merk").ok().flatten(),
            bmn_tipe: row.try_get("bmn_tipe").ok().flatten(),
            bmn_tahun_perolehan: row.try_get("bmn_tahun_perolehan").ok().flatten(),
            bmn_kondisi: row.try_get("bmn_kondisi").ok().flatten(),
            no_polisi: row.try_get("no_polisi").ok().flatten(),
            no_bpkb: row.try_get("no_bpkb").ok().flatten(),
            no_stnk: row.try_get("no_stnk").ok().flatten(),
            no_rangka: row.try_get("no_rangka").ok().flatten(),
            no_mesin: row.try_get("no_mesin").ok().flatten(),
            alamat: row.try_get("alamat").ok().flatten(),
            luas_tanah: row.try_get("luas_tanah").ok().flatten(),
            luas_bangunan: row.try_get("luas_bangunan").ok().flatten(),
            serial_number: row.try_get("serial_number").ok().flatten(),
            spesifikasi: row.try_get("spesifikasi").ok().flatten(),
            lokasi_pemakaian: row.try_get("lokasi_pemakaian").ok().flatten(),
            tanggal_mulai: row.get("tanggal_mulai"),
            tanggal_selesai: row.get("tanggal_selesai"),
            detail_bmn: row.try_get("detail_bmn").unwrap_or(serde_json::json!({})),
            keterangan: row.try_get("keterangan").ok().flatten(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
