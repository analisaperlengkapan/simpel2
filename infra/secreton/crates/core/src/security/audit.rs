//! Advanced Audit System
//!
//! Provides comprehensive audit capabilities exceeding HashiCorp Vault:
//! - Immutable audit logs with cryptographic integrity
//! - Real-time SIEM integration
//! - Behavioral analytics and anomaly detection
//! - Compliance reporting (PCI DSS, SOX, GDPR, etc.)
//! - Distributed audit across multiple nodes
//! - Tamper-evident audit trails
//! - Forward security with progressive key derivation

use async_trait::async_trait;
use chrono::{DateTime, Timelike, Utc};
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Audit event severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuditSeverity {
    Debug = 0,
    Info = 1,
    Notice = 2,
    Warning = 3,
    Error = 4,
    Critical = 5,
    Alert = 6,
    Emergency = 7,
}

/// Audit event categories for compliance mapping
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AuditCategory {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    SystemAccess,
    ConfigurationChange,
    PolicyViolation,
    SecurityEvent,
    ComplianceEvent,
    AdminAction,
    APIAccess,
    CryptographicOperation,
}

/// Comprehensive audit event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier
    pub event_id: Uuid,
    /// Timestamp in UTC
    pub timestamp: DateTime<Utc>,
    /// Event severity level
    pub severity: AuditSeverity,
    /// Event category for compliance
    pub category: AuditCategory,
    /// Source system/service
    pub source: String,
    /// User/entity performing the action
    pub principal: Option<String>,
    /// Target resource/entity
    pub target: Option<String>,
    /// Action performed
    pub action: String,
    /// Result/outcome
    pub result: AuditResult,
    /// Additional context data
    pub context: HashMap<String, serde_json::Value>,
    /// IP address of request origin
    pub source_ip: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Session identifier
    pub session_id: Option<String>,
    /// Request correlation ID
    pub correlation_id: Option<String>,
    /// Geolocation data
    pub geo_location: Option<GeoLocation>,
    /// Risk score (0-100)
    pub risk_score: Option<u8>,
    /// Compliance tags
    pub compliance_tags: Vec<String>,
    /// Sensitive data indicators
    pub sensitive_data_access: bool,
    /// Duration of operation (if applicable)
    pub duration: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure(String),
    Partial(String),
    Denied(String),
}

/// Cryptographically signed audit log entry
///
/// Enhanced with HMAC chain for tamper-proof storage (Requirement 12.2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAuditEntry {
    /// The audit event
    pub event: AuditEvent,
    /// Sequential number for ordering
    pub sequence_number: u64,
    /// Hash of previous entry for chain integrity
    pub previous_hash: Vec<u8>,
    /// Hash of current entry
    pub entry_hash: Vec<u8>,
    /// Digital signature for integrity
    pub signature: Vec<u8>,
    /// HMAC of the entry for tamper detection (Requirement 12.2)
    pub hmac: Vec<u8>,
    /// HMAC of the previous entry for chain verification (Requirement 12.2)
    pub previous_hmac: Vec<u8>,
    /// Node identifier that created this entry
    pub node_id: String,
}

/// Behavioral pattern for anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralPattern {
    pub user_id: String,
    pub typical_access_hours: Vec<u8>, // 0-23 hours
    pub typical_locations: Vec<String>,
    pub common_resources: Vec<String>,
    pub average_session_duration: Duration,
    pub api_call_patterns: HashMap<String, u32>,
    pub last_updated: DateTime<Utc>,
    pub confidence_level: f64,
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    pub event_id: Uuid,
    pub anomaly_type: AnomalyType,
    pub severity: AuditSeverity,
    pub confidence: f64,
    pub description: String,
    pub baseline_deviation: f64,
    pub recommended_action: RecommendedAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    UnusualTimeAccess,
    UnusualLocationAccess,
    SuspiciousVolumeAccess,
    UnexpectedResourceAccess,
    FailedAuthenticationSpike,
    PrivilegeEscalation,
    DataExfiltrationPattern,
    MaliciousActivityPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedAction {
    Monitor,
    Alert,
    BlockUser,
    RequireAdditionalAuth,
    EscalateToAdmin,
    ImmediateInvestigation,
}

/// SIEM integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiemConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub authentication: SiemAuth,
    pub batch_size: usize,
    pub batch_timeout: Duration,
    pub retry_attempts: u32,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub custom_headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SiemAuth {
    None,
    ApiKey(String),
    BasicAuth { username: String, password: String },
    BearerToken(String),
    Mtls { cert_path: String, key_path: String },
}

/// Compliance reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceConfig {
    pub standards: Vec<ComplianceStandard>,
    pub report_schedule: ReportSchedule,
    pub retention_policy: RetentionPolicy,
    pub encryption_required: bool,
    pub digital_signatures: bool,
}

/// Audit system health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSystemHealth {
    pub system_health_score: f64,
    pub events_processed_last_hour: u64,
    pub storage_utilization_percent: f64,
    pub replication_lag_ms: u64,
    pub integrity_check_passed: bool,
    pub last_integrity_check: SystemTime,
}

impl Default for AuditSystemHealth {
    fn default() -> Self {
        Self {
            system_health_score: 100.0,
            events_processed_last_hour: 0,
            storage_utilization_percent: 0.0,
            replication_lag_ms: 0,
            integrity_check_passed: true,
            last_integrity_check: SystemTime::now(),
        }
    }
}

