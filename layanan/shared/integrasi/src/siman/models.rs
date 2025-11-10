use serde::{Deserialize, Serialize};

/// Response dari SSO Kemenkeu OAuth2 token endpoint
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimanTokenResponse {
    /// Access token untuk API calls
    pub access_token: String,
    /// Token type (biasanya "Bearer")
    pub token_type: String,
    /// Waktu expire dalam detik
    pub expires_in: u64,
    /// Scope yang diberikan (opsional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Request parameters untuk mendapatkan data aset
#[derive(Debug, Serialize, Clone)]
pub struct SimanDataRequest {
    /// BA_KEY - Kode satuan kerja
    #[serde(rename = "BA_KEY")]
    pub ba_key: String,
    /// ID_1 - Starting row index
    #[serde(rename = "ID_1")]
    pub id_1: String,
    /// ID_2 - Ending row index
    #[serde(rename = "ID_2")]
    pub id_2: String,
}

impl SimanDataRequest {
    /// Membuat request baru dengan pagination
    pub fn new(ba_key: String, start_id: u32, end_id: u32) -> Self {
        Self {
            ba_key,
            id_1: start_id.to_string(),
            id_2: end_id.to_string(),
        }
    }
}

/// Generic response structure untuk SIMAN API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimanResponse<T> {
    /// Status code dari response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    /// Message dari API
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Data hasil query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<T>>,
    /// Error message jika ada
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Response untuk getRowCount endpoint
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RowCountResponse {
    /// Nama tabel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_name: Option<String>,
    /// Total rows
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_count: Option<i64>,
    /// Total rows (alternative field name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}

/// Kategori aset yang tersedia di SIMAN API v2.0
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimanAssetCategory {
    /// Aset Alat Besar
    AlatBesar,
    /// Aset Angkutan Bermotor
    AngkutanBermotor,
    /// Aset Alat Persenjataan
    AlatPersenjataan,
    /// Aset Tak Berwujud
    TakBerwujud,
    /// Aset Bangunan Air
    BangunanAir,
    /// Aset Gedung dan Bangunan
    GedungBangunan,
    /// Aset Instalasi dan Jaringan
    InstalasiJaringan,
    /// Aset Jalan dan Jembatan
    JalandanJembatan,
    /// Aset Non-TIK
    NonTIK,
    /// Aset Rumah
    Rumah,
    /// Aset Tanah
    Tanah,
    /// Aset Tetap Lainnya
    TetapLainnya,
    /// Konstruksi Dalam Pengerjaan (KDP)
    KDP,
    /// Aset Khusus TIK
    KhususTIK,
    /// Aset Tetap Renovasi
    TetapRenovasi,
}

impl SimanAssetCategory {
    /// Mendapatkan nama endpoint untuk kategori aset
    pub fn endpoint(&self) -> &'static str {
        match self {
            Self::AlatBesar => "getAsetAlatBesar",
            Self::AngkutanBermotor => "getAsetAngkutanBermotor",
            Self::AlatPersenjataan => "getAsetAlatPersenjataan",
            Self::TakBerwujud => "getAsetTakBerwujud",
            Self::BangunanAir => "getAsetBangunanAir",
            Self::GedungBangunan => "getAsetGedungBangunan",
            Self::InstalasiJaringan => "getAsetInstalasiJaringan",
            Self::JalandanJembatan => "getAsetJalandanJembatan",
            Self::NonTIK => "getAsetNonTIK",
            Self::Rumah => "getAsetRumah",
            Self::Tanah => "getAsetTanah",
            Self::TetapLainnya => "getAsetTetapLainnya",
            Self::KDP => "getAsetKDP",
            Self::KhususTIK => "getAsetKhususTIK",
            Self::TetapRenovasi => "getAsetTetapRenovasi",
        }
    }

    /// Mendapatkan nama tabel untuk getRowCount
    /// Sesuai dengan Panduan Penggunaan Web Service SLDK-Kejaksaan RI (Tabel 1)
    pub fn table_name(&self) -> &'static str {
        match self {
            Self::AlatBesar => "SIMAN2_M_ASET_ALAT_BESAR",
            Self::AngkutanBermotor => "SIMAN2_M_ASET_ANGKUTAN_BERMOTOR",
            Self::AlatPersenjataan => "SIMAN2_M_ASET_ALAT_PERSENJATAAN",
            Self::TakBerwujud => "SIMAN2_M_ASET_ASET_TAK_BERWUJUD", // Ada prefix ASET_ ganda
            Self::BangunanAir => "SIMAN2_M_ASET_BANGUNAN_AIR",
            Self::GedungBangunan => "SIMAN2_M_ASET_GEDUNG_BANGUNAN",
            Self::InstalasiJaringan => "SIMAN2_M_ASET_INSTALASI_JARINGAN",
            Self::JalandanJembatan => "SIMAN2_M_ASET_JALAN_DAN_JEMBATAN", // Ada "_DAN_"
            Self::NonTIK => "SIMAN2_M_ASET_NON_TIK",
            Self::Rumah => "SIMAN2_M_ASET_RUMAH",
            Self::Tanah => "SIMAN2_M_ASET_TANAH",
            Self::TetapLainnya => "SIMAN2_M_ASET_ASET_TETAP_LAINNYA", // FIXED: Dengan prefix ASET_ ganda
            Self::KDP => "SIMAN2_M_ASET_KDP",
            Self::KhususTIK => "SIMAN2_M_ASET_KHUSUS_TIK",
            Self::TetapRenovasi => "SIMAN2_M_ASET_ASET_TETAP_RENOVASI", // FIXED: Dengan prefix ASET_ ganda
        }
    }

    /// Mendapatkan nama kategori aset (SHORT FORM - sesuai database column `kategori_aset`)
    /// Gunakan method ini untuk query database dan menyimpan data
    pub fn description(&self) -> &'static str {
        match self {
            Self::AlatBesar => "Alat Besar",
            Self::AngkutanBermotor => "Angkutan Bermotor",
            Self::AlatPersenjataan => "Alat Persenjataan",
            Self::TakBerwujud => "Tak Berwujud",
            Self::BangunanAir => "Bangunan Air",
            Self::GedungBangunan => "Gedung Bangunan",
            Self::InstalasiJaringan => "Instalasi Jaringan",
            Self::JalandanJembatan => "Jalan dan Jembatan",
            Self::NonTIK => "Non TIK",
            Self::Rumah => "Rumah",
            Self::Tanah => "Tanah",
            Self::TetapLainnya => "Tetap Lainnya",
            Self::KDP => "KDP",
            Self::KhususTIK => "Khusus TIK",
            Self::TetapRenovasi => "Tetap Renovasi",
        }
    }

    /// Mendapatkan nama lengkap kategori aset (LONG FORM - untuk display/UI)
    /// Gunakan method ini untuk menampilkan ke user
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::AlatBesar => "Aset Alat Besar",
            Self::AngkutanBermotor => "Aset Angkutan Bermotor",
            Self::AlatPersenjataan => "Aset Alat Persenjataan",
            Self::TakBerwujud => "Aset Tak Berwujud",
            Self::BangunanAir => "Aset Bangunan Air",
            Self::GedungBangunan => "Aset Gedung dan Bangunan",
            Self::InstalasiJaringan => "Aset Instalasi dan Jaringan",
            Self::JalandanJembatan => "Aset Jalan dan Jembatan",
            Self::NonTIK => "Peralatan dan Mesin Non-TIK",
            Self::Rumah => "Rumah Negara",
            Self::Tanah => "Tanah",
            Self::TetapLainnya => "Aset Tetap Lainnya",
            Self::KDP => "Konstruksi Dalam Pengerjaan (KDP)",
            Self::KhususTIK => "Peralatan dan Mesin Khusus TIK",
            Self::TetapRenovasi => "Aset Tetap Renovasi",
        }
    }

    /// Mendapatkan semua kategori aset
    pub fn all() -> Vec<Self> {
        vec![
            Self::AlatBesar,
            Self::AngkutanBermotor,
            Self::AlatPersenjataan,
            Self::TakBerwujud,
            Self::BangunanAir,
            Self::GedungBangunan,
            Self::InstalasiJaringan,
            Self::JalandanJembatan,
            Self::NonTIK,
            Self::Rumah,
            Self::Tanah,
            Self::TetapLainnya,
            Self::KDP,
            Self::KhususTIK,
            Self::TetapRenovasi,
        ]
    }

    /// Parse dari string kategori_aset di database ke enum
    /// Returns None jika string tidak valid
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Alat Besar" => Some(Self::AlatBesar),
            "Angkutan Bermotor" => Some(Self::AngkutanBermotor),
            "Alat Persenjataan" => Some(Self::AlatPersenjataan),
            "Tak Berwujud" => Some(Self::TakBerwujud),
            "Bangunan Air" => Some(Self::BangunanAir),
            "Gedung Bangunan" => Some(Self::GedungBangunan),
            "Instalasi Jaringan" => Some(Self::InstalasiJaringan),
            "Jalan dan Jembatan" => Some(Self::JalandanJembatan),
            "Non TIK" => Some(Self::NonTIK),
            "Rumah" => Some(Self::Rumah),
            "Tanah" => Some(Self::Tanah),
            "Tetap Lainnya" => Some(Self::TetapLainnya),
            "KDP" => Some(Self::KDP),
            "Khusus TIK" => Some(Self::KhususTIK),
            "Tetap Renovasi" => Some(Self::TetapRenovasi),
            _ => None,
        }
    }

    /// Get SQL view name untuk kategori ini
    pub fn view_name(&self) -> &'static str {
        match self {
            Self::AlatBesar => "v_siman_aset_alat_besar",
            Self::AngkutanBermotor => "v_siman_aset_angkutan_bermotor",
            Self::AlatPersenjataan => "v_siman_aset_alat_persenjataan",
            Self::TakBerwujud => "v_siman_aset_tak_berwujud",
            Self::BangunanAir => "v_siman_aset_bangunan_air",
            Self::GedungBangunan => "v_siman_aset_gedung_bangunan",
            Self::InstalasiJaringan => "v_siman_aset_instalasi_jaringan",
            Self::JalandanJembatan => "v_siman_aset_jalan_jembatan",
            Self::NonTIK => "v_siman_aset_non_tik",
            Self::Rumah => "v_siman_aset_rumah",
            Self::Tanah => "v_siman_aset_tanah",
            Self::TetapLainnya => "v_siman_aset_tetap_lainnya",
            Self::KDP => "v_siman_aset_kdp",
            Self::KhususTIK => "v_siman_aset_khusus_tik",
            Self::TetapRenovasi => "v_siman_aset_tetap_renovasi",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_description_matches_database() {
        // Pastikan description() return value yang exact match dengan database
        assert_eq!(SimanAssetCategory::AlatBesar.description(), "Alat Besar");
        assert_eq!(
            SimanAssetCategory::TakBerwujud.description(),
            "Tak Berwujud"
        );
        assert_eq!(
            SimanAssetCategory::GedungBangunan.description(),
            "Gedung Bangunan"
        );
        assert_eq!(SimanAssetCategory::NonTIK.description(), "Non TIK");
        assert_eq!(SimanAssetCategory::KDP.description(), "KDP");
    }

    #[test]
    fn test_from_str_roundtrip() {
        // Test bahwa description() -> from_str() roundtrip works
        for category in SimanAssetCategory::all() {
            let desc = category.description();
            let parsed = SimanAssetCategory::from_str(desc);
            assert!(parsed.is_some(), "Failed to parse: {}", desc);
            assert_eq!(parsed.unwrap(), category);
        }
    }

    #[test]
    fn test_view_names() {
        // Verify view names are correct
        assert_eq!(
            SimanAssetCategory::AlatBesar.view_name(),
            "v_siman_aset_alat_besar"
        );
        assert_eq!(SimanAssetCategory::KDP.view_name(), "v_siman_aset_kdp");
    }

    #[test]
    fn test_all_categories_count() {
        // Harus ada 15 kategori
        assert_eq!(SimanAssetCategory::all().len(), 15);
    }
}
