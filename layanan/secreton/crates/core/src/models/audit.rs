//! Enhanced Audit Model for SIMKARI Super App
//!
//! Provides comprehensive audit models for Attorney General's Office compliance
//! with hierarchical audit trails, satker-based organization, and compliance flags.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Enhanced audit event for SIMKARI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier
    pub event_id: Uuid,
    /// Timestamp in UTC
    pub timestamp: DateTime<Utc>,
    /// Event type classification
    pub event_type: AuditEventType,
    /// NIP (Nomor Induk Pegawai) of the user performing the action
    pub nip: Option<String>,
    /// Satker code (Kode Satuan Kerja) for organizational hierarchy
    pub satker_code: Option<String>,
    /// Admin level performing the operation
    pub admin_level: Option<AdminLevel>,
    /// Authenc session identifier for correlation
    pub authenc_session_id: Option<String>,
    /// Resource path or identifier being accessed
    pub resource_path: String,
    /// Operation performed on the resource
    pub operation: Operation,
    /// Result of the operation
    pub result: OperationResult,
    /// Security context for the operation
    pub security_context: SecurityContext,
    /// Risk score (0-100) for security assessment
    pub risk_score: Option<f64>,
    /// Compliance flags for kejaksaan operations
    pub compliance_flags: Vec<ComplianceFlag>,
    /// Additional metadata for the audit event
    pub metadata: HashMap<String, serde_json::Value>,
    /// Source IP address of the request
    pub source_ip: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Geolocation information if available
    pub geo_location: Option<GeoLocation>,
    /// Duration of the operation in milliseconds
    pub duration_ms: Option<u64>,
}

/// Types of audit events in SIMKARI system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AuditEventType {
    /// Authentication events (login, logout, token refresh)
    Authentication,
    /// Authorization events (permission checks, role assignments)
    Authorization,
    /// Secret access events (read, write, delete)
    SecretAccess,
    /// Secret modification events (create, update, delete)
    SecretModification,
    /// Administrative operations (user management, role changes)
    AdminOperation,
    /// System configuration changes
    ConfigurationChange,
    /// Security policy violations
    PolicyViolation,
    /// Compliance-related events
    ComplianceEvent,
    /// Cryptographic operations (key generation, encryption, decryption)
    CryptographicOperation,
    /// API access events
    ApiAccess,
    /// Data export/import operations
    DataTransfer,
    /// System maintenance operations
    SystemMaintenance,
}

/// Administrative levels in the Attorney General's Office hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdminLevel {
    /// Administrator for a specific satker (satuan kerja)
    AdminSatker(String),
    /// Administrator for a wilayah (regional level - kejaksaan tinggi)
    AdminWilayah(String),
    /// Administrator for Eselon I level at kejaksaan agung
    AdminEselonI,
    /// Administrator at the central/national level
    AdminPusat,
    /// System administrator (technical operations)
    SystemAdmin,
}

/// Operations that can be performed on resources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Operation {
    /// Read operation (view, list, search)
    Read,
    /// Write operation (create, update)
    Write,
    /// Delete operation
    Delete,
    /// Execute operation (run, process)
    Execute,
    /// Administrative operation (manage, configure)
    Admin,
    /// Audit operation (review, export)
    Audit,
    /// Backup operation
    Backup,
    /// Restore operation
    Restore,
    /// Import operation
    Import,
    /// Export operation
    Export,
}

/// Result of an operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationResult {
    /// Operation completed successfully
    Success,
    /// Operation failed with error details
    Failure(String),
    /// Operation partially completed
    Partial(String),
    /// Operation was denied due to permissions
    Denied(String),
    /// Operation was blocked by security policy
    Blocked(String),
    /// Operation timed out
    Timeout,
}

