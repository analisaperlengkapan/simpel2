use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BankAsetItem {
    pub id: Uuid,
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
