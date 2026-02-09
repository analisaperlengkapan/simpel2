use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Jenis kejahatan khusus
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpecialCrimeType {
    Corruption,         // Korupsi
    MoneyLaundering,    // Pencucian Uang
    Terrorism,          // Terorisme
    HumanTrafficking,   // Perdagangan Manusia
    Cybercrime,         // Kejahatan Siber
    EnvironmentalCrime, // Kejahatan Lingkungan
    OrganizedCrime,     // Kejahatan Terorganisir
    DrugTrafficking,    // Perdagangan Narkoba
}

/// Status kasus khusus
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpecialCaseStatus {
    Investigation, // Penyidikan
    Evidence,      // Pengumpulan Bukti
    Analysis,      // Analisis
    Prosecution,   // Penuntutan
    Trial,         // Persidangan
    Appeal,        // Banding
    Execution,     // Eksekusi
    Closed,        // Ditutup
}

/// Prioritas kasus khusus
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpecialCasePriority {
    Urgent, // Mendesak
    High,   // Tinggi
    Medium, // Sedang
    Low,    // Rendah
}

/// Instansi terkait
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelatedAgency {
    KPK,   // Komisi Pemberantasan Korupsi
    BNN,   // Badan Narkotika Nasional
    BNPT,  // Badan Nasional Penanggulangan Terorisme
    PPATK, // Pusat Pelaporan dan Analisis Transaksi Keuangan
    KLHK,  // Kementerian Lingkungan Hidup
    Polri, // Kepolisian
    TNI,   // Tentara Nasional Indonesia
    BEA,   // Bea dan Cukai
}

/// Tingkat klasifikasi kasus
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClassificationLevel {
    TopSecret,    // Sangat Rahasia
    Secret,       // Rahasia
    Confidential, // Terbatas
    Internal,     // Internal
    Public,       // Publik
}

/// Status tersangka dalam kasus khusus
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpecialSuspectStatus {
    Identified, // Teridentifikasi
    Wanted,     // DPO (Daftar Pencarian Orang)
    Detained,   // Ditahan
    Released,   // Dibebaskan
    Convicted,  // Divonis
    Fled,       // Melarikan Diri
}

/// Kasus pidana khusus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialCase {
    pub id: String,
    pub case_number: String,
    pub crime_type: SpecialCrimeType,
    pub title: String,
    pub description: String,
    pub status: SpecialCaseStatus,
    pub priority: SpecialCasePriority,
    pub classification: ClassificationLevel,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub lead_investigator: String,
    pub team_members: Vec<String>,
    pub related_agencies: Vec<RelatedAgency>,
    pub location: String,
    pub estimated_loss: Option<f64>,
    pub suspects_count: i32,
    pub evidence_count: i32,
    pub witnesses_count: i32,
}

/// Tersangka dalam kasus khusus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialSuspect {
    pub id: String,
    pub case_id: String,
    pub name: String,
    pub identity_number: String,
    pub nationality: String,
    pub age: i32,
    pub occupation: String,
    pub address: String,
    pub status: SpecialSuspectStatus,
    pub role_in_crime: String,
    pub arrest_date: Option<DateTime<Utc>>,
    pub charges: Vec<String>,
    pub threat_level: String,
}

/// Penyidikan khusus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialInvestigation {
    pub id: String,
    pub case_id: String,
    pub investigation_type: String,
    pub lead_investigator: String,
    pub team_size: i32,
    pub status: String,
    pub start_date: DateTime<Utc>,
    pub target_completion: DateTime<Utc>,
    pub progress_percentage: f32,
    pub budget_allocated: f64,
    pub budget_used: f64,
    pub findings: Vec<String>,
    pub challenges: Vec<String>,
}

/// Bukti khusus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialEvidence {
    pub id: String,
    pub case_id: String,
    pub evidence_type: String,
    pub description: String,
    pub collected_date: DateTime<Utc>,
    pub collected_by: String,
    pub location_found: String,
    pub chain_of_custody: Vec<String>,
    pub analysis_status: String,
    pub analysis_results: Option<String>,
    pub classification: ClassificationLevel,
}

/// Analisis khusus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialAnalysis {
    pub id: String,
    pub case_id: String,
    pub analysis_type: String,
    pub analyst_name: String,
    pub start_date: DateTime<Utc>,
    pub completion_date: Option<DateTime<Utc>>,
    pub status: String,
    pub methodology: String,
    pub findings: Vec<String>,
    pub recommendations: Vec<String>,
    pub risk_assessment: String,
}

/// Operasi khusus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialOperation {
    pub id: String,
    pub case_id: String,
    pub operation_name: String,
    pub operation_type: String,
    pub commander: String,
    pub team_size: i32,
    pub planned_date: DateTime<Utc>,
    pub execution_date: Option<DateTime<Utc>>,
    pub status: String,
    pub objectives: Vec<String>,
    pub resources_needed: Vec<String>,
    pub risk_level: String,
    pub success_rate: Option<f32>,
}

/// Kerjasama internasional
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternationalCooperation {
    pub id: String,
    pub case_id: String,
    pub partner_country: String,
    pub partner_agency: String,
    pub cooperation_type: String,
    pub request_date: DateTime<Utc>,
    pub response_date: Option<DateTime<Utc>>,
    pub status: String,
    pub information_shared: Vec<String>,
    pub assistance_provided: Vec<String>,
    pub contact_person: String,
}

/// Statistik PIDSUS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PidsusStatistics {
    pub total_cases: i32,
    pub active_cases: i32,
    pub closed_cases: i32,
    pub total_suspects: i32,
    pub convicted_suspects: i32,
    pub conviction_rate: f32,
    pub total_evidence: i32,
    pub total_recovered_assets: f64,
    pub average_case_duration: i32,
    pub success_rate: f32,
    pub international_cases: i32,
}
