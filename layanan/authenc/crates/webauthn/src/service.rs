//! WebAuthn service implementation

use std::sync::Arc;

use tracing::{error, info};
use url::Url;
use webauthn_rs::prelude::*;
use webauthn_rs_proto::ResidentKeyRequirement;

use authenc_types::{AuthencError, Result, UserId};

use crate::store::CredentialStore;

/// WebAuthn service configuration
#[derive(Debug, Clone)]
pub struct WebAuthnConfig {
    /// Relying Party ID (domain name)
    pub rp_id: String,
    /// Relying Party origin (full URL)
    pub rp_origin: Url,
    /// Relying Party name (display name)
    pub rp_name: String,
}

impl Default for WebAuthnConfig {
    fn default() -> Self {
        Self {
            rp_id: "localhost".to_string(),
            rp_origin: Url::parse("http://localhost:8080").expect("Valid URL"),
            rp_name: "Authenc".to_string(),
        }
    }
}

/// WebAuthn service for passkey authentication
pub struct WebAuthnService {
    /// WebAuthn instance from webauthn-rs
    webauthn: Arc<Webauthn>,
    /// Credential storage
    credential_store: Arc<dyn CredentialStore>,
    /// Configuration
    config: WebAuthnConfig,
}

impl WebAuthnService {
    /// Create a new WebAuthn service
    ///
    /// # Arguments
    /// * `config` - WebAuthn configuration (RP ID, origin, name)
    /// * `credential_store` - Storage for credentials
    ///
    /// # Returns
    /// * `Result<Self>` - WebAuthn service instance or error
    ///
    /// # Example
    /// ```no_run
    /// use authenc_webauthn::{WebAuthnService, WebAuthnConfig};
    /// use url::Url;
    ///
    /// let config = WebAuthnConfig {
    ///     rp_id: "example.com".to_string(),
    ///     rp_origin: Url::parse("https://example.com").unwrap(),
    ///     rp_name: "Example App".to_string(),
    /// };
    ///
    /// // let service = WebAuthnService::new(config, credential_store)?;
    /// ```
    pub fn new(config: WebAuthnConfig, credential_store: Arc<dyn CredentialStore>) -> Result<Self> {
        info!(
            "Initializing WebAuthn service with RP ID: {}, Origin: {}",
            config.rp_id, config.rp_origin
        );

        // Build WebAuthn instance
        let builder = WebauthnBuilder::new(&config.rp_id, &config.rp_origin)
            .map_err(|e| {
                error!("Failed to create WebAuthn builder: {}", e);
                AuthencError::config(format!("Invalid WebAuthn configuration: {}", e))
            })?
            .rp_name(&config.rp_name);

        let webauthn = Arc::new(builder.build().map_err(|e| {
            error!("Failed to build WebAuthn instance: {}", e);
            AuthencError::config(format!("Failed to build WebAuthn: {}", e))
        })?);

        info!("WebAuthn service initialized successfully");

        Ok(Self {
            webauthn,
            credential_store,
            config,
        })
    }

    /// Get the relying party ID
    pub fn rp_id(&self) -> &str {
        &self.config.rp_id
    }

    /// Get the relying party origin
    pub fn rp_origin(&self) -> &Url {
        &self.config.rp_origin
    }

    /// Get the relying party name
    pub fn rp_name(&self) -> &str {
        &self.config.rp_name
    }