/// Security context for audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Authentication method used
    pub auth_method: Option<String>,
    /// Token type (JWT, OAuth, etc.)
    pub token_type: Option<String>,
    /// Security level of the operation
    pub security_level: SecurityLevel,
    /// Whether multi-factor authentication was used
    pub mfa_used: bool,
    /// Client certificate information
    pub client_cert_info: Option<String>,
    /// Encryption algorithm used
    pub encryption_algorithm: Option<String>,
    /// Whether the operation involved sensitive data
    pub sensitive_data_involved: bool,
}

/// Security levels for operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityLevel {
    /// Public information, no special security required
    Public,
    /// Internal information, basic security required
    Internal,
    /// Confidential information, enhanced security required
    Confidential,
    /// Secret information, high security required
    Secret,
    /// Top secret information, maximum security required
    TopSecret,
}

/// Compliance flags for kejaksaan operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ComplianceFlag {
    /// Requires approval from higher authority
    RequiresApproval,
    /// Must be reported to supervisory body
    MustReport,
    /// Involves classified information
    ClassifiedData,
    /// Financial transaction compliance
    FinancialCompliance,
    /// Legal proceeding related
    LegalProceeding,
    /// Evidence handling compliance
    EvidenceHandling,
    /// Data retention policy applies
    DataRetention,
    /// Privacy protection required
    PrivacyProtection,
    /// Audit trail required
    AuditTrailRequired,
    /// Regulatory compliance (OJK, BI, etc.)
    RegulatoryCompliance(String),
    /// Internal policy compliance
    InternalPolicy(String),
}

/// Geolocation information for audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    /// Country code (ISO 3166-1 alpha-2)
    pub country: Option<String>,
    /// Province or state
    pub province: Option<String>,
    /// City name
    pub city: Option<String>,
    /// Latitude coordinate
    pub latitude: Option<f64>,
    /// Longitude coordinate
    pub longitude: Option<f64>,
    /// Timezone identifier
    pub timezone: Option<String>,
}

/// Audit trail entry with cryptographic integrity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailEntry {
    /// The audit event
    pub event: AuditEvent,
    /// Sequential number for ordering and integrity
    pub sequence_number: u64,
    /// Hash of the previous entry for chain integrity
    pub previous_hash: Vec<u8>,
    /// Hash of this entry
    pub entry_hash: Vec<u8>,
    /// Digital signature for integrity verification
    pub signature: Vec<u8>,
    /// Node identifier that created this entry
    pub node_id: String,
    /// Merkle tree root for batch verification
    pub merkle_root: Option<Vec<u8>>,
}

/// Hierarchical audit summary for different organizational levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalAuditSummary {
    /// Summary identifier
    pub summary_id: Uuid,
    /// Time period for this summary
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    /// Organizational level this summary covers
    pub organizational_level: OrganizationalLevel,
    /// Total number of audit events in this period
    pub total_events: u64,
    /// Events by type
    pub events_by_type: HashMap<AuditEventType, u64>,
    /// Events by result
    pub events_by_result: HashMap<String, u64>,
    /// Security incidents count
    pub security_incidents: u64,
    /// Compliance violations count
    pub compliance_violations: u64,
    /// Average risk score for the period
    pub average_risk_score: f64,
    /// Top users by activity
    pub top_users: Vec<UserActivitySummary>,
    /// Top resources accessed
    pub top_resources: Vec<ResourceAccessSummary>,
    /// Compliance status by flag
    pub compliance_status: HashMap<ComplianceFlag, ComplianceStatus>,
}

/// Organizational levels in the hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OrganizationalLevel {
    /// National/central level
    Pusat,
    /// Regional level (kejaksaan tinggi)
    Wilayah(String),
    /// Satker level (satuan kerja)
    Satker(String),
    /// Individual user level
    User(String),
}

/// User activity summary for audit reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivitySummary {
    /// NIP of the user
    pub nip: String,
    /// User's name
    pub name: Option<String>,
    /// Satker code
    pub satker_code: String,
    /// Total number of operations
    pub total_operations: u64,
    /// Failed operations count
    pub failed_operations: u64,
    /// Average risk score
    pub average_risk_score: f64,
    /// Most common operation type
    pub most_common_operation: Operation,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
}

