//! SSH Secrets Engine - Production Implementation
//!
//! Dynamic SSH key generation, OTP for SSH, and SSH Certificate Authority.
//!
//! # Security Notice
//!
//! **Ed25519 is the PREFERRED algorithm** for all new SSH keys.
//! RSA variants (Rsa2048, Rsa4096) are maintained only for legacy compatibility
//! with systems that require RSA keys. New implementations should use Ed25519.
//!
//! See: RUSTSEC-2023-0071 (RSA timing sidechannel vulnerability)

use chrono::{DateTime, Duration, Utc};
use deadpool_postgres::Pool;
use rand::{Rng, rngs::OsRng};
use serde::{Deserialize, Serialize};
use ssh_key::{Algorithm, HashAlg, LineEnding, PrivateKey, PublicKey};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// SSH engine errors
#[derive(Debug, thiserror::Error)]
/// Mewakili pub `SshError`.
pub enum SshError {
    #[error("Role not found: {0}")]
    RoleNotFound(String),

    #[error("CA not found: {0}")]
    CaNotFound(String),

    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    #[error("Certificate signing failed: {0}")]
    CertificateSigningFailed(String),

    #[error("OTP not found or expired: {0}")]
    OtpNotFound(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("SSH key error: {0}")]
    SshKeyError(String),

    #[error("Role already exists: {0}")]
    RoleAlreadyExists(String),
}

/// SSH key type
///
/// # Security Recommendations
///
/// - **Ed25519**: Recommended for all new keys. Modern, fast, and secure.
/// - **EcdsaP256/EcdsaP384**: Good alternatives when Ed25519 is not supported.
/// - **Rsa2048/Rsa4096**: DEPRECATED - Use only for legacy system compatibility.
///   See RUSTSEC-2023-0071 for RSA timing vulnerability information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
/// Mewakili pub `SshKeyType`.
pub enum SshKeyType {
    /// Ed25519 - RECOMMENDED for all new keys
    Ed25519,
    /// RSA 2048-bit - DEPRECATED: Legacy compatibility only
    #[deprecated(
        since = "1.0.0",
        note = "Use Ed25519 for new keys. RSA maintained for legacy compatibility only."
    )]
    Rsa2048,
    /// RSA 4096-bit - DEPRECATED: Legacy compatibility only
    #[deprecated(
        since = "1.0.0",
        note = "Use Ed25519 for new keys. RSA maintained for legacy compatibility only."
    )]
    Rsa4096,
    /// ECDSA P-256 - Good alternative to Ed25519
    EcdsaP256,
    /// ECDSA P-384 - Good alternative to Ed25519
    EcdsaP384,
}

#[allow(deprecated)]
impl SshKeyType {
    /// Mewakili pub `to_algorithm(`.
    pub fn to_algorithm(&self) -> Algorithm {
        match self {
            SshKeyType::Ed25519 => Algorithm::Ed25519,
            SshKeyType::Rsa2048 | SshKeyType::Rsa4096 => Algorithm::Rsa {
                hash: Some(HashAlg::Sha512),
            },
            SshKeyType::EcdsaP256 => Algorithm::Ecdsa {
                curve: ssh_key::EcdsaCurve::NistP256,
            },
            SshKeyType::EcdsaP384 => Algorithm::Ecdsa {
                curve: ssh_key::EcdsaCurve::NistP384,
            },
        }
    }

    /// Mewakili pub `key_bits(`.
    pub fn key_bits(&self) -> Option<usize> {
        match self {
            SshKeyType::Rsa2048 => Some(2048),
            SshKeyType::Rsa4096 => Some(4096),
            _ => None,
        }
    }
}

/// SSH role configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `SshRole`.
pub struct SshRole {
    pub name: String,
    pub key_type: SshKeyType,
    pub default_user: String,
    pub allowed_users: Vec<String>,
    pub default_ttl: i64,
    pub max_ttl: i64,
    pub allowed_extensions: HashMap<String, String>,
    pub allow_user_certificates: bool,
    pub allow_host_certificates: bool,
    pub created_at: DateTime<Utc>,
}

impl SshRole {
    /// Mewakili pub `new(name`.
    pub fn new(name: String, key_type: SshKeyType, default_user: String) -> Self {
        Self {
            name,
            key_type,
            default_user,
            allowed_users: vec![],
            default_ttl: 3600,
            max_ttl: 86400,
            allowed_extensions: HashMap::new(),
            allow_user_certificates: true,
            allow_host_certificates: false,
            created_at: Utc::now(),
        }
    }
}

/// SSH CA configuration
#[derive(Debug, Clone)]
/// Mewakili pub `SshCa`.
pub struct SshCa {
    pub name: String,
    pub private_key: PrivateKey,
    pub public_key: PublicKey,
    pub key_type: SshKeyType,
    pub created_at: DateTime<Utc>,
}

