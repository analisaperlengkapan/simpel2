//! Credential storage trait and implementations

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use webauthn_rs::prelude::*;

use authenc_types::{Result, UserId};

use crate::models::StoredCredential;

/// Trait for storing and retrieving WebAuthn credentials
#[async_trait]
pub trait CredentialStore: Send + Sync {
    /// Store a new credential
    async fn store_credential(&self, credential: &StoredCredential) -> Result<()>;

    /// Get a credential by its database ID
    async fn get_credential(&self, id: Uuid) -> Result<StoredCredential>;

    /// Get a credential by its WebAuthn credential ID
    async fn get_credential_by_id(&self, cred_id: &CredentialID) -> Result<StoredCredential>;

    /// Get all credentials for a user
    async fn get_credentials_for_user(&self, user_id: UserId) -> Result<Vec<StoredCredential>>;

    /// Delete a credential
    async fn delete_credential(&self, id: Uuid) -> Result<()>;

    /// Update the last used timestamp for a credential
    async fn update_last_used(&self, id: Uuid, timestamp: DateTime<Utc>) -> Result<()>;

    /// Update the credential counter (for replay attack prevention)
    async fn update_counter(&self, id: Uuid, counter: u32) -> Result<()>;

    /// Update the credential nickname
    async fn update_nickname(&self, id: Uuid, nickname: String) -> Result<()>;
}

// PostgreSQL implementation will be added in authenc-storage crate
