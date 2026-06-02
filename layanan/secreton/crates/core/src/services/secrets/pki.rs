//! PKI Secrets Engine - Simplified Production Implementation
//!
//! Provides PKI functionality for certificate lifecycle management.
//! Compatible with rcgen 0.13.x API.

use chrono::{DateTime, Duration, Utc};
use rcgen::{CertificateParams, DistinguishedName, DnType, IsCa, KeyPair};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// PKI Engine errors
#[derive(Debug, thiserror::Error)]
pub enum PkiError {
    #[error("Certificate generation failed: {0}")]
    GenerationFailed(String),

    #[error("Certificate not found: {0}")]
    CertificateNotFound(String),

    #[error("CA not found: {0}")]
    CaNotFound(String),

    #[error("Role not found: {0}")]
    RoleNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Key algorithm types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyAlgorithm {
    EcdsaP256,
    Ed25519,
}

/// Certificate Authority type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CaType {
    Root,
    Intermediate,
}

/// CA Certificate storage
#[derive(Debug, Clone)]
pub struct CertificateAuthority {
    pub name: String,
    pub ca_type: CaType,
    pub certificate_pem: String,
    pub private_key_pem: String,
    pub serial_counter: u64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub parent_ca: Option<String>, // Name of parent CA for intermediate CAs
}

/// Certificate template (enhanced PKI role with constraints)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateTemplate {
    pub name: String,
    pub ttl: Duration,
    pub max_ttl: Duration,
    pub allow_any_name: bool,
    pub allowed_domains: Vec<String>,
    pub key_usage: Vec<String>,
    pub ext_key_usage: Vec<String>,
    pub require_cn: bool,
    pub allow_localhost: bool,
    pub allow_ip_sans: bool,
    pub server_flag: bool,
    pub client_flag: bool,
    pub code_signing_flag: bool,
    pub email_protection_flag: bool,
}

impl Default for CertificateTemplate {
    fn default() -> Self {
        Self {
            name: String::new(),
            ttl: Duration::days(90),
            max_ttl: Duration::days(365),
            allow_any_name: false,
            allowed_domains: Vec::new(),
            key_usage: vec![
                "DigitalSignature".to_string(),
                "KeyEncipherment".to_string(),
            ],
            ext_key_usage: vec!["ServerAuth".to_string()],
            require_cn: true,
            allow_localhost: false,
            allow_ip_sans: false,
            server_flag: true,
            client_flag: false,
            code_signing_flag: false,
            email_protection_flag: false,
        }
    }
}

/// PKI Role (certificate template) - alias for backward compatibility
pub type PkiRole = CertificateTemplate;

/// Issued certificate response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuedCertificate {
    pub serial_number: String,
    pub certificate_pem: String,
    pub private_key_pem: String,
    pub ca_chain: Vec<String>,
    pub expires_at: DateTime<Utc>,
}

/// Certificate issue request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCertificateRequest {
    pub common_name: String,
    pub alt_names: Vec<String>,
    pub ttl: Option<Duration>,
}

/// Revoked certificate entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokedCertificate {
    pub serial_number: String,
    pub revoked_at: DateTime<Utc>,
}

/// OCSP certificate status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OcspStatus {
    Good,
    Revoked,
    Unknown,
}

/// OCSP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcspResponse {
    pub serial_number: String,
    pub status: OcspStatus,
    pub this_update: DateTime<Utc>,
    pub next_update: DateTime<Utc>,
    pub revocation_time: Option<DateTime<Utc>>,
}

/// Certificate renewal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenewalConfig {
    pub enabled: bool,
    pub threshold_days: i64,
    pub check_interval_seconds: u64,
}

impl Default for RenewalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold_days: 30,
            check_interval_seconds: 3600, // Check every hour
        }
    }
}

/// PKI Secrets Engine
pub struct PkiEngine {
    cas: Arc<RwLock<HashMap<String, CertificateAuthority>>>,
    roles: Arc<RwLock<HashMap<String, PkiRole>>>,
    issued_certificates: Arc<RwLock<HashMap<String, IssuedCertificate>>>,
    revoked_certificates: Arc<RwLock<HashMap<String, RevokedCertificate>>>,
    renewal_config: Arc<RwLock<RenewalConfig>>,
}