/// Resource access summary for audit reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAccessSummary {
    /// Resource path or identifier
    pub resource_path: String,
    /// Resource type
    pub resource_type: String,
    /// Total access count
    pub access_count: u64,
    /// Unique users who accessed this resource
    pub unique_users: u64,
    /// Failed access attempts
    pub failed_attempts: u64,
    /// Average risk score for accesses
    pub average_risk_score: f64,
    /// Last access timestamp
    pub last_access: DateTime<Utc>,
}

/// Compliance status for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    /// Total events with this compliance flag
    pub total_events: u64,
    /// Events that passed compliance checks
    pub compliant_events: u64,
    /// Events that failed compliance checks
    pub non_compliant_events: u64,
    /// Compliance percentage
    pub compliance_percentage: f64,
    /// Last compliance check timestamp
    pub last_check: DateTime<Utc>,
}

/// Audit query parameters for searching audit logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    /// Start time for the query range
    pub start_time: Option<DateTime<Utc>>,
    /// End time for the query range
    pub end_time: Option<DateTime<Utc>>,
    /// Filter by NIP
    pub nip: Option<String>,
    /// Filter by satker code
    pub satker_code: Option<String>,
    /// Filter by admin level
    pub admin_level: Option<AdminLevel>,
    /// Filter by event type
    pub event_type: Option<AuditEventType>,
    /// Filter by operation
    pub operation: Option<Operation>,
    /// Filter by result type
    pub result_type: Option<String>,
    /// Filter by resource path (supports wildcards)
    pub resource_path: Option<String>,
    /// Filter by compliance flags
    pub compliance_flags: Option<Vec<ComplianceFlag>>,
    /// Minimum risk score threshold
    pub min_risk_score: Option<f64>,
    /// Maximum risk score threshold
    pub max_risk_score: Option<f64>,
    /// Source IP address filter
    pub source_ip: Option<String>,
    /// Security level filter
    pub security_level: Option<SecurityLevel>,
    /// Limit number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
    /// Sort order (field, ascending/descending)
    pub sort_by: Option<(String, bool)>,
}

/// Audit configuration for SIMKARI system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Whether audit logging is enabled
    pub enabled: bool,
    /// Minimum security level to audit
    pub min_security_level: SecurityLevel,
    /// Event types to audit
    pub audit_event_types: Vec<AuditEventType>,
    /// Compliance flags that require immediate alerting
    pub alert_compliance_flags: Vec<ComplianceFlag>,
    /// Risk score threshold for alerts
    pub alert_risk_threshold: f64,
    /// Retention policy for audit logs
    pub retention_policy: AuditRetentionPolicy,
    /// Whether to enable real-time monitoring
    pub real_time_monitoring: bool,
    /// SIEM integration settings
    pub siem_integration: Option<SiemIntegrationConfig>,
    /// Hierarchical reporting configuration
    pub hierarchical_reporting: HierarchicalReportingConfig,
}

/// Audit log retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRetentionPolicy {
    /// Default retention period in days
    pub default_retention_days: u32,
    /// Retention by event type
    pub retention_by_type: HashMap<AuditEventType, u32>,
    /// Retention by compliance flag
    pub retention_by_compliance: HashMap<ComplianceFlag, u32>,
    /// Whether to archive old logs
    pub enable_archiving: bool,
    /// Archive location
    pub archive_location: Option<String>,
    /// Compression settings for archived logs
    pub archive_compression: bool,
}

/// SIEM integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiemIntegrationConfig {
    /// SIEM endpoint URL
    pub endpoint: String,
    /// Authentication method
    pub auth_method: SiemAuthMethod,
    /// Batch size for sending events
    pub batch_size: usize,
    /// Batch timeout in seconds
    pub batch_timeout_seconds: u32,
    /// Event types to send to SIEM
    pub event_types: Vec<AuditEventType>,
    /// Minimum risk score to send to SIEM
    pub min_risk_score: f64,
}

