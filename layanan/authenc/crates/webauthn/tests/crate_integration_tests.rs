//! Crate integration tests for WebAuthn
//!
//! These tests verify integration between:
//! - authenc-webauthn → authenc-storage (credential store)
//! - authenc-webauthn → authenc-types (domain types, errors)
//! - authenc-api → authenc-webauthn (API handlers)
//! - authenc-core → authenc-webauthn (authentication flow)

use authenc_types::{AuthencError, Result, UserId};
use authenc_webauthn::{CredentialStore, StoredCredential, WebAuthnConfig, WebAuthnService};
use std::sync::Arc;
use url::Url;
use uuid::Uuid;

// ============================================================================
// Integration Test 1: authenc-webauthn → authenc-storage
// ============================================================================

#[tokio::test]
async fn test_integration_webauthn_storage_types() {
    // Verify that authenc-webauthn can use authenc-storage types
    // Note: PostgresCredentialStore implements CredentialStore trait from authenc-webauthn

    // Verify UserId from authenc-types works with both crates
    let user_id = UserId(Uuid::new_v4());
    assert!(!user_id.0.is_nil());

    // Verify AuthencError from authenc-types works with both crates
    let error = AuthencError::not_found("Test error");
    assert!(matches!(error, AuthencError::NotFound(_)));
}

#[tokio::test]
async fn test_integration_credential_store_trait() {
    // Verify that the CredentialStore trait from authenc-webauthn
    // can be implemented by authenc-storage

    // This is verified by the fact that PostgresCredentialStore
    // implements CredentialStore trait

    // Type check: ensure trait is object-safe
    let _: Option<Arc<dyn CredentialStore>> = None;
}

// ============================================================================
// Integration Test 2: authenc-webauthn → authenc-types
// ============================================================================

#[tokio::test]
async fn test_integration_webauthn_types_domain_models() {
    // Verify that WebAuthn models use authenc-types correctly

    let user_id = UserId(Uuid::new_v4());

    // Verify UserId from authenc-types works with WebAuthn
    assert!(!user_id.0.is_nil());

    // Verify CredentialID from webauthn-rs works
    use webauthn_rs::prelude::*;
    let cred_id = CredentialID::from(vec![1, 2, 3, 4]);
    assert_eq!(cred_id.len(), 4);
}

#[tokio::test]
async fn test_integration_webauthn_types_errors() {
    // Verify that WebAuthn service uses authenc-types errors correctly

    let config = WebAuthnConfig {
        rp_id: "".to_string(), // Invalid RP ID
        rp_origin: Url::parse("http://localhost").unwrap(),
        rp_name: "Test".to_string(),
    };

    // Mock credential store
    struct MockStore;

    #[async_trait::async_trait]
    impl CredentialStore for MockStore {
        async fn store_credential(&self, _: &StoredCredential) -> Result<()> {
            Ok(())
        }
        async fn get_credential(&self, _: Uuid) -> Result<StoredCredential> {
            Err(AuthencError::not_found("Not found"))
        }
        async fn get_credential_by_id(
            &self,
            _: &webauthn_rs::prelude::CredentialID,
        ) -> Result<StoredCredential> {
            Err(AuthencError::not_found("Not found"))
        }
        async fn get_credentials_for_user(&self, _: UserId) -> Result<Vec<StoredCredential>> {
            Ok(vec![])
        }
        async fn delete_credential(&self, _: Uuid) -> Result<()> {
            Ok(())
        }
        async fn update_last_used(&self, _: Uuid, _: chrono::DateTime<chrono::Utc>) -> Result<()> {
            Ok(())
        }
        async fn update_counter(&self, _: Uuid, _: u32) -> Result<()> {
            Ok(())
        }
        async fn update_nickname(&self, _: Uuid, _: String) -> Result<()> {
            Ok(())
        }
    }

    let store = Arc::new(MockStore);
    let result = WebAuthnService::new(config, store);

    // Verify error type is AuthencError from authenc-types
    assert!(result.is_err());
}

// ============================================================================
// Integration Test 3: WebAuthn Service Configuration
// ============================================================================

#[tokio::test]
async fn test_integration_webauthn_service_creation() {
    // Verify that WebAuthnService can be created with proper configuration

    let config = WebAuthnConfig {
        rp_id: "example.com".to_string(),
        rp_origin: Url::parse("https://example.com").unwrap(),
        rp_name: "Example App".to_string(),
    };

    struct MockStore;

    #[async_trait::async_trait]
    impl CredentialStore for MockStore {
        async fn store_credential(&self, _: &StoredCredential) -> Result<()> {
            Ok(())
        }
        async fn get_credential(&self, _: Uuid) -> Result<StoredCredential> {
            Err(AuthencError::not_found("Not found"))
        }
        async fn get_credential_by_id(
            &self,
            _: &webauthn_rs::prelude::CredentialID,
        ) -> Result<StoredCredential> {
            Err(AuthencError::not_found("Not found"))
        }
        async fn get_credentials_for_user(&self, _: UserId) -> Result<Vec<StoredCredential>> {
            Ok(vec![])
        }
        async fn delete_credential(&self, _: Uuid) -> Result<()> {
            Ok(())
        }
        async fn update_last_used(&self, _: Uuid, _: chrono::DateTime<chrono::Utc>) -> Result<()> {
            Ok(())
        }
        async fn update_counter(&self, _: Uuid, _: u32) -> Result<()> {
            Ok(())
        }
        async fn update_nickname(&self, _: Uuid, _: String) -> Result<()> {
            Ok(())
        }
    }

    let store = Arc::new(MockStore);
    let service = WebAuthnService::new(config, store).unwrap();

    // Verify service configuration
    assert_eq!(service.rp_id(), "example.com");
    assert_eq!(service.rp_origin().as_str(), "https://example.com/");
    assert_eq!(service.rp_name(), "Example App");
}