impl PkiEngine {
    /// Create new PKI engine instance
    pub fn new() -> Self {
        Self {
            cas: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            issued_certificates: Arc::new(RwLock::new(HashMap::new())),
            revoked_certificates: Arc::new(RwLock::new(HashMap::new())),
            renewal_config: Arc::new(RwLock::new(RenewalConfig::default())),
        }
    }

    /// Create new PKI engine with custom renewal configuration
    pub fn with_renewal_config(renewal_config: RenewalConfig) -> Self {
        Self {
            cas: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            issued_certificates: Arc::new(RwLock::new(HashMap::new())),
            revoked_certificates: Arc::new(RwLock::new(HashMap::new())),
            renewal_config: Arc::new(RwLock::new(renewal_config)),
        }
    }

    /// Update renewal configuration
    pub async fn set_renewal_config(&self, config: RenewalConfig) {
        let mut renewal_config = self.renewal_config.write().await;
        *renewal_config = config;
    }

    /// Get renewal configuration
    pub async fn get_renewal_config(&self) -> RenewalConfig {
        self.renewal_config.read().await.clone()
    }

    /// Generate Intermediate CA signed by parent CA
    pub async fn generate_intermediate_ca(
        &self,
        common_name: String,
        ttl_days: i64,
        parent_ca_name: &str,
    ) -> Result<CertificateAuthority, PkiError> {
        // Get parent CA
        let mut cas = self.cas.write().await;
        let parent_ca = cas
            .get_mut(parent_ca_name)
            .ok_or_else(|| PkiError::CaNotFound(parent_ca_name.to_string()))?;

        // Parse parent CA key pair
        let parent_key_pair = KeyPair::from_pem(&parent_ca.private_key_pem).map_err(|e| {
            PkiError::GenerationFailed(format!("Failed to parse parent CA key: {}", e))
        })?;

        // Reconstruct parent CA certificate params for signing
        let mut parent_dn = DistinguishedName::new();
        parent_dn.push(DnType::CommonName, parent_ca.name.clone());

        let mut parent_params = CertificateParams::default();
        parent_params.distinguished_name = parent_dn;
        parent_params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

        // Reconstruct parent CA issuer
        let parent_issuer = rcgen::Issuer::from_params(&parent_params, &parent_key_pair);

        // Create intermediate CA distinguished name
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, common_name.clone());

