use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use garde::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationType {
    SurveillanceOperation,
    InformationGathering,
    CounterIntelligence,
    CriminalInvestigation,
    CorruptionMonitoring,
    TerrorismPrevention,
    CyberSecurity,
    FinancialCrimeTracking,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationStatus {
    Planning,
    Active,
    OnHold,
    Completed,
    Terminated,
    UnderReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PriorityLevel {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClassificationLevel {
    TopSecret,
    Secret,
    Confidential,
    Restricted,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateOperationRequest {
    #[garde(length(min = 3, max = 100))]
    pub operation_name: String,

    #[garde(skip)]
    pub operation_type: OperationType,

    #[garde(skip)]
    pub priority: PriorityLevel,

    #[garde(length(min = 10))]
    pub target_description: String,

    #[garde(skip)]
    pub classification_level: ClassificationLevel,

    pub budget_allocated: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelOperation {
    pub id: Uuid,
    pub operation_name: String,
    pub operation_type: OperationType,
    pub status: OperationStatus,
    pub priority: PriorityLevel,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub target_description: String,
    pub assigned_agents: Vec<String>,
    pub classification_level: ClassificationLevel,
    pub progress_percentage: u8,
    pub collected_data_count: u32,
    pub reports_generated: u32,
    pub budget_allocated: f64,
    pub budget_used: f64,
}

impl IntelOperation {
    pub fn new(req: CreateOperationRequest) -> Self {
        Self {
            id: Uuid::new_v4(),
            operation_name: req.operation_name,
            operation_type: req.operation_type,
            status: OperationStatus::Planning,
            priority: req.priority,
            start_date: Utc::now(),
            end_date: None,
            target_description: req.target_description,
            assigned_agents: Vec::new(),
            classification_level: req.classification_level,
            progress_percentage: 0,
            collected_data_count: 0,
            reports_generated: 0,
            budget_allocated: req.budget_allocated,
            budget_used: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportType {
    SituationReport,
    IntelligenceAssessment,
    ThreatAnalysis,
    OperationalUpdate,
    FinalReport,
    IncidentReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewStatus {
    Draft,
    UnderReview,
    Approved,
    Rejected,
    RequiresRevision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelReport {
    pub id: Uuid,
    pub operation_id: Uuid,
    pub title: String,
    pub report_type: ReportType,
    pub created_date: DateTime<Utc>,
    pub author: String,
    pub classification: ClassificationLevel,
    pub summary: String,
    pub key_findings: Vec<String>,
    pub recommendations: Vec<String>,
    pub attachments_count: u32,
    pub review_status: ReviewStatus,
}
