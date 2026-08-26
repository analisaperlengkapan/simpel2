use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BankAsetItem {
    /// `integrasi.siman_aset.id` (BIGSERIAL). Serialized as a string because
    /// the FE contract types `id` as String.
    pub id: String,
    pub kategori_aset: String,
    pub no_aset: String,
    pub nama_aset: Option<String>,
    pub kode_barang: Option<String>,
    pub merk: Option<String>,
    pub tipe: Option<String>,
    pub kondisi: Option<String>,
    pub lokasi: Option<String>,
    pub satker: Option<String>,
    pub kode_satker: Option<String>,
    pub nup: Option<String>,
    pub nilai_perolehan: Option<f64>,
    pub tgl_perolehan: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// Slim lookup response used by the pemakaian-bmn form when the user types a
/// NUP — the frontend auto-fills `bmn_kode_barang` + `bmn_nama_barang`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BankAsetLookup {
    /// `integrasi.siman_aset.id` (BIGSERIAL), string-typed per the FE contract.
    pub id: String,
    pub nup: String,
    pub kode_barang: Option<String>,
    pub nama_barang: Option<String>,
    pub merk: Option<String>,
    pub tahun_perolehan: Option<String>,
    pub kondisi: Option<String>,
    pub satker: Option<String>,
    /// Nilai perolehan aset dari SIMAN (sumber otoritatif). Dipakai oleh
    /// penghapusan-bmn form & service untuk mengisi `nilai_perolehan`
    /// otomatis daripada percaya input operator.
    pub nilai_perolehan: Option<f64>,
}

/// How the domain identifies one BMN asset: **kode satker + kode barang +
/// NUP**.
///
/// Each part is separately optional so a caller can pass what it holds — the
/// pemakaian form has only a NUP typed into a box, while a penghapusan record
/// carries all three. Every part supplied narrows the match; a `None` is an
/// admission of ambiguity, not a wildcard the caller wanted.
///
/// NUP is required because it is the only part that is never implied by the
/// others: "Kejari Mamuju / kendaraan unit tahanan / NUP 2" names exactly one
/// physical asset, and dropping any of the three names a set.
#[derive(Debug, Clone, Copy)]
pub struct AsetIdentity<'a> {
    /// SIMAN `no_aset` — the Nomor Urut Pendaftaran.
    pub nup: &'a str,
    /// Barang code in either spelling; normalised before comparison.
    pub kode_barang: Option<&'a str>,
    /// MySIMKARI `kode_satker` of the owning satker (NOT SIMAN's
    /// `kdsatker_keu` — the lookup translates through
    /// `integrasi.v_satker_code_map`).
    pub satker_code: Option<&'a str>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BankAsetDetail {
    #[serde(flatten)]
    pub item: BankAsetItem,
    pub riwayat_pemakaian: Vec<RiwayatEntry>,
    pub riwayat_penghapusan: Vec<RiwayatEntry>,
    pub riwayat_kebutuhan: Vec<RiwayatEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RiwayatEntry {
    pub id: Uuid,
    pub ref_no: Option<String>,
    pub status: Option<String>,
    pub deskripsi: Option<String>,
    pub tanggal: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BankAsetDashboard {
    pub total_aset: i64,
    pub total_nilai_perolehan: f64,
    pub total_satker: i64,
    pub total_kategori: i64,
    pub kondisi_breakdown: Vec<KondisiStat>,
    pub kategori_breakdown: Vec<KategoriStat>,
    pub top_satker: Vec<SatkerStat>,
    pub per_tahun: Vec<TahunStat>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KondisiStat {
    pub kondisi: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KategoriStat {
    pub kategori: String,
    pub count: i64,
    pub nilai: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SatkerStat {
    pub satker: String,
    pub count: i64,
    pub nilai: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TahunStat {
    pub tahun: i32,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BankAsetSebaran {
    pub satker: Vec<SebaranSatker>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SebaranSatker {
    pub kode_satker: Option<String>,
    pub nama_satker: String,
    pub total_aset: i64,
    pub nilai_perolehan: f64,
    pub aset_baik: i64,
    pub aset_rusak: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LastSyncInfo {
    pub last_sync_at: Option<DateTime<Utc>>,
    pub total_aset: i64,
    pub source: String,
}

/// One selectable filter value plus how many assets carry it (for dropdowns).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilterOption {
    pub value: String,
    pub count: i64,
}

/// Distinct values per filterable column, served to the FE so filter dropdowns
/// are populated dynamically from real SIMAN data (jenis BMN, kategori, kondisi,
/// satker) rather than hard-coded lists.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BankAsetFilterOptions {
    pub jenis: Vec<FilterOption>,
    pub kategori: Vec<FilterOption>,
    pub kondisi: Vec<FilterOption>,
    pub satker: Vec<FilterOption>,
}
