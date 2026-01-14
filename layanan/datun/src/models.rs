use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Model data utama untuk Perkara Datun (Perdata dan Tata Usaha Negara)
#[derive(Clone, Debug, Serialize, Deserialize)]
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

impl DatunCase {
    pub fn new(
        no_skk: String,
        instansi_pemohon: String,
        pihak_lawan: String,
        judul_perkara: String,
        jenis_layanan: ServiceType,
        posisi_kasus: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            no_skk,
            tanggal_skk: chrono::Local::now().format("%Y-%m-%d").to_string(),
            instansi_pemohon,
            pihak_lawan,
            judul_perkara,
            jenis_layanan,
            status: CaseStatus::Permohonan,
            tim_jpn: vec![],
            nilai_pemulihan: None,
            posisi_kasus,
            updated_at: chrono::Local::now().to_rfc3339(),
        }
    }
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

/// Request payload untuk membuat kasus baru
#[derive(Deserialize, Validate, Debug)]
pub struct CreateCaseRequest {
    #[validate(length(min = 1, message = "Nomor SKK wajib diisi"))]
    pub no_skk: String,
    #[validate(length(min = 1, message = "Instansi Pemohon wajib diisi"))]
    pub instansi_pemohon: String,
    #[validate(length(min = 1, message = "Pihak Lawan wajib diisi"))]
    pub pihak_lawan: String,
    #[validate(length(min = 5, message = "Judul Perkara minimal 5 karakter"))]
    pub judul_perkara: String,
    pub jenis_layanan: ServiceType,
    #[validate(length(min = 10, message = "Posisi Kasus minimal 10 karakter"))]
    pub posisi_kasus: String,
}

/// Request payload untuk update kasus
#[derive(Deserialize, Validate, Debug)]
pub struct UpdateCaseRequest {
    pub status: Option<CaseStatus>,
    pub tim_jpn: Option<Vec<String>>,
    pub nilai_pemulihan: Option<f64>,
}
