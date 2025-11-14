/// Software Statement JWT Validator (RFC 7591 Section 2.3)
///
/// Validates software statements (signed JWTs containing client metadata)
/// from trusted issuers

use async_trait::async_trait;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::database::operations::client_registration as db_ops;
use crate::database::Database;
use crate::error::{AuthencError, Result};

/// Software Statement claims (from JWT payload)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareStatementClaims {
    /// Issuer of the software statement
    pub iss: String,

    /// Subject (usually the software_id)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,

    /// Software identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_id: Option<String>,

    /// Software version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_version: Option<String>,

    /// Client name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,

    /// Client URI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_uri: Option<String>,

    /// Logo URI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_uri: Option<String>,

    /// Redirect URIs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uris: Option<Vec<String>>,

    /// Grant types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grant_types: Option<Vec<String>>,

    /// Response types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_types: Option<Vec<String>>,

    /// Scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// Contacts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contacts: Option<Vec<String>>,

    /// Token endpoint authentication method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint_auth_method: Option<String>,

    /// Additional claims
    #[serde(flatten)]
    pub additional_claims: HashMap<String, serde_json::Value>,
}

/// Trait for validating software statements
#[async_trait]
pub trait SoftwareStatementValidator: Send + Sync {
    /// Validate a software statement JWT and extract claims
    async fn validate_statement(
        &self,
        statement_jwt: &str,
    ) -> Result<SoftwareStatementClaims>;

    /// Check if an issuer is trusted
    async fn is_trusted_issuer(&self, issuer: &str) -> Result<bool>;
}

/// Production implementation of software statement validator
pub struct ProductionSoftwareStatementValidator {
    db: Arc<Database>,
}

impl ProductionSoftwareStatementValidator {
    /// Create a new validator
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Fetch JWKS from URL (for issuers with jwks_uri)
    async fn fetch_jwks(&self, jwks_uri: &str) -> Result<serde_json::Value> {
        let response = reqwest::get(jwks_uri)
            .await
            .map_err(|e| AuthencError::InternalError {
                message: format!("Failed to fetch JWKS: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(AuthencError::InternalError {
                message: format!("Failed to fetch JWKS: HTTP {}", response.status()),
            });
        }

        response.json().await.map_err(|e| AuthencError::InternalError {
            message: format!("Failed to parse JWKS: {}", e),
        })
    }

    /// Extract public key from JWKS for the given key ID
    fn extract_decoding_key(
        &self,
        jwks: &serde_json::Value,
        kid: Option<&str>,
    ) -> Result<DecodingKey> {
        let keys = jwks.get("keys").and_then(|v| v.as_array()).ok_or_else(|| {
            AuthencError::ValidationError {
                message: "Invalid JWKS structure".to_string(),
            }
        })?;

        // Find key by kid if specified, otherwise use first key
        let key = if let Some(kid) = kid {
            keys.iter()
                .find(|k| k.get("kid").and_then(|v| v.as_str()) == Some(kid))
                .ok_or_else(|| AuthencError::ValidationError {
                    message: format!("Key with kid '{}' not found in JWKS", kid),
                })?
        } else {
            keys.first().ok_or_else(|| AuthencError::ValidationError {
                message: "Empty JWKS".to_string(),
            })?
        };

        // Extract key type and algorithm
        let kty = key.get("kty").and_then(|v| v.as_str()).ok_or_else(|| {
            AuthencError::ValidationError {
                message: "Missing 'kty' in JWK".to_string(),
            }
        })?;

        match kty {
            "RSA" => {
                let n = key.get("n").and_then(|v| v.as_str()).ok_or_else(|| {
                    AuthencError::ValidationError {
                        message: "Missing 'n' in RSA key".to_string(),
                    }
                })?;
                let e = key.get("e").and_then(|v| v.as_str()).ok_or_else(|| {
                    AuthencError::ValidationError {
                        message: "Missing 'e' in RSA key".to_string(),
                    }
                })?;

                DecodingKey::from_rsa_components(n, e).map_err(|e| {
                    AuthencError::ValidationError {
                        message: format!("Invalid RSA key: {}", e),
                    }
                })
            }
            "EC" => {
                let x = key.get("x").and_then(|v| v.as_str()).ok_or_else(|| {
                    AuthencError::ValidationError {
                        message: "Missing 'x' in EC key".to_string(),
                    }
                })?;
                let y = key.get("y").and_then(|v| v.as_str()).ok_or_else(|| {
                    AuthencError::ValidationError {
                        message: "Missing 'y' in EC key".to_string(),
                    }
                })?;

                DecodingKey::from_ec_components(x, y).map_err(|e| {
                    AuthencError::ValidationError {
                        message: format!("Invalid EC key: {}", e),
                    }
                })
            }
            "OKP" => {
                // Ed25519 keys
                let x = key.get("x").and_then(|v| v.as_str()).ok_or_else(|| {
                    AuthencError::ValidationError {
                        message: "Missing 'x' in OKP key".to_string(),
                    }
                })?;

                DecodingKey::from_ed_components(x).map_err(|e| {
                    AuthencError::ValidationError {
                        message: format!("Invalid Ed25519 key: {}", e),
                    }
                })
            }
            _ => Err(AuthencError::ValidationError {
                message: format!("Unsupported key type: {}", kty),
            }),
        }
    }

