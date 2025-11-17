use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Metadata, SecurityLevel};

/// Enhanced Secret model for SIMKARI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub id: i64,
    pub path: String,
    pub version: u32,
    pub data: EncryptedValue,
    pub metadata: SecretMetadata,
    pub access_control: AccessControl,
    pub audit_trail: AuditTrail,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by_nip: Option<String>, // NIP pegawai yang membuat
    pub satker_owner: String,           // Satker pemilik secret (replaces instansi_owner)
    pub last_accessed: DateTime<Utc>,
    pub namespace: String, // Kept for backward compatibility
}

/// Encrypted value with algorithm information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedValue {
    pub data: serde_json::Value,
    pub encryption_algorithm: EncryptionAlgorithm,
    pub encrypted_at: DateTime<Utc>,
    pub key_id: Option<String>,
}

/// Supported encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM (classical)
    Aes256Gcm,
    /// ChaCha20-Poly1305 (classical)
    ChaCha20Poly1305,
    /// ML-KEM (post-quantum)
    MlKem,
    /// Hybrid classical + post-quantum
    Hybrid {
        classical: Box<EncryptionAlgorithm>,
        post_quantum: Box<EncryptionAlgorithm>,
    },
}

/// Flexible metadata system for secrets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub security_level: SecurityLevel,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub custom_fields: Metadata,
    pub compliance_flags: Vec<String>, // Compliance flags for kejaksaan operations
    pub risk_score: Option<f64>,       // Risk assessment score
}

/// Role-based access control for secrets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControl {
    pub required_roles: Vec<String>,  // Role yang diperlukan (fleksibel)
    pub required_satker: Vec<String>, // Satker yang diizinkan
    pub nip_whitelist: Option<Vec<String>>, // NIP yang diizinkan
    pub nip_blacklist: Option<Vec<String>>, // NIP yang dilarang
    pub time_based_access: Option<TimeBasedAccess>,
    pub audit_required: bool,                     // Wajib audit setiap akses
    pub admin_level_required: Option<AdminLevel>, // Level admin yang diperlukan
}

/// Time-based access restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBasedAccess {
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub allowed_hours: Option<Vec<u8>>, // Hours of day (0-23)
    pub allowed_days: Option<Vec<u8>>,  // Days of week (0-6, Sunday=0)
    pub timezone: String,               // Timezone for time checks
}

/// Admin levels for hierarchical access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdminLevel {
    AdminSatker(String),  // Admin satker tertentu
    AdminWilayah(String), // Admin wilayah tertentu
    AdminEselonI,         // Admin eselon I di kejaksaan agung
    AdminPusat,           // Admin tingkat pusat
}

/// Audit trail for secret operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    pub creation_event: AuditEvent,
    pub access_events: Vec<AuditEvent>,
    pub modification_events: Vec<AuditEvent>,
    pub last_audit_check: Option<DateTime<Utc>>,
}

/// Individual audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub nip: Option<String>,         // NIP pegawai
    pub satker_code: Option<String>, // Kode satuan kerja
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub operation: Operation,
    pub result: OperationResult,
    pub risk_score: Option<f64>,
    pub compliance_flags: Vec<String>, // Flag compliance kejaksaan
    pub admin_level: Option<AdminLevel>, // Level admin yang melakukan operasi
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    Create,
    Read,
    Update,
    Delete,
    AccessGranted,
    AccessDenied,
    Export,
    Import,
    Backup,
    Restore,
}

/// Operations performed on secrets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    Create,
    Read,
    Update,
    Delete,
    List,
    Export,
    Import,
    Backup,
    Restore,
    Encrypt,
    Decrypt,
    Share,
    Revoke,
}

/// Result of operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationResult {
    Success,
    Failure(String),
    PartialSuccess(String),
    AccessDenied(String),
    NotFound,
    Timeout,
}

impl Default for SecretMetadata {
    fn default() -> Self {
        Self {
            security_level: SecurityLevel::Internal,
            tags: Vec::new(),
            description: None,
            custom_fields: Metadata::new(),
            compliance_flags: Vec::new(),
            risk_score: None,
        }
    }
}

impl Default for AccessControl {
    fn default() -> Self {
        Self {
            required_roles: Vec::new(),
            required_satker: Vec::new(),
            nip_whitelist: None,
            nip_blacklist: None,
            time_based_access: None,
            audit_required: true, // Default to requiring audit
            admin_level_required: None,
        }
    }
}

impl Default for AuditTrail {
    fn default() -> Self {
        let creation_event = AuditEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::Create,
            nip: None,
            satker_code: None,
            session_id: None,
            ip_address: None,
            user_agent: None,
            operation: Operation::Create,
            result: OperationResult::Success,
            risk_score: None,
            compliance_flags: Vec::new(),
            admin_level: None,
        };

        Self {
            creation_event,
            access_events: Vec::new(),
            modification_events: Vec::new(),
            last_audit_check: None,
        }
    }
}

impl Secret {
    /// Create a new secret with enhanced metadata
    pub fn new(
        path: String,
        data: serde_json::Value,
        satker_owner: String,
        created_by_nip: Option<String>,
    ) -> Self {
        let encrypted_value = EncryptedValue {
            data,
            encryption_algorithm: EncryptionAlgorithm::Aes256Gcm, // Default algorithm
            encrypted_at: Utc::now(),
            key_id: None,
        };

        Self {
            id: 0, // Will be set by storage
            path,
            version: 1,
            data: encrypted_value,
            metadata: SecretMetadata::default(),
            access_control: AccessControl::default(),
            audit_trail: AuditTrail::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by_nip,
            satker_owner,
            last_accessed: Utc::now(),
            namespace: "default".to_string(), // Backward compatibility
        }
    }