        // Create intermediate CA certificate params with path length constraint
        let mut params = CertificateParams::default();
        params.distinguished_name = dn;
        params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Constrained(0)); // Path length = 0

        // Generate intermediate CA key pair
        let key_pair = KeyPair::generate().map_err(|e| {
            PkiError::GenerationFailed(format!("Failed to generate intermediate CA key: {}", e))
        })?;

        // Sign intermediate CA with parent CA
        let cert = params.signed_by(&key_pair, &parent_issuer).map_err(|e| {
            PkiError::GenerationFailed(format!("Failed to sign intermediate CA: {}", e))
        })?;

        let certificate_pem = cert.pem();
        let private_key_pem = key_pair.serialize_pem();

        let created_at = Utc::now();
        let expires_at = created_at + Duration::days(ttl_days);

        let intermediate_name = format!("intermediate-{}", common_name);
        let ca = CertificateAuthority {
            name: intermediate_name.clone(),
            ca_type: CaType::Intermediate,
            certificate_pem,
            private_key_pem,
            serial_counter: 1,
            created_at,
            expires_at,
            parent_ca: Some(parent_ca_name.to_string()),
        };

        // Store intermediate CA
        cas.insert(intermediate_name, ca.clone());

        Ok(ca)
    }

    /// Generate Root CA
    pub async fn generate_root_ca(
        &self,
        common_name: String,
        ttl_days: i64,
    ) -> Result<CertificateAuthority, PkiError> {
        // Create distinguished name
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, common_name);

        // Create certificate params
        let mut params = CertificateParams::default();
        params.distinguished_name = dn;
        params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

        // Generate key pair
        let key_pair = KeyPair::generate().map_err(|e| {
            PkiError::GenerationFailed(format!("Failed to generate key pair: {}", e))
        })?;

        // Self-sign the certificate
        let cert = params.self_signed(&key_pair).map_err(|e| {
            PkiError::GenerationFailed(format!("Failed to generate root CA: {}", e))
        })?;

        let certificate_pem = cert.pem();
        let private_key_pem = key_pair.serialize_pem();

        let created_at = Utc::now();
        let expires_at = created_at + Duration::days(ttl_days);

        let ca = CertificateAuthority {
            name: "root".to_string(),
            ca_type: CaType::Root,
            certificate_pem,
            private_key_pem,
            serial_counter: 1,
            created_at,
            expires_at,
            parent_ca: None,
        };

        // Store CA
        let mut cas = self.cas.write().await;
        cas.insert("root".to_string(), ca.clone());

        Ok(ca)
    }

    /// Create PKI role
    pub async fn create_role(&self, role: PkiRole) -> Result<(), PkiError> {
        if role.name.is_empty() {
            return Err(PkiError::InvalidConfig(
                "Role name cannot be empty".to_string(),
            ));
        }

        let mut roles = self.roles.write().await;
        roles.insert(role.name.clone(), role);

        Ok(())
    }

    /// Issue certificate from role
    pub async fn issue_certificate(
        &self,
        role_name: &str,
        request: IssueCertificateRequest,
    ) -> Result<IssuedCertificate, PkiError> {
        // Get role and clone it to avoid holding the lock
        let role = {
            let roles = self.roles.read().await;
            roles
                .get(role_name)
                .ok_or_else(|| PkiError::RoleNotFound(role_name.to_string()))?
                .clone()
        }; // Lock is dropped here

        // Validate common name requirement
        if role.require_cn && request.common_name.is_empty() {
            return Err(PkiError::InvalidConfig(
                "Common name is required by template".to_string(),
            ));
        }

        // Validate domain
        if !role.allow_any_name && !role.allowed_domains.is_empty() {
            let allowed = role
                .allowed_domains
                .iter()
                .any(|d| request.common_name.ends_with(d));
            if !allowed {
                return Err(PkiError::InvalidConfig(format!(
                    "Domain {} not allowed by role",
                    request.common_name
                )));
            }
        }

        // Validate localhost
        if !role.allow_localhost && request.common_name == "localhost" {
            return Err(PkiError::InvalidConfig(
                "Localhost not allowed by template".to_string(),
            ));
        }

        // Validate TTL against max_ttl
        let requested_ttl = request.ttl.unwrap_or(role.ttl);
        if requested_ttl > role.max_ttl {
            return Err(PkiError::InvalidConfig(format!(
                "Requested TTL {:?} exceeds maximum {:?}",
                requested_ttl, role.max_ttl
            )));
        }

        // Get CA (now safe to acquire write lock)
        let mut cas = self.cas.write().await;
        let ca = cas
            .get_mut("root")
            .ok_or_else(|| PkiError::CaNotFound("root".to_string()))?;

        // Parse CA key pair
        let ca_key_pair = KeyPair::from_pem(&ca.private_key_pem)
            .map_err(|e| PkiError::GenerationFailed(format!("Failed to parse CA key: {}", e)))?;

        // Reconstruct CA certificate params for signing
        let mut ca_dn = DistinguishedName::new();
        ca_dn.push(DnType::CommonName, ca.name.clone());

        let mut ca_params = CertificateParams::default();
        ca_params.distinguished_name = ca_dn;
        ca_params.is_ca = IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

        // Reconstruct CA issuer for signing
        let ca_issuer = rcgen::Issuer::from_params(&ca_params, &ca_key_pair);

        // Create leaf certificate params
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, request.common_name.clone());

        let mut params = CertificateParams::default();
        params.distinguished_name = dn;
        params.is_ca = IsCa::NoCa;

        // Generate leaf key pair
        let leaf_key_pair = KeyPair::generate().map_err(|e| {
            PkiError::GenerationFailed(format!("Failed to generate leaf key: {}", e))
        })?;

        // Sign with CA
        let cert = params.signed_by(&leaf_key_pair, &ca_issuer).map_err(|e| {
            PkiError::GenerationFailed(format!("Failed to sign certificate: {}", e))
        })?;

        let certificate_pem = cert.pem();
        let private_key_pem = leaf_key_pair.serialize_pem();

        // Generate serial
        let serial_number = format!("{:016x}", ca.serial_counter);
        ca.serial_counter += 1;

        let ttl = request.ttl.unwrap_or(role.ttl);
        let expires_at = Utc::now() + ttl;

        let issued = IssuedCertificate {
            serial_number: serial_number.clone(),
            certificate_pem,
            private_key_pem,
            ca_chain: vec![ca.certificate_pem.clone()],
            expires_at,
        };

        // Store issued certificate
        drop(cas);
        let mut issued_certs = self.issued_certificates.write().await;
        issued_certs.insert(serial_number, issued.clone());

        Ok(issued)
    }

    /// Revoke certificate
    pub async fn revoke_certificate(&self, serial_number: &str) -> Result<(), PkiError> {
        let issued = self.issued_certificates.read().await;
        if !issued.contains_key(serial_number) {
            return Err(PkiError::CertificateNotFound(serial_number.to_string()));
        }

        let revoked = RevokedCertificate {
            serial_number: serial_number.to_string(),
            revoked_at: Utc::now(),
        };

        let mut revoked_certs = self.revoked_certificates.write().await;
        revoked_certs.insert(serial_number.to_string(), revoked);

        Ok(())
    }

    /// List CAs
    pub async fn list_cas(&self) -> Vec<String> {
        let cas = self.cas.read().await;
        cas.keys().cloned().collect()
    }

    /// List roles
    pub async fn list_roles(&self) -> Vec<String> {
        let roles = self.roles.read().await;
        roles.keys().cloned().collect()
    }

    /// Get certificate by serial
    pub async fn get_certificate(
        &self,
        serial_number: &str,
    ) -> Result<IssuedCertificate, PkiError> {
        let certs = self.issued_certificates.read().await;
        certs
            .get(serial_number)
            .cloned()
            .ok_or_else(|| PkiError::CertificateNotFound(serial_number.to_string()))
    }

    /// Generate CRL (Certificate Revocation List)
    pub async fn generate_crl(&self) -> Result<Vec<RevokedCertificate>, PkiError> {
        let revoked = self.revoked_certificates.read().await;
        Ok(revoked.values().cloned().collect())
    }

    /// Get OCSP status for a certificate
    pub async fn get_ocsp_status(&self, serial_number: &str) -> Result<OcspResponse, PkiError> {
        let now = Utc::now();
        let next_update = now + Duration::hours(24);

        // Check if certificate is revoked
        let revoked = self.revoked_certificates.read().await;
        if let Some(revoked_cert) = revoked.get(serial_number) {
            return Ok(OcspResponse {
                serial_number: serial_number.to_string(),
                status: OcspStatus::Revoked,
                this_update: now,
                next_update,
                revocation_time: Some(revoked_cert.revoked_at),
            });
        }

        // Check if certificate exists and is valid
        let issued = self.issued_certificates.read().await;
        if let Some(cert) = issued.get(serial_number) {
            // Check if certificate has expired
            if cert.expires_at < now {
                return Ok(OcspResponse {
                    serial_number: serial_number.to_string(),
                    status: OcspStatus::Revoked,
                    this_update: now,
                    next_update,
                    revocation_time: Some(cert.expires_at),
                });
            }

            return Ok(OcspResponse {
                serial_number: serial_number.to_string(),
                status: OcspStatus::Good,
                this_update: now,
                next_update,
                revocation_time: None,
            });
        }

        // Certificate not found
        Ok(OcspResponse {
            serial_number: serial_number.to_string(),
            status: OcspStatus::Unknown,
            this_update: now,
            next_update,
            revocation_time: None,
        })
    }

    /// Check certificates that need renewal
    pub async fn check_certificates_for_renewal(&self) -> Result<Vec<String>, PkiError> {
        let config = self.renewal_config.read().await;
        if !config.enabled {
            return Ok(vec![]);
        }

        let threshold = Duration::days(config.threshold_days);
        let now = Utc::now();
        let renewal_deadline = now + threshold;

        let issued = self.issued_certificates.read().await;
        let revoked = self.revoked_certificates.read().await;

        let mut certificates_to_renew = Vec::new();

        for (serial, cert) in issued.iter() {
            // Skip revoked certificates
            if revoked.contains_key(serial) {
                continue;
            }

            // Check if certificate expires within threshold
            if cert.expires_at <= renewal_deadline {
                certificates_to_renew.push(serial.clone());
            }
        }

        Ok(certificates_to_renew)
    }

    /// Renew a certificate by serial number
    pub async fn renew_certificate(
        &self,
        serial_number: &str,
        role_name: &str,
    ) -> Result<IssuedCertificate, PkiError> {
        // Get the original certificate
        let issued = self.issued_certificates.read().await;
        let _original_cert = issued
            .get(serial_number)
            .ok_or_else(|| PkiError::CertificateNotFound(serial_number.to_string()))?;

        // Extract common name from the original certificate
        // For simplicity, we'll use a placeholder. In production, you'd parse the cert.
        let common_name = format!("renewed-{}", serial_number);

        drop(issued);

        // Issue a new certificate with the same parameters
        let request = IssueCertificateRequest {
            common_name,
            alt_names: vec![],
            ttl: None,
        };

        // Issue new certificate
        let new_cert = self.issue_certificate(role_name, request).await?;

        // Revoke the old certificate
        self.revoke_certificate(serial_number).await?;

        Ok(new_cert)
    }

    /// Start automatic renewal background task
    /// Returns a handle that can be used to stop the task
    pub fn start_renewal_scheduler(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                let config = self.renewal_config.read().await.clone();

                if !config.enabled {
                    // If renewal is disabled, check again in 1 hour
                    tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                    continue;
                }

                // Check for certificates that need renewal
                match self.check_certificates_for_renewal().await {
                    Ok(serials) => {
                        if !serials.is_empty() {
                            tracing::info!(
                                "Found {} certificates approaching expiration",
                                serials.len()
                            );
                            // In production, you would trigger renewal here
                            // For now, we just log the certificates
                            for serial in serials {
                                tracing::info!("Certificate {} needs renewal", serial);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to check certificates for renewal: {}", e);
                    }
                }

                // Sleep for the configured interval
                tokio::time::sleep(tokio::time::Duration::from_secs(
                    config.check_interval_seconds,
                ))
                .await;
            }
        })
    }
}

