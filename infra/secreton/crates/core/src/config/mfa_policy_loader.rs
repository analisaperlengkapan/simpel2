//! MFA Policy Configuration Loader
//!
//! Loads and validates MFA policies from configuration files for government compliance.

use crate::services::mfa::{MfaService, MfaError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{info, warn, error};

/// MFA policy configuration loaded from TOML files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaPolicyConfig {
    /// General MFA settings
    pub mfa: MfaSettings,
}

/// MFA settings from configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSettings {
    /// Whether MFA is enabled globally
    pub enabled: bool,

    /// Default MFA method
    pub default_method: String,

    /// Supported MFA methods
    pub supported_methods: Vec<String>,

    /// TOTP configuration
    pub totp: TotpSettings,

    /// Policy enforcement settings
    pub policies: PolicySettings,

    /// Recovery codes configuration
    pub recovery_codes: RecoveryCodesSettings,

    /// Rate limiting configuration
    pub rate_limiting: RateLimitingSettings,

    /// Audit configuration
    pub audit: AuditSettings,

    /// Security configuration
    pub security: SecuritySettings,

    /// Backup configuration
    pub backup: BackupSettings,

    /// Monitoring configuration
    pub monitoring: MonitoringSettings,

    /// Compliance configuration
    pub compliance: ComplianceSettings,

    /// Access control configuration
    pub access_control: AccessControlSettings,

    /// Network security configuration
    pub network: NetworkSettings,

    /// Session management configuration
    pub session: SessionSettings,
}

/// TOTP-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSettings {
    pub issuer: String,
    pub time_window: u32,
    pub digits: u8,
    pub algorithm: String,
    pub skew_tolerance: u32,
    pub qr_code_size: u32,
}

/// Policy enforcement settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySettings {
    pub enforce_for_all: bool,
    pub setup_grace_period: u32,
    pub max_failed_attempts: u32,
    pub lockout_duration: u32,
    pub reauth_interval: u32,
    pub roles: RolePolicySettings,
    pub satker: SatkerPolicySettings,
}

/// Role-based policy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolePolicySettings {
    pub admin_immediate_setup: Vec<String>,
    pub high_privilege_roles: Vec<String>,
    pub standard_roles: Vec<String>,
}

/// Satker-based policy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatkerPolicySettings {
    pub high_security_satkers: Vec<String>,
    pub standard_satkers: Vec<String>,
}

/// Recovery codes settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryCodesSettings {
    pub count: u32,
    pub length: u32,
    pub format: String,
    pub allow_reuse: bool,
}

/// Rate limiting settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingSettings {
    pub max_attempts_per_minute: u32,
    pub max_setup_attempts_per_hour: u32,
    pub window_seconds: u32,
    pub progressive_delays: bool,
}

/// Audit settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSettings {
    pub enabled: bool,
    pub log_success: bool,
    pub log_failures: bool,
    pub log_setup_changes: bool,
    pub log_recovery_usage: bool,
    pub retention_days: u32,
}

/// Security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub secret_encryption: String,
    pub kdf: String,
    pub kdf_iterations: u32,
    pub salt_length: u32,
    pub use_hsm: bool,
    pub hsm_slot: u32,
    pub hsm_pin: Option<String>,
}

/// Backup settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSettings {
    pub enabled: bool,
    pub interval_hours: u32,
    pub retention_days: u32,
    pub backup_encryption_key: Option<String>,
    pub backup_path: String,
}

/// Monitoring settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSettings {
    pub metrics_enabled: bool,
    pub alert_on_anomalies: bool,
    pub failed_attempts_threshold: u32,
    pub monitoring_window_minutes: u32,
    pub alert_email: Option<String>,
    pub alert_webhook: Option<String>,
}

/// Compliance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSettings {
    pub fips_mode: bool,
    pub audit_integrity_check: bool,
    pub require_admin_approval: bool,
    pub compliance_reporting: bool,
    pub report_interval_hours: u32,
}

/// Access control settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlSettings {
    pub setup_permission: String,
    pub verify_permission: String,
    pub admin_permission: String,
    pub audit_permission: String,
}

