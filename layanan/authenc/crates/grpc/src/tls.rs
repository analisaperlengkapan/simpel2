//! mTLS configuration for gRPC

use std::path::Path;
use tonic::transport::{Certificate, Identity, ServerTlsConfig};

/// TLS configuration for gRPC server
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Server certificate
    pub cert: Vec<u8>,
    /// Server private key
    pub key: Vec<u8>,
    /// CA certificate for client verification
    pub ca_cert: Option<Vec<u8>>,
}

impl TlsConfig {
    /// Create TLS config from file paths
    pub fn from_files(
        cert_path: impl AsRef<Path>,
        key_path: impl AsRef<Path>,
        ca_cert_path: Option<impl AsRef<Path>>,
    ) -> Result<Self, std::io::Error> {
        let cert = std::fs::read(cert_path)?;
        let key = std::fs::read(key_path)?;
        let ca_cert = ca_cert_path.map(|path| std::fs::read(path)).transpose()?;

        Ok(Self { cert, key, ca_cert })
    }

    /// Create TLS config from PEM strings
    pub fn from_pem(cert_pem: String, key_pem: String, ca_cert_pem: Option<String>) -> Self {
        Self {
            cert: cert_pem.into_bytes(),
            key: key_pem.into_bytes(),
            ca_cert: ca_cert_pem.map(|s| s.into_bytes()),
        }
    }

    /// Build Tonic ServerTlsConfig
    pub fn build_server_config(&self) -> Result<ServerTlsConfig, tonic::transport::Error> {
        let identity = Identity::from_pem(&self.cert, &self.key);

        let mut tls_config = ServerTlsConfig::new().identity(identity);

        // Enable mTLS if CA certificate is provided
        if let Some(ca_cert) = &self.ca_cert {
            let ca = Certificate::from_pem(ca_cert);
            tls_config = tls_config.client_ca_root(ca);
        }

        Ok(tls_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_config_from_pem() {
        let cert_pem = "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----".to_string();
        let key_pem = "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----".to_string();
        let ca_cert_pem =
            Some("-----BEGIN CERTIFICATE-----\nca\n-----END CERTIFICATE-----".to_string());

        let config = TlsConfig::from_pem(cert_pem, key_pem, ca_cert_pem);

        assert!(!config.cert.is_empty());
        assert!(!config.key.is_empty());
        assert!(config.ca_cert.is_some());
    }
}
