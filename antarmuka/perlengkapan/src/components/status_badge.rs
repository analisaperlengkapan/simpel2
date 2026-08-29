//! Kelas CSS untuk badge status, dipetakan dari nada semantik yang dikirim
//! backend.
//!
//! Frontend dulu menyimpan kosakata statusnya sendiri: `match status {"ACTIVE"
//! => "Aktif", …}`. Salinan itu ada empat, masing-masing dengan celah berbeda,
//! sehingga status V035 (`SUBMITTED_APPROVER_SATKER`, `REVISI_OPERATOR`) dan
//! cabang wilayah V029 (`KONSEP_SK_WILAYAH_GENERATED`, `SK_SIGNED_WILAYAH`)
//! tampil sebagai **"Lainnya"**, sementara tabel monitoring mencetak `ACTIVE`
//! mentah.
//!
//! Sekarang label datang dari `status_label` (diturunkan dari enum di
//! backend) dan yang tersisa di sini hanyalah pemetaan **lima nada** ke kelas
//! Tailwind. Lima itu tidak bertambah saat status workflow bertambah — itulah
//! yang membuat berkas ini tidak bisa ketinggalan lagi.

/// Kelas Tailwind untuk badge, dari nada semantik (`neutral`, `info`,
/// `success`, `warning`, `danger`).
///
/// Nada di luar kelima itu jatuh ke netral: badge tetap terbaca, hanya tanpa
/// muatan warna — jauh lebih baik daripada teks tanpa gaya sama sekali.
pub fn status_tone_classes(tone: &str) -> &'static str {
    match tone {
        "info" => "bg-info-500/10 text-info-300 ring-info-500/20",
        "success" => "bg-success-500/10 text-success-300 ring-success-500/20",
        "warning" => "bg-warning-500/10 text-warning-300 ring-warning-500/20",
        "danger" => "bg-danger-500/10 text-danger-300 ring-danger-500/20",
        _ => "bg-slate-500/10 text-slate-300 ring-slate-500/20",
    }
}

/// Kelas lengkap sebuah badge inline (bentuk + warna) — dipakai tabel dan
/// halaman detail supaya bentuknya seragam.
pub fn status_badge_classes(tone: &str) -> String {
    format!(
        "inline-flex items-center rounded-full px-2.5 py-1 text-xs font-medium ring-1 ring-inset {}",
        status_tone_classes(tone)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_tone_gets_its_own_palette() {
        let tones = ["neutral", "info", "success", "warning", "danger"];
        let mut seen: Vec<&str> = tones.iter().map(|t| status_tone_classes(t)).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), tones.len(), "dua nada berbagi kelas yang sama");
    }

    /// Kanari arah sebaliknya: nada yang tak dikenal harus tetap menghasilkan
    /// kelas yang terbaca, bukan string kosong.
    #[test]
    fn an_unknown_tone_still_renders_readable() {
        assert_eq!(
            status_tone_classes("belum-ada-nadanya"),
            status_tone_classes("neutral")
        );
        assert!(!status_badge_classes("belum-ada-nadanya").is_empty());
    }
}
