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

use crate::api::client::{api_delete, api_delete_empty, api_get, api_post, api_put};
use crate::api::error::AppResult;
use crate::api::pakaian_dinas::{RosterPegawai, RosterPengisian, Ukuran};

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

/// Pre-flight readiness for one workflow gate. Mirrors backend
/// `pakaian_dinas::models::KesiapanPakaianDinas`; every count is advisory and
/// the person may still proceed.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct KesiapanPakaianDinas {
    /// `satker` | `wilayah` | `pusat`.
    #[serde(default)]
    pub tahap: String,
    #[serde(default)]
    pub lengkap: bool,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub pegawai_belum_diisi: i64,
    #[serde(default)]
    pub belum_mengajukan: i64,
    #[serde(default)]
    pub belum_lengkap: i64,
    #[serde(default)]
    pub belum_diteruskan: i64,
    #[serde(default)]
    pub belum_selesai: i64,
    #[serde(default)]
    pub selesai: i64,
}

/// `DELETE /pakaian-dinas/pengajuan/{id}/satker/{kode}/pegawai/{nip}/keluarkan`
/// — undo "Keluarkan dari Pengajuan". The person goes back to "belum mengisi":
/// their sizes are not restored, because they never had any when excluded.
pub async fn masukkan_kembali_pegawai(
    pengajuan_id: &str,
    satker_code: &str,
    nip: &str,
) -> AppResult<RosterPengisian> {
    let r: Bungkus<RosterPengisian> = api_delete(&format!(
        "/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{satker_code}/pegawai/{nip}/keluarkan"
    ))
    .await?;
    Ok(r.data)
}

/// `GET /pakaian-dinas/pengajuan/{id}/kesiapan` — counts for the confirmation
/// dialog. `kecuali` lists the participation rows about to be moved (one for a
/// single action, many for a bulk move), so they are not counted against the
/// person moving them.
pub async fn fetch_kesiapan_pakaian_dinas(
    pengajuan_id: &str,
    tahap: &str,
    satker_code: Option<&str>,
    kecuali: &[String],
) -> AppResult<KesiapanPakaianDinas> {
    let mut jalan = format!("/pakaian-dinas/pengajuan/{pengajuan_id}/kesiapan?tahap={tahap}");
    if let Some(kode) = satker_code.filter(|k| !k.trim().is_empty()) {
        jalan.push_str(&format!("&satker_code={kode}"));
    }
    let kecuali = kecuali
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(",");
    if !kecuali.is_empty() {
        jalan.push_str(&format!("&kecuali={kecuali}"));
    }
    let r: Bungkus<KesiapanPakaianDinas> = api_get(&jalan).await?;
    Ok(r.data)
}

/// How many people owe a size and how many have given one.
///
/// People the operator excluded ("Keluarkan dari Pengajuan") are out of BOTH
/// numbers: they owe nothing, so counting them would raise a shortfall warning
/// that does not exist — the server's `kesiapan` does not count them either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitungPengisian {
    pub total: usize,
    pub terisi: usize,
}

impl HitungPengisian {
    /// People who still owe a size.
    pub fn belum(&self) -> usize {
        self.total.saturating_sub(self.terisi)
    }
}

pub fn hitung_pengisian(pegawai: &[RosterPegawai]) -> HitungPengisian {
    let aktif = || pegawai.iter().filter(|p| !p.dikeluarkan);
    HitungPengisian {
        total: aktif().count(),
        terisi: aktif().filter(|p| p.sudah_diisi).count(),
    }
}

/// One-sentence warning for the confirmation dialog, or `None` when there is
/// nothing outstanding to mention. Pure, so the wording is testable.
pub fn ringkasan_kesiapan(k: &KesiapanPakaianDinas) -> Option<String> {
    let mut bagian = Vec::new();
    if k.tahap == "satker" {
        if k.pegawai_belum_diisi > 0 {
            bagian.push(format!(
                "{} dari {} pegawai belum mengisi ukuran",
                k.pegawai_belum_diisi, k.total
            ));
        }
    } else {
        if k.belum_mengajukan > 0 {
            bagian.push(format!(
                "{} satker belum mengajukan ke validator wilayah",
                k.belum_mengajukan
            ));
        }
        if k.belum_lengkap > 0 {
            bagian.push(format!(
                "{} satker sudah diajukan tetapi ukurannya belum lengkap",
                k.belum_lengkap
            ));
        }
        if k.tahap == "pusat" && k.belum_diteruskan > 0 {
            bagian.push(format!(
                "{} satker belum diteruskan ke validator pusat",
                k.belum_diteruskan
            ));
        }
    }
    if bagian.is_empty() {
        None
    } else {
        Some(format!("{}.", bagian.join("; ")))
    }
}