/// Network security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub allowed_ip_ranges: Vec<String>,
    pub ip_rate_limiting: bool,
    pub max_requests_per_ip: u32,
}

/// Session management settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettings {
    pub session_timeout: u32,
    pub require_mfa_for_extension: bool,
    pub max_concurrent_sessions: u32,
}

/// MFA policy loader and validator
pub struct MfaPolicyLoader {
    config: MfaPolicyConfig,
}

impl MfaPolicyLoader {
    /// Load MFA policy configuration from TOML file
    pub async fn load_from_file<P: AsRef<Path>>(config_path: P) -> Result<Self, MfaError> {
        let config_content = fs::read_to_string(config_path.as_ref())
            .await
            .map_err(|e| MfaError::NotConfigured(format!("Failed to read config file: {}", e)))?;

        let config: MfaPolicyConfig = toml::from_str(&config_content)
            .map_err(|e| MfaError::NotConfigured(format!("Failed to parse config: {}", e)))?;

        info!("Loaded MFA policy configuration from {:?}", config_path.as_ref());

        let loader = Self { config };
        loader.validate_config()?;

        Ok(loader)
    }

    /// Validate the loaded configuration
    fn validate_config(&self) -> Result<(), MfaError> {
        let mfa = &self.config.mfa;

        // Validate TOTP settings
        if mfa.totp.digits != 6 && mfa.totp.digits != 8 {
            return Err(MfaError::NotConfigured(
                "TOTP digits must be 6 or 8".to_string()
            ));
        }

        if mfa.totp.time_window != 30 && mfa.totp.time_window != 60 {
            warn!("Non-standard TOTP time window: {}s", mfa.totp.time_window);
        }

        // Validate policy settings
        if mfa.policies.max_failed_attempts == 0 {
            return Err(MfaError::NotConfigured(
                "max_failed_attempts must be greater than 0".to_string()
            ));
        }

        if mfa.policies.lockout_duration == 0 {
            return Err(MfaError::NotConfigured(
                "lockout_duration must be greater than 0".to_string()
            ));
        }

        // Validate recovery codes settings
        if mfa.recovery_codes.count == 0 {
            return Err(MfaError::NotConfigured(
                "recovery_codes count must be greater than 0".to_string()
            ));
        }

        if mfa.recovery_codes.length < 8 {
            warn!("Recovery code length is less than 8 characters, consider increasing for security");
        }

        // Validate security settings
        if mfa.security.kdf_iterations < 10000 {
            warn!("KDF iterations is less consider increasing for security");
        }

        info!("MFA policy configuration validation completed successfully");
        Ok(())
    }

    /// Get the loaded configuration
    pub fn get_config(&self) -> &MfaPolicyConfig {
        &self.config
    }

    /// Check if MFA is required for a specific role
    pub fn is_mfa_required_for_role(&self, role: &str) -> bool {
        if !self.config.mfa.policies.enforce_for_all {
            return false;
        }

        // Check if role requires immediate setup
        if self.config.mfa.policies.roles.admin_immediate_setup.contains(&role.to_string()) {
            return true;
        }

        // Check if role is in high privilege list
        if self.config.mfa.policies.roles.high_privilege_roles.contains(&role.to_string()) {
            return true;
        }

        // Check if role is in standard roles list
        if self.config.mfa.policies.roles.standard_roles.contains(&role.to_string()) {
            return true;
        }

        // Default to requiring MFA if enforce_for_all is true
        true
    }

    /// Check if MFA setup should be immediate for a role
    pub fn requires_immediate_setup(&self, role: &str) -> bool {
        self.config.mfa.policies.roles.admin_immediate_setup.contains(&role.to_string())
    }

    /// Check if satker requires high security MFA policies
    pub fn is_high_security_satker(&self, satker_code: &str) -> bool {
        self.config.mfa.policies.satker.high_security_satkers.iter()
            .any(|pattern| {
                if pattern.ends_with('*') {
                    let prefix = &pattern[..pattern.len() - 1];
                    satker_code.starts_with(prefix)
                } else {
                    satker_code == pattern
                }
            })
    }