    /// Determine algorithm from JWT header
    fn get_algorithm_from_header(&self, alg: Algorithm) -> Result<Algorithm> {
        match alg {
            Algorithm::RS256
            | Algorithm::RS384
            | Algorithm::RS512
            | Algorithm::ES256
            | Algorithm::ES384
            | Algorithm::EdDSA => Ok(alg),
            _ => Err(AuthencError::ValidationError {
                message: format!("Unsupported JWT algorithm: {:?}", alg),
            }),
        }
    }
}

#[async_trait]
impl SoftwareStatementValidator for ProductionSoftwareStatementValidator {
    async fn validate_statement(
        &self,
        statement_jwt: &str,
    ) -> Result<SoftwareStatementClaims> {
        // Decode header to get algorithm and key ID
        let header = decode_header(statement_jwt).map_err(|e| AuthencError::ValidationError {
            message: format!("Invalid JWT header: {}", e),
        })?;

        let algorithm = header.alg;
        let kid = header.kid.as_deref();

        // Decode without validation first to get the issuer
        // Use a temporary validation that doesn't check signature
        let mut temp_validation = Validation::default();
        temp_validation.insecure_disable_signature_validation();
        temp_validation.validate_exp = false;

        let untrusted_claims: SoftwareStatementClaims =
            decode::<SoftwareStatementClaims>(
                statement_jwt,
                &DecodingKey::from_secret(&[]), // Dummy key since we're not validating
                &temp_validation,
            )
            .map_err(|e| AuthencError::ValidationError {
                message: format!("Failed to decode JWT: {}", e),
            })?
            .claims;

        // Get issuer configuration from database
        let issuer_config =
            db_ops::get_software_statement_issuer_by_issuer(&self.db, &untrusted_claims.iss)
                .await?
                .ok_or_else(|| AuthencError::AuthenticationFailed)?;

        if !issuer_config.enabled {
            return Err(AuthencError::AuthenticationFailed);
        }

        // Get JWKS (either from database or fetch from URL)
        let jwks = if let Some(jwks) = issuer_config.jwks {
            jwks
        } else if let Some(jwks_uri) = issuer_config.jwks_uri {
            self.fetch_jwks(&jwks_uri).await?
        } else {
            return Err(AuthencError::ConfigurationError {
                message: "Issuer has neither jwks nor jwks_uri configured".to_string(),
            });
        };

        // Extract decoding key from JWKS
        let decoding_key = self.extract_decoding_key(&jwks, kid)?;

        // Validate JWT signature and claims
        let mut validation = Validation::new(algorithm);
        validation.set_issuer(&[&issuer_config.issuer]);
        validation.validate_exp = false; // Software statements typically don't expire

        let token_data = decode::<SoftwareStatementClaims>(
            statement_jwt,
            &decoding_key,
            &validation,
        )
        .map_err(|e| AuthencError::AuthenticationFailed)?;

        Ok(token_data.claims)
    }

    async fn is_trusted_issuer(&self, issuer: &str) -> Result<bool> {
        match db_ops::get_software_statement_issuer_by_issuer(&self.db, issuer).await? {
            Some(config) => Ok(config.enabled),
            None => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_software_statement_claims() {
        // Test JWT claims parsing
        let claims = SoftwareStatementClaims {
            iss: "https://trusted-issuer.example.com".to_string(),
            sub: Some("client-123".to_string()),
            software_id: Some("app-xyz".to_string()),
            software_version: Some("1.0.0".to_string()),
            client_name: Some("Test Application".to_string()),
            client_uri: Some("https://example.com".to_string()),
            logo_uri: None,
            redirect_uris: Some(vec!["https://example.com/callback".to_string()]),
            grant_types: Some(vec!["authorization_code".to_string()]),
            response_types: Some(vec!["code".to_string()]),
            scope: Some("openid profile email".to_string()),
            contacts: Some(vec!["admin@example.com".to_string()]),
            token_endpoint_auth_method: Some("client_secret_basic".to_string()),
            additional_claims: HashMap::new(),
        };

        assert_eq!(claims.iss, "https://trusted-issuer.example.com");
        assert_eq!(claims.software_id.unwrap(), "app-xyz");
    }
}