    /// Start passkey registration flow
    ///
    /// # Arguments
    /// * `user_id` - User ID registering the passkey
    /// * `username` - Username for display
    /// * `display_name` - Display name for the user
    ///
    /// # Returns
    /// * `Result<(CreationChallengeResponse, RegistrationSession)>` - Challenge to send to client and session state
    ///
    /// # Example
    /// ```no_run
    /// # use authenc_webauthn::WebAuthnService;
    /// # use authenc_types::UserId;
    /// # use uuid::Uuid;
    /// # async fn example(service: &WebAuthnService) -> authenc_types::Result<()> {
    /// let user_id = UserId(Uuid::new_v4());
    /// let (challenge, session) = service.start_registration(
    ///     user_id,
    ///     "alice",
    ///     "Alice Smith"
    /// ).await?;
    /// // Send challenge to client
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_registration(
        &self,
        user_id: UserId,
        username: &str,
        display_name: &str,
    ) -> Result<(
        CreationChallengeResponse,
        crate::models::RegistrationSession,
    )> {
        info!(
            "Starting passkey registration for user: {} ({})",
            username, user_id.0
        );

        // The WebAuthn user handle is the user's own id, stable across
        // registrations. It used to be a fresh random UUID per ceremony, so the
        // handle an authenticator stored (and hands back at a usernameless
        // login) pointed at nobody.
        let user_unique_id = user_id.0;

        // Get existing credentials for this user (for excluding from registration)
        let existing_credentials = self
            .credential_store
            .get_credentials_for_user(user_id)
            .await
            .unwrap_or_else(|e| {
                error!("Failed to get existing credentials: {}", e);
                vec![]
            });

        let excluded_credentials: Vec<CredentialID> = existing_credentials
            .iter()
            .map(|c| c.cred_id.clone())
            .collect();

        info!(
            "Excluding {} existing credentials from registration",
            excluded_credentials.len()
        );

        // Start registration ceremony
        let (mut ccr, reg_state) = self
            .webauthn
            .start_passkey_registration(
                user_unique_id,
                username,
                display_name,
                Some(excluded_credentials),
            )
            .map_err(|e| {
                error!("Failed to start passkey registration: {}", e);
                AuthencError::webauthn(format!("Registration start failed: {}", e))
            })?;

        require_discoverable_credential(&mut ccr);

        // Create registration session
        let registration = crate::models::RegistrationSession {
            user_id,
            state: reg_state,
            created_at: chrono::Utc::now(),
        };

        info!(
            "Passkey registration started successfully for user: {}",
            username
        );

        Ok((ccr, registration))
    }