#[cfg(test)]
mod kesiapan_ringkasan_tests {
    use super::{KesiapanPakaianDinas, ringkasan_kesiapan};

    fn satker(belum: i64, total: i64) -> KesiapanPakaianDinas {
        KesiapanPakaianDinas {
            tahap: "satker".to_string(),
            lengkap: belum == 0,
            total,
            pegawai_belum_diisi: belum,
            ..Default::default()
        }
    }

    #[test]
    fn a_complete_gate_has_nothing_to_warn_about() {
        assert_eq!(ringkasan_kesiapan(&satker(0, 40)), None);
        let wilayah = KesiapanPakaianDinas {
            tahap: "wilayah".to_string(),
            lengkap: true,
            total: 10,
            ..Default::default()
        };
        assert_eq!(ringkasan_kesiapan(&wilayah), None);
    }

    #[test]
    fn the_satker_warning_names_the_unfilled_employees() {
        let teks = ringkasan_kesiapan(&satker(3, 40)).unwrap();
        assert!(teks.contains("3 dari 40 pegawai"), "{teks}");
    }

    #[test]
    fn the_wilayah_warning_names_unsubmitted_and_half_filled_satker() {
        let k = KesiapanPakaianDinas {
            tahap: "wilayah".to_string(),
            lengkap: false,
            total: 10,
            belum_mengajukan: 2,
            belum_lengkap: 1,
            // Not forwarded yet is the wilayah's own queue, not a warning here.
            belum_diteruskan: 4,
            ..Default::default()
        };
        let teks = ringkasan_kesiapan(&k).unwrap();
        assert!(teks.contains("2 satker belum mengajukan"), "{teks}");
        assert!(teks.contains("1 satker sudah diajukan"), "{teks}");
        assert!(!teks.contains("diteruskan"), "{teks}");
    }

    #[test]
    fn the_pusat_warning_also_names_satker_wilayah_has_not_forwarded() {
        let k = KesiapanPakaianDinas {
            tahap: "pusat".to_string(),
            lengkap: false,
            total: 100,
            belum_mengajukan: 3,
            belum_lengkap: 0,
            belum_diteruskan: 7,
            belum_selesai: 5,
            ..Default::default()
        };
        let teks = ringkasan_kesiapan(&k).unwrap();
        assert!(teks.contains("3 satker belum mengajukan"), "{teks}");
        assert!(teks.contains("7 satker belum diteruskan"), "{teks}");
        // Satker already at pusat waiting for its own approval is not a
        // subordinate failure and is deliberately not warned about.
        assert!(!teks.contains("menunggu persetujuan"), "{teks}");
    }
}

#[cfg(test)]
mod hitung_pengisian_tests {
    use super::{HitungPengisian, hitung_pengisian};
    use crate::api::pakaian_dinas::RosterPegawai;

    fn orang(nip: &str, sudah_diisi: bool, dikeluarkan: bool) -> RosterPegawai {
        RosterPegawai {
            nip: nip.to_string(),
            nama: nip.to_string(),
            jabatan: None,
            pangkat: None,
            gol_kd: None,
            eselon: None,
            jenis_kelamin: "L".to_string(),
            jenis: None,
            foto: None,
            with_hijab: false,
            sudah_diisi,
            dikeluarkan,
            ukuran: Vec::new(),
            saran: Vec::new(),
        }
    }

    #[test]
    fn excluded_people_are_out_of_both_numbers() {
        let roster = vec![orang("1", true, false), orang("2", false, true)];
        assert_eq!(
            hitung_pengisian(&roster),
            HitungPengisian {
                total: 1,
                terisi: 1
            },
            "satu-satunya yang wajib mengisi sudah mengisi: tidak ada peringatan"
        );
        assert_eq!(hitung_pengisian(&roster).belum(), 0);
    }

    #[test]
    fn an_active_person_who_has_not_filled_is_still_counted() {
        let roster = vec![
            orang("1", true, false),
            orang("2", false, false),
            orang("3", false, true),
        ];
        let h = hitung_pengisian(&roster);
        assert_eq!((h.total, h.terisi, h.belum()), (2, 1, 1));
    }

    #[test]
    fn an_empty_roster_owes_nothing() {
        assert_eq!(hitung_pengisian(&[]).belum(), 0);
    }
}
