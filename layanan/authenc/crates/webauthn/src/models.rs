//! WebAuthn data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::*;

use authenc_types::UserId;

/// Stored credential in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredential {
    /// Unique credential ID (database primary key)
    pub id: Uuid,
    /// User ID this credential belongs to
    pub user_id: UserId,
    /// WebAuthn credential ID
    pub cred_id: CredentialID,
    /// The passkey credential
    pub cred: Passkey,
    /// Optional user-assigned nickname for this credential
    pub nickname: Option<String>,
    /// When this credential was created
    pub created_at: DateTime<Utc>,
    /// When this credential was last used for authentication
    pub last_used: Option<DateTime<Utc>>,
}

/// Passkey registration session (temporary, stored during registration flow)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationSession {
    /// User ID registering the passkey
    pub user_id: UserId,
    /// WebAuthn registration state from webauthn-rs
    pub state: PasskeyRegistration,
    /// When this registration was initiated
    pub created_at: DateTime<Utc>,
}

/// Passkey authentication session (temporary, stored during authentication flow)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationSession {
    /// Optional user ID (None for usernameless authentication)
    pub user_id: Option<UserId>,
    /// WebAuthn authentication state from webauthn-rs
    pub state: PasskeyAuthentication,
    /// When this authentication was initiated
    pub created_at: DateTime<Utc>,
}

/// Authentication result
#[derive(Debug, Clone)]
pub enum AuthenticationResult {
    /// Authentication succeeded
    Success {
        /// User ID that was authenticated
        user_id: UserId,
        /// Credential ID that was used
        credential_id: Uuid,
    },
    /// Authentication failed
    Failed {
        /// Reason for failure
        reason: String,
    },
}
