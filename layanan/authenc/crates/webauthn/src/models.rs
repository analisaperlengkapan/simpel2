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

/// The webauthn-rs state of one authentication ceremony.
///
/// The two flows are different ceremonies in webauthn-rs and a state started
/// by one cannot be finished by the other:
///
/// - `Passkey`: the caller named the user, so the allow-list holds that user's
///   credentials.
/// - `Discoverable`: usernameless. The allow-list is empty on purpose, the
///   authenticator picks the credential, and the server finds the user from the
///   credential id it presents. Finishing this with the `Passkey` flow, which an
///   empty allow-list used to do, can never succeed: webauthn-rs looks the
///   credential up in the list it was started with and there is nothing in it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CeremonyState {
    /// User-specific ceremony (`start_passkey_authentication`).
    Passkey(PasskeyAuthentication),
    /// Usernameless ceremony (`start_discoverable_authentication`).
    Discoverable(DiscoverableAuthentication),
}

/// Passkey authentication session (temporary, stored during authentication flow)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationSession {
    /// Optional user ID (None for usernameless authentication)
    pub user_id: Option<UserId>,
    /// WebAuthn authentication state from webauthn-rs
    pub state: CeremonyState,
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
