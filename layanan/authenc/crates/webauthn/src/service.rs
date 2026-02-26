//! WebAuthn service implementation

use std::sync::Arc;

use tracing::{error, info};
use url::Url;
use webauthn_rs::prelude::*;

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

        // Generate unique user handle
        let user_unique_id = uuid::Uuid::new_v4();

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
        let (ccr, reg_state) = self
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

        // Get allowed credentials
        let allowed_credentials = if let Some(uid) = user_id {
            // User-specific authentication (username provided)
            self.credential_store
                .get_credentials_for_user(uid)
                .await
                .unwrap_or_else(|e| {
                    error!("Failed to get credentials for user: {}", e);
                    vec![]
                })
        } else {
            // Usernameless authentication (discoverable credentials)
            vec![]
        };

        info!(
            "Allowing {} credentials for authentication",
            allowed_credentials.len()
        );

        // Start authentication ceremony
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

        // Create authentication session
        let authentication = crate::models::AuthenticationSession {
            user_id,
            state: auth_state,
            created_at: chrono::Utc::now(),
        };

        info!("Passkey authentication started successfully");

        Ok((rcr, authentication))
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

        // Finish authentication ceremony (verify assertion response)
        let auth_result = self
            .webauthn
            .finish_passkey_authentication(auth, &state.state)
            .map_err(|e| {
                error!("Failed to finish passkey authentication: {}", e);
                return crate::models::AuthenticationResult::Failed {
                    reason: format!("Authentication verification failed: {}", e),
                };
            })
            .map_err(|_| AuthencError::unauthorized("Authentication failed"))?;

        // Get credential from database
        let credential = self
            .credential_store
            .get_credential_by_id(&auth_result.cred_id())
            .await
            .map_err(|e| {
                error!("Failed to get credential: {}", e);
                AuthencError::not_found("Credential not found")
            })?;

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

#[cfg(test)]
mod tests {
    use super::*;

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