    /// Finish passkey registration flow
    ///
    /// # Arguments
    /// * `user_id` - User ID registering the passkey
    /// * `reg` - Registration response from client
    /// * `state` - Registration session state from start_registration
    ///
    /// # Returns
    /// * `Result<StoredCredential>` - The stored credential
    ///
    /// # Example
    /// ```no_run
    /// # use authenc_webauthn::WebAuthnService;
    /// # use authenc_types::UserId;
    /// # use uuid::Uuid;
    /// # use webauthn_rs::prelude::*;
    /// # async fn example(service: &WebAuthnService, reg: RegisterPublicKeyCredential, session: authenc_webauthn::RegistrationSession) -> authenc_types::Result<()> {
    /// let user_id = UserId(Uuid::new_v4());
    /// let credential = service.finish_registration(user_id, &reg, &session).await?;
    /// println!("Credential registered: {:?}", credential.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn finish_registration(
        &self,
        user_id: UserId,
        reg: &RegisterPublicKeyCredential,
        state: &crate::models::RegistrationSession,
    ) -> Result<crate::models::StoredCredential> {
        info!("Finishing passkey registration for user: {}", user_id.0);

        // Verify user ID matches
        if state.user_id != user_id {
            error!(
                "User ID mismatch: expected {}, got {}",
                state.user_id.0, user_id.0
            );
            return Err(AuthencError::unauthorized("User ID mismatch"));
        }

        // Finish registration ceremony (verify attestation response)
        let passkey = self
            .webauthn
            .finish_passkey_registration(reg, &state.state)
            .map_err(|e| {
                error!("Failed to finish passkey registration: {}", e);
                AuthencError::webauthn(format!("Registration verification failed: {}", e))
            })?;

        // Create stored credential
        let stored_credential = crate::models::StoredCredential {
            id: uuid::Uuid::new_v4(),
            user_id,
            cred_id: passkey.cred_id().clone(),
            cred: passkey,
            nickname: None,
            created_at: chrono::Utc::now(),
            last_used: None,
        };

        // Store credential in database
        self.credential_store
            .store_credential(&stored_credential)
            .await
            .map_err(|e| {
                error!("Failed to store credential: {}", e);
                e
            })?;

        info!(
            "Passkey registration completed successfully for user: {}",
            user_id.0
        );

        Ok(stored_credential)
    }

    /// Start passkey authentication flow
    ///
    /// # Arguments
    /// * `user_id` - Optional user ID (None for usernameless authentication with discoverable credentials)
    ///
    /// # Returns
    /// * `Result<(RequestChallengeResponse, AuthenticationSession)>` - Challenge to send to client and session state
    ///
    /// # Example
    /// ```no_run
    /// # use authenc_webauthn::WebAuthnService;
    /// # use authenc_types::UserId;
    /// # use uuid::Uuid;
    /// # async fn example(service: &WebAuthnService) -> authenc_types::Result<()> {
    /// // User-specific authentication (username provided)
    /// let user_id = Some(UserId(Uuid::new_v4()));
    /// let (challenge, session) = service.start_authentication(user_id).await?;
    ///
    /// // Usernameless authentication (discoverable credentials)
    /// let (challenge, session) = service.start_authentication(None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_authentication(
        &self,
        user_id: Option<UserId>,
    ) -> Result<(
        RequestChallengeResponse,
        crate::models::AuthenticationSession,
    )> {
        if let Some(uid) = user_id {
            info!("Starting passkey authentication for user: {}", uid.0);
        } else {
            info!("Starting usernameless passkey authentication (discoverable credentials)");
        }

        let Some(uid) = user_id else {
            // Usernameless: the authenticator chooses the credential. This must
            // be the discoverable ceremony; a passkey ceremony started with an
            // empty allow-list cannot be finished (nothing to verify against).
            let (rcr, state) = self
                .webauthn
                .start_discoverable_authentication()
                .map_err(|e| {
                    error!("Failed to start discoverable authentication: {}", e);
                    AuthencError::webauthn(format!("Authentication start failed: {}", e))
                })?;
            info!("Usernameless passkey authentication started successfully");
            return Ok((
                rcr,
                crate::models::AuthenticationSession {
                    user_id: None,
                    state: crate::models::CeremonyState::Discoverable(state),
                    created_at: chrono::Utc::now(),
                },
            ));
        };

        // User-specific authentication: the allow-list is that user's credentials.
        let allowed_credentials = self
            .credential_store
            .get_credentials_for_user(uid)
            .await
            .unwrap_or_else(|e| {
                error!("Failed to get credentials for user: {}", e);
                vec![]
            });

        info!(
            "Allowing {} credentials for authentication",
            allowed_credentials.len()
        );

        let (rcr, auth_state) = self
            .webauthn
            .start_passkey_authentication(
                &allowed_credentials
                    .iter()
                    .map(|c| c.cred.clone())
                    .collect::<Vec<_>>(),
            )
            .map_err(|e| {
                error!("Failed to start passkey authentication: {}", e);
                AuthencError::webauthn(format!("Authentication start failed: {}", e))
            })?;

        info!("Passkey authentication started successfully");

        Ok((
            rcr,
            crate::models::AuthenticationSession {
                user_id,
                state: crate::models::CeremonyState::Passkey(auth_state),
                created_at: chrono::Utc::now(),
            },
        ))
    }

    /// Finish passkey authentication flow
    ///
    /// # Arguments
    /// * `auth` - Authentication response from client
    /// * `state` - Authentication session state from start_authentication
    ///
    /// # Returns
    /// * `Result<AuthenticationResult>` - Authentication result with user ID and credential ID
    ///
    /// # Example
    /// ```no_run
    /// # use authenc_webauthn::WebAuthnService;
    /// # use webauthn_rs::prelude::*;
    /// # async fn example(service: &WebAuthnService, auth: PublicKeyCredential, session: authenc_webauthn::AuthenticationSession) -> authenc_types::Result<()> {
    /// let result = service.finish_authentication(&auth, &session).await?;
    /// match result {
    ///     authenc_webauthn::AuthenticationResult::Success { user_id, credential_id } => {
    ///         println!("User {} authenticated with credential {}", user_id.0, credential_id);
    ///     }
    ///     authenc_webauthn::AuthenticationResult::Failed { reason } => {
    ///         println!("Authentication failed: {}", reason);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn finish_authentication(
        &self,
        auth: &PublicKeyCredential,
        state: &crate::models::AuthenticationSession,
    ) -> Result<crate::models::AuthenticationResult> {
        info!("Finishing passkey authentication");

        // Finish the ceremony that was started (verify the assertion), then load
        // the stored credential it was made with.
        let (auth_result, credential) = match &state.state {
            crate::models::CeremonyState::Passkey(passkey_state) => {
                let result = self
                    .webauthn
                    .finish_passkey_authentication(auth, passkey_state)
                    .map_err(|e| {
                        error!("Failed to finish passkey authentication: {}", e);
                        AuthencError::unauthorized("Authentication failed")
                    })?;
                let credential = self
                    .credential_store
                    .get_credential_by_id(result.cred_id())
                    .await
                    .map_err(|e| {
                        error!("Failed to get credential: {}", e);
                        AuthencError::not_found("Credential not found")
                    })?;
                (result, credential)
            }
            crate::models::CeremonyState::Discoverable(discoverable_state) => {
                // The authenticator picked the credential, so the credential id
                // it presents is what identifies the user. The user handle in the
                // assertion is not used: credentials registered before the handle
                // became the user's id carry a random one.
                let (_user_handle, presented_id) = self
                    .webauthn
                    .identify_discoverable_authentication(auth)
                    .map_err(|e| {
                        error!("Failed to identify discoverable credential: {}", e);
                        AuthencError::unauthorized("Authentication failed")
                    })?;
                let credential = self
                    .credential_store
                    .get_credential_by_id(&CredentialID::from(presented_id.to_vec()))
                    .await
                    .map_err(|e| {
                        error!("Failed to get credential: {}", e);
                        AuthencError::unauthorized("Authentication failed")
                    })?;
                let result = self
                    .webauthn
                    .finish_discoverable_authentication(
                        auth,
                        discoverable_state.clone(),
                        &[DiscoverableKey::from(&credential.cred)],
                    )
                    .map_err(|e| {
                        error!("Failed to finish discoverable authentication: {}", e);
                        AuthencError::unauthorized("Authentication failed")
                    })?;
                (result, credential)
            }
        };

        // A ceremony started for one user must not be finished with another
        // user's credential.
        if let Some(expected) = state.user_id
            && credential.user_id != expected
        {
            error!(
                "Credential belongs to user {} but the ceremony was started for {}",
                credential.user_id.0, expected.0
            );
            return Err(AuthencError::unauthorized("Authentication failed"));
        }

        // Verify credential counter (replay attack prevention)
        // Note: webauthn-rs handles counter validation internally during finish_passkey_authentication
        // If the counter doesn't increment, the authentication will fail above
        // We just need to update our stored counter value
        let new_counter = auth_result.counter();

        // Update credential counter (prevents replay attacks)
        self.credential_store
            .update_counter(credential.id, new_counter)
            .await
            .map_err(|e| {
                error!("Failed to update credential counter: {}", e);
                e
            })?;

        // Update last used timestamp
        self.credential_store
            .update_last_used(credential.id, chrono::Utc::now())
            .await
            .map_err(|e| {
                error!("Failed to update last used timestamp: {}", e);
                e
            })?;

        info!(
            "Passkey authentication completed successfully for user: {}",
            credential.user_id.0
        );

        Ok(crate::models::AuthenticationResult::Success {
            user_id: credential.user_id,
            credential_id: credential.id,
        })
    }

    /// List all credentials for a user
    ///
    /// # Arguments
    /// * `user_id` - User ID to list credentials for
    ///
    /// # Returns
    /// * `Result<Vec<StoredCredential>>` - List of credentials with metadata
    ///
    /// # Example
    /// ```no_run
    /// # use authenc_webauthn::WebAuthnService;
    /// # use authenc_types::UserId;
    /// # use uuid::Uuid;
    /// # async fn example(service: &WebAuthnService) -> authenc_types::Result<()> {
    /// let user_id = UserId(Uuid::new_v4());
    /// let credentials = service.list_credentials(user_id).await?;
    /// for cred in credentials {
    ///     println!("Credential: {} (created: {})", cred.id, cred.created_at);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_credentials(
        &self,
        user_id: UserId,
    ) -> Result<Vec<crate::models::StoredCredential>> {
        info!("Listing credentials for user: {}", user_id.0);

        let credentials = self
            .credential_store
            .get_credentials_for_user(user_id)
            .await?;

        info!(
            "Found {} credentials for user: {}",
            credentials.len(),
            user_id.0
        );

        Ok(credentials)
    }

    /// Delete a credential
    ///
    /// # Arguments
    /// * `user_id` - User ID (for ownership verification)
    /// * `credential_id` - Credential ID to delete
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Errors
    /// * `AuthencError::Unauthorized` - If user does not own the credential
    /// * `AuthencError::NotFound` - If credential does not exist
    ///
    /// # Example
    /// ```no_run
    /// # use authenc_webauthn::WebAuthnService;
    /// # use authenc_types::UserId;
    /// # use uuid::Uuid;
    /// # async fn example(service: &WebAuthnService) -> authenc_types::Result<()> {
    /// let user_id = UserId(Uuid::new_v4());
    /// let credential_id = Uuid::new_v4();
    /// service.delete_credential(user_id, credential_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_credential(
        &self,
        user_id: UserId,
        credential_id: uuid::Uuid,
    ) -> Result<()> {
        info!(
            "Deleting credential {} for user: {}",
            credential_id, user_id.0
        );

        // Verify ownership
        let credential = self
            .credential_store
            .get_credential(credential_id)
            .await
            .map_err(|_| AuthencError::not_found("Credential not found"))?;

        if credential.user_id != user_id {
            error!(
                "User {} attempted to delete credential {} owned by user {}",
                user_id.0, credential_id, credential.user_id.0
            );
            return Err(AuthencError::unauthorized("You do not own this credential"));
        }

        // Delete credential
        self.credential_store
            .delete_credential(credential_id)
            .await?;

        info!(
            "Credential {} deleted successfully for user: {}",
            credential_id, user_id.0
        );

        Ok(())
    }

    /// Update credential nickname
    ///
    /// # Arguments
    /// * `user_id` - User ID (for ownership verification)
    /// * `credential_id` - Credential ID to update
    /// * `nickname` - New nickname for the credential
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Errors
    /// * `AuthencError::Unauthorized` - If user does not own the credential
    /// * `AuthencError::NotFound` - If credential does not exist
    ///
    /// # Example
    /// ```no_run
    /// # use authenc_webauthn::WebAuthnService;
    /// # use authenc_types::UserId;
    /// # use uuid::Uuid;
    /// # async fn example(service: &WebAuthnService) -> authenc_types::Result<()> {
    /// let user_id = UserId(Uuid::new_v4());
    /// let credential_id = Uuid::new_v4();
    /// service.update_credential_nickname(user_id, credential_id, "My YubiKey".to_string()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_credential_nickname(
        &self,
        user_id: UserId,
        credential_id: uuid::Uuid,
        nickname: String,
    ) -> Result<()> {
        info!(
            "Updating nickname for credential {} (user: {})",
            credential_id, user_id.0
        );

        // Verify ownership
        let credential = self
            .credential_store
            .get_credential(credential_id)
            .await
            .map_err(|_| AuthencError::not_found("Credential not found"))?;

        if credential.user_id != user_id {
            error!(
                "User {} attempted to update credential {} owned by user {}",
                user_id.0, credential_id, credential.user_id.0
            );
            return Err(AuthencError::unauthorized("You do not own this credential"));
        }

        // Update nickname
        self.credential_store
            .update_nickname(credential_id, nickname.clone())
            .await?;

        info!(
            "Credential {} nickname updated to '{}' for user: {}",
            credential_id, nickname, user_id.0
        );

        Ok(())
    }
}

