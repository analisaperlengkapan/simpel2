use serde::{Deserialize, Serialize};

/// Model data utama untuk Perkara Datun (Perdata dan Tata Usaha Negara)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DatunCase {
    /// ID unik perkara
    pub id: String,
    /// Nomor Surat Kuasa Khusus (SKK)
    pub no_skk: String,
    /// Tanggal SKK
    pub tanggal_skk: String,
    /// Instansi Pemohon (e.g., Pemda, BUMN)
    pub instansi_pemohon: String,
    /// Pihak Lawan (e.g., PT. X, Sdr. Y)
    pub pihak_lawan: String,
    /// Judul/Nama Perkara
    pub judul_perkara: String,
    /// Jenis Layanan (Bantuan Hukum, Pertimbangan Hukum, etc.)
    pub jenis_layanan: ServiceType,
    /// Status Perkara
    pub status: CaseStatus,
    /// Jaksa Pengacara Negara (JPN) yang menangani
    pub tim_jpn: Vec<String>,
    /// Potensi Pemulihan Keuangan Negara (Rp)
    pub nilai_pemulihan: Option<f64>,
    /// Posisi Kasus Singkat
    pub posisi_kasus: String,
    /// Tanggal update terakhir
    pub updated_at: String,
}

/// Jenis Layanan Datun
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceType {
    /// Bantuan Hukum (Litigasi)
    BantuanHukumLitigasi,
    /// Bantuan Hukum (Non Litigasi)
    BantuanHukumNonLitigasi,
    /// Pertimbangan Hukum (Legal Opinion)
    LegalOpinion,
    /// Pertimbangan Hukum (Legal Assistance)
    LegalAssistance,
    /// Pertimbangan Hukum (Legal Audit)
    LegalAudit,
    /// Penegakan Hukum
    PenegakanHukum,
    /// Tindakan Hukum Lain
    TindakanHukumLain,
    /// Pelayanan Hukum
    PelayananHukum,
}

impl ServiceType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::BantuanHukumLitigasi => "BanHuk Litigasi",
            Self::BantuanHukumNonLitigasi => "BanHuk Non-Litigasi",
            Self::LegalOpinion => "Legal Opinion (LO)",
            Self::LegalAssistance => "Legal Assistance (LA)",
            Self::LegalAudit => "Legal Audit",
            Self::PenegakanHukum => "Penegakan Hukum",
            Self::TindakanHukumLain => "Tindakan Hukum Lain",
            Self::PelayananHukum => "Pelayanan Hukum",
        }
    }

    pub fn color_class(&self) -> &'static str {
        match self {
            Self::BantuanHukumLitigasi | Self::BantuanHukumNonLitigasi => {
                "text-blue-600 bg-blue-50"
            }
            Self::LegalOpinion | Self::LegalAssistance | Self::LegalAudit => {
                "text-purple-600 bg-purple-50"
            }
            Self::PenegakanHukum => "text-red-600 bg-red-50",
            Self::TindakanHukumLain => "text-orange-600 bg-orange-50",
            Self::PelayananHukum => "text-green-600 bg-green-50",
        }
    }
}

/// Status Perkara Datun
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CaseStatus {
    /// Permohonan Masuk
    Permohonan,
    /// Telaah Kasus
    Telaah,
    /// Proses Penanganan (Persidangan/Mediasi)
    Proses,
    /// Selesai / Putusan / Kesepakatan
    Selesai,
    /// Arsip
    Arsip,
}

impl CaseStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Permohonan => "Permohonan",
            Self::Telaah => "Telaah",
            Self::Proses => "Proses",
            Self::Selesai => "Selesai",
            Self::Arsip => "Arsip",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Permohonan => "text-gray-600 bg-gray-50",
            Self::Telaah => "text-yellow-600 bg-yellow-50",
            Self::Proses => "text-blue-600 bg-blue-50",
            Self::Selesai => "text-green-600 bg-green-50",
            Self::Arsip => "text-gray-400 bg-gray-100",
        }
    }
}

/// Tingkat Prioritas
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PriorityLevel {
    Rendah,
    Sedang,
    Tinggi,
}

impl PriorityLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Rendah => "Rendah",
            Self::Sedang => "Sedang",
            Self::Tinggi => "Tinggi",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Rendah => "text-gray-500",
            Self::Sedang => "text-blue-500",
            Self::Tinggi => "text-red-500",
        }
    }
}

/// Jenis Pemohon
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrincipalType {
    PemerintahPusat,
    PemerintahDaerah,
    BUMN,
    BUMD,
    LembagaNegara,
    Masyarakat, // Untuk Pelayanan Hukum
}