/// SIEM authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SiemAuthMethod {
    /// No authentication
    None,
    /// API key authentication
    ApiKey(String),
    /// Basic authentication
    BasicAuth { username: String, password: String },
    /// Bearer token authentication
    BearerToken(String),
    /// Mutual TLS authentication
    MutualTls { cert_path: String, key_path: String },
}

/// Hierarchical reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalReportingConfig {
    /// Whether to enable hierarchical reporting
    pub enabled: bool,
    /// Reporting intervals by organizational level
    pub reporting_intervals: HashMap<OrganizationalLevel, ReportingInterval>,
    /// Report recipients by level
    pub report_recipients: HashMap<OrganizationalLevel, Vec<String>>,
    /// Report formats
    pub report_formats: Vec<ReportFormat>,
}

/// Reporting intervals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportingInterval {
    /// Real-time reporting
    RealTime,
    /// Hourly reports
    Hourly,
    /// Daily reports
    Daily,
    /// Weekly reports
    Weekly,
    /// Monthly reports
    Monthly,
    /// Quarterly reports
    Quarterly,
    /// Annual reports
    Annual,
}

/// Report formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    /// JSON format
    Json,
    /// PDF format
    Pdf,
    /// Excel format
    Excel,
    /// CSV format
    Csv,
    /// HTML format
    Html,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_security_level: SecurityLevel::Internal,
            audit_event_types: vec![
                AuditEventType::Authentication,
                AuditEventType::Authorization,
                AuditEventType::SecretAccess,
                AuditEventType::SecretModification,
                AuditEventType::AdminOperation,
            ],
            alert_compliance_flags: vec![
                ComplianceFlag::RequiresApproval,
                ComplianceFlag::ClassifiedData,
                ComplianceFlag::EvidenceHandling,
            ],
            alert_risk_threshold: 70.0,
            retention_policy: AuditRetentionPolicy::default(),
            real_time_monitoring: true,
            siem_integration: None,
            hierarchical_reporting: HierarchicalReportingConfig::default(),
        }
    }
}

impl Default for AuditRetentionPolicy {
    fn default() -> Self {
        Self {
            default_retention_days: 2555, // 7 years for legal compliance
            retention_by_type: HashMap::new(),
            retention_by_compliance: HashMap::new(),
            enable_archiving: true,
            archive_location: None,
            archive_compression: true,
        }
    }
}

impl Default for HierarchicalReportingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            reporting_intervals: HashMap::new(),
            report_recipients: HashMap::new(),
            report_formats: vec![ReportFormat::Json, ReportFormat::Pdf],
        }
    }
}

