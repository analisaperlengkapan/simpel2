//! BADIKLAT Types
//!
//! Data structures for education and training module

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PelatihanItem {
    pub id: String,
    pub judul: String,
    pub deskripsi: String,
    pub tanggal_mulai: String,
    pub tanggal_selesai: String,
    pub instruktur: String,
    pub kuota: i32,
    pub peserta_terdaftar: i32,
    pub status: String,
    pub kategori: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PesertaItem {
    pub id: String,
    pub nama: String,
    pub nip: String,
    pub unit_kerja: String,
    pub email: String,
    pub pelatihan_diikuti: Vec<String>,
    pub sertifikat: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InstrukturItem {
    pub id: String,
    pub nama: String,
    pub spesialisasi: String,
    pub email: String,
    pub pengalaman: String,
    pub rating: f32,
    pub pelatihan_diampu: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SertifikatItem {
    pub id: String,
    pub nomor_sertifikat: String,
    pub nama_peserta: String,
    pub nama_pelatihan: String,
    pub tanggal_terbit: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatistikBadiklat {
    pub total_pelatihan: i32,
    pub total_peserta: i32,
    pub total_instruktur: i32,
    pub sertifikat_terbit: i32,
    pub tingkat_kepuasan: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterPelatihan {
    pub kategori: String,
    pub status: String,
    pub instruktur: String,
    pub bulan: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}
