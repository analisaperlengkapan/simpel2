use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a military criminal case
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MilitaryCase {
    pub id: String,
    pub case_number: String,
    pub case_type: CaseType,
    pub title: String,
    pub description: String,
    pub status: CaseStatus,
    pub priority: CasePriority,
    pub created_date: DateTime<Utc>,
    pub updated_date: DateTime<Utc>,
    pub assigned_investigator: String,
    pub unit_involved: String,
    pub location: String,
    pub suspects_count: u32,
    pub evidence_count: u32,
}

/// Represents a military suspect
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MilitarySuspect {
    pub id: String,
    pub nrp: String, // Nomor Registrasi Pokok
    pub name: String,
    pub rank: MilitaryRank,
    pub unit: String,
    pub position: String,
    pub case_id: String,
    pub status: SuspectStatus,
    pub arrest_date: Option<DateTime<Utc>>,
    pub detention_status: DetentionStatus,
    pub charges: Vec<String>,
    pub contact_info: ContactInfo,
}

/// Represents investigation progress
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Investigation {
    pub id: String,
    pub case_id: String,
    pub investigator_name: String,
    pub investigation_type: InvestigationType,
    pub status: InvestigationStatus,
    pub start_date: DateTime<Utc>,
    pub target_completion: Option<DateTime<Utc>>,
    pub progress_percentage: u8,
    pub findings: Vec<Finding>,
    pub notes: String,
    pub next_actions: Vec<String>,
}

/// Represents evidence in a military case
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub case_id: String,
    pub evidence_type: EvidenceType,
    pub description: String,
    pub collected_date: DateTime<Utc>,
    pub collected_by: String,
    pub location_found: String,
    pub chain_of_custody: Vec<CustodyRecord>,
    pub status: EvidenceStatus,
    pub analysis_results: Option<String>,
}

/// Contact information for suspects
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ContactInfo {
    pub phone: Option<String>,
    pub emergency_contact: Option<String>,
    pub address: Option<String>,
    pub next_of_kin: Option<String>,
}

/// Chain of custody record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustodyRecord {
    pub timestamp: DateTime<Utc>,
    pub handler: String,
    pub action: String,
    pub location: String,
}

/// Investigation findings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub description: String,
    pub significance: FindingSignificance,
    pub evidence_ids: Vec<String>,
    pub date_discovered: DateTime<Utc>,
}

/// Military case statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MilitaryStatistics {
    pub total_cases: u32,
    pub active_cases: u32,
    pub completed_cases: u32,
    pub total_suspects: u32,
    pub suspects_in_custody: u32,
    pub conviction_rate: f32,
    pub average_investigation_time: u32, // in days
}

// Enums for various classifications

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CaseType {
    Corruption,
    Desertion,
    Insubordination,
    TheftMilitaryProperty,
    Violence,
    DrugOffense,
    EspionageTraitorism,
    AbusePower,
    Fraud,
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CaseStatus {
    Reported,
    UnderInvestigation,
    EvidenceCollection,
    SuspectIdentified,
    AwaitingTrial,
    InTrial,
    Concluded,
    Dismissed,
    Appealed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CasePriority {
    Low,
    Medium,
    High,
    Critical,
    TopSecret,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MilitaryRank {
    // Enlisted Personnel
    Tamtama,
    KopralDua,
    KopralSatu,
    KopralKepala,
    Sersan,
    SerSan,
    SerKep,
    SerMa,

    // Officers
    Letda,
    Lettu,
    Kapten,
    Mayor,
    Letkol,
    Kolonel,
    Brigjen,
    Mayjen,
    Letjen,
    Jenderal,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SuspectStatus {
    PersonOfInterest,
    Suspect,
    Accused,
    Convicted,
    Acquitted,
    Fugitive,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DetentionStatus {
    Free,
    Detained,
    OnBail,
    HousearRest,
    MilitaryConfinement,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum InvestigationType {
    Preliminary,
    Formal,
    SpecialInvestigation,
    CourtMartialPrep,
    Appeal,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum InvestigationStatus {
    Initiated,
    InProgress,
    OnHold,
    Completed,
    Terminated,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EvidenceType {
    Physical,
    Documentary,
    Digital,
    Testimonial,
    Photographic,
    Audio,
    Video,
    Forensic,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EvidenceStatus {
    Collected,
    UnderAnalysis,
    Analyzed,
    Preserved,
    Presented,
    Destroyed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FindingSignificance {
    Minor,
    Moderate,
    Significant,
    Critical,
    Conclusive,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
}
