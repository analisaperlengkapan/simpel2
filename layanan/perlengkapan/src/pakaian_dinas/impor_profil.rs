//! Impor profil ukuran awal: tipe permintaan/hasil dan validasi identitas yang murni.
//!
//! Data awal ukuran pakaian dinas datang dari berkas milik satker (jutaan sel, ditulis tangan),
//! bukan dari aplikasi. Berkasnya dibersihkan DI LUAR aplikasi oleh alat penyiapan
//! (`infra/scripts/siapkan-impor-pakaian-dinas.py`), yang memisahkan baris yang jelas dari yang
//! masih diragukan (NIP rusak atau ganda). Yang sampai ke sini hanya baris yang jelas, dan
//! endpoint ini TETAP memvalidasi ulang dengan aturan yang sama dengan setiap jalur penyimpan
//! ukuran (`validasi_ukuran`): alat penyiapan bisa salah, aturan di sini tidak boleh longgar.
//!
//! Data nyata tidak pernah ada di repositori: berkasnya dikirim operator per lot lewat
//! permintaan, dan tak satu pun NIP atau nama ditulis ke log (hanya jumlah).

use serde::{Deserialize, Serialize};

/// Paling banyak baris per permintaan. 43.000 baris = 44 lot; batas ini menjaga satu transaksi
/// dan satu badan permintaan tetap kecil.
pub const MAKS_BARIS: usize = 2000;

/// Satu ukuran pada satu kolom pakaian, dinamai menurut jenis dan spesifikasinya (bukan id), supaya
/// berkas yang disiapkan tak bergantung pada id yang dibuat basis data.
#[derive(Debug, Clone, Deserialize)]
pub struct ImporUkuran {
    pub jenis: String,
    pub spesifikasi: String,
    pub ukuran: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImporBaris {
    /// Rujukan ke baris di berkas sumber (mis. `kejati:1234`), dikembalikan apa adanya pada baris
    /// yang ditolak agar bisa dicari lagi tanpa membawa NIP.
    pub rujukan: String,
    pub nip: String,
    pub nama: String,
    #[serde(default)]
    pub jenis_kelamin: Option<String>,
    #[serde(default)]
    pub with_hijab: Option<bool>,
    #[serde(default)]
    pub ukuran: Vec<ImporUkuran>,
}

#[derive(Debug, Deserialize)]
pub struct ImporProfilRequest {
    /// `true`: validasi dan hitung apa yang AKAN terjadi, tanpa menulis apa pun.
    #[serde(default)]
    pub dry_run: bool,
    pub baris: Vec<ImporBaris>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ImporDitolak {
    pub rujukan: String,
    /// NIP yang disamarkan: cukup untuk mengenali baris, tidak cukup untuk menyalin identitasnya.
    pub nip_tersamar: String,
    pub alasan: String,
}

#[derive(Debug, Default, Serialize)]
pub struct ImporProfilHasil {
    pub dry_run: bool,
    pub diterima: usize,
    /// Baris yang lolos validasi (dimuat, atau akan dimuat pada `dry_run`).
    pub baris_dimuat: usize,
    pub profil_baru: usize,
    pub profil_sudah_ada: usize,
    pub ukuran_ditulis: usize,
    /// Ukuran yang dilewati karena sudah diisi orangnya sendiri lewat aplikasi: impor tidak menimpa.
    pub ukuran_dilewati_sudah_diisi: usize,
    pub ditolak: Vec<ImporDitolak>,
}

/// NIP yang dibersihkan dari spasi: tepat 18 digit, tidak lebih dan tidak kurang. Pemotongan
/// "NIP + nomor lama" dan perbaikan angka rusak dilakukan alat penyiapan dan TIDAK ditebak di sini.
pub fn nip_bersih(nip: &str) -> Result<String, String> {
    let n = nip.trim();
    if n.len() == 18 && n.bytes().all(|b| b.is_ascii_digit()) {
        Ok(n.to_string())
    } else {
        Err("NIP harus tepat 18 digit".to_string())
    }
}

/// NIP untuk ditampilkan di galat: enam digit pertama dan dua terakhir.
pub fn nip_tersamar(nip: &str) -> String {
    let chars: Vec<char> = nip.trim().chars().collect();
    if chars.len() < 10 {
        return "…".to_string();
    }
    let depan: String = chars[..6].iter().collect();
    let belakang: String = chars[chars.len() - 2..].iter().collect();
    format!("{depan}…{belakang}")
}

/// Nama, jenis kelamin, dan bendera hijab sebuah baris, dirapikan. Galatnya satu kalimat yang
/// menyebut kolomnya.
pub fn identitas_bersih(b: &ImporBaris) -> Result<(String, String, Option<String>), String> {
    let nip = nip_bersih(&b.nip)?;
    let nama = b.nama.split_whitespace().collect::<Vec<_>>().join(" ");
    if nama.is_empty() {
        return Err("Nama kosong".to_string());
    }
    if nama.chars().count() > 255 {
        return Err("Nama lebih dari 255 karakter".to_string());
    }
    let jk = match b.jenis_kelamin.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(v) if v.eq_ignore_ascii_case("L") => Some("L".to_string()),
        Some(v) if v.eq_ignore_ascii_case("P") => Some("P".to_string()),
        Some(_) => return Err("Jenis kelamin harus L atau P".to_string()),
    };
    Ok((nip, nama, jk))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baris(nip: &str, nama: &str, jk: Option<&str>) -> ImporBaris {
        ImporBaris {
            rujukan: "uji:1".to_string(),
            nip: nip.to_string(),
            nama: nama.to_string(),
            jenis_kelamin: jk.map(str::to_string),
            with_hijab: None,
            ukuran: vec![],
        }
    }

    #[test]
    fn nip_harus_tepat_18_digit() {
        assert_eq!(
            nip_bersih(" 199203142014031001 ").unwrap(),
            "199203142014031001"
        );
        for salah in [
            "",
            "19920314201403100",
            "1992031420140310011",
            "19920314201403100x",
            "1.99E+17",
            "199203142014031001 123456",
        ] {
            assert!(nip_bersih(salah).is_err(), "{salah:?} harus ditolak");
        }
    }

    #[test]
    fn nip_tersamar_tak_memuat_identitas_utuh() {
        let t = nip_tersamar("199203142014031001");
        assert_eq!(t, "199203…01");
        assert!(!t.contains("2014031"));
        assert_eq!(nip_tersamar("123"), "…");
    }

    #[test]
    fn identitas_dirapikan_dan_digalatkan() {
        let (nip, nama, jk) =
            identitas_bersih(&baris("199203142014031001", "  Budi   Santoso ", Some("l"))).unwrap();
        assert_eq!(
            (nip.as_str(), nama.as_str(), jk.as_deref()),
            ("199203142014031001", "Budi Santoso", Some("L"))
        );
        assert_eq!(
            identitas_bersih(&baris("199203142014031001", "A", Some("")))
                .unwrap()
                .2,
            None
        );
        assert!(identitas_bersih(&baris("199203142014031001", "   ", None)).is_err());
        assert!(identitas_bersih(&baris("199203142014031001", "A", Some("X"))).is_err());
        assert!(identitas_bersih(&baris("1", "A", None)).is_err());
        assert!(identitas_bersih(&baris("199203142014031001", &"A".repeat(256), None)).is_err());
    }
}