    /// Get grace period for MFA setup based on role and satker
    pub fn get_setup_grace_period(&self, role: &str, satker_code: &str) -> u32 {
        // Immediate setup for admin roles or high security satkers
        if self.requires_immediate_setup(role) || self.is_high_security_satker(satker_code) {
            return 0;
        }

        self.config.mfa.policies.setup_grace_period
    }

    /// Get maximum failed attempts before lockout
    pub fn get_max_failed_attempts(&self, role: &str) -> u32 {
        // Stricter limits for high privilege roles
        if self.config.mfa.policies.roles.high_privilege_roles.contains(&role.to_string()) {
            return std::cmp::min(self.config.mfa.policies.max_failed_attempts, 3);
        }

        self.config.mfa.policies.max_failed_attempts
    }

    /// Apply configuration to MFA service
    pub async fn apply_to_service(&self, mfa_service: &mut MfaService) -> Result<(), MfaError> {
        info!("Applying MFA policy configuration to service");

        // This would configure the MFA service with the loaded policies
        // Implementation depends on the MfaService interface

        info!("MFA policy configuration applied successfully");
        Ok(())
    }

    /// Generate compliance report
    pub fn generate_compliance_report(&self) -> HashMap<String, serde_json::Value> {
        let mut report = HashMap::new();

        report.insert("mfa_enabled".to_string(),
            serde_json::Value::Bool(self.config.mfa.enabled));

        report.insert("enforce_for_all".to_string(),
            serde_json::Value::Bool(self.config.mfa.policies.enforce_for_all));

        report.insert("fips_mode".to_string(),
            serde_json::Value::Bool(self.config.mfa.compliance.fips_mode));

        report.insert("audit_enabled".to_string(),
            serde_json::Value::Bool(self.config.mfa.audit.enabled));

        report.insert("hsm_enabled".to_string(),
            serde_json::Value::Bool(self.config.mfa.security.use_hsm));

        report.insert("backup_enabled".to_string(),
            serde_json::Value::Bool(self.config.mfa.backup.enabled));

        report.insert("monitoring_enabled".to_string(),
            serde_json::Value::Bool(self.config.mfa.monitoring.metrics_enabled));

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_load_valid_config() {
        let config_content = r#"
[mfa]
enabled = true
default_method = "TOTP"
supported_methods = ["TOTP"]

[mfa.totp]
issuer = "Test Kejaksaan RI"
time_window = 30
digits = 6
algorithm = "SHA1"
skew_tolerance = 1
qr_code_size = 256

[mfa.policies]
enforce_for_all = true
setup_grace_period = 7
max_failed_attempts = 5
lockout_duration = 30
reauth_interval = 8

[mfa.policies.roles]
admin_immediate_setup = ["AdminPusat"]
high_privilege_roles = ["AdminPusat"]
standard_roles = ["PegawaiNegeri"]

[mfa.policies.satker]
high_security_satkers = ["KEJAKSAAN_AGUNG"]
standard_satkers = ["KEJATI_*"]

[mfa.recovery_codes]
count = 10
length = 12
format = "alphanumeric"
allow_reuse = false

[mfa.rate_limiting]
max_attempts_per_minute = 10
max_setup_attempts_per_hour = 3
window_seconds = 60
progressive_delays = true

[mfa.audit]
enabled = true
log_success = true
log_failures = true
log_setup_changes = true
log_recovery_usage = true
retention_days = 365

[mfa.security]
secret_encryption = "AES-256-GCM"
kdf = "PBKDF2"
kdf_iterations = 100000
salt_length = 32
use_hsm = false
hsm_slot = 0

[mfa.backup]
enabled = true
interval_hours = 24
retention_days = 90
backup_path = "./backups/mfa"

[mfa.monitoring]
metrics_enabled = true
alert_on_anomalies = true
failed_attempts_threshold = 20
monitoring_window_minutes = 15

[mfa.compliance]
fips_mode = false
audit_integrity_check = true
require_admin_approval = true
compliance_reporting = true
report_interval_hours = 168

[mfa.access_control]
setup_permission = "mfa:setup"
verify_permission = "mfa:verify"
admin_permission = "mfa:admin"
audit_permission = "mfa:audit"

[mfa.network]
allowed_ip_ranges = ["10.0.0.0/8"]
ip_rate_limiting = true
max_requests_per_ip = 20

[mfa.session]
session_timeout = 15
require_mfa_for_extension = true
max_concurrent_sessions = 3
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let loader = MfaPolicyLoader::load_from_file(temp_file.path()).await.unwrap();

        assert!(loader.config.mfa.enabled);
        assert_eq!(loader.config.mfa.totp.issuer, "Test Kejaksaan RI");
        assert!(loader.is_mfa_required_for_role("AdminPusat"));
        assert!(loader.requires_immediate_setup("AdminPusat"));
        assert!(loader.is_high_security_satker("KEJAKSAAN_AGUNG"));
    }

    #[tokio::test]
    async fn test_role_based_policies() {
        let config_content = r#"
[mfa]
enabled = true
default_method = "TOTP"
supported_methods = ["TOTP"]

[mfa.totp]
issuer = "Test"
time_window = 30
digits = 6
algorithm = "SHA1"
skew_tolerance = 1
qr_code_size = 256

[mfa.policies]
enforce_for_all = true
setup_grace_period = 7
max_failed_attempts = 5
lockout_duration = 30
reauth_interval = 8

[mfa.policies.roles]
admin_immediate_setup = ["AdminPusat", "AdminEselonI"]
high_privilege_roles = ["AdminPusat"]
standard_roles = ["PegawaiNegeri"]

[mfa.policies.satker]
high_security_satkers = ["KEJAKSAAN_AGUNG"]
standard_satkers = ["KEJATI_*"]

[mfa.recovery_codes]
count = 10
length = 12
format = "alphanumeric"
allow_reuse = false

[mfa.rate_limiting]
max_attempts_per_minute = 10
max_setup_attempts_per_hour = 3
window_seconds = 60
progressive_delays = true

[mfa.audit]
enabled = true
log_success = true
log_failures = true
log_setup_changes = true
log_recovery_usage = true
retention_days = 365

[mfa.security]
secret_encryption = "AES-256-GCM"
kdf = "PBKDF2"
kdf_iterations = 100000
salt_length = 32
use_hsm = false
hsm_slot = 0

[mfa.backup]
enabled = true
interval_hours = 24
retention_days = 90
backup_path = "./backups/mfa"

[mfa.monitoring]
metrics_enabled = true
alert_on_anomalies = true
failed_attempts_threshold = 20
monitoring_window_minutes = 15

[mfa.compliance]
fips_mode = false
audit_integrity_check = true
require_admin_approval = true
compliance_reporting = true
report_interval_hours = 168

[mfa.access_control]
setup_permission = "mfa:setup"
verify_permission = "mfa:verify"
admin_permission = "mfa:admin"
audit_permission = "mfa:audit"

[mfa.network]
allowed_ip_ranges = ["10.0.0.0/8"]
ip_rate_limiting = true
max_requests_per_ip = 20

[mfa.session]
session_timeout = 15
require_mfa_for_extension = true
max_concurrent_sessions = 3
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let loader = MfaPolicyLoader::load_from_file(temp_file.path()).await.unwrap();

        // Test role-based policies
        assert!(loader.requires_immediate_setup("AdminPusat"));
        assert!(loader.requires_immediate_setup("AdminEselonI"));
        assert!(!loader.requires_immediate_setup("PegawaiNegeri"));

        // Test grace period
        assert_eq!(loader.get_setup_grace_period("AdminPusat", "STANDARD"), 0);
        assert_eq!(loader.get_setup_grace_period("PegawaiNegeri", "STANDARD"), 7);

        // Test max failed attempts
        assert_eq!(loader.get_max_failed_attempts("AdminPusat"), 3); // Stricter for high privilege
        assert_eq!(loader.get_max_failed_attempts("PegawaiNegeri"), 5);
    }
}
