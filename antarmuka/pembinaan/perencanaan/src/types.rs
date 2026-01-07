use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{NaiveDate, DateTime, Utc};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RencanaPengadaan {
    pub id: Uuid,
    pub nama_kegiatan: String,
    pub kode_rekening: String,
    pub pagu_anggaran: i64,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateRencanaRequest {
    pub nama_kegiatan: String,
    pub kode_rekening: String,
    pub pagu_anggaran: i64,
    pub tanggal_mulai: NaiveDate,
    pub tanggal_selesai: NaiveDate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateRencanaRequest {
    pub nama_kegiatan: Option<String>,
    pub kode_rekening: Option<String>,
    pub pagu_anggaran: Option<i64>,
    pub tanggal_mulai: Option<NaiveDate>,
    pub tanggal_selesai: Option<NaiveDate>,
    pub status: Option<String>,
}

// Keeping existing types to avoid breaking other potential usages,
// though they seem like placeholders.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Barang {
    pub id: String,
    pub kode_barang: String,
    pub nama_barang: String,
    pub kategori: String,
    pub jumlah: i32,
    pub satuan: String,
    pub harga_satuan: f64,
    pub total_harga: f64,
    pub status: String,
    pub lokasi: String,
    pub tanggal_pengadaan: String,
    pub supplier: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pengadaan {
    pub id: String,
    pub nomor_pengadaan: String,
    pub nama_pengadaan: String,
    pub jenis_pengadaan: String,
    pub nilai_pengadaan: f64,
    pub status: String,
    pub tanggal_mulai: String,
    pub tanggal_selesai: String,
    pub supplier: String,
    pub progress: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Laporan {
    pub id: String,
    pub judul: String,
    pub jenis: String,
    pub periode: String,
    pub status: String,
    pub tanggal_generate: String,
    pub file_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Statistik {
    pub total_barang: i32,
    pub total_nilai: f64,
    pub pengadaan_aktif: i32,
    pub perlu_perbaikan: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}
