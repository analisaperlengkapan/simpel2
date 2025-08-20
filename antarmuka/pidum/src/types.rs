//! Types untuk PIDUM (Penyidikan Pidana Umum) microfrontend

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Struktur data perkara pidana umum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralCase {
    pub case_number: String,
    pub crime_type: GeneralCrimeType,
    pub status: GeneralCaseStatus,
    pub priority: CasePriority,
    pub defendant_count: u32,
    pub witness_count: u32,
    pub evidence_count: u32,
    pub prosecutor: String,
    pub investigator: String,
    pub registered_date: DateTime<Utc>,
    pub estimated_trial_date: Option<DateTime<Utc>>,
    pub court: String,
    pub case_summary: String,
    pub total_loss: Option<f64>,
}

/// Jenis tindak pidana umum
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GeneralCrimeType {
    Theft,          // Pencurian
    Fraud,          // Penipuan
    Embezzlement,   // Penggelapan
    Assault,        // Penganiayaan
    DrugPossession, // Narkotika tingkat ringan
    Traffic,        // Lalu lintas
    Domestic,       // KDRT
    Property,       // Properti
    Commercial,     // Perdagangan
    Other,          // Lainnya
}

impl GeneralCrimeType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Theft => "Pencurian",
            Self::Fraud => "Penipuan",
            Self::Embezzlement => "Penggelapan",
            Self::Assault => "Penganiayaan",
            Self::DrugPossession => "Narkotika Ringan",
            Self::Traffic => "Lalu Lintas",
            Self::Domestic => "KDRT",
            Self::Property => "Properti",
            Self::Commercial => "Perdagangan",
            Self::Other => "Lainnya",
        }
    }
}

/// Status perkara pidana umum
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GeneralCaseStatus {
    Investigation, // Penyidikan
    Prosecution,   // Penuntutan
    Trial,         // Persidangan
    Verdict,       // Putusan
    Execution,     // Eksekusi
    Appeal,        // Banding
    Cassation,     // Kasasi
    Closed,        // Ditutup
}

impl GeneralCaseStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Investigation => "Penyidikan",
            Self::Prosecution => "Penuntutan",
            Self::Trial => "Persidangan",
            Self::Verdict => "Putusan",
            Self::Execution => "Eksekusi",
            Self::Appeal => "Banding",
            Self::Cassation => "Kasasi",
            Self::Closed => "Ditutup",
        }
    }
}

/// Prioritas perkara
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CasePriority {
    Low,
    Medium,
    High,
    Urgent,
}

impl CasePriority {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Low => "Rendah",
            Self::Medium => "Sedang",
            Self::High => "Tinggi",
            Self::Urgent => "Darurat",
        }
    }
}

/// Data defendant/terdakwa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralDefendant {
    pub name: String,
    pub id_number: String,
    pub age: u32,
    pub address: String,
    pub occupation: String,
    pub status: DefendantStatus,
    pub legal_counsel: Option<String>,
}

/// Status terdakwa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DefendantStatus {
    Suspect,   // Tersangka
    Defendant, // Terdakwa
    Convicted, // Terpidana
    Acquitted, // Bebas
}

impl DefendantStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Suspect => "Tersangka",
            Self::Defendant => "Terdakwa",
            Self::Convicted => "Terpidana",
            Self::Acquitted => "Bebas",
        }
    }
}

/// Data investigation/penyidikan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralInvestigation {
    pub case_number: String,
    pub investigator: String,
    pub start_date: DateTime<Utc>,
    pub target_completion: DateTime<Utc>,
    pub progress_percentage: f64,
    pub investigation_activities: Vec<InvestigationActivity>,
    pub findings: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Aktivitas penyidikan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestigationActivity {
    pub activity_type: String,
    pub description: String,
    pub date: DateTime<Utc>,
    pub officer: String,
    pub location: String,
    pub result: String,
}

/// Data evidence/barang bukti
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralEvidence {
    pub evidence_id: String,
    pub case_number: String,
    pub evidence_type: EvidenceType,
    pub description: String,
    pub seized_date: DateTime<Utc>,
    pub seized_location: String,
    pub condition: EvidenceCondition,
    pub custodian: String,
    pub storage_location: String,
    pub chain_of_custody: Vec<CustodyRecord>,
}

/// Jenis barang bukti
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    Physical,   // Fisik
    Digital,    // Digital
    Document,   // Dokumen
    Financial,  // Keuangan
    Biological, // Biologis
}

impl EvidenceType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Physical => "Fisik",
            Self::Digital => "Digital",
            Self::Document => "Dokumen",
            Self::Financial => "Keuangan",
            Self::Biological => "Biologis",
        }
    }
}

/// Kondisi barang bukti
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceCondition {
    Good,
    Fair,
    Poor,
    Damaged,
}

impl EvidenceCondition {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Good => "Baik",
            Self::Fair => "Cukup",
            Self::Poor => "Buruk",
            Self::Damaged => "Rusak",
        }
    }
}

/// Record custody/penyimpanan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyRecord {
    pub date: DateTime<Utc>,
    pub from_officer: String,
    pub to_officer: String,
    pub purpose: String,
    pub location: String,
}

/// Data prosecution/penuntutan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralProsecution {
    pub case_number: String,
    pub prosecutor: String,
    pub indictment_number: String,
    pub charges: Vec<String>,
    pub trial_schedule: Vec<TrialSession>,
    pub prosecution_status: ProsecutionStatus,
    pub court: String,
    pub judge: String,
}

/// Status penuntutan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProsecutionStatus {
    PreparingIndictment, // Menyiapkan dakwaan
    IndictmentFiled,     // Dakwaan diajukan
    TrialOngoing,        // Sidang berlangsung
    WaitingVerdict,      // Menunggu putusan
    AppealProcess,       // Proses banding
    CassationProcess,    // Proses kasasi
    FinalVerdict,        // Putusan final
}

impl ProsecutionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::PreparingIndictment => "Menyiapkan Dakwaan",
            Self::IndictmentFiled => "Dakwaan Diajukan",
            Self::TrialOngoing => "Sidang Berlangsung",
            Self::WaitingVerdict => "Menunggu Putusan",
            Self::AppealProcess => "Proses Banding",
            Self::CassationProcess => "Proses Kasasi",
            Self::FinalVerdict => "Putusan Final",
        }
    }
}

/// Session sidang
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialSession {
    pub session_number: u32,
    pub date: DateTime<Utc>,
    pub agenda: String,
    pub attendees: Vec<String>,
    pub notes: String,
    pub next_session: Option<DateTime<Utc>>,
}

/// Statistik PIDUM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidumStatistics {
    pub total_cases: u32,
    pub investigation_cases: u32,
    pub prosecution_cases: u32,
    pub trial_cases: u32,
    pub completed_cases: u32,
    pub monthly_new_cases: u32,
    pub case_resolution_rate: f64,
    pub average_processing_time: u32, // dalam hari
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}