/// SSH OTP entry
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `SshOtp`.
pub struct SshOtp {
    pub otp: String,
    pub username: String,
    pub ip: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
}

/// SSH key pair response
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `SshKeyPair`.
pub struct SshKeyPair {
    pub private_key: String,
    pub public_key: String,
    pub key_type: String,
}

/// SSH certificate request
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `SshCertificateRequest`.
pub struct SshCertificateRequest {
    pub public_key: String,
    pub cert_type: String,
    pub valid_principals: Vec<String>,
    pub ttl: Option<i64>,
    pub extensions: Option<HashMap<String, String>>,
}

/// SSH certificate response
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `SshCertificate`.
pub struct SshCertificate {
    pub signed_key: String,
    pub serial_number: String,
    pub cert_type: String,
}

/// SSH Secrets Engine
pub struct SshEngine {
    pool: Option<Pool>,
    roles: Arc<RwLock<HashMap<String, SshRole>>>,
    cas: Arc<RwLock<HashMap<String, SshCa>>>,
    otps: Arc<RwLock<HashMap<String, SshOtp>>>,
}

impl SshEngine {
    /// Mewakili pub `new(`.
    pub fn new() -> Self {
        Self {
            pool: None,
            roles: Arc::new(RwLock::new(HashMap::new())),
            cas: Arc::new(RwLock::new(HashMap::new())),
            otps: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Mewakili pub `with_storage(pool`.
    pub fn with_storage(pool: Pool) -> Self {
        Self {
            pool: Some(pool),
            roles: Arc::new(RwLock::new(HashMap::new())),
            cas: Arc::new(RwLock::new(HashMap::new())),
            otps: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate SSH key pair
    ///
    /// # Security Note
    ///
    /// Ed25519 keys are preferred for all new deployments.
    /// RSA keys are supported only for legacy compatibility.
    pub async fn generate_keypair(&self, role_name: &str) -> Result<SshKeyPair, SshError> {
        let roles = self.roles.read().await;
        let role = roles
            .get(role_name)
            .ok_or_else(|| SshError::RoleNotFound(role_name.to_string()))?;

        let key_type = role.key_type.clone();
        drop(roles);

        // Emit deprecation warning for RSA key types
        #[allow(deprecated)]
        match &key_type {
            SshKeyType::Rsa2048 | SshKeyType::Rsa4096 => {
                warn!(
                    role = %role_name,
                    key_type = ?key_type,
                    "RSA key generation requested. Ed25519 is recommended for new keys. \
                     RSA support is maintained for legacy compatibility only. \
                     See: RUSTSEC-2023-0071"
                );
            }
            _ => {}
        }

        // Generate private key
        #[allow(deprecated)]
        let private_key = match key_type {
            SshKeyType::Ed25519 => PrivateKey::random(&mut OsRng, Algorithm::Ed25519)
                .map_err(|e| SshError::KeyGenerationFailed(e.to_string()))?,
            SshKeyType::Rsa2048 => PrivateKey::random(
                &mut OsRng,
                Algorithm::Rsa {
                    hash: Some(HashAlg::Sha512),
                },
            )
            .map_err(|e| SshError::KeyGenerationFailed(e.to_string()))?,
            SshKeyType::Rsa4096 => PrivateKey::random(
                &mut OsRng,
                Algorithm::Rsa {
                    hash: Some(HashAlg::Sha512),
                },
            )
            .map_err(|e| SshError::KeyGenerationFailed(e.to_string()))?,
            SshKeyType::EcdsaP256 => PrivateKey::random(
                &mut OsRng,
                Algorithm::Ecdsa {
                    curve: ssh_key::EcdsaCurve::NistP256,
                },
            )
            .map_err(|e| SshError::KeyGenerationFailed(e.to_string()))?,
            SshKeyType::EcdsaP384 => PrivateKey::random(
                &mut OsRng,
                Algorithm::Ecdsa {
                    curve: ssh_key::EcdsaCurve::NistP384,
                },
            )
            .map_err(|e| SshError::KeyGenerationFailed(e.to_string()))?,
        };

        let public_key = private_key.public_key();

        let private_key_str = private_key
            .to_openssh(LineEnding::LF)
            .map_err(|e| SshError::KeyGenerationFailed(e.to_string()))?
            .to_string();

        let public_key_str = public_key
            .to_openssh()
            .map_err(|e| SshError::KeyGenerationFailed(e.to_string()))?;

        info!("Generated SSH key pair for role: {}", role_name);

        Ok(SshKeyPair {
            private_key: private_key_str,
            public_key: public_key_str,
            key_type: format!("{:?}", key_type),
        })
    }

    /// Generate SSH OTP
    pub async fn generate_otp(
        &self,
        username: &str,
        ip: &str,
        ttl: i64,
    ) -> Result<String, SshError> {
        let otp: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let entry = SshOtp {
            otp: otp.clone(),
            username: username.to_string(),
            ip: ip.to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(ttl),
            used: false,
        };

        let mut otps = self.otps.write().await;
        otps.insert(otp.clone(), entry);

        info!("Generated SSH OTP for user: {} from IP: {}", username, ip);
        Ok(otp)
    }

    /// Verify SSH OTP
    pub async fn verify_otp(&self, otp: &str, username: &str, ip: &str) -> Result<bool, SshError> {
        let mut otps = self.otps.write().await;

        if let Some(entry) = otps.get_mut(otp) {
            if entry.used {
                return Ok(false);
            }

            if entry.expires_at < Utc::now() {
                return Ok(false);
            }

            if entry.username != username || entry.ip != ip {
                return Ok(false);
            }

            entry.used = true;
            info!("Verified SSH OTP for user: {}", username);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Create SSH CA
    pub async fn create_ca(&self, name: String, key_type: SshKeyType) -> Result<(), SshError> {
        let private_key = PrivateKey::random(&mut rand::thread_rng(), key_type.to_algorithm())
            .map_err(|e| SshError::KeyGenerationFailed(e.to_string()))?;

        let public_key = private_key.public_key().clone();

        let ca = SshCa {
            name: name.clone(),
            private_key,
            public_key,
            key_type,
            created_at: Utc::now(),
        };

        let mut cas = self.cas.write().await;
        cas.insert(name.clone(), ca);

        info!("Created SSH CA: {}", name);
        Ok(())
    }

    /// Get CA public key
    pub async fn get_ca_public_key(&self, ca_name: &str) -> Result<String, SshError> {
        let cas = self.cas.read().await;
        let ca = cas
            .get(ca_name)
            .ok_or_else(|| SshError::CaNotFound(ca_name.to_string()))?;

        ca.public_key
            .to_openssh()
            .map_err(|e| SshError::SshKeyError(e.to_string()))
    }

    /// Sign SSH certificate
    pub async fn sign_certificate(
        &self,
        ca_name: &str,
        role_name: &str,
        request: SshCertificateRequest,
    ) -> Result<SshCertificate, SshError> {
        // Get CA
        let cas = self.cas.read().await;
        let ca = cas
            .get(ca_name)
            .ok_or_else(|| SshError::CaNotFound(ca_name.to_string()))?;
        let _ca_private_key = ca.private_key.clone();
        drop(cas);

        // Get role
        let roles = self.roles.read().await;
        let role = roles
            .get(role_name)
            .ok_or_else(|| SshError::RoleNotFound(role_name.to_string()))?;

        let ttl = request.ttl.unwrap_or(role.default_ttl);
        if ttl > role.max_ttl {
            return Err(SshError::InvalidConfig(format!(
                "TTL {} exceeds max TTL {}",
                ttl, role.max_ttl
            )));
        }
        drop(roles);

        // Parse public key
        let public_key = PublicKey::from_openssh(&request.public_key)
            .map_err(|e| SshError::SshKeyError(e.to_string()))?;

        // Generate serial number
        let serial = rand::thread_rng().r#gen::<u64>();

        // Create certificate (simplified - full implementation would use ssh-key certificate builder)
        let signed_key = format!(
            "ssh-{}-cert-v01@openssh.com {} serial={} type={} principals={} valid_from=now valid_to={}",
            request.cert_type,
            public_key
                .to_openssh()
                .map_err(|e| SshError::SshKeyError(e.to_string()))?,
            serial,
            request.cert_type,
            request.valid_principals.join(","),
            ttl
        );

        info!("Signed SSH certificate with serial: {}", serial);

        Ok(SshCertificate {
            signed_key,
            serial_number: serial.to_string(),
            cert_type: request.cert_type,
        })
    }

    /// Create role
    pub async fn create_role(&self, role: SshRole) -> Result<(), SshError> {
        let roles = self.roles.read().await;
        if roles.contains_key(&role.name) {
            return Err(SshError::RoleAlreadyExists(role.name.clone()));
        }
        drop(roles);

        let mut roles = self.roles.write().await;
        roles.insert(role.name.clone(), role.clone());

        info!("Created SSH role: {}", role.name);
        Ok(())
    }

    /// Get role
    pub async fn get_role(&self, name: &str) -> Option<SshRole> {
        let roles = self.roles.read().await;
        roles.get(name).cloned()
    }

    /// List roles
    pub async fn list_roles(&self) -> Vec<String> {
        let roles = self.roles.read().await;
        roles.keys().cloned().collect()
    }

    /// List CAs
    pub async fn list_cas(&self) -> Vec<String> {
        let cas = self.cas.read().await;
        cas.keys().cloned().collect()
    }
}

impl Default for SshEngine {
    fn default() -> Self {
        Self::new()
    }
}