impl AuditEvent {
    /// Create a new audit event with basic information
    pub fn new(
        event_type: AuditEventType,
        operation: Operation,
        resource_path: String,
        result: OperationResult,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            nip: None,
            satker_code: None,
            admin_level: None,
            authenc_session_id: None,
            resource_path,
            operation,
            result,
            security_context: SecurityContext::default(),
            risk_score: None,
            compliance_flags: Vec::new(),
            metadata: HashMap::new(),
            source_ip: None,
            user_agent: None,
            geo_location: None,
            duration_ms: None,
        }
    }

    /// Set the user information for this audit event
    pub fn with_user(mut self, nip: String, satker_code: String) -> Self {
        self.nip = Some(nip);
        self.satker_code = Some(satker_code);
        self
    }

    /// Set the admin level for this audit event
    pub fn with_admin_level(mut self, admin_level: AdminLevel) -> Self {
        self.admin_level = Some(admin_level);
        self
    }

    /// Add compliance flags to this audit event
    pub fn with_compliance_flags(mut self, flags: Vec<ComplianceFlag>) -> Self {
        self.compliance_flags = flags;
        self
    }

    /// Set the risk score for this audit event
    pub fn with_risk_score(mut self, risk_score: f64) -> Self {
        self.risk_score = Some(risk_score.clamp(0.0, 100.0));
        self
    }

    /// Add metadata to this audit event
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set the security context for this audit event
    pub fn with_security_context(mut self, security_context: SecurityContext) -> Self {
        self.security_context = security_context;
        self
    }

    /// Calculate risk score based on event properties
    pub fn calculate_risk_score(&mut self) {
        let mut score: f64 = 0.0;

        // Base score by event type
        score += match self.event_type {
            AuditEventType::Authentication => 10.0,
            AuditEventType::Authorization => 15.0,
            AuditEventType::SecretAccess => 30.0,
            AuditEventType::SecretModification => 50.0,
            AuditEventType::AdminOperation => 60.0,
            AuditEventType::ConfigurationChange => 70.0,
            AuditEventType::PolicyViolation => 80.0,
            AuditEventType::CryptographicOperation => 40.0,
            _ => 20.0,
        };

        // Increase score based on operation result
        score += match &self.result {
            OperationResult::Success => 0.0,
            OperationResult::Failure(_) => 20.0,
            OperationResult::Denied(_) => 30.0,
            OperationResult::Blocked(_) => 40.0,
            OperationResult::Timeout => 25.0,
            OperationResult::Partial(_) => 15.0,
        };

        // Increase score based on security level
        score += match self.security_context.security_level {
            SecurityLevel::Public => 0.0,
            SecurityLevel::Internal => 5.0,
            SecurityLevel::Confidential => 15.0,
            SecurityLevel::Secret => 25.0,
            SecurityLevel::TopSecret => 35.0,
        };

        // Increase score for compliance flags
        for flag in &self.compliance_flags {
            score += match flag {
                ComplianceFlag::ClassifiedData => 20.0,
                ComplianceFlag::EvidenceHandling => 25.0,
                ComplianceFlag::RequiresApproval => 15.0,
                ComplianceFlag::LegalProceeding => 30.0,
                _ => 10.0,
            };
        }

        // Decrease score if MFA was used
        if self.security_context.mfa_used {
            score -= 10.0;
        }

        self.risk_score = Some(score.clamp(0.0, 100.0));
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            auth_method: None,
            token_type: None,
            security_level: SecurityLevel::Internal,
            mfa_used: false,
            client_cert_info: None,
            encryption_algorithm: None,
            sensitive_data_involved: false,
        }
    }
}

impl AdminLevel {
    /// Get the hierarchical level as a numeric value (higher = more authority)
    pub fn hierarchy_level(&self) -> u8 {
        match self {
            AdminLevel::AdminSatker(_) => 1,
            AdminLevel::AdminWilayah(_) => 2,
            AdminLevel::AdminEselonI => 3,
            AdminLevel::AdminPusat => 4,
            AdminLevel::SystemAdmin => 5,
        }
    }

    /// Check if this admin level can manage the specified level
    pub fn can_manage(&self, other: &AdminLevel) -> bool {
        self.hierarchy_level() >= other.hierarchy_level()
    }

    /// Get the scope of authority for this admin level
    pub fn authority_scope(&self) -> String {
        match self {
            AdminLevel::AdminSatker(satker) => format!("satker:{}", satker),
            AdminLevel::AdminWilayah(wilayah) => format!("wilayah:{}", wilayah),
            AdminLevel::AdminEselonI => "eselon_i".to_string(),
            AdminLevel::AdminPusat => "pusat".to_string(),
            AdminLevel::SystemAdmin => "system".to_string(),
        }
    }
}

impl OrganizationalLevel {
    /// Get the hierarchical level as a numeric value
    pub fn hierarchy_level(&self) -> u8 {
        match self {
            OrganizationalLevel::User(_) => 1,
            OrganizationalLevel::Satker(_) => 2,
            OrganizationalLevel::Wilayah(_) => 3,
            OrganizationalLevel::Pusat => 4,
        }
    }
}