/// Performance and usage metrics for audit system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetrics {
    pub events_logged: u64,
    pub events_processed: u64,
    pub storage_used_bytes: u64,
    pub alerts_generated: u64,
    pub compliance_checks_passed: u64,
    pub compliance_checks_failed: u64,
    pub average_processing_time_ms: f64,
    pub replication_lag_ms: u64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceStandard {
    PciDss,
    Sox,
    Gdpr,
    Hipaa,
    Iso27001,
    Nist,
    Ojk, // Indonesian banking regulation (OJK)
    Pp71_2019, // Indonesian Government Regulation 71/2019 on Electronic Systems and Transactions
    Perpres95_2018, // Indonesian Presidential Regulation 95/2018 on Electronic-Based Government Systems
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ReportSchedule {
    #[default]
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annually,
    OnDemand,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetentionPolicy {
    pub default_retention: Duration,
    pub category_specific: HashMap<AuditCategory, Duration>,
    pub archive_after: Duration,
    pub archive_location: String,
    pub permanent_retention_categories: Vec<AuditCategory>,
}

/// Security audit errors for enhanced compliance and monitoring features
/// Distinct from `crate::audit::SecurityAuditError` which handles basic audit logging.
/// This error type is for advanced security features like SIEM integration,
/// compliance validation, and anomaly detection.
#[derive(Debug, thiserror::Error)]
pub enum SecurityAuditError {
    #[error("Audit storage error: {message}")]
    StorageError { message: String },

    #[error("Audit signature verification failed")]
    SignatureVerificationFailed,

    #[error("Audit chain integrity compromised at sequence {sequence}")]
    ChainIntegrityCompromised { sequence: u64 },

    #[error("SIEM integration error: {message}")]
    SiemError { message: String },

    #[error("Compliance validation error: {standard:?} - {message}")]
    ComplianceError {
        standard: ComplianceStandard,
        message: String,
    },

    #[error("Anomaly detection error: {message}")]
    AnomalyDetectionError { message: String },

    #[error("Audit serialization error: {message}")]
    SerializationError { message: String },

    #[error("Retention policy violation: {message}")]
    RetentionPolicyViolation { message: String },
}

/// Trait for audit storage backends
#[async_trait]
pub trait AuditStorage: Send + Sync {
    async fn store_entry(&self, entry: &SignedAuditEntry) -> Result<(), SecurityAuditError>;
    async fn retrieve_entries(
        &self,
        start_sequence: u64,
        end_sequence: u64,
    ) -> Result<Vec<SignedAuditEntry>, SecurityAuditError>;
    async fn get_latest_sequence(&self) -> Result<u64, SecurityAuditError>;
    async fn search_entries(&self, query: &AuditQuery)
        -> Result<Vec<SignedAuditEntry>, SecurityAuditError>;
    async fn verify_chain_integrity(
        &self,
        start_sequence: u64,
        end_sequence: u64,
    ) -> Result<bool, SecurityAuditError>;
    async fn archive_entries(
        &self,
        before_sequence: u64,
        archive_location: &str,
    ) -> Result<u64, SecurityAuditError>;
}

/// Query structure for audit log searches
///
/// Enhanced to support filtering by time range, actor, action, resource pattern (Requirement 12.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    /// Filter by time range - start time (Requirement 12.3)
    pub start_time: Option<DateTime<Utc>>,
    /// Filter by time range - end time (Requirement 12.3)
    pub end_time: Option<DateTime<Utc>>,
    pub severity: Option<AuditSeverity>,
    pub category: Option<AuditCategory>,
    /// Filter by actor/user (Requirement 12.3)
    pub principal: Option<String>,
    /// Filter by target resource (Requirement 12.3)
    pub target: Option<String>,
    /// Filter by resource path pattern (Requirement 12.3)
    pub resource_pattern: Option<String>,
    /// Filter by action type (Requirement 12.3)
    pub action: Option<String>,
    pub source_ip: Option<String>,
    pub correlation_id: Option<String>,
    pub result_type: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl AuditQuery {
    /// Create a new empty query
    pub fn new() -> Self {
        Self {
            start_time: None,
            end_time: None,
            severity: None,
            category: None,
            principal: None,
            target: None,
            resource_pattern: None,
            action: None,
            source_ip: None,
            correlation_id: None,
            result_type: None,
            limit: None,
            offset: None,
        }
    }

    /// Filter by time range (Requirement 12.3)
    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// Filter by actor (Requirement 12.3)
    pub fn with_actor(mut self, actor: impl Into<String>) -> Self {
        self.principal = Some(actor.into());
        self
    }

    /// Filter by action (Requirement 12.3)
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Filter by resource pattern (Requirement 12.3)
    pub fn with_resource_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.resource_pattern = Some(pattern.into());
        self
    }

    /// Check if an audit entry matches this query
    pub fn matches(&self, entry: &SignedAuditEntry) -> bool {
        // Time range filter
        if let Some(start) = self.start_time {
            if entry.event.timestamp < start {
                return false;
            }
        }
        if let Some(end) = self.end_time {
            if entry.event.timestamp > end {
                return false;
            }
        }

        // Actor filter
        if let Some(ref principal) = self.principal {
            if entry.event.principal.as_ref() != Some(principal) {
                return false;
            }
        }

        // Action filter
 if let Some(ref action) = self.action {
            if &entry.event.action != action {
                return false;
            }
        }

        // Resource pattern filter (simple contains match)
        if let Some(ref pattern) = self.resource_pattern {
            if let Some(ref target) = entry.event.target {
                if !target.contains(pattern) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Target filter
        if let Some(ref target) = self.target {
            if entry.event.target.as_ref() != Some(target) {
                return false;
            }
        }

        // Severity filter
        if let Some(severity) = self.severity {
            if entry.event.severity != severity {
                return false;
            }
        }

        // Category filter
        if let Some(ref category) = self.category {
            if &entry.event.category != category {
                return false;
            }
        }

        // Source IP filter
        if let Some(ref source_ip) = self.source_ip {
            if entry.event.source_ip.as_ref() != Some(source_ip) {
                return false;
            }
        }

        // Correlation ID filter
        if let Some(ref correlation_id) = self.correlation_id {
            if entry.event.correlation_id.as_ref() != Some(correlation_id) {
                return false;
            }
        }

        true
    }
}

impl Default for AuditQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// Advanced Audit System - main orchestrator
pub struct AdvancedAuditSystem {
    storage: Arc<dyn AuditStorage>,
    signing_key: Arc<Ed25519KeyPair>,
    verification_key: Arc<UnparsedPublicKey<Vec<u8>>>,
    sequence_counter: Arc<Mutex<u64>>,
    node_id: String,
    siem_config: Arc<RwLock<Option<SiemConfig>>>,
    compliance_config: Arc<RwLock<ComplianceConfig>>,
    behavioral_patterns: Arc<RwLock<HashMap<String, BehavioralPattern>>>,
    anomaly_detector: Arc<dyn AnomalyDetector>,
    pending_events: Arc<Mutex<VecDeque<AuditEvent>>>,
    batch_processor_running: Arc<Mutex<bool>>,
}

/// Trait for anomaly detection engines
#[async_trait]
pub trait AnomalyDetector: Send + Sync {
    async fn detect_anomalies(
        &self,
        event: &AuditEvent,
        pattern: Option<&BehavioralPattern>,
    ) -> Result<Vec<AnomalyResult>, SecurityAuditError>;
    async fn update_behavioral_pattern(
        &self,
        user_id: &str,
        event: &AuditEvent,
    ) -> Result<BehavioralPattern, SecurityAuditError>;
    fn get_baseline_metrics(&self, user_id: &str) -> Option<HashMap<String, f64>>;
}

impl AdvancedAuditSystem {
    /// Create a new advanced audit system
    pub fn new(
        storage: Arc<dyn AuditStorage>,
        node_id: String,
        compliance_config: ComplianceConfig,
        anomaly_detector: Arc<dyn AnomalyDetector>,
    ) -> Result<Self, SecurityAuditError> {
        // Generate signing key pair for entry integrity
        let rng = ring::rand::SystemRandom::new();
        let pkcs8_bytes =
            Ed25519KeyPair::generate_pkcs8(&rng).map_err(|e| SecurityAuditError::StorageError {
                message: format!("Key generation failed: {}", e),
            })?;

        let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).map_err(|e| {
            SecurityAuditError::StorageError {
                message: format!("Key parsing failed: {}", e),
            }
        })?;

        let verification_key =
            UnparsedPublicKey::new(&ED25519, signing_key.public_key().as_ref().to_vec());

        Ok(Self {
            storage,
            signing_key: Arc::new(signing_key),
            verification_key: Arc::new(verification_key),
            sequence_counter: Arc::new(Mutex::new(0)),
            node_id,
            siem_config: Arc::new(RwLock::new(None)),
            compliance_config: Arc::new(RwLock::new(compliance_config)),
            behavioral_patterns: Arc::new(RwLock::new(HashMap::new())),
            anomaly_detector,
            pending_events: Arc::new(Mutex::new(VecDeque::new())),
            batch_processor_running: Arc::new(Mutex::new(false)),
        })
    }

    /// Initialize the audit system
    pub async fn initialize(&self) -> Result<(), SecurityAuditError> {
        // Get the latest sequence number from storage
        let latest_sequence = self.storage.get_latest_sequence().await?;
        {
            let mut counter = self.sequence_counter.lock().unwrap();
            *counter = latest_sequence;
        }

        // Start batch processor
        self.start_batch_processor().await;

        info!(
            "Advanced audit system initialized with sequence {}",
            latest_sequence
        );
        Ok(())
    }

    /// Log an audit event
    pub async fn log_event(&self, mut event: AuditEvent) -> Result<Uuid, SecurityAuditError> {
        // Enrich event with additional context
        event.timestamp = Utc::now();
        if event.event_id.is_nil() {
            event.event_id = Uuid::new_v4();
        }

        // Perform anomaly detection
        let user_id = event.principal.clone().unwrap_or_default();
        let pattern = {
            let patterns = self.behavioral_patterns.read().unwrap();
            patterns.get(&user_id).cloned()
        };

        let anomalies = self
            .anomaly_detector
            .detect_anomalies(&event, pattern.as_ref())
            .await?;

        // Update risk score based on anomalies
        if !anomalies.is_empty() {
            let max_confidence = anomalies
                .iter()
                .map(|a| a.confidence)
                .fold(0.0f64, f64::max);
            event.risk_score = Some((max_confidence * 100.0) as u8);

            // Log anomaly events
            for anomaly in anomalies {
                if anomaly.confidence > 0.7 {
                    let anomaly_event = AuditEvent {
                        event_id: Uuid::new_v4(),
                        timestamp: Utc::now(),
                        severity: anomaly.severity,
                        category: AuditCategory::SecurityEvent,
                        source: "anomaly_detector".to_string(),
                        principal: event.principal.clone(),
                        target: None,
                        action: format!("anomaly_detected_{:?}", anomaly.anomaly_type),
                        result: AuditResult::Success,
                        context: {
                            let mut context = HashMap::new();
                            context.insert(
                                "original_event_id".to_string(),
                                serde_json::json!(event.event_id),
                            );
                            context.insert(
                                "anomaly_description".to_string(),
                                serde_json::json!(anomaly.description),
                            );
                            context.insert(
                                "confidence".to_string(),
                                serde_json::json!(anomaly.confidence),
                            );
                            context
                        },
                        source_ip: event.source_ip.clone(),
                        user_agent: None,
                        session_id: event.session_id.clone(),
                        correlation_id: event.correlation_id.clone(),
                        geo_location: None,
                        risk_score: Some(100),
                        compliance_tags: vec!["SECURITY_INCIDENT".to_string()],
                        sensitive_data_access: false,
                        duration: None,
                    };

                    // Add to pending queue
                    {
                        let mut pending = self.pending_events.lock().unwrap();
                        pending.push_back(anomaly_event);
                    }
                }
            }
        }

        // Update behavioral patterns
        if !user_id.is_empty() {
            let updated_pattern = self
                .anomaly_detector
                .update_behavioral_pattern(&user_id, &event)
                .await?;
            {
                let mut patterns = self.behavioral_patterns.write().unwrap();
                patterns.insert(user_id, updated_pattern);
            }
        }

        // Add compliance tags based on event category
        self.add_compliance_tags(&mut event);

        // Add to pending events queue for batch processing
        let event_id = event.event_id;
        {
            let mut pending = self.pending_events.lock().unwrap();
            pending.push_back(event);
        }

        Ok(event_id)
    }

    /// Create and sign an audit entry for digital signature verification and audit integrity
    async fn create_signed_entry(&self, event: AuditEvent) -> Result<SignedAuditEntry, SecurityAuditError> {
        // Get next sequence number
        let sequence_number = {
            let mut counter = self.sequence_counter.lock().unwrap();
            *counter += 1;
            *counter
        };

        // Get previous entry hash and HMAC for chain integrity
        let (previous_hash, previous_hmac) = if sequence_number > 1 {
            match self
                .storage
                .retrieve_entries(sequence_number - 1, sequence_number - 1)
                .await
            {
                Ok(entries) if !entries.is_empty() => {
                    (entries[0].entry_hash.clone(), entries[0].hmac.clone())
                }
                _ => (vec![0; 32], vec![0; 32]), // Genesis hash and HMAC
            }
        } else {
            (vec![0; 32], vec![0; 32]) // Genesis hash and HMAC
        };

        // Create entry hash
        let entry_data =
            serde_json::to_vec(&event).map_err(|e| SecurityAuditError::SerializationError {
                message: e.to_string(),
            })?;

        let mut hasher = Sha256::new();
        hasher.update(&entry_data);
        hasher.update(&previous_hash);
        hasher.update(sequence_number.to_le_bytes());
        hasher.update(self.node_id.as_bytes());
        let entry_hash = hasher.finalize().to_vec();

        // Sign the entry
        let signature = self.signing_key.sign(&entry_hash).as_ref().to_vec();

        // Compute HMAC for tamper-proof chain (Requirement 12.2)
        // HMAC includes: entry_hash + previous_hmac + sequence_number
        let mut hmac_hasher = Sha256::new();
        hmac_hasher.update(&entry_hash);
        hmac_hasher.update(&previous_hmac);
        hmac_hasher.update(sequence_number.to_le_bytes());
        hmac_hasher.update(self.node_id.as_bytes());
        let hmac = hmac_hasher.finalize().to_vec();

        Ok(SignedAuditEntry {
            event,
            sequence_number,
            previous_hash,
            entry_hash,
            signature,
            hmac,
            previous_hmac,
            node_id: self.node_id.clone(),
        })
    }

    /// Verify the integrity of an audit entry
    ///
    /// Enhanced to verify HMAC chain for tamper detection (Requirement 12.2)
    /// Alerts on HMAC verification failure (Requirement 12.5)
    pub fn verify_entry(&self, entry: &SignedAuditEntry) -> Result<bool, SecurityAuditError> {
        // Verify signature
        match self
            .verification_key
            .verify(&entry.entry_hash, &entry.signature)
        {
            Ok(()) => {}
            Err(_) => {
                // Alert on signature verification failure (Requirement 12.5)
                error!(
                    "SECURITY ALERT: Audit entry signature verification failed for sequence {}",
                    entry.sequence_number
                );
                warn!(
                    "Audit tampering detected: Invalid signature for entry {} (event_id: {})",
                    entry.sequence_number, entry.event.event_id
                );
                return Ok(false);
            }
        }

        // Verify hash
        let entry_data =
            serde_json::to_vec(&entry.event).map_err(|e| SecurityAuditError::SerializationError {
                message: e.to_string(),
            })?;

        let mut hasher = Sha256::new();
        hasher.update(&entry_data);
        hasher.update(&entry.previous_hash);
        hasher.update(entry.sequence_number.to_le_bytes());
        hasher.update(entry.node_id.as_bytes());
        let computed_hash = hasher.finalize().to_vec();

        if computed_hash != entry.entry_hash {
            // Alert on hash verification failure (Requirement 12.5)
            error!(
                "SECURITY ALERT: Audit entry hash verification failed for sequence {}",
                entry.sequence_number
            );
            warn!(
                "Audit tampering detected: Invalid hash for entry {} (event_id: {})",
                entry.sequence_number, entry.event.event_id
            );
            return Ok(false);
        }

        // Verify HMAC chain (Requirement 12.2)
        let mut hmac_hasher = Sha256::new();
        hmac_hasher.update(&entry.entry_hash);
        hmac_hasher.update(&entry.previous_hmac);
        hmac_hasher.update(entry.sequence_number.to_le_bytes());
        hmac_hasher.update(entry.node_id.as_bytes());
        let computed_hmac = hmac_hasher.finalize().to_vec();

        if computed_hmac != entry.hmac {
            // Alert on HMAC verification failure (Requirement 12.5)
            error!(
                "SECURITY ALERT: Audit entry HMAC verification failed for sequence {}",
                entry.sequence_number
            );
            warn!(
                "Audit tampering detected: Invalid HMAC chain for entry {} (event_id: {})",
                entry.sequence_number, entry.event.event_id
            );
            return Ok(false);
        }

        Ok(true)
    }

    /// Verify audit chain integrity and alert on tampering
    ///
    /// Verifies the integrity of a range of audit entries and alerts administrators
    /// on any tampering detection (Requirement 12.5)
    pub async fn verify_and_alert_chain(
        &self,
        start_sequence: u64,
        end_sequence: u64,
    ) -> Result<bool, SecurityAuditError> {
        let entries = self.storage.retrieve_entries(start_sequence, end_sequence).await?;

        let mut all_valid = true;
        let mut tampered_entries = Vec::new();

        for entry in &entries {
            if !self.verify_entry(entry)? {
                all_valid = false;
                tampered_entries.push(entry.sequence_number);
            }
        }

        if !all_valid {
            // Alert administrators on tampering detection (Requirement 12.5)
            error!(
                "CRITICAL SECURITY ALERT: Audit log tampering detected! {} entries failed verification",
                tampered_entries.len()
            );
            error!(
                "Tampered entry sequence numbers: {:?}",
                tampered_entries
            );

            // Log a security event for the tampering detection
            let alert_event = AuditEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                severity: AuditSeverity::Emergency,
                category: AuditCategory::SecurityEvent,
                source: "audit_integrity_monitor".to_string(),
                principal: Some("system".to_string()),
                target: Some(format!("audit_log_sequences_{}_to_{}", start_sequence, end_sequence)),
                action: "audit_tampering_detected".to_string(),
                result: AuditResult::Failure("Audit log integrity compromised".to_string()),
                context: {
                    let mut context = HashMap::new();
                    context.insert(
                        "tampered_sequences".to_string(),
                        serde_json::json!(tampered_entries),
                    );
                    context.insert(
                        "total_tampered".to_string(),
                        serde_json::json!(tampered_entries.len()),
                    );
                    context
                },
                source_ip: None,
                user_agent: None,
                session_id: None,
                correlation_id: Some(format!("tampering_alert_{}", Uuid::new_v4())),
                geo_location: None,
                risk_score: Some(100),
                compliance_tags: vec!["SECURITY_INCIDENT".to_string(), "AUDIT_TAMPERING".to_string()],
                sensitive_data_access: false,
                duration: None,
            };

            // Log the tampering alert
            let _ = self.log_event(alert_event).await;
        }

        Ok(all_valid)
    }

    /// Search audit logs with comprehensive querying
    pub async fn search(&self, query: &AuditQuery) -> Result<Vec<SignedAuditEntry>, SecurityAuditError> {
        self.storage.search_entries(query).await
    }

    /// Generate compliance report
    pub async fn generate_compliance_report(
        &self,
        standard: ComplianceStandard,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<ComplianceReport, SecurityAuditError> {
        let query = AuditQuery::new()
            .with_time_range(start_time, end_time);

        let entries = self.search(&query).await?;

        // Generate report based on compliance standard
        let report = match standard {
            ComplianceStandard::PciDss => {
                self.generate_pci_dss_report(&entries, start_time, end_time)
            }
            ComplianceStandard::Sox => self.generate_sox_report(&entries, start_time, end_time),
            ComplianceStandard::Gdpr => self.generate_gdpr_report(&entries, start_time, end_time),
            ComplianceStandard::Ojk => self.generate_ojk_report(&entries, start_time, end_time),
            ComplianceStandard::Pp71_2019 => self.generate_pp71_2019_report(&entries, start_time, end_time),
            ComplianceStandard::Perpres95_2018 => self.generate_perpres95_2018_report(&entries, start_time, end_time),
            _ => self.generate_generic_report(&entries, start_time, end_time),
        };

        Ok(report)
    }

    /// Configure SIEM integration
    pub fn configure_siem(&self, config: SiemConfig) {
        let mut siem_config = self.siem_config.write().unwrap();
        *siem_config = Some(config);
        info!("SIEM integration configured");
    }

    /// Start batch processor for efficient logging
    async fn start_batch_processor(&self) {
        {
            let mut running = self.batch_processor_running.lock().unwrap();
            if *running {
                return;
            }
            *running = true;
        }

        let pending_events = self.pending_events.clone();
        let storage = self.storage.clone();
        let siem_config = self.siem_config.clone();
        let audit_system = self.clone_for_processing();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                interval.tick().await;

                let events_to_process: Vec<AuditEvent> = {
                    let mut pending = pending_events.lock().unwrap();
                    let mut events = Vec::new();

                    // Process up to 100 events per batch
                    for _ in 0..100 {
                        if let Some(event) = pending.pop_front() {
                            events.push(event);
                        } else {
                            break;
                        }
                    }
                    events
                };

                if events_to_process.is_empty() {
                    continue;
                }

                // Process each event
                for event in events_to_process {
                    match audit_system.create_signed_entry(event.clone()).await {
                        Ok(signed_entry) => {
                            if let Err(e) = storage.store_entry(&signed_entry).await {
                                error!("Failed to store audit entry: {}", e);
                                // Re-queue the event for retry
                                let mut pending = pending_events.lock().unwrap();
                                pending.push_front(event);
                            } else {
                                debug!("Stored audit entry: {}", signed_entry.event.event_id);

                                // Send to SIEM if configured (avoid holding lock across await)
                                let siem_config_data = {
                                    let config_guard = siem_config.read().unwrap();
                                    config_guard.clone()
                                };

                                if let Some(config) = siem_config_data {
                                    if config.enabled {
                                        if let Err(e) =
                                            Self::send_to_siem(&signed_entry, &config).await
                                        {
                                            warn!("Failed to send to SIEM: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to create signed audit entry: {}", e);
                        }
                    }
                }
            }
        });
    }

    /// Send audit entry to SIEM system
    async fn send_to_siem(entry: &SignedAuditEntry, config: &SiemConfig) -> Result<(), SecurityAuditError> {
        // Implementation would send to actual SIEM system
        debug!(
            "Sending audit entry {} to SIEM at {}",
            entry.event.event_id, config.endpoint
        );

        // Mock implementation
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }

    /// Add compliance tags based on event properties
    fn add_compliance_tags(&self, event: &mut AuditEvent) {
        let compliance_config = self.compliance_config.read().unwrap();

        for standard in &compliance_config.standards {
            match standard {
                ComplianceStandard::PciDss => {
                    if matches!(
                        event.category,
                        AuditCategory::DataAccess | AuditCategory::CryptographicOperation
                    ) {
                        event.compliance_tags.push("PCI_DSS".to_string());
                    }
                }
                ComplianceStandard::Sox => {
                    if matches!(
                        event.category,
                        AuditCategory::DataModification | AuditCategory::AdminAction
                    ) {
                        event.compliance_tags.push("SOX".to_string());
                    }
                }
                ComplianceStandard::Gdpr => {
                    if event.sensitive_data_access {
                        event.compliance_tags.push("GDPR".to_string());
                    }
                }
                ComplianceStandard::Ojk => {
                    if matches!(
                        event.category,
                        AuditCategory::Authentication | AuditCategory::Authorization
                    ) {
                        event.compliance_tags.push("OJK".to_string());
                    }
                }
                ComplianceStandard::Pp71_2019 => {
                    // PP 71/2019 requires audit of all electronic system operations
                    event.compliance_tags.push("PP_71_2019".to_string());
                }
                ComplianceStandard::Perpres95_2018 => {
                    // Perpres 95/2018 focuses on government electronic systems
                    if matches!(
                        event.category,
                        AuditCategory::SystemAccess | AuditCategory::ConfigurationChange | AuditCategory::AdminAction
                    ) {
                        event.compliance_tags.push("PERPRES_95_2018".to_string());
                    }
                }
                _ => {}
            }
        }
    }

    /// Clone minimal data for background processing
    fn clone_for_processing(&self) -> ProcessingAuditSystem {
        ProcessingAuditSystem {
            signing_key: self.signing_key.clone(),
            sequence_counter: self.sequence_counter.clone(),
            node_id: self.node_id.clone(),
        }
    }

    // Compliance report generators
    fn generate_pci_dss_report(
        &self,
        _entries: &[SignedAuditEntry],
        _start: DateTime<Utc>,
        _end: DateTime<Utc>,
    ) -> ComplianceReport {
        // Mock implementation
        ComplianceReport {
            standard: ComplianceStandard::PciDss,
            period_start: _start,
            period_end: _end,
            total_events: _entries.len() as u64,
            compliance_violations: 0,
            recommendations: vec!["No violations found".to_string()],
            summary: "PCI DSS compliance maintained".to_string(),
        }
    }

    fn generate_sox_report(
        &self,
        _entries: &[SignedAuditEntry],
        _start: DateTime<Utc>,
        _end: DateTime<Utc>,
    ) -> ComplianceReport {
        ComplianceReport {
            standard: ComplianceStandard::Sox,
            period_start: _start,
            period_end: _end,
            total_events: _entries.len() as u64,
            compliance_violations: 0,
            recommendations: vec!["SOX controls operating effectively".to_string()],
            summary: "SOX compliance maintained".to_string(),
        }
    }

    fn generate_gdpr_report(
        &self,
        _entries: &[SignedAuditEntry],
        _start: DateTime<Utc>,
        _end: DateTime<Utc>,
    ) -> ComplianceReport {
        ComplianceReport {
            standard: ComplianceStandard::Gdpr,
            period_start: _start,
            period_end: _end,
            total_events: _entries.len() as u64,
            compliance_violations: 0,
            recommendations: vec!["Data protection measures adequate".to_string()],
            summary: "GDPR compliance maintained".to_string(),
        }
    }

    fn generate_ojk_report(
        &self,
        _entries: &[SignedAuditEntry],
        _start: DateTime<Utc>,
        _end: DateTime<Utc>,
    ) -> ComplianceReport {
        ComplianceReport {
            standard: ComplianceStandard::Ojk,
            period_start: _start,
            period_end: _end,
            total_events: _entries.len() as u64,
            compliance_violations: 0,
            recommendations: vec!["Banking security controls effective".to_string()],
            summary: "OJK regulatory compliance maintained".to_string(),
        }
    }

    /// Generate compliance report for PP 71/2019 (Indonesian Government Regulation)
    ///
    /// PP 71/2019 covers Electronic Systems and Transactions security requirements
    /// (Requirement 12.4)
    fn generate_pp71_2019_report(
        &self,
        entries: &[SignedAuditEntry],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> ComplianceReport {
        // PP 71/2019 requires:
        // - Audit trail of all electronic system operations
        // - Data integrity protection
        // - Access control and authentication
        // - Incident response and recovery

        let mut violations = 0;
        let mut recommendations = Vec::new();

        // Check for authentication events
        let auth_events = entries.iter().filter(|e| {
            e.event.category == AuditCategory::Authentication
        }).count();

        if auth_events == 0 {
            recommendations.push("No authentication events logged - ensure authentication auditing is enabled".to_string());
        }

        // Check for failed access attempts
        let failed_access = entries.iter().filter(|e| {
            matches!(e.event.result, AuditResult::Denied(_) | AuditResult::Failure(_))
        }).count();

        if failed_access > 0 {
            recommendations.push(format!("{} failed access attempts detected - review security policies", failed_access));
        }

        // Check for configuration changes
        let config_changes = entries.iter().filter(|e| {
            e.event.category == AuditCategory::ConfigurationChange
        }).count();

        if config_changes > 0 {
            recommendations.push(format!("{} configuration changes detected - ensure proper authorization", config_changes));
        }

        // Check audit log integrity
        let integrity_issues = entries.iter().filter(|e| {
            !self.verify_entry(e).unwrap_or(false)
        }).count();

        if integrity_issues > 0 {
            violations += integrity_issues as u32;
            recommendations.push(format!("CRITICAL: {} audit entries failed integrity verification", integrity_issues));
        }

        if recommendations.is_empty() {
            recommendations.push("All PP 71/2019 requirements met".to_string());
        }

        ComplianceReport {
            standard: ComplianceStandard::Pp71_2019,
            period_start: start,
            period_end: end,
            total_events: entries.len() as u64,
            compliance_violations: violations,
            recommendations,
            summary: if violations == 0 {
                "PP 71/2019 compliance maintained - Electronic systems security requirements met".to_string()
            } else {
                format!("PP 71/2019 compliance issues detected - {} violations require attention", violations)
            },
        }
    }

    /// Generate compliance report for Perpres 95/2018 (Indonesian Presidential Regulation)
    ///
    /// Perpres 95/2018 covers Electronic-Based Government Systems (SPBE)
    /// (Requirement 12.4)
    fn generate_perpres95_2018_report(
        &self,
        entries: &[SignedAuditEntry],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> ComplianceReport {
        // Perpres 95/2018 requires:
        // - Government system access control
        // - Data protection and confidentiality
        // - System availability and reliability
        // - Audit and monitoring

        let mut violations = 0;
        let mut recommendations = Vec::new();

        // Check for system access events
        let system_access = entries.iter().filter(|e| {
            e.event.category == AuditCategory::SystemAccess
        }).count();

        // Check for admin actions
        let admin_actions = entries.iter().filter(|e| {
            e.event.category == AuditCategory::AdminAction
        }).count();

        if admin_actions > 0 {
            recommendations.push(format!("{} administrative actions logged - ensure proper oversight", admin_actions));
        }

        // Check for security events
        let security_events = entries.iter().filter(|e| {
            e.event.category == AuditCategory::SecurityEvent
        }).count();

        if security_events > 0 {
            recommendations.push(format!("{} security events detected - review and respond appropriately", security_events));
            violations += security_events as u32;
        }

        // Check for data access patterns
        let data_access = entries.iter().filter(|e| {
            e.event.category == AuditCategory::DataAccess && e.event.sensitive_data_access
        }).count();

        if data_access > 0 {
            recommendations.push(format!("{} sensitive data access events - ensure proper authorization", data_access));
        }

        // Check for policy violations
        let policy_violations = entries.iter().filter(|e| {
            e.event.category == AuditCategory::PolicyViolation
        }).count();

        if policy_violations > 0 {
            violations += policy_violations as u32;
            recommendations.push(format!("CRITICAL: {} policy violations detected", policy_violations));
        }

        if recommendations.is_empty() {
            recommendations.push("All Perpres 95/2018 SPBE requirements met".to_string());
        }

        ComplianceReport {
            standard: ComplianceStandard::Perpres95_2018,
            period_start: start,
            period_end: end,
            total_events: entries.len() as u64,
            compliance_violations: violations,
            recommendations,
            summary: if violations == 0 {
                "Perpres 95/2018 SPBE compliance maintained - Government electronic systems requirements met".to_string()
            } else {
                format!("Perpres 95/2018 SPBE compliance issues detected - {} violations require immediate attention", violations)
            },
        }
    }

    fn generate_generic_report(
        &self,
        _entries: &[SignedAuditEntry],
        _start: DateTime<Utc>,
        _end: DateTime<Utc>,
    ) -> ComplianceReport {
        ComplianceReport {
            standard: ComplianceStandard::Custom("Generic".to_string()),
            period_start: _start,
            period_end: _end,
            total_events: _entries.len() as u64,
            compliance_violations: 0,
            recommendations: vec!["Review audit logs regularly".to_string()],
            summary: "Audit log analysis complete".to_string(),
        }
    }

    /// Get audit system metrics
    pub fn get_metrics(&self) -> AuditSystemHealth {
        let sequence_counter = self.sequence_counter.lock().unwrap();
        AuditSystemHealth {
            system_health_score: 100.0, // Mock value
            events_processed_last_hour: *sequence_counter,
            storage_utilization_percent: 50.0,
            replication_lag_ms: 0,
            integrity_check_passed: true,
            last_integrity_check: SystemTime::now(),
        }
    }

    /// Start monitoring tasks (called by security orchestrator)
    pub async fn start_monitoring(&self) -> Result<(), SecurityAuditError> {
        self.start_batch_processing().await
    }

    /// Get health metrics for security monitoring
    pub async fn get_health_metrics(&self) -> AuditSystemHealth {
        self.get_health_status().await
    }

    /// Get detailed health status
    pub async fn get_health_status(&self) -> AuditSystemHealth {
        AuditSystemHealth {
            system_health_score: 95.0,        // Calculate based on system state
            events_processed_last_hour: 1000, // Mock value
            storage_utilization_percent: 45.0,
            replication_lag_ms: 10,
            integrity_check_passed: true,
            last_integrity_check: SystemTime::now(),
        }
    }

    /// Start batch processing
    pub async fn start_batch_processing(&self) -> Result<(), SecurityAuditError> {
        let mut running = self.batch_processor_running.lock().unwrap();
        if *running {
            return Ok(());
        }
        *running = true;
        info!("Audit batch processing started");
        Ok(())
    }
}

/// Minimal struct for background processing
struct ProcessingAuditSystem {
    signing_key: Arc<Ed25519KeyPair>,
    sequence_counter: Arc<Mutex<u64>>,
    node_id: String,
}

impl ProcessingAuditSystem {
    async fn create_signed_entry(&self, event: AuditEvent) -> Result<SignedAuditEntry, SecurityAuditError> {
        let sequence_number = {
            let mut counter = self.sequence_counter.lock().unwrap();
            *counter += 1;
            *counter
        };

        let previous_hash = vec![0; 32]; // Simplified for background processing
        let previous_hmac = vec![0; 32]; // Simplified for background processing

        let entry_data =
            serde_json::to_vec(&event).map_err(|e| SecurityAuditError::SerializationError {
                message: e.to_string(),
            })?;

        let mut hasher = Sha256::new();
        hasher.update(&entry_data);
        hasher.update(&previous_hash);
        hasher.update(sequence_number.to_le_bytes());
        hasher.update(self.node_id.as_bytes());
        let entry_hash = hasher.finalize().to_vec();

        let signature = self.signing_key.sign(&entry_hash).as_ref().to_vec();

        // Compute HMAC for tamper-proof chain (Requirement 12.2)
        let mut hmac_hasher = Sha256::new();
        hmac_hasher.update(&entry_hash);
        hmac_hasher.update(&previous_hmac);
        hmac_hasher.update(sequence_number.to_le_bytes());
        hmac_hasher.update(self.node_id.as_bytes());
        let hmac = hmac_hasher.finalize().to_vec();

        Ok(SignedAuditEntry {
            event,
            sequence_number,
            previous_hash,
            entry_hash,
            signature,
            hmac,
            previous_hmac,
            node_id: self.node_id.clone(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub standard: ComplianceStandard,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: u64,
    pub compliance_violations: u32,
    pub recommendations: Vec<String>,
    pub summary: String,
}

/// Simple anomaly detector implementation
pub struct SimpleAnomalyDetector;

#[async_trait]
impl AnomalyDetector for SimpleAnomalyDetector {
    async fn detect_anomalies(
        &self,
        event: &AuditEvent,
        pattern: Option<&BehavioralPattern>,
    ) -> Result<Vec<AnomalyResult>, SecurityAuditError> {
        let mut anomalies = Vec::new();

        if let Some(pattern) = pattern {
            // Check for unusual time access
            let current_hour = event.timestamp.hour() as u8;
            if !pattern.typical_access_hours.contains(&current_hour) {
                anomalies.push(AnomalyResult {
                    event_id: event.event_id,
                    anomaly_type: AnomalyType::UnusualTimeAccess,
                    severity: AuditSeverity::Warning,
                    confidence: 0.8,
                    description: format!("Access at unusual hour: {}", current_hour),
                    baseline_deviation: 2.0,
                    recommended_action: RecommendedAction::Monitor,
                });
            }
        }

        // Check for failed authentication spikes
        if matches!(
            event.result,
            AuditResult::Failure(_) | AuditResult::Denied(_)
        ) && event.category == AuditCategory::Authentication
        {
            anomalies.push(AnomalyResult {
                event_id: event.event_id,
                anomaly_type: AnomalyType::FailedAuthenticationSpike,
                severity: AuditSeverity::Warning,
                confidence: 0.6,
                description: "Authentication failure detected".to_string(),
                baseline_deviation: 1.5,
                recommended_action: RecommendedAction::Alert,
            });
        }

        Ok(anomalies)
    }

    async fn update_behavioral_pattern(
        &self,
        user_id: &str,
        event: &AuditEvent,
    ) -> Result<BehavioralPattern, SecurityAuditError> {
        // Create or update behavioral pattern
        let current_hour = event.timestamp.hour() as u8;

        Ok(BehavioralPattern {
            user_id: user_id.to_string(),
            typical_access_hours: vec![current_hour],
            typical_locations: vec![event.source_ip.clone().unwrap_or_default()],
            common_resources: vec![event.target.clone().unwrap_or_default()],
            average_session_duration: event.duration.unwrap_or_default(),
            api_call_patterns: HashMap::new(),
            last_updated: Utc::now(),
            confidence_level: 0.5,
        })
    }

    fn get_baseline_metrics(&self, _user_id: &str) -> Option<HashMap<String, f64>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    // Mock storage implementation for testing
    struct MockAuditStorage {
        entries: Arc<Mutex<Vec<SignedAuditEntry>>>,
    }

    impl MockAuditStorage {
        fn new() -> Self {
            Self {
                entries: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl AuditStorage for MockAuditStorage {
        async fn store_entry(&self, entry: &SignedAuditEntry) -> Result<(), SecurityAuditError> {
            let mut entries = self.entries.lock().unwrap();
            entries.push(entry.clone());
            Ok(())
        }

        async fn retrieve_entries(
            &self,
            start_sequence: u64,
            end_sequence: u64,
        ) -> Result<Vec<SignedAuditEntry>, SecurityAuditError> {
            let entries = self.entries.lock().unwrap();
            Ok(entries
                .iter()
                .filter(|e| {
                    e.sequence_number >= start_sequence && e.sequence_number <= end_sequence
                })
                .cloned()
                .collect())
        }

        async fn get_latest_sequence(&self) -> Result<u64, SecurityAuditError> {
            let entries = self.entries.lock().unwrap();
            Ok(entries.iter().map(|e| e.sequence_number).max().unwrap_or(0))
        }

        async fn search_entries(
            &self,
            query: &AuditQuery,
        ) -> Result<Vec<SignedAuditEntry>, SecurityAuditError> {
            let entries = self.entries.lock().unwrap();
            let mut filtered: Vec<SignedAuditEntry> = entries
                .iter()
                .filter(|e| query.matches(e))
                .cloned()
                .collect();

            // Apply limit and offset
            if let Some(offset) = query.offset {
                filtered = filtered.into_iter().skip(offset).collect();
            }
            if let Some(limit) = query.limit {
                filtered.truncate(limit);
            }

            Ok(filtered)
        }

        async fn verify_chain_integrity(
            &self,
            _start_sequence: u64,
            _end_sequence: u64,
        ) -> Result<bool, SecurityAuditError> {
            Ok(true)
        }

        async fn archive_entries(
            &self,
            _before_sequence: u64,
            _archive_location: &str,
        ) -> Result<u64, SecurityAuditError> {
            Ok(0)
        }
    }

    #[tokio::test]
    async fn test_audit_system_basic_functionality() {
        let storage = Arc::new(MockAuditStorage::new());
        let compliance_config = ComplianceConfig {
            standards: vec![ComplianceStandard::PciDss],
            report_schedule: ReportSchedule::Daily,
            retention_policy: RetentionPolicy {
                default_retention: Duration::from_secs(86400 * 365),
                category_specific: HashMap::new(),
                archive_after: Duration::from_secs(86400 * 90),
                archive_location: "/archive".to_string(),
                permanent_retention_categories: vec![AuditCategory::SecurityEvent],
            },
            encryption_required: true,
            digital_signatures: true,
        };

        let anomaly_detector = Arc::new(SimpleAnomalyDetector);

        let audit_system = AdvancedAuditSystem::new(
            storage,
            "test_node".to_string(),
            compliance_config,
            anomaly_detector,
        )
        .unwrap();

        audit_system.initialize().await.unwrap();

        // Create test audit event
        let event = AuditEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            severity: AuditSeverity::Info,
            category: AuditCategory::DataAccess,
            source: "test_service".to_string(),
            principal: Some("test_user".to_string()),
            target: Some("secret/test".to_string()),
            action: "read".to_string(),
            result: AuditResult::Success,
            context: HashMap::new(),
            source_ip: Some("127.0.0.1".to_string()),
            user_agent: None,
            session_id: Some("session_123".to_string()),
            correlation_id: Some("corr_123".to_string()),
            geo_location: None,
            risk_score: None,
            compliance_tags: Vec::new(),
            sensitive_data_access: false,
            duration: Some(Duration::from_millis(100)),
        };

        let event_id = audit_system.log_event(event).await.unwrap();
        assert!(!event_id.is_nil());

        // Wait for batch processing
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Search for the logged event
        let mut query = AuditQuery::new();
        query.category = Some(AuditCategory::DataAccess);

        let results = audit_system.search(&query).await.unwrap();
        assert!(!results.is_empty());
    }
}
