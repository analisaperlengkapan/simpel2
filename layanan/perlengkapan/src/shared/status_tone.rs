//! Nada semantik sebuah status workflow, dikirim bersama labelnya.
//!
//! Latar: label status pernah disalin ke frontend sebagai `match` atas nama
//! state (`"ACTIVE" => "Aktif"`). Salinan itu tumbuh empat kali dengan celah
//! yang berbeda-beda — negara bagian V035 (`SUBMITTED_APPROVER_SATKER`,
//! `REVISI_OPERATOR`) dan V029 (`KONSEP_SK_WILAYAH_GENERATED`,
//! `SK_SIGNED_WILAYAH`) jatuh ke lengan `_ =>` dan tampil sebagai "Lainnya",
//! sementara tabel monitoring tidak memetakan sama sekali dan mencetak
//! `ACTIVE` mentah ke layar.
//!
//! Akarnya: kosakata status hidup di dua tempat. Perbaikannya bukan menambal
//! lengan yang hilang (itu hanya menunda salinan kelima) melainkan
//! **memindahkan seluruh kosakata ke sisi enum** — satu-satunya hal yang
//! benar-benar tahu daftar statusnya. Backend mengirim `status_label` (teks
//! untuk manusia) dan `status_tone` (kelas semantik), frontend hanya memetakan
//! lima nada ini ke kelas CSS. Daftar lima itu **lengkap menurut konstruksi**:
//! menambah status workflow baru tidak bisa lagi membuatnya usang.

use serde::{Deserialize, Serialize};

/// Kelas semantik badge status. Sengaja hanya lima dan tidak bertambah
/// seiring bertambahnya status workflow — itulah yang membuat peta di
/// frontend tidak bisa ketinggalan.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusTone {
    /// Belum bergerak / tanpa muatan penilaian (draft, dibatalkan).
    /// Juga nada untuk state yang tak dikenal — netral tidak mengklaim apa pun.
    #[default]
    Neutral,
    /// Sedang berjalan, menunggu pihak lain.
    Info,
    /// Langkah berhasil / selesai.
    Success,
    /// Perlu tindakan pengaju, atau tenggat terlampaui.
    Warning,
    /// Berhenti dengan hasil negatif.
    Danger,
}

impl StatusTone {
    /// Nilai yang dikirim ke frontend. Sama persis dengan representasi serde-nya.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Danger => "danger",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `as_str` dan serde harus sepakat — frontend mencocokkan string itu,
    /// jadi keduanya menyimpang berarti badge kehilangan warnanya diam-diam.
    #[test]
    fn as_str_matches_the_serde_representation() {
        for tone in [
            StatusTone::Neutral,
            StatusTone::Info,
            StatusTone::Success,
            StatusTone::Warning,
            StatusTone::Danger,
        ] {
            let json = serde_json::to_string(&tone).expect("tone serialises");
            assert_eq!(json, format!("\"{}\"", tone.as_str()));
        }
    }
}
