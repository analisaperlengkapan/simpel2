use std::sync::{Arc, RwLock};
use crate::models::{DatunCase, ServiceType, CaseStatus};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// In-memory storage for cases (simulating database)
    cases: Arc<RwLock<Vec<DatunCase>>>,
}

impl AppState {
    /// Initialize new state with mock data
    pub fn new() -> Self {
        let mock_data = vec![
            DatunCase {
                id: "DTN001".to_string(),
                no_skk: "SKK-123/A/JA/01/2025".to_string(),
                tanggal_skk: "2025-01-15".to_string(),
                instansi_pemohon: "PT. PLN (Persero)".to_string(),
                pihak_lawan: "PT. Energi Abadi".to_string(),
                judul_perkara: "Wanprestasi Kontrak Pembangunan Gardu Induk".to_string(),
                jenis_layanan: ServiceType::BantuanHukumLitigasi,
                status: CaseStatus::Proses,
                tim_jpn: vec!["Andi SH".to_string(), "Budi SH".to_string()],
                nilai_pemulihan: Some(15_000_000_000.0),
                posisi_kasus: "Tergugat tidak menyelesaikan pekerjaan sesuai deadline kontrak".to_string(),
                updated_at: "2025-02-01T10:00:00Z".to_string(),
            },
            DatunCase {
                id: "DTN002".to_string(),
                no_skk: "SPRIN-LO-45/B/02/2025".to_string(),
                tanggal_skk: "2025-02-01".to_string(),
                instansi_pemohon: "Pemkot Surabaya".to_string(),
                pihak_lawan: "-".to_string(),
                judul_perkara: "Legal Opinion terkait Aset Lahan Pasar Turi".to_string(),
                jenis_layanan: ServiceType::LegalOpinion,
                status: CaseStatus::Telaah,
                tim_jpn: vec!["Siti SH MH".to_string()],
                nilai_pemulihan: None,
                posisi_kasus: "Permohonan pendapat hukum mengenai status HPL lahan".to_string(),
                updated_at: "2025-02-10T14:30:00Z".to_string(),
            },
        ];

        Self {
            cases: Arc::new(RwLock::new(mock_data)),
        }
    }

    /// Add a new case
    pub fn add_case(&self, case: DatunCase) {
        let mut cases = self.cases.write().unwrap();
        cases.push(case);
    }

    /// Get all cases
    pub fn get_cases(&self) -> Vec<DatunCase> {
        self.cases.read().unwrap().clone()
    }

    /// Get a case by ID
    pub fn get_case(&self, id: &str) -> Option<DatunCase> {
        let cases = self.cases.read().unwrap();
        cases.iter().find(|c| c.id == id).cloned()
    }

    /// Update a case
    pub fn update_case(&self, id: &str, status: Option<CaseStatus>, tim_jpn: Option<Vec<String>>, nilai: Option<f64>) -> Option<DatunCase> {
        let mut cases = self.cases.write().unwrap();
        if let Some(case) = cases.iter_mut().find(|c| c.id == id) {
            if let Some(s) = status {
                case.status = s;
            }
            if let Some(t) = tim_jpn {
                case.tim_jpn = t;
            }
            if let Some(n) = nilai {
                case.nilai_pemulihan = Some(n);
            }
            case.updated_at = chrono::Local::now().to_rfc3339();
            return Some(case.clone());
        }
        None
    }
}