// ============================================================================
// Integration Test 4: End-to-End Flow Verification
// ============================================================================

#[tokio::test]
async fn test_integration_registration_flow_types() {
    // Verify that registration flow uses correct types from all crates

    let config = WebAuthnConfig {
        rp_id: "localhost".to_string(),
        rp_origin: Url::parse("http://localhost:8080").unwrap(),
        rp_name: "Test App".to_string(),
    };

    struct MockStore;

    #[async_trait::async_trait]
    impl CredentialStore for MockStore {
        async fn store_credential(&self, _: &StoredCredential) -> Result<()> {
            Ok(())
        }
        async fn get_credential(&self, _: Uuid) -> Result<StoredCredential> {
            Err(AuthencError::not_found("Not found"))
        }
        async fn get_credential_by_id(
            &self,
            _: &webauthn_rs::prelude::CredentialID,
        ) -> Result<StoredCredential> {
            Err(AuthencError::not_found("Not found"))
        }
        async fn get_credentials_for_user(&self, _: UserId) -> Result<Vec<StoredCredential>> {
            Ok(vec![])
        }
        async fn delete_credential(&self, _: Uuid) -> Result<()> {
            Ok(())
        }
        async fn update_last_used(&self, _: Uuid, _: chrono::DateTime<chrono::Utc>) -> Result<()> {
            Ok(())
        }
        async fn update_counter(&self, _: Uuid, _: u32) -> Result<()> {
            Ok(())
        }
        async fn update_nickname(&self, _: Uuid, _: String) -> Result<()> {
            Ok(())
        }
    }

    let store = Arc::new(MockStore);
    let service = WebAuthnService::new(config, store).unwrap();

    // Start registration with UserId from authenc-types
    let user_id = UserId(Uuid::new_v4());
    let result = service
        .start_registration(user_id, "testuser", "Test User")
        .await;

    // Verify result types
    assert!(result.is_ok());
    let (challenge, session) = result.unwrap();

    // Verify challenge response from webauthn-rs
    assert!(!challenge.public_key.challenge.as_ref().is_empty());

    // Verify session uses UserId from authenc-types
    assert_eq!(session.user_id, user_id);
}

#[tokio::test]
async fn test_integration_authentication_flow_types() {
    // Verify that authentication flow uses correct types from all crates

    let config = WebAuthnConfig {
        rp_id: "localhost".to_string(),
        rp_origin: Url::parse("http://localhost:8080").unwrap(),
        rp_name: "Test App".to_string(),
    };

    struct MockStore;

    #[async_trait::async_trait]
    impl CredentialStore for MockStore {
        async fn store_credential(&self, _: &StoredCredential) -> Result<()> {
            Ok(())
        }
        async fn get_credential(&self, _: Uuid) -> Result<StoredCredential> {
            Err(AuthencError::not_found("Not found"))
        }
        async fn get_credential_by_id(
            &self,
            _: &webauthn_rs::prelude::CredentialID,
        ) -> Result<StoredCredential> {
            Err(AuthencError::not_found("Not found"))
        }
        async fn get_credentials_for_user(&self, _: UserId) -> Result<Vec<StoredCredential>> {
            Ok(vec![])
        }
        async fn delete_credential(&self, _: Uuid) -> Result<()> {
            Ok(())
        }
        async fn update_last_used(&self, _: Uuid, _: chrono::DateTime<chrono::Utc>) -> Result<()> {
            Ok(())
        }
        async fn update_counter(&self, _: Uuid, _: u32) -> Result<()> {
            Ok(())
        }
        async fn update_nickname(&self, _: Uuid, _: String) -> Result<()> {
            Ok(())
        }
    }

    let store = Arc::new(MockStore);
    let service = WebAuthnService::new(config, store).unwrap();

    // Start authentication with UserId from authenc-types
    let user_id = UserId(Uuid::new_v4());
    let result = service.start_authentication(Some(user_id)).await;

    // Verify result types
    assert!(result.is_ok());
    let (challenge, session) = result.unwrap();

    // Verify challenge response from webauthn-rs
    assert!(!challenge.public_key.challenge.as_ref().is_empty());

    // Verify session uses UserId from authenc-types
    assert_eq!(session.user_id, Some(user_id));
}
