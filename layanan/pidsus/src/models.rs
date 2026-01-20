use chrono::{DateTime, Utc};
use garde::Validate;
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSql, FromSql)]
#[postgres(name = "special_crime_type")]
pub enum SpecialCrimeType {
    Corruption,
    MoneyLaundering,
    Terrorism,
    HumanTrafficking,
    Cybercrime,
    EnvironmentalCrime,
    OrganizedCrime,
    DrugTrafficking,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSql, FromSql)]
#[postgres(name = "special_case_status")]
pub enum SpecialCaseStatus {
    Investigation,
    Evidence,
    Analysis,
    Prosecution,
    Trial,
    Appeal,
    Execution,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSql, FromSql)]
#[postgres(name = "special_case_priority")]
pub enum SpecialCasePriority {
    Urgent,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSql, FromSql)]
#[postgres(name = "classification_level")]
pub enum ClassificationLevel {
    TopSecret,
    Secret,
    Confidential,
    Internal,
    Public,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelatedAgency {
    KPK,
    BNN,
    BNPT,
    PPATK,
    KLHK,
    Polri,
    TNI,
    BEA,
}

// Request Models

#[derive(Debug, Validate, Deserialize)]
pub struct CreateCaseRequest {
    #[garde(length(min = 1, max = 50))]
    pub case_number: String,
    #[garde(skip)]
    pub crime_type: SpecialCrimeType,
    #[garde(length(min = 1, max = 255))]
    pub title: String,
    #[garde(skip)]
    pub description: String,
    #[garde(skip)]
    pub status: SpecialCaseStatus,
    #[garde(skip)]
    pub priority: SpecialCasePriority,
    #[garde(skip)]
    pub classification: ClassificationLevel,
    #[garde(length(min = 1, max = 255))]
    pub lead_investigator: String,
    #[garde(length(min = 1, max = 255))]
    pub location: String,
    #[garde(skip)]
    pub estimated_loss: Option<f64>,
    #[garde(skip)]
    pub team_members: Vec<String>,
    #[garde(skip)]
    pub related_agencies: Vec<String>,
}

// Domain/Response Model

#[derive(Debug, Serialize, Deserialize)]
pub struct SpecialCase {
    pub id: Uuid,
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
    pub location: String,
    pub estimated_loss: Option<f64>,
    pub suspects_count: i32,
    pub evidence_count: i32,
    pub witnesses_count: i32,
    pub team_members: Vec<String>,
    pub related_agencies: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_cases: i64,
    pub active_cases: i64,
    pub closed_cases: i64,
    pub total_suspects: i64,
    pub total_evidence: i64,
    pub total_recovered_assets: f64,
    pub conviction_rate: f64,
}
