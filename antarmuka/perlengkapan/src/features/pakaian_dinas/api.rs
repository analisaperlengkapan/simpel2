//! Klien untuk grup ukuran sebagai DATA REFERENSI dan profil ukuran per spesifikasi (V019).
//!
//! Backend: `layanan/perlengkapan/src/pakaian_dinas/` (`/pakaian-dinas/ukuran-grup`,
//! `/pakaian-dinas/ukuran`, `/pakaian-dinas/ukuran-profil`). Nama tipe di sini sama dengan
//! kembarannya di backend, supaya `check-fe-be-dto-drift.py` memasangkan dan memeriksanya.
//!
//! Sengaja memakai `api_get`/`api_put`/… dari `client`, bukan pasangan `cfg(wasm32)` ganda
//! seperti berkas `pakaian_dinas.rs` yang lebih tua: satu salinan, satu jalur galat.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::api::client::{api_delete_empty, api_get, api_post, api_put};
use crate::api::error::AppResult;
use crate::api::pakaian_dinas::Ukuran;

#[derive(Debug, Clone, Deserialize)]
struct Bungkus<T> {
    data: T,
}

/// Satu grup ukuran: `PILIHAN` (nilai harus salah satu dari daftarnya) atau `TEKS` (isian bebas).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UkuranGrup {
    pub kode: String,
    pub nama: String,
    pub tipe: String,
    pub urutan: i32,
    pub is_active: bool,
}

impl UkuranGrup {
    pub const TIPE_TEKS: &'static str = "TEKS";

    pub fn is_teks(&self) -> bool {
        self.tipe.eq_ignore_ascii_case(Self::TIPE_TEKS)
    }
}

/// Satu kolom pakaian yang ditawarkan di halaman Ukuran pegawai, dengan isiannya.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UkuranProfilItem {
    pub spesifikasi_id: String,
    pub jenis_pakaian_nama: String,
    pub spesifikasi_nama: String,
    pub ukuran_group: String,
    pub tipe: String,
    pub gender: String,
    #[serde(default)]
    pub ukuran: Option<String>,
}