    /// Check if user has access to this secret
    pub fn check_access(
        &self,
        user_nip: Option<&str>,
        user_roles: &[String],
        user_satker: &str,
        admin_level: Option<&AdminLevel>,
    ) -> bool {
        // Check satker access
        if !self.access_control.required_satker.is_empty()
            && !self
                .access_control
                .required_satker
                .contains(&user_satker.to_string())
        {
            return false;
        }

        // Check NIP whitelist/blacklist
        if let Some(nip) = user_nip {
            if let Some(blacklist) = &self.access_control.nip_blacklist
                && blacklist.contains(&nip.to_string())
            {
                return false;
            }

            if let Some(whitelist) = &self.access_control.nip_whitelist
                && !whitelist.contains(&nip.to_string())
            {
                return false;
            }
        }

        // Check role requirements
        if !self.access_control.required_roles.is_empty() {
            let has_required_role = self
                .access_control
                .required_roles
                .iter()
                .any(|required_role| user_roles.contains(required_role));

            if !has_required_role {
                return false;
            }
        }

        // Check admin level requirements
        if let Some(_required_admin_level) = &self.access_control.admin_level_required
            && admin_level.is_none()
        {
            return false;
        }
        // In production, implement proper admin level hierarchy checking

        // Check time-based access
        if let Some(time_access) = &self.access_control.time_based_access
            && !self.check_time_based_access(time_access)
        {
            return false;
        }

        true
    }

    /// Check time-based access restrictions
    fn check_time_based_access(&self, time_access: &TimeBasedAccess) -> bool {
        let now = Utc::now();

        // Check validity period
        if let Some(valid_from) = time_access.valid_from
            && now < valid_from
        {
            return false;
        }

        if let Some(valid_until) = time_access.valid_until
            && now > valid_until
        {
            return false;
        }

        // Check allowed hours and days
        // In production, implement proper timezone handling
        let hour = now.hour() as u8;
        let weekday = now.weekday().num_days_from_sunday() as u8;

        if let Some(allowed_hours) = &time_access.allowed_hours
            && !allowed_hours.contains(&hour)
        {
            return false;
        }

        if let Some(allowed_days) = &time_access.allowed_days
            && !allowed_days.contains(&weekday)
        {
            return false;
        }

        true
    }

    /// Add an audit event to the trail
    pub fn add_audit_event(&mut self, event: AuditEvent) {
        match event.event_type {
            AuditEventType::Read => {
                self.audit_trail.access_events.push(event);
                self.last_accessed = Utc::now();
            }
            AuditEventType::Update => {
                self.audit_trail.modification_events.push(event);
                self.updated_at = Utc::now();
            }
            _ => {
                // For other events, add to access events
                self.audit_trail.access_events.push(event);
            }
        }
    }

    /// Get risk score based on access patterns and metadata
    pub fn calculate_risk_score(&self) -> f64 {
        let mut risk_score = 0.0;

        // Base risk from security level
        risk_score += match self.metadata.security_level {
            SecurityLevel::Public => 0.1,
            SecurityLevel::Internal => 0.3,
            SecurityLevel::Confidential => 0.6,
            SecurityLevel::Secret => 0.8,
            SecurityLevel::TopSecret => 1.0,
        };

        // Risk from access frequency
        let access_count = self.audit_trail.access_events.len() as f64;
        if access_count > 100.0 {
            risk_score += 0.2; // High access frequency increases risk
        }

        // Risk from failed access attempts
        let failed_attempts = self
            .audit_trail
            .access_events
            .iter()
            .filter(|event| matches!(event.result, OperationResult::AccessDenied(_)))
            .count() as f64;

        if failed_attempts > 5.0 {
            risk_score += 0.3; // Multiple failed attempts increase risk
        }

        // Risk from compliance flags
        if !self.metadata.compliance_flags.is_empty() {
            risk_score += 0.1 * self.metadata.compliance_flags.len() as f64;
        }

        // Cap at 1.0
        risk_score.min(1.0)
    }

    /// Check if secret requires audit for access
    pub fn requires_audit(&self) -> bool {
        self.access_control.audit_required
            || self.metadata.security_level >= SecurityLevel::Confidential
    }

    /// Get satker owner (replaces instansi_owner)
    pub fn get_satker_owner(&self) -> &str {
        &self.satker_owner
    }

    /// Set satker owner
    pub fn set_satker_owner(&mut self, satker_code: String) {
        self.satker_owner = satker_code;
        self.updated_at = Utc::now();
    }

    /// Add compliance flag
    pub fn add_compliance_flag(&mut self, flag: String) {
        if !self.metadata.compliance_flags.contains(&flag) {
            self.metadata.compliance_flags.push(flag);
            self.updated_at = Utc::now();
        }
    }

    /// Remove compliance flag
    pub fn remove_compliance_flag(&mut self, flag: &str) {
        self.metadata.compliance_flags.retain(|f| f != flag);
        self.updated_at = Utc::now();
    }

    /// Check if secret has compliance flag
    pub fn has_compliance_flag(&self, flag: &str) -> bool {
        self.metadata.compliance_flags.contains(&flag.to_string())
    }
}