/// Ask the client for a discoverable (resident) credential.
///
/// The portal's primary login is usernameless: the user taps "Masuk dengan
/// Passkey" without typing a name and the authenticator offers its credential.
/// An authenticator can only do that for a discoverable credential. webauthn-rs
/// registers with `residentKey: discouraged`, which platform authenticators
/// (Touch ID, Windows Hello, Google Password Manager) override on their own but
/// security keys and virtual authenticators obey, leaving a credential that
/// registers fine and can never be used to sign in. `required` makes an
/// authenticator that cannot do it fail at registration, where the user sees it,
/// instead of at the first login.
pub(crate) fn require_discoverable_credential(ccr: &mut CreationChallengeResponse) {
    let selection = ccr
        .public_key
        .authenticator_selection
        .get_or_insert_with(Default::default);
    selection.resident_key = Some(ResidentKeyRequirement::Required);
    selection.require_resident_key = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use chrono::{DateTime, Utc};

    use crate::models::{CeremonyState, StoredCredential};

    /// A store with no credentials. The ceremonies under test start without one.
    #[derive(Default)]
    struct EmptyStore(Mutex<Vec<StoredCredential>>);

    #[async_trait]
    impl CredentialStore for EmptyStore {
        async fn store_credential(&self, credential: &StoredCredential) -> Result<()> {
            self.0.lock().unwrap().push(credential.clone());
            Ok(())
        }
        async fn get_credential(&self, id: uuid::Uuid) -> Result<StoredCredential> {
            self.0
                .lock()
                .unwrap()
                .iter()
                .find(|c| c.id == id)
                .cloned()
                .ok_or_else(|| AuthencError::not_found("Credential not found"))
        }
        async fn get_credential_by_id(&self, cred_id: &CredentialID) -> Result<StoredCredential> {
            self.0
                .lock()
                .unwrap()
                .iter()
                .find(|c| &c.cred_id == cred_id)
                .cloned()
                .ok_or_else(|| AuthencError::not_found("Credential not found"))
        }
        async fn get_credentials_for_user(&self, user_id: UserId) -> Result<Vec<StoredCredential>> {
            Ok(self
                .0
                .lock()
                .unwrap()
                .iter()
                .filter(|c| c.user_id == user_id)
                .cloned()
                .collect())
        }
        async fn delete_credential(&self, _id: uuid::Uuid) -> Result<()> {
            Ok(())
        }
        async fn update_last_used(&self, _id: uuid::Uuid, _ts: DateTime<Utc>) -> Result<()> {
            Ok(())
        }
        async fn update_counter(&self, _id: uuid::Uuid, _counter: u32) -> Result<()> {
            Ok(())
        }
        async fn update_nickname(&self, _id: uuid::Uuid, _nickname: String) -> Result<()> {
            Ok(())
        }
    }

    fn service() -> WebAuthnService {
        WebAuthnService::new(WebAuthnConfig::default(), Arc::new(EmptyStore::default()))
            .expect("the default localhost configuration is valid")
    }

    #[test]
    fn registration_asks_for_a_discoverable_credential_with_the_users_id_as_handle() {
        let svc = service();
        let user_id = UserId(uuid::Uuid::new_v4());
        let (ccr, _session) = tokio_test::block_on(svc.start_registration(user_id, "ani", "Ani"))
            .expect("registration starts");

        let selection = ccr
            .public_key
            .authenticator_selection
            .expect("authenticatorSelection is present");
        assert_eq!(
            selection.resident_key,
            Some(ResidentKeyRequirement::Required)
        );
        assert!(selection.require_resident_key);

        // The handle is the user's own id, the same one on every registration.
        let handle: &[u8] = ccr.public_key.user.id.as_ref();
        assert_eq!(handle, user_id.0.as_bytes());
        let (again, _) = tokio_test::block_on(svc.start_registration(user_id, "ani", "Ani"))
            .expect("registration starts again");
        let handle_again: &[u8] = again.public_key.user.id.as_ref();
        assert_eq!(handle, handle_again);
    }

    #[test]
    fn usernameless_start_is_a_discoverable_ceremony_with_an_empty_allow_list() {
        let svc = service();
        let (rcr, session) = tokio_test::block_on(svc.start_authentication(None))
            .expect("usernameless authentication starts");

        // This is the regression: the state used to be a plain passkey ceremony
        // built from an empty list, which finish_authentication can never verify.
        assert!(
            matches!(session.state, CeremonyState::Discoverable(_)),
            "usernameless must start the discoverable ceremony"
        );
        assert!(session.user_id.is_none());
        assert!(rcr.public_key.allow_credentials.is_empty());
    }

    #[test]
    fn a_named_user_starts_the_passkey_ceremony() {
        let svc = service();
        let user_id = UserId(uuid::Uuid::new_v4());
        let (_rcr, session) = tokio_test::block_on(svc.start_authentication(Some(user_id)))
            .expect("authentication starts");
        assert!(matches!(session.state, CeremonyState::Passkey(_)));
        assert_eq!(session.user_id, Some(user_id));
    }

    #[test]
    fn ceremony_state_survives_a_round_trip_through_json() {
        // The ceremony state is kept in the database between start and finish so
        // that any replica can finish what another started; it has to serialise.
        let svc = service();
        let (_rcr, session) = tokio_test::block_on(svc.start_authentication(None)).expect("starts");
        let json = serde_json::to_string(&session).expect("serialises");
        let back: crate::models::AuthenticationSession =
            serde_json::from_str(&json).expect("deserialises");
        assert!(matches!(back.state, CeremonyState::Discoverable(_)));

        let (_ccr, reg) = tokio_test::block_on(svc.start_registration(
            UserId(uuid::Uuid::new_v4()),
            "ani",
            "Ani",
        ))
        .expect("starts");
        let json = serde_json::to_string(&reg).expect("serialises");
        let _: crate::models::RegistrationSession =
            serde_json::from_str(&json).expect("deserialises");
    }

    #[test]
    fn test_webauthn_config_default() {
        let config = WebAuthnConfig::default();
        assert_eq!(config.rp_id, "localhost");
        assert_eq!(config.rp_origin.as_str(), "http://localhost:8080/");
        assert_eq!(config.rp_name, "Authenc");
    }

    #[test]
    fn test_webauthn_config_custom() {
        let config = WebAuthnConfig {
            rp_id: "example.com".to_string(),
            rp_origin: Url::parse("https://example.com").unwrap(),
            rp_name: "Example App".to_string(),
        };
        assert_eq!(config.rp_id, "example.com");
        assert_eq!(config.rp_origin.as_str(), "https://example.com/");
        assert_eq!(config.rp_name, "Example App");
    }
}