impl UkuranProfilItem {
    pub fn is_teks(&self) -> bool {
        self.tipe.eq_ignore_ascii_case(UkuranGrup::TIPE_TEKS)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateUkuranGrupRequest {
    pub kode: String,
    pub nama: String,
    pub tipe: Option<String>,
    pub urutan: Option<i32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UpdateUkuranGrupRequest {
    pub nama: String,
    pub urutan: Option<i32>,
    pub is_active: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateUkuranRequest {
    pub ukuran: String,
    pub group: String,
    pub urutan: Option<i32>,
}

/// Seluruh daftar ukuran sebuah grup dalam urutan baru: satu permintaan, satu transaksi di server.
#[derive(Clone, Debug, Serialize)]
pub struct UrutkanUkuranRequest {
    pub ukuran: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UkuranProfilInput {
    pub spesifikasi_id: String,
    pub ukuran: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SimpanUkuranProfilRequest {
    pub ukuran: Vec<UkuranProfilInput>,
    pub with_hijab: Option<bool>,
}

/// Pembungkus `{success,data,message}` yang dikembalikan pembuatan/pembaruan; isinya tak dipakai
/// pemanggil (daftar dimuat ulang), jadi tak perlu dibaca.
#[derive(Debug, Clone, Deserialize)]
struct Abaikan {}

/// Semua grup (urutan tampil). `hanya_aktif`: yang ditawarkan saat membuat spesifikasi baru.
pub async fn fetch_ukuran_grup(hanya_aktif: bool) -> AppResult<Vec<UkuranGrup>> {
    let jalan = if hanya_aktif {
        "/pakaian-dinas/ukuran-grup?aktif=true"
    } else {
        "/pakaian-dinas/ukuran-grup"
    };
    let r: Bungkus<Vec<UkuranGrup>> = api_get(jalan).await?;
    Ok(r.data)
}

pub async fn buat_ukuran_grup(request: &CreateUkuranGrupRequest) -> AppResult<()> {
    let _: Abaikan = api_post("/pakaian-dinas/ukuran-grup", request).await?;
    Ok(())
}

pub async fn ubah_ukuran_grup(kode: &str, request: &UpdateUkuranGrupRequest) -> AppResult<()> {
    let _: Abaikan = api_put(&format!("/pakaian-dinas/ukuran-grup/{kode}"), request).await?;
    Ok(())
}

pub async fn hapus_ukuran_grup(kode: &str) -> AppResult<()> {
    api_delete_empty(&format!("/pakaian-dinas/ukuran-grup/{kode}")).await
}

pub async fn buat_ukuran(request: &CreateUkuranRequest) -> AppResult<()> {
    let _: Abaikan = api_post("/pakaian-dinas/ukuran", request).await?;
    Ok(())
}

/// Susun ulang seluruh daftar ukuran sebuah grup. Ditolak utuh bila daftar di server sudah
/// berbeda dari yang dikirim (ukuran ditambah atau dihapus sejak layar memuatnya).
pub async fn urutkan_ukuran_grup(grup: &str, ukuran: Vec<String>) -> AppResult<()> {
    let _: Abaikan = api_put(
        &format!("/pakaian-dinas/ukuran-grup/{grup}/urutan-ukuran"),
        &UrutkanUkuranRequest { ukuran },
    )
    .await?;
    Ok(())
}

pub async fn hapus_ukuran(group: &str, ukuran: &str) -> AppResult<()> {
    api_delete_empty(&format!("/pakaian-dinas/ukuran/{group}/{ukuran}")).await
}

/// Profil ukuran milik pemanggil (NIP dari klaim): satu entri per kolom pakaian aktif.
pub async fn fetch_ukuran_profil() -> AppResult<Vec<UkuranProfilItem>> {
    let r: Bungkus<Vec<UkuranProfilItem>> = api_get("/pakaian-dinas/ukuran-profil").await?;
    Ok(r.data)
}

/// Simpan profil; yang dikembalikan adalah keadaan terbaru seluruh kolom.
pub async fn simpan_ukuran_profil(
    request: &SimpanUkuranProfilRequest,
) -> AppResult<Vec<UkuranProfilItem>> {
    let r: Bungkus<Vec<UkuranProfilItem>> =
        api_put("/pakaian-dinas/ukuran-profil", request).await?;
    Ok(r.data)
}

/// Daftar ukuran per kode grup (huruf besar), terurut menurut `urutan`. Satu permintaan untuk
/// seluruh master (puluhan baris) mengalahkan satu permintaan per grup, dan grup baru dari
/// referensi ikut tanpa mengubah kode.
pub fn kelompokkan_ukuran(ukuran: &[Ukuran]) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<(i32, String)>> = HashMap::new();
    for u in ukuran {
        map.entry(u.group.to_uppercase())
            .or_default()
            .push((u.urutan, u.ukuran.clone()));
    }
    map.into_iter()
        .map(|(k, mut v)| {
            v.sort_by_key(|(urutan, _)| *urutan);
            (k, v.into_iter().map(|(_, u)| u).collect())
        })
        .collect()
}

/// Ikon (nama Font Awesome yang dikenal `icon_from_fa_class`) dan kelas warnanya untuk sebuah
/// grup. Grup baru dari referensi yang tak dikenal mendapat penggaris netral, bukan ikon tanda tanya.
pub fn ikon_grup(kode: &str) -> &'static str {
    match kode.to_uppercase().as_str() {
        "BAJU" => "fas fa-tshirt text-info-400",
        "CELANA" => "fas fa-male text-success-400",
        "SEPATU" => "fas fa-shoe-prints text-gold-400",
        "NAMA_DADA" => "fas fa-user-tag text-gold-400",
        _ => "fas fa-ruler text-slate-300",
    }
}
