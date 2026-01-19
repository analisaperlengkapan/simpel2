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
/// Mewakili pub `PkiError`.
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
/// Mewakili pub `KeyAlgorithm`.
pub enum KeyAlgorithm {
    EcdsaP256,
    Ed25519,
}

/// Certificate Authority type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Mewakili pub `CaType`.
pub enum CaType {
    Root,
}

/// CA Certificate storage
#[derive(Debug, Clone)]
/// Mewakili pub `CertificateAuthority`.
pub struct CertificateAuthority {
    pub name: String,
    pub ca_type: CaType,
    pub certificate_pem: String,
    pub private_key_pem: String,
    pub serial_counter: u64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// PKI Role (certificate template)
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `PkiRole`.
pub struct PkiRole {
    pub name: String,
    pub ttl: Duration,
    pub max_ttl: Duration,
    pub allow_any_name: bool,
    pub allowed_domains: Vec<String>,
}

impl Default for PkiRole {
    fn default() -> Self {
        Self {
            name: String::new(),
            ttl: Duration::days(90),
            max_ttl: Duration::days(365),
            allow_any_name: false,
            allowed_domains: Vec::new(),
        }
    }
}

/// Issued certificate response
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `IssuedCertificate`.
pub struct IssuedCertificate {
    pub serial_number: String,
    pub certificate_pem: String,
    pub private_key_pem: String,
    pub ca_chain: Vec<String>,
    pub expires_at: DateTime<Utc>,
}

/// Certificate issue request
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `IssueCertificateRequest`.
pub struct IssueCertificateRequest {
    pub common_name: String,
    pub alt_names: Vec<String>,
    pub ttl: Option<Duration>,
}

/// Revoked certificate entry
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `RevokedCertificate`.
pub struct RevokedCertificate {
    pub serial_number: String,
    pub revoked_at: DateTime<Utc>,
}

/// PKI Secrets Engine
pub struct PkiEngine {
    cas: Arc<RwLock<HashMap<String, CertificateAuthority>>>,
    roles: Arc<RwLock<HashMap<String, PkiRole>>>,
    issued_certificates: Arc<RwLock<HashMap<String, IssuedCertificate>>>,
    revoked_certificates: Arc<RwLock<HashMap<String, RevokedCertificate>>>,
}

impl PkiEngine {
    /// Create new PKI engine instance
    pub fn new() -> Self {
        Self {
            cas: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            issued_certificates: Arc::new(RwLock::new(HashMap::new())),
            revoked_certificates: Arc::new(RwLock::new(HashMap::new())),
        }
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
        // Get role
        let roles = self.roles.read().await;
        let role = roles
            .get(role_name)
            .ok_or_else(|| PkiError::RoleNotFound(role_name.to_string()))?;

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

        // Get CA
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

        // Reconstruct CA certificate for signing
        let ca_cert = ca_params
            .self_signed(&ca_key_pair)
            .map_err(|e| PkiError::GenerationFailed(format!("Failed to reconstruct CA: {}", e)))?;

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
        let cert = params
            .signed_by(&leaf_key_pair, &ca_cert, &ca_key_pair)
            .map_err(|e| {
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
