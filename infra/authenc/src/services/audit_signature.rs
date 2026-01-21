//! Audit Signature Service
//!
//! Provides tamper-proof audit logging with HMAC-SHA256 signatures.
//! Ensures integrity of audit trail for compliance requirements.

use crate::models::events::{AdminEvent, Event};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Errors related to audit signature operations
#[derive(Debug, Error)]
pub enum AuditSignatureError {
    /// Failed to genesignature
    #[error("Failed to generate signature: {0}")]
    SignatureGenerationError(String),

    /// Failed to verify signature
    #[error("Failed to verify signature: {0}")]
    SignatureVerificationError(String),

    /// Invalid signature format
    #[error("Invalid signature format: {0}")]
    InvalidSignatureFormat(String),

    /// Signature mismatch
    #[error("Signature mismatch: expected {expected}, got {actual}")]
    SignatureMismatch { expected: String, actual: String },
}

/// Audit signature service for tamper-proof logging
#[derive(Clone)]
pub struct AuditSignatureService {
    /// HMAC secret key for signing
    secret_key: Arc<Vec<u8>>,
}

impl AuditSignatureService {
    /// Create a new audit signature service
    ///
    /// # Arguments
    /// * `secret_key` - Secret key for HMAC signing (should be at least 32 bytes)
    pub fn new(secret_key: Vec<u8>) -> Self {
        Self {
            secret_key: Arc::new(secret_key),
        }
    }

    /// Create from base64-encoded secret key
    pub fn from_base64(secret_key_b64: &str) -> Result<Self, AuditSignatureError> {
        let secret_key = crate::utils::encoding::base64_decode(secret_key_b64)
            .map_err(|e| AuditSignatureError::InvalidSignatureFormat(e.to_string()))?;

        if secret_key.len() < 32 {
            return Err(AuditSignatureError::SignatureGenerationError(
                "Secret key must be at least 32 bytes".to_string(),
            ));
        }

        Ok(Self::new(secret_key))
    }

    /// Generate a signature for an event
    ///
    /// Creates a canonical representation of the event and signs it with HMAC-SHA256
    pub fn sign_event(&self, event: &Event) -> Result<String, AuditSignatureError> {
        let canonical = self.canonicalize_event(event);
        self.sign_data(&canonical)
    }

    /// Generate a signature for an admin event
    ///
    /// Creates a canonical representation of the admin event and signs it with HMAC-SHA256
    pub fn sign_admin_event(&self, event: &AdminEvent) -> Result<String, AuditSignatureError> {
        let canonical = self.canonicalize_admin_event(event);
        self.sign_data(&canonical)
    }

    /// Verify an event signature
    ///
    /// Returns Ok(()) if signature is valid, Err otherwise
    pub fn verify_event(&self, event: &Event, signature: &str) -> Result<(), AuditSignatureError> {
        let expected_signature = self.sign_event(event)?;

        if expected_signature != signature {
            return Err(AuditSignatureError::SignatureMismatch {
                expected: expected_signature,
                actual: signature.to_string(),
            });
        }

        Ok(())
    }

    /// Verify an admin event signature
    ///
    /// Returns Ok(()) if signature is valid, Err otherwise
    pub fn verify_admin_event(
        &self,
        event: &AdminEvent,
        signature: &str,
    ) -> Result<(), AuditSignatureError> {
        let expected_signature = self.sign_admin_event(event)?;

        if expected_signature != signature {
            return Err(AuditSignatureError::SignatureMismatch {
                expected: expected_signature,
                actual: signature.to_string(),
            });
        }

        Ok(())
    }

    /// Sign arbitrary data with HMAC-SHA256
    fn sign_data(&self, data: &str) -> Result<String, AuditSignatureError> {
        let mut mac = HmacSha256::new_from_slice(&self.secret_key).map_err(|e| {
            AuditSignatureError::SignatureGenerationError(format!("Invalid key length: {}", e))
        })?;

        mac.update(data.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();

        // Convert to hex string
        Ok(hex::encode(code_bytes))
    }

    /// Create canonical representation of an event for signing
    ///
    /// Format: id|time|event_type|realm_id|user_id|session_id|ip_address|error|details_json
    fn canonicalize_event(&self, event: &Event) -> String {
        let details_json = serde_json::to_string(&event.details).unwrap_or_default();

        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            event.id,
            event.time.to_rfc3339(),
            event.event_type.as_str(),
            event.realm_id,
            event.realm_name.as_deref().unwrap_or(""),
            event.client_id.as_deref().unwrap_or(""),
            event.user_id.as_deref().unwrap_or(""),
            event.session_id.as_deref().unwrap_or(""),
            event.ip_address.as_deref().unwrap_or(""),
            event.error.as_deref().unwrap_or(""),
            details_json
        )
    }

