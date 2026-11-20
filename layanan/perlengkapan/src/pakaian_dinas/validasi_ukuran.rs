//! Aturan nilai ukuran per grup (V019).
//!
//! Sebuah grup adalah `PILIHAN` (nilainya harus salah satu dari daftar `ms_ukuran` grup
//! itu) atau `TEKS` (isian bebas, mis. nama dada). Aturan ini dipakai di SATU tempat oleh
//! semua jalur yang menyimpan ukuran (pengisian kampanye, profil pegawai, impor), supaya
//! jalur-jalur itu tidak bisa menyimpang: sebelum ini daftar ukuran hanya dijaga oleh
//! pilihan di layar, sehingga permintaan langsung ke API menyimpan nilai apa pun.

use super::models::UkuranGrup;

/// Panjang maksimum isian teks bebas.
pub const MAKS_TEKS: usize = 100;

/// Periksa dan kanonkan satu nilai ukuran terhadap aturan grupnya.
///
/// `daftar` = ukuran yang sah untuk grup `PILIHAN` (diabaikan untuk `TEKS`). Hasilnya nilai
/// yang akan disimpan: untuk `PILIHAN` penulisan dari daftar (jadi `xl` tersimpan `XL`),
/// untuk `TEKS` nilai yang sudah dirapikan spasinya.
pub fn validasi_ukuran(
    grup: &UkuranGrup,
    daftar: &[String],
    nilai: &str,
) -> Result<String, String> {
    let nilai = nilai.trim();
    if nilai.is_empty() {
        return Err("ukuran kosong".to_string());
    }
    if grup.is_teks() {
        if nilai.chars().count() > MAKS_TEKS {
            return Err(format!(
                "isian \"{}\" melebihi {MAKS_TEKS} karakter",
                grup.nama
            ));
        }
        return Ok(nilai.to_string());
    }
    daftar
        .iter()
        .find(|u| u.eq_ignore_ascii_case(nilai))
        .cloned()
        .ok_or_else(|| format!("ukuran \"{nilai}\" tidak ada di daftar grup {}", grup.kode))
}

/// Normalkan kode grup: HURUF BESAR, spasi menjadi `_`, hanya huruf, angka, dan `_`.
pub fn normalisasi_kode_grup(kode: &str) -> Result<String, String> {
    let k = kode.trim().to_uppercase().replace([' ', '-'], "_");
    if k.is_empty() {
        return Err("Kode grup harus diisi".to_string());
    }
    if !k
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
    {
        return Err("Kode grup hanya boleh huruf, angka, dan garis bawah".to_string());
    }
    Ok(k)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grup(tipe: &str) -> UkuranGrup {
        UkuranGrup {
            kode: "BAJU".into(),
            nama: "Baju".into(),
            tipe: tipe.into(),
            urutan: 1,
            is_active: true,
        }
    }

    fn daftar() -> Vec<String> {
        ["S", "M", "L", "XL", "2XL"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn a_choice_must_be_on_the_groups_list() {
        assert_eq!(
            validasi_ukuran(&grup("PILIHAN"), &daftar(), "L"),
            Ok("L".into())
        );
        assert!(validasi_ukuran(&grup("PILIHAN"), &daftar(), "XXXL").is_err());
        assert!(validasi_ukuran(&grup("PILIHAN"), &daftar(), "").is_err());
    }

    #[test]
    fn the_stored_spelling_is_the_lists_not_the_callers() {
        assert_eq!(
            validasi_ukuran(&grup("PILIHAN"), &daftar(), "  xl "),
            Ok("XL".into())
        );
        assert_eq!(
            validasi_ukuran(&grup("PILIHAN"), &daftar(), "2xl"),
            Ok("2XL".into())
        );
    }

    #[test]
    fn free_text_is_trimmed_and_bounded_but_not_matched_against_a_list() {
        assert_eq!(
            validasi_ukuran(&grup("TEKS"), &[], "  E. SILALAHI "),
            Ok("E. SILALAHI".into())
        );
        let panjang = "A".repeat(MAKS_TEKS + 1);
        assert!(validasi_ukuran(&grup("TEKS"), &[], &panjang).is_err());
        assert!(validasi_ukuran(&grup("TEKS"), &[], "   ").is_err());
    }

    #[test]
    fn a_group_code_is_normalised_or_refused() {
        assert_eq!(normalisasi_kode_grup(" nama dada "), Ok("NAMA_DADA".into()));
        assert_eq!(normalisasi_kode_grup("sepatu-pdl"), Ok("SEPATU_PDL".into()));
        assert!(normalisasi_kode_grup("").is_err());
        assert!(normalisasi_kode_grup("topi;DROP").is_err());
    }
}
