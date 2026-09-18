//! Foto pegawai dari MySIMKARI.
//!
//! `integrasi.mysimkari_pegawai.foto` menyimpan NAMA BERKAS, bukan URL —
//! sebuah awalan acak 32 karakter diikuti nama berkas aslinya. Nama itu bebas:
//! contoh nyata dari basis data memuat spasi, tanda kurung, dan titik ganda:
//!
//! ```text
//! Ywv1QHmgThRbGQmXh6BSsU9ENAx3NtkNLODA ATANDIMA.jpeg
//! Qj5B0HYfPR1yL8qBYNP7DS1QD51CzluJSUASTAWA-removebg-preview (1).jpg
//! sYpuyk7nf02ulRufWhNg91Gf5sIWRkrnWhatsApp Image 2024-06-25 at 07.23.25.jpeg
//! ```
//!
//! Ketiganya diverifikasi mengembalikan `200 image/jpeg` dari host media,
//! TANPA autentikasi — tetapi hanya setelah di-percent-encode. Menempelkan
//! nama berkas apa adanya menghasilkan URL rusak untuk sebagian besar pegawai.
//!
//! Rantai datanya sudah utuh sebelum ini (kolom ada, sinkronisasi menulisnya,
//! API mengembalikannya); yang tidak pernah ada adalah penampilnya. Nol `<img>`
//! foto pegawai di kedua frontend.
//!
//! Ada di `lib-core`, bukan `lib-ui`, karena dua sisi memerlukannya: frontend
//! merakit URL untuk `<img>`, dan `layanan-integrasi` merakit URL yang SAMA
//! untuk mengunduh bytenya ke SK izin pemakaian BMN — ia satu-satunya pod yang
//! punya egress ke host media. Satu implementasi, bukan dua yang boleh
//! menyimpang diam-diam.

/// Host penyimpanan foto MySIMKARI.
///
/// Disebut eksplisit, bukan diturunkan dari konfigurasi: ia bukan API SIMPel
/// dan tidak berubah per-lingkungan. Bila kelak berubah, satu tempat ini yang
/// diubah — dan CSP `img-src` harus ikut, kalau tidak browser menolaknya tanpa
/// satu pun pesan di layar.
pub const MEDIA_MYSIMKARI: &str =
    "https://media-mysimkari.kejaksaan.go.id/app-storage/pegawai/images";

/// Percent-encode satu segmen path.
///
/// Ditulis tangan alih-alih menarik dependensi: lib-core dikompilasi untuk
/// wasm32 dan host, dan aturannya cukup sempit — pertahankan huruf, angka, dan
/// `-._~`, sandikan sisanya. Itu lebih ketat dari yang diwajibkan RFC 3986 dan
/// itu disengaja: menyandikan yang sebenarnya aman tidak merusak apa pun,
/// sedangkan melewatkan satu karakter merusak URL-nya.
fn encode_segment(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 8);
    for byte in raw.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(*byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// URL foto pegawai, atau `None` bila tidak ada fotonya.
///
/// `None` untuk nilai kosong maupun yang hanya berisi spasi: keduanya berarti
/// "tidak tercatat", dan mengembalikan URL yang pasti 404 hanya membuat
/// browser mengunduh halaman galat lalu menampilkan ikon rusak.
pub fn foto_pegawai_url(foto: Option<&str>) -> Option<String> {
    let nama = foto?.trim();
    if nama.is_empty() {
        return None;
    }
    Some(format!("{MEDIA_MYSIMKARI}/{}", encode_segment(nama)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kosong_bukan_url() {
        assert_eq!(foto_pegawai_url(None), None);
        assert_eq!(foto_pegawai_url(Some("")), None);
        assert_eq!(foto_pegawai_url(Some("   ")), None);
    }

    #[test]
    fn nama_berkas_nyata_dari_mysimkari_tersandi() {
        // Ketiganya nama berkas SUNGGUHAN dari dump simpelv1, dan ketiganya
        // mengembalikan 200 image/jpeg setelah disandikan seperti ini.
        assert_eq!(
            foto_pegawai_url(Some("Ywv1QHmgThRbGQmXh6BSsU9ENAx3NtkNLODA ATANDIMA.jpeg")).unwrap(),
            format!("{MEDIA_MYSIMKARI}/Ywv1QHmgThRbGQmXh6BSsU9ENAx3NtkNLODA%20ATANDIMA.jpeg")
        );
        let kurung = foto_pegawai_url(Some("SUASTAWA-removebg-preview (1).jpg")).unwrap();
        assert!(
            kurung.ends_with("SUASTAWA-removebg-preview%20%281%29.jpg"),
            "{kurung}"
        );
        let titik_dua =
            foto_pegawai_url(Some("WhatsApp Image 2024-06-25 at 07.23.25.jpeg")).unwrap();
        assert!(
            titik_dua.contains("07.23.25.jpeg"),
            "titik tidak perlu disandikan: {titik_dua}"
        );
        assert!(
            !titik_dua.contains(' '),
            "spasi wajib tersandi: {titik_dua}"
        );
    }

    #[test]
    fn tiap_karakter_tak_aman_tersandi_tepat() {
        // Kanari arah sebaliknya. Diperiksa dengan mencocokkan escape yang
        // BENAR, bukan dengan "tidak memuat karakter itu": `%` menyandi
        // menjadi `%25`, yang tentu saja memuat `%`, dan asersi naif itu
        // gagal pada implementasi yang justru benar.
        for (c, escape) in [
            ('/', "%2F"),
            ('?', "%3F"),
            ('#', "%23"),
            ('&', "%26"),
            ('%', "%25"),
            (' ', "%20"),
            ('(', "%28"),
            (')', "%29"),
            ('\'', "%27"),
        ] {
            let url = foto_pegawai_url(Some(&format!("a{c}b.jpg"))).unwrap();
            assert!(
                url.ends_with(&format!("a{escape}b.jpg")),
                "{c:?} seharusnya menjadi {escape}: {url}"
            );
        }
    }

    #[test]
    fn karakter_aman_tidak_disandikan() {
        // Arah sebaliknya lagi: penyandi yang menyandikan SEGALANYA akan lolos
        // uji di atas sambil menghasilkan URL yang tak perlu jelek.
        let url = foto_pegawai_url(Some("aZ09-._~.jpg")).unwrap();
        assert!(url.ends_with("/aZ09-._~.jpg"), "{url}");
    }
}