impl Default for PkiEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_root_ca() {
        let engine = PkiEngine::new();
        let result = engine
            .generate_root_ca("Test Root CA".to_string(), 3650)
            .await;

        assert!(result.is_ok());
        let ca = result.unwrap();
        assert_eq!(ca.ca_type, CaType::Root);
        assert!(!ca.certificate_pem.is_empty());
    }

    #[tokio::test]
    async fn test_create_role() {
        let engine = PkiEngine::new();
        let role = PkiRole {
            name: "web-server".to_string(),
            allowed_domains: vec!["example.com".to_string()],
            ..Default::default()
        };

        assert!(engine.create_role(role).await.is_ok());
        let roles = engine.list_roles().await;
        assert!(roles.contains(&"web-server".to_string()));
    }

    #[tokio::test]
    async fn test_issue_certificate() {
        let engine = PkiEngine::new();

        // Setup
        let _ = engine
            .generate_root_ca("Test CA".to_string(), 365)
            .await
            .unwrap();

        let role = PkiRole {
            name: "test-role".to_string(),
            allow_any_name: true,
            ..Default::default()
        };
        engine.create_role(role).await.unwrap();

        // Issue certificate
        let request = IssueCertificateRequest {
            common_name: "test.example.com".to_string(),
            alt_names: vec![],
            ttl: None,
        };

        let result = engine.issue_certificate("test-role", request).await;
        assert!(result.is_ok());

        let cert = result.unwrap();
        assert!(!cert.certificate_pem.is_empty());
        assert!(!cert.private_key_pem.is_empty());
    }

    #[tokio::test]
    async fn test_revoke_certificate() {
        let engine = PkiEngine::new();

        // Setup and issue
        let _ = engine
            .generate_root_ca("Test CA".to_string(), 365)
            .await
            .unwrap();

        let role = PkiRole {
            name: "test".to_string(),
            allow_any_name: true,
            ..Default::default()
        };
        engine.create_role(role).await.unwrap();

        let request = IssueCertificateRequest {
            common_name: "test.com".to_string(),
            alt_names: Vec::new(),
            ttl: None,
        };

        let cert = engine.issue_certificate("test", request).await.unwrap();

        // Revoke
        assert!(engine.revoke_certificate(&cert.serial_number).await.is_ok());

        // Verify CRL
        let crl = engine.generate_crl().await.unwrap();
        assert!(!crl.is_empty());
    }
}
