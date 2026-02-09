use serde::{Deserialize, Serialize};

/// Audit Management
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Audit {
    pub id: String,
    pub nomor_audit: String,
    pub judul_audit: String,
    pub jenis_audit: AuditType,
    pub status: AuditStatus,
    pub tanggal_mulai: String,
    pub tanggal_selesai: Option<String>,
    pub auditor_utama: String,
    pub tim_audit: Vec<String>,
    pub scope_audit: String,
    pub objektif: String,
    pub progress: u8,
    pub temuan_count: u32,
    pub prioritas: Priority,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AuditType {
    Internal,
    External,
    Compliance,
    Operational,
    Financial,
    IT,
    Risk,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AuditStatus {
    Planning,
    InProgress,
    Review,
    Completed,
    Cancelled,
    OnHold,
}

/// Findings Management
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Finding {
    pub id: String,
    pub audit_id: String,
    pub nomor_temuan: String,
    pub kategori: FindingCategory,
    pub deskripsi: String,
    pub kondisi: String,
    pub kriteria: String,
    pub sebab: String,
    pub akibat: String,
    pub rekomendasi: String,
    pub prioritas: Priority,
    pub status: FindingStatus,
    pub pic_responsible: String,
    pub target_completion: String,
    pub actual_completion: Option<String>,
    pub evidence: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FindingCategory {
    Financial,
    Operational,
    Compliance,
    Security,
    Process,
    System,
    Documentation,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FindingStatus {
    Open,
    InProgress,
    UnderReview,
    Closed,
    Overdue,
    Recurring,
}

/// Monitoring and Compliance
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MonitoringItem {
    pub id: String,
    pub nama_sistem: String,
    pub deskripsi: String,
    pub kategori: MonitoringCategory,
    pub status: MonitoringStatus,
    pub compliance_rate: f32,
    pub last_check: String,
    pub next_check: String,
    pub frequency: MonitoringFrequency,
    pub pic_monitoring: String,
    pub alert_threshold: f32,
    pub current_score: f32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MonitoringCategory {
    Financial,
    Operational,
    Security,
    Compliance,
    Performance,
    Risk,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MonitoringStatus {
    Normal,
    Warning,
    Critical,
    Unknown,
    Maintenance,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MonitoringFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annually,
    OnDemand,
}

/// Follow-up Actions
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FollowUpAction {
    pub id: String,
    pub finding_id: String,
    pub action_description: String,
    pub pic_responsible: String,
    pub target_date: String,
    pub actual_date: Option<String>,
    pub status: ActionStatus,
    pub progress: u8,
    pub evidence_completion: Vec<String>,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ActionStatus {
    NotStarted,
    InProgress,
    Completed,
    Overdue,
    Cancelled,
    OnHold,
}

/// Reporting
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SupervisionReport {
    pub id: String,
    pub report_number: String,
    pub title: String,
    pub report_type: ReportType,
    pub period_start: String,
    pub period_end: String,
    pub executive_summary: String,
    pub findings_summary: String,
    pub recommendations: Vec<String>,
    pub status: ReportStatus,
    pub generated_by: String,
    pub approved_by: Option<String>,
    pub generated_at: String,
    pub approved_at: Option<String>,
    pub file_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ReportType {
    Monthly,
    Quarterly,
    Annual,
    AuditReport,
    ComplianceReport,
    MonitoringReport,
    SpecialReport,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ReportStatus {
    Draft,
    UnderReview,
    Approved,
    Published,
    Archived,
}

/// Risk Assessment
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RiskAssessment {
    pub id: String,
    pub risk_title: String,
    pub description: String,
    pub category: RiskCategory,
    pub likelihood: RiskLevel,
    pub impact: RiskLevel,
    pub risk_score: u8,
    pub mitigation_strategy: String,
    pub control_measures: Vec<String>,
    pub owner: String,
    pub review_date: String,
    pub status: RiskStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RiskCategory {
    Financial,
    Operational,
    Strategic,
    Compliance,
    Reputation,
    Technology,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RiskStatus {
    Active,
    Mitigated,
    Accepted,
    Transferred,
    Avoided,
}

/// Common Priority Level
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Dashboard Statistics
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SupervisionStats {
    pub active_audits: u32,
    pub total_findings: u32,
    pub open_findings: u32,
    pub overdue_actions: u32,
    pub compliance_rate: f32,
    pub monitoring_items: u32,
    pub critical_risks: u32,
    pub pending_reports: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}
