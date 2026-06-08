use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::Row;
use uuid::Uuid;

use super::*;

// ============================================================================
// Main Entity: Pengajuan Kebutuhan BMN
// ============================================================================

/// Main entity for BMN needs analysis request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanKebutuhanBmn {
    pub id: Uuid,
    pub nama: String,
    pub deskripsi: Option<String>,
    pub tahun: i32,
    pub tgl_mulai: NaiveDate,
    pub tgl_selesai: NaiveDate,
    /// Legacy field (V029 deprecates). Aplikasi baru harus baca `scope_satker`;
    /// kolom legacy dipertahankan utk backward compat consumer lama.
    pub pilihan_satker: PilihanSatker,
    /// Cakupan satker per V029 (Fase 1.7): semua | sebagian | wilayah.
    pub scope_satker: PilihanSatker,
    /// Nama wilayah Kejaksaan Tinggi (text label, match
    /// `integrasi.mysimkari_satker.wilayah`). WAJIB jika scope=wilayah;
    /// `None` untuk scope lain.
    pub wilayah_id: Option<String>,
    pub id_jenis_asset: Value,
    pub is_appv_daskrimti: bool,
    pub status_kode: i32,
    pub status: KebutuhanBmnStatus,
    // Report generation
    pub laporan_url: Option<String>,
    pub laporan_format: Option<String>,
    pub laporan_generated_at: Option<DateTime<Utc>>,
    // Audit
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}
impl PengajuanKebutuhanBmn {
    /// Create from database row
    pub fn from_row(row: &Row) -> Self {
        let status_kode: i32 = row.get("status_kode");
        Self {
            id: row.get("id"),
            nama: row.get("nama"),
            deskripsi: row.get("deskripsi"),
            tahun: row.get("tahun"),
            tgl_mulai: row.get("tgl_mulai"),
            tgl_selesai: row.get("tgl_selesai"),
            pilihan_satker: PilihanSatker::from_str(
                row.get::<_, String>("pilihan_satker").as_str(),
            ),
            scope_satker: row
                .try_get::<_, String>("scope_satker")
                .ok()
                .map(|s| PilihanSatker::from_str(&s))
                .unwrap_or_default(),
            wilayah_id: row.try_get("wilayah_id").ok().flatten(),
            id_jenis_asset: row.get("id_jenis_asset"),
            is_appv_daskrimti: row.get("is_appv_daskrimti"),
            status_kode,
            status: KebutuhanBmnStatus::from_code(status_kode).unwrap_or_default(),
            laporan_url: row.try_get("laporan_url").ok().flatten(),
            laporan_format: row.try_get("laporan_format").ok().flatten(),
            laporan_generated_at: row.try_get("laporan_generated_at").ok().flatten(),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            version: row.get("version"),
        }
    }
}
// ============================================================================
// Entity: Pengajuan Kebutuhan BMN Asset
// ============================================================================

