//! Requesting Party Token (RPT) Implementation
//!
//! RPT is a special JWT token that contains permissions granted to the requesting party.
//! It's the core artifact in UMA 2.0 that proves authorization.

use crate::error::{AuthencError, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// RPT (Requesting Party Token) Claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RptClaims {
    /// Issuer
    pub iss: String,
    /// Subject (requesting party)
    pub sub: String,
    /// Audience (resource server)
    pub aud: Vec<String>,
    /// Expiration time
    pub exp: i64,
    /// Issued at
    pub iat: i64,
    /// Not before
    pub nbf: i64,
    /// JWT ID
    pub jti: String,
    /// Authorization details (UMA permissions)
    pub authorization: AuthorizationDetails,
    /// Client ID that obtained the RPT
    pub azp: String,
    /// Realm
    pub realm: String,
}

/// Authorization details in RPT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationDetails {
    /// Permissions granted
    pub permissions: Vec<Permission>,
}

/// Permission in RPT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Resource ID
    #[serde(rename = "rsid")]
    pub resource_id: String,
    /// Resource name
    #[serde(rename = "rsname")]
    pub resource_name: Option<String>,
    /// Scopes granted
    pub scopes: Vec<String>,
    /// Claims that led to this permission
    pub claims: Option<HashMap<String, serde_json::Value>>,
}

/// RPT token structure
#[derive(Debug, Clone)]
pub struct Rpt {
    /// Raw token string
    pub token: String,
    /// Decoded claims
    pub claims: RptClaims,
}

impl Rpt {
    /// Create new RPT from token string
    pub fn from_token(token: String, claims: RptClaims) -> Self {
        Self { token, claims }
    }

    /// Get token string
    pub fn as_str(&self) -> &str {
        &self.token
    }

    /// Check if RPT is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() >= self.claims.exp
    }

    /// Check if RPT has permission for resource and scopes
    pub fn has_permission(&self, resource_id: &str, required_scopes: &[String]) -> bool {
        for permission in &self.claims.authorization.permissions {
            if permission.resource_id == resource_id {
                // Check if all required scopes are present
                return required_scopes
                    .iter()
                    .all(|scope| permission.scopes.contains(scope));
            }
        }
        false
    }

    /// Get all permissions
    pub fn permissions(&self) -> &[Permission] {
        &self.claims.authorization.permissions
    }
}

/// RPT Service for creating and managing RPTs
pub struct RptService {
    /// Issuer URL
    issuer: String,
    /// Signing key
    signing_key: EncodingKey,
    /// Verification key
    verification_key: DecodingKey,
    /// Token lifetime in seconds
    token_lifetime: i64,
}

impl RptService {
    /// Create new RPT service
    pub fn new(issuer: String, signing_key: EncodingKey, verification_key: DecodingKey) -> Self {
        Self {
            issuer,
            signing_key,
            verification_key,
            token_lifetime: 300, // 5 minutes default
        }
    }

    /// Set token lifetime
    pub fn with_lifetime(mut self, lifetime_seconds: i64) -> Self {
        self.token_lifetime = lifetime_seconds;
        self
    }