    /// Create canonical representation of an admin event for signing
    ///
    /// Format: id|time|realm_id|auth_user_id|auth_ip|resource_type|operation_type|resource_path|representation|error
    fn canonicalize_admin_event(&self, event: &AdminEvent) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            event.id,
            event.time.to_rfc3339(),
            event.realm_id,
            event.realm_name.as_deref().unwrap_or(""),
            event.auth_details.user_id,
            event.auth_details.username.as_deref().unwrap_or(""),
            event.auth_details.ip_address.as_deref().unwrap_or(""),
            event.auth_details.user_agent.as_deref().unwrap_or(""),
            event.resource_type.as_str(),
            event.operation_type.as_str(),
            event.resource_path,
            event.representation.as_deref().unwrap_or(""),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::events::{AuthDetails, EventType, OperationType, ResourceType};
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_service() -> AuditSignatureService {
        // Use a test secret key (32 bytes)
        let secret_key = b"test_secret_key_32_bytes_long!!!".to_vec();
        AuditSignatureService::new(secret_key)
    }

    #[test]
    fn test_sign_event() {
        let service = create_test_service();

        let event = Event {
            id: "test-id".to_string(),
            time: Utc::now(),
            event_type: EventType::Login,
            realm_id: "test-realm".to_string(),
            realm_name: Some("Test Realm".to_string()),
            client_id: Some("test-client".to_string()),
            user_id: Some("test-user".to_string()),
            session_id: Some("test-session".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            error: None,
            details: HashMap::new(),
        };

        let signature = service.sign_event(&event).unwrap();
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64); // SHA256 produces 32 bytes = 64 hex chars
    }

    #[test]
    fn test_verify_event_valid() {
        let service = create_test_service();

        let event = Event {
            id: "test-id".to_string(),
            time: Utc::now(),
            event_type: EventType::Login,
            realm_id: "test-realm".to_string(),
            realm_name: Some("Test Realm".to_string()),
            client_id: Some("test-client".to_string()),
            user_id: Some("test-user".to_string()),
            session_id: Some("test-session".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            error: None,
            details: HashMap::new(),
        };

        let signature = service.sign_event(&event).unwrap();
        let result = service.verify_event(&event, &signature);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_event_invalid() {
        let service = create_test_service();

        let event = Event {
            id: "test-id".to_string(),
            time: Utc::now(),
            event_type: EventType::Login,
            realm_id: "test-realm".to_string(),
            realm_name: Some("Test Realm".to_string()),
            client_id: Some("test-client".to_string()),
            user_id: Some("test-user".to_string()),
            session_id: Some("test-session".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            error: None,
            details: HashMap::new(),
        };

        let invalid_signature = "0000000000000000000000000000000000000000000000000000000000000000";
        let result = service.verify_event(&event, invalid_signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_admin_event() {
        let service = create_test_service();

        let event = AdminEvent {
            id: "test-id".to_string(),
            time: Utc::now(),
            realm_id: "test-realm".to_string(),
            realm_name: Some("Test Realm".to_string()),
            auth_details: AuthDetails {
                user_id: "admin-user".to_string(),
                username: Some("admin".to_string()),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("test-agent".to_string()),
            },
            resource_type: ResourceType::User,
            operation_type: OperationType::Create,
            resource_path: "/users/test-user".to_string(),
            representation: Some("{}".to_string()),
            error: None,
        };

        let signature = service.sign_admin_event(&event).unwrap();
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_verify_admin_event_valid() {
        let service = create_test_service();

        let event = AdminEvent {
            id: "test-id".to_string(),
            time: Utc::now(),
            realm_id: "test-realm".to_string(),
            realm_name: Some("Test Realm".to_string()),
            auth_details: AuthDetails {
                user_id: "admin-user".to_string(),
                username: Some("admin".to_string()),
                ip_address: Some("127.0.0.1".to_string()),
                user_agent: Some("test-agent".to_string()),
            },
            resource_type: ResourceType::User,
            operation_type: OperationType::Create,
            resource_path: "/users/test-user".to_string(),
            representation: Some("{}".to_string()),
            error: None,
        };

        let signature = service.sign_admin_event(&event).unwrap();
        let result = service.verify_admin_event(&event, &signature);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deterministic_signatures() {
        let service = create_test_service();

        let event = Event {
            id: "test-id".to_string(),
            time: Utc::now(),
            event_type: EventType::Login,
            realm_id: "test-realm".to_string(),
            realm_name: Some("Test Realm".to_string()),
            client_id: Some("test-client".to_string()),
            user_id: Some("test-user".to_string()),
            session_id: Some("test-session".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            error: None,
            details: HashMap::new(),
        };

        let sig1 = service.sign_event(&event).unwrap();
        let sig2 = service.sign_event(&event).unwrap();

        // Same event should produce same signature
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_from_base64() {
        let secret_key = b"test_secret_key_32_bytes_long!!!";
        let secret_key_b64 = crate::utils::encoding::base64_encode(secret_key);

        let service = AuditSignatureService::from_base64(&secret_key_b64).unwrap();

        let event = Event {
            id: "test-id".to_string(),
            time: Utc::now(),
            event_type: EventType::Login,
            realm_id: "test-realm".to_string(),
            realm_name: None,
            client_id: None,
            user_id: None,
            session_id: None,
            ip_address: None,
            error: None,
            details: HashMap::new(),
        };

        let signature = service.sign_event(&event);
        assert!(signature.is_ok());
    }
}