/// Asset type included in a BMN request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanKebutuhanBmnAsset {
    pub id: Uuid,
    pub pengajuan_id: Uuid,
    pub kode_barang: Option<String>,
    pub nm_barang: Option<String>,
    pub ms_jenis_asset_id: Option<i32>,
    pub keterangan: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl PengajuanKebutuhanBmnAsset {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pengajuan_id: row.get("pengajuan_id"),
            kode_barang: row.get("kode_barang"),
            nm_barang: row.get("nm_barang"),
            ms_jenis_asset_id: row.get("ms_jenis_asset_id"),
            keterangan: row.get("keterangan"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
// ============================================================================
// Entity: Pengajuan Kebutuhan BMN Satker
// ============================================================================

/// Per-satker tracking for BMN request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanKebutuhanBmnSatker {
    pub id: Uuid,
    pub pengajuan_id: Uuid,
    pub ms_satker_id: String,
    pub ms_satker_pusat_id: Option<String>,
    pub nm_satker: Option<String>,
    pub status_kode: i32,
    pub status: KebutuhanBmnStatus,
    pub prioritas: i32,
    // Lampiran & catatan operator satker
    pub catatan_satker: Option<String>,
    pub lampiran_surat_permohonan: Option<String>,
    pub lampiran_pendukung: Value,
    // Validator wilayah
    pub catatan_validator_wilayah: Option<String>,
    pub validator_wilayah_id: Option<Uuid>,
    pub tanggal_submit_wilayah: Option<DateTime<Utc>>,
    // Validator pusat
    pub catatan_validator_pusat: Option<String>,
    pub validator_pusat_id: Option<Uuid>,
    pub tanggal_submit_pusat: Option<DateTime<Utc>>,
    // Analysis data (from SIMAN & MySIMKARI)
    pub data_eksisting_siman: Value,
    pub data_pegawai_mysimkari: Value,
    pub rekap_eselon: Value,
    pub rekap_non_eselon: Value,
    pub hasil_analisis: Value,
    pub is_approved: Option<bool>,
    pub alasan_keputusan: Option<String>,
    // Audit
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl PengajuanKebutuhanBmnSatker {
    pub fn from_row(row: &Row) -> Self {
        let status_kode: i32 = row.get("status_kode");
        Self {
            id: row.get("id"),
            pengajuan_id: row.get("pengajuan_id"),
            ms_satker_id: row.get("ms_satker_id"),
            ms_satker_pusat_id: row.get("ms_satker_pusat_id"),
            nm_satker: row.get("nm_satker"),
            status_kode,
            status: KebutuhanBmnStatus::from_code(status_kode).unwrap_or_default(),
            prioritas: row.get("prioritas"),
            catatan_satker: row.try_get("catatan_satker").ok().flatten(),
            lampiran_surat_permohonan: row.try_get("lampiran_surat_permohonan").ok().flatten(),
            lampiran_pendukung: row
                .try_get("lampiran_pendukung")
                .unwrap_or(serde_json::json!([])),
            catatan_validator_wilayah: row.try_get("catatan_validator_wilayah").ok().flatten(),
            validator_wilayah_id: row.try_get("validator_wilayah_id").ok().flatten(),
            tanggal_submit_wilayah: row.try_get("tanggal_submit_wilayah").ok().flatten(),
            catatan_validator_pusat: row.try_get("catatan_validator_pusat").ok().flatten(),
            validator_pusat_id: row.try_get("validator_pusat_id").ok().flatten(),
            tanggal_submit_pusat: row.try_get("tanggal_submit_pusat").ok().flatten(),
            data_eksisting_siman: row
                .try_get("data_eksisting_siman")
                .unwrap_or(serde_json::json!({})),
            data_pegawai_mysimkari: row
                .try_get("data_pegawai_mysimkari")
                .unwrap_or(serde_json::json!({})),
            rekap_eselon: row.try_get("rekap_eselon").unwrap_or(serde_json::json!({})),
            rekap_non_eselon: row
                .try_get("rekap_non_eselon")
                .unwrap_or(serde_json::json!({})),
            hasil_analisis: row
                .try_get("hasil_analisis")
                .unwrap_or(serde_json::json!({})),
            is_approved: row.try_get("is_approved").ok().flatten(),
            alasan_keputusan: row.try_get("alasan_keputusan").ok().flatten(),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
// ============================================================================
// Entity: Pengajuan Kebutuhan BMN Satker Barang
// ============================================================================

/// Individual goods requested by a satker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanKebutuhanBmnBarang {
    pub id: Uuid,
    pub pengajuan_satker_id: Uuid,
    pub nama: String,
    pub kode_barang: Option<String>,
    pub jumlah: i32,
    pub satuan: String,
    pub jml_setuju: i32,
    pub alasan: Option<String>,
    pub keterangan: Option<String>,
    pub prioritas: i32,
    pub skor: f64,
    pub file_pendukung: Value,
    pub existing_count: i32,
    pub existing_condition: Option<String>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl PengajuanKebutuhanBmnBarang {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pengajuan_satker_id: row.get("pengajuan_satker_id"),
            nama: row.get("nama"),
            kode_barang: row.get("kode_barang"),
            jumlah: row.get("jumlah"),
            satuan: row.get("satuan"),
            jml_setuju: row.get("jml_setuju"),
            alasan: row.get("alasan"),
            keterangan: row.get("keterangan"),
            prioritas: row.get("prioritas"),
            skor: row.try_get::<_, f64>("skor").unwrap_or(0.0),
            file_pendukung: row.get("file_pendukung"),
            existing_count: row.get("existing_count"),
            existing_condition: row.get("existing_condition"),
            created_by: row.get("created_by"),
            updated_by: row.get("updated_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
// ============================================================================
// Entity: Pengajuan Kebutuhan BMN Satker Aktivitas
// ============================================================================

/// Workflow history audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanKebutuhanBmnAktivitas {
    pub id: Uuid,
    pub pengajuan_satker_id: Uuid,
    pub from_status_kode: Option<i32>,
    pub to_status_kode: i32,
    pub user_id: Option<Uuid>,
    pub nip: Option<String>,
    pub nama: Option<String>,
    pub pangkat: Option<String>,
    pub jabatan: Option<String>,
    pub role: Option<String>,
    pub aksi: String,
    pub komentar: Option<String>,
    pub created_at: DateTime<Utc>,
}
impl PengajuanKebutuhanBmnAktivitas {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pengajuan_satker_id: row.get("pengajuan_satker_id"),
            from_status_kode: row.get("from_status_kode"),
            to_status_kode: row.get("to_status_kode"),
            user_id: row.get("user_id"),
            nip: row.get("nip"),
            nama: row.get("nama"),
            pangkat: row.get("pangkat"),
            jabatan: row.get("jabatan"),
            role: row.get("role"),
            aksi: row.get("aksi"),
            komentar: row.get("komentar"),
            created_at: row.get("created_at"),
        }
    }
}
// ============================================================================
// Summary View Model
// ============================================================================

/// Summary data for dashboard and lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KebutuhanBmnSummary {
    pub id: Uuid,
    pub nama: String,
    pub tahun: i32,
    pub status_kode: i32,
    pub status_nama: String,
    pub total_satker: i64,
    pub total_barang: i64,
    pub total_jumlah_diminta: i64,
    pub total_jumlah_disetujui: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl KebutuhanBmnSummary {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            nama: row.get("nama"),
            tahun: row.get("tahun"),
            status_kode: row.get("status_kode"),
            status_nama: row
                .get::<_, Option<String>>("status_nama")
                .unwrap_or_default(),
            total_satker: row.get::<_, i64>("total_satker"),
            total_barang: row.get::<_, i64>("total_barang"),
            total_jumlah_diminta: row.get::<_, i64>("total_jumlah_diminta"),
            total_jumlah_disetujui: row.get::<_, i64>("total_jumlah_disetujui"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}
/// V029 (Fase 1.6): Satu entry allowed BMN (entity).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PengajuanBmnReferensi {
    pub id: Uuid,
    pub pengajuan_id: Uuid,
    pub kode_barang: String,
    pub nama_barang: String,
    pub keterangan: Option<String>,
    pub created_at: DateTime<Utc>,
}
impl PengajuanBmnReferensi {
    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            pengajuan_id: row.get("pengajuan_id"),
            kode_barang: row.get("kode_barang"),
            nama_barang: row.get("nama_barang"),
            keterangan: row.try_get("keterangan").ok().flatten(),
            created_at: row.get("created_at"),
        }
    }
}