    /// Create RPT with permissions
    pub fn create_rpt(
        &self,
        subject: &str,
        audience: Vec<String>,
        authorized_party: &str,
        realm: &str,
        permissions: Vec<Permission>,
    ) -> Result<Rpt> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.token_lifetime);

        let claims = RptClaims {
            iss: self.issuer.clone(),
            sub: subject.to_string(),
            aud: audience,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            authorization: AuthorizationDetails { permissions },
            azp: authorized_party.to_string(),
            realm: realm.to_string(),
        };

        // Create JWT token
        let token = jsonwebtoken::encode(&Header::default(), &claims, &self.signing_key)
            .map_err(|e| AuthencError::internal(format!("Failed to create RPT: {}", e)))?;

        Ok(Rpt::from_token(token, claims))
    }

    /// Verify RPT token
    pub fn verify_rpt(&self, token: &str) -> Result<RptClaims> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.issuer]);
        validation.validate_exp = true;

        let token_data =
            jsonwebtoken::decode::<RptClaims>(token, &self.verification_key, &validation)
                .map_err(|e| AuthencError::unauthorized(format!("Invalid RPT token: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Upgrade RPT with additional permissions
    pub fn upgrade_rpt(
        &self,
        existing_rpt: &Rpt,
        additional_permissions: Vec<Permission>,
    ) -> Result<Rpt> {
        let mut all_permissions = existing_rpt.claims.authorization.permissions.clone();

        // Merge permissions - avoid duplicates
        for new_perm in additional_permissions {
            let exists = all_permissions.iter().any(|p| {
                p.resource_id == new_perm.resource_id
                    && p.scopes.iter().all(|s| new_perm.scopes.contains(s))
            });

            if !exists {
                all_permissions.push(new_perm);
            }
        }

        self.create_rpt(
            &existing_rpt.claims.sub,
            existing_rpt.claims.aud.clone(),
            &existing_rpt.claims.azp,
            &existing_rpt.claims.realm,
            all_permissions,
        )
    }

    /// Create RPT from permission ticket evaluation
    pub fn create_rpt_from_ticket(
        &self,
        requesting_party: &str,
        client_id: &str,
        realm: &str,
        granted_permissions: Vec<Permission>,
    ) -> Result<Rpt> {
        // Audience is all resource servers involved
        let audience: Vec<String> = granted_permissions
            .iter()
            .map(|p| format!("resource-server-{}", p.resource_id))
            .collect();

        self.create_rpt(
            requesting_party,
            audience,
            client_id,
            realm,
            granted_permissions,
        )
    }

    /// Introspect RPT and return permission details
    pub fn introspect(&self, token: &str) -> Result<RptIntrospectionResponse> {
        match self.verify_rpt(token) {
            Ok(claims) => {
                let permissions = claims
                    .authorization
                    .permissions
                    .iter()
                    .map(|p| super::RptPermission {
                        resource_id: p.resource_id.clone(),
                        resource_name: p.resource_name.clone(),
                        scopes: p.scopes.clone(),
                        claims: p.claims.clone(),
                    })
                    .collect();

                Ok(RptIntrospectionResponse {
                    active: true,
                    exp: Some(claims.exp),
                    iat: Some(claims.iat),
                    permissions: Some(permissions),
                })
            }
            Err(_) => Ok(RptIntrospectionResponse {
                active: false,
                exp: None,
                iat: None,
                permissions: None,
            }),
        }
    }
}

use super::RptIntrospectionResponse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpt_creation() {
        let signing_key = EncodingKey::from_secret(b"test_secret");
        let verification_key = DecodingKey::from_secret(b"test_secret");

        let service = RptService::new(
            "https://authenc.example.com".to_string(),
            signing_key,
            verification_key,
        );

        let permissions = vec![Permission {
            resource_id: "resource-123".to_string(),
            resource_name: Some("Test Resource".to_string()),
            scopes: vec!["read".to_string(), "write".to_string()],
            claims: None,
        }];

        let rpt = service
            .create_rpt(
                "user-456",
                vec!["resource-server-123".to_string()],
                "client-789",
                "test-realm",
                permissions,
            )
            .expect("Failed to create RPT");

        assert!(!rpt.is_expired());
        assert!(rpt.has_permission("resource-123", &vec!["read".to_string()]));
        assert!(!rpt.has_permission("resource-999", &vec!["read".to_string()]));
    }

    #[test]
    fn test_rpt_verification() {
        let signing_key = EncodingKey::from_secret(b"test_secret");
        let verification_key = DecodingKey::from_secret(b"test_secret");

        let service = RptService::new(
            "https://authenc.example.com".to_string(),
            signing_key,
            verification_key,
        );

        let permissions = vec![Permission {
            resource_id: "resource-123".to_string(),
            resource_name: Some("Test Resource".to_string()),
            scopes: vec!["read".to_string()],
            claims: None,
        }];

        let rpt = service
            .create_rpt(
                "user-456",
                vec!["resource-server-123".to_string()],
                "client-789",
                "test-realm",
                permissions,
            )
            .expect("Failed to create RPT");

        let verified_claims = service
            .verify_rpt(rpt.as_str())
            .expect("Failed to verify RPT");

        assert_eq!(verified_claims.sub, "user-456");
        assert_eq!(verified_claims.azp, "client-789");
        assert_eq!(verified_claims.authorization.permissions.len(), 1);
    }

    #[test]
    fn test_rpt_upgrade() {
        let signing_key = EncodingKey::from_secret(b"test_secret");
        let verification_key = DecodingKey::from_secret(b"test_secret");

        let service = RptService::new(
            "https://authenc.example.com".to_string(),
            signing_key,
            verification_key,
        );

        let initial_permissions = vec![Permission {
            resource_id: "resource-123".to_string(),
            resource_name: Some("Resource 1".to_string()),
            scopes: vec!["read".to_string()],
            claims: None,
        }];

        let rpt = service
            .create_rpt(
                "user-456",
                vec!["resource-server-123".to_string()],
                "client-789",
                "test-realm",
                initial_permissions,
            )
            .expect("Failed to create RPT");

        let additional_permissions = vec![Permission {
            resource_id: "resource-456".to_string(),
            resource_name: Some("Resource 2".to_string()),
            scopes: vec!["write".to_string()],
            claims: None,
        }];

        let upgraded_rpt = service
            .upgrade_rpt(&rpt, additional_permissions)
            .expect("Failed to upgrade RPT");

        assert_eq!(upgraded_rpt.permissions().len(), 2);
        assert!(upgraded_rpt.has_permission("resource-123", &vec!["read".to_string()]));
        assert!(upgraded_rpt.has_permission("resource-456", &vec!["write".to_string()]));
    }
}
