//! Authentication service implementation
//!
//! This module implements the core authentication logic, including:
//! - Username/password authentication
//! - Brute force protection integration
//! - MFA requirement checking
//! - Session creation on successful authentication

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};

use authenc_types::{
    domain::*,
    domain_types::{AuthFailureReason, AuthResult, Credentials, RealmId, SessionId, UserId},
    error::AuthencError,
    result::Result,
    traits::*,
};

/// Implementation of the authentication service
///
/// This service orchestrates the authentication flow by coordinating between:
/// - UserStore: Fetching user data
/// - PasswordHasher: Verifying passwords
/// - BruteForceProtector: Preventing brute force attacks
/// - SessionStore: Creating sessions on successful authentication
pub struct AuthenticationServiceImpl {
    /// User storage for fetching user data
    user_store: Arc<dyn UserStore>,
    /// Session storage for creating sessions
    session_store: Arc<dyn SessionStore>,
    /// Password hasher for verifying passwords
    password_hasher: Arc<dyn PasswordHasher>,
    /// Brute force protector for preventing attacks
    brute_force_protector: Arc<dyn BruteForceProtector>,
}

impl AuthenticationServiceImpl {
    /// Create a new authentication service
    ///
    /// # Arguments
    ///
    /// * `user_store` - User storage implementation
    /// * `session_store` - Session storage implementation
    /// * `password_hasher` - Password hasher implementation
    /// * `brute_force_protector` - Brute force protector implementation
    pub fn new(
        user_store: Arc<dyn UserStore>,
        session_store: Arc<dyn SessionStore>,
        password_hasher: Arc<dyn PasswordHasher>,
        brute_force_protector: Arc<dyn BruteForceProtector>,
    ) -> Self {
        Self {
            user_store,
            session_store,
            password_hasher,
            brute_force_protector,
        }
    }

    /// Verify a plaintext password against a stored hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        self.password_hasher.verify(password, hash)
    }

    /// Generate a temporary MFA token for MFA verification
    ///
    /// This is a placeholder implementation. In production, this should:
    /// - Generate a cryptographically secure random token
    /// - Store the token with expiration (e.g., 5 minutes)
    /// - Associate the token with the user ID
    ///
    /// # Arguments
    ///
    /// * `user` - The user requiring MFA
    ///
    /// # Returns
    ///
    /// A temporary MFA token string
    async fn generate_mfa_token(&self, user: &User) -> Result<String> {
        // TODO: Implement proper MFA token generation with Secreton
        // For now, generate a simple token
        let token = format!("mfa_{}", uuid::Uuid::new_v4());
        debug!(user_id = %user.id, "Generated MFA token");
        Ok(token)
    }
}

#[async_trait]
impl AuthenticationService for AuthenticationServiceImpl {
    /// Authenticate a user with credentials
    ///
    /// This method implements the complete authentication flow:
    /// 1. Check brute force protection
    /// 2. Fetch user from database
    /// 3. Verify password
    /// 4. Check if MFA is required
    /// 5. Create session if authentication succeeds
    ///
    /// # Arguments
    ///
    /// * `credentials` - User credentials (username and password)
    /// * `realm_id` - Realm ID for multi-tenant isolation
    ///
    /// # Returns
    ///
    /// * `AuthResult::Success` - Authentication succeeded, session created
    /// * `AuthResult::MfaRequired` - MFA verification required
    /// * `AuthResult::Failed` - Authentication failed with reason
    async fn authenticate(
        &self,
        credentials: Credentials,
        realm_id: RealmId,
    ) -> Result<AuthResult> {
        debug!(
            username = %credentials.username,
            realm_id = %realm_id,
            "Starting authentication"
        );

        // Step 1: Check brute force protection
        if let Err(e) = self
            .brute_force_protector
            .check(&credentials.username)
            .await
        {
            warn!(
                username = %credentials.username,
                error = %e,
                "Brute force protection triggered"
            );
            return Ok(AuthResult::Failed {
                reason: AuthFailureReason::AccountLocked,
            });
        }

        // Step 2: Get user from store
        let user = match self
            .user_store
            .get_user_by_username(&credentials.username, realm_id)
            .await
        {
            Ok(user) => user,
            Err(AuthencError::UserNotFound(_)) => {
                // User not found - record failure and return invalid credentials
                self.brute_force_protector
                    .record_failure(&credentials.username)
                    .await?;
                warn!(
                    username = %credentials.username,
                    "User not found"
                );
                return Ok(AuthResult::Failed {
                    reason: AuthFailureReason::InvalidCredentials,
                });
            }
            Err(e) => {
                // Other error - return internal error
                warn!(
                    username = %credentials.username,
                    error = %e,
                    "Error fetching user"
                );
                return Ok(AuthResult::Failed {
                    reason: AuthFailureReason::InternalError(e.to_string()),
                });
            }
        };

        // Step 3: Check if user is enabled
        if !user.enabled {
            warn!(
                user_id = %user.id,
                username = %credentials.username,
                "User account is disabled"
            );
            return Ok(AuthResult::Failed {
                reason: AuthFailureReason::UserDisabled,
            });
        }

        // Step 4: Verify password
        let password_valid = match self.password_hasher.verify(
            &credentials.password,
            user.password_hash.as_deref().unwrap_or(""),
        ) {
            Ok(valid) => valid,
            Err(e) => {
                warn!(
                    user_id = %user.id,
                    error = %e,
                    "Error verifying password"
                );
                return Ok(AuthResult::Failed {
                    reason: AuthFailureReason::InternalError(e.to_string()),
                });
            }
        };

        if !password_valid {
            // Password invalid - record failure and return invalid credentials
            self.brute_force_protector
                .record_failure(&credentials.username)
                .await?;
            warn!(
                user_id = %user.id,
                username = %credentials.username,
                "Invalid password"
            );
            return Ok(AuthResult::Failed {
                reason: AuthFailureReason::InvalidCredentials,
            });
        }

        // Step 5: Password is valid - record success (reset failure count)
        self.brute_force_protector
            .record_success(&credentials.username)
            .await?;

        // Step 6: Check MFA requirement
        if user.mfa_enabled {
            let mfa_token = self.generate_mfa_token(&user).await?;
            info!(
                user_id = %user.id,
                username = %credentials.username,
                "Authentication succeeded, MFA required"
            );
            return Ok(AuthResult::MfaRequired {
                user_id: UserId::from_uuid(user.id),
                mfa_token,
            });
        }

        // Step 7: Create session
        let session = match self
            .session_store
            .create_session(UserId::from_uuid(user.id))
            .await
        {
            Ok(session) => session,
            Err(e) => {
                warn!(
                    user_id = %user.id,
                    error = %e,
                    "Error creating session"
                );
                return Ok(AuthResult::Failed {
                    reason: AuthFailureReason::InternalError(e.to_string()),
                });
            }
        };

        info!(
            user_id = %user.id,
            session_id = %session.id,
            username = %credentials.username,
            "Authentication succeeded"
        );

        Ok(AuthResult::Success {
            user_id: UserId::from_uuid(user.id),
            session_id: SessionId::from_uuid(session.id),
        })
    }

    /// Verify MFA code
    ///
    /// This method verifies a multi-factor authentication code and creates a session
    /// if verification succeeds.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID requiring MFA verification
    /// * `_code` - MFA code to verify (TOTP or backup code)
    ///
    /// # Returns
    ///
    /// * `AuthResult::Success` - MFA verification succeeded, session created
    /// * `AuthResult::Failed` - MFA verification failed
    async fn verify_mfa(&self, user_id: UserId, _code: String) -> Result<AuthResult> {
        debug!(
            user_id = %user_id,
            "Verifying MFA code"
        );

        // TODO: Implement MFA verification with authenc-mfa crate
        // For now, return a placeholder error
        warn!(
            user_id = %user_id,
            "MFA verification not yet implemented"
        );

        Ok(AuthResult::Failed {
            reason: AuthFailureReason::InternalError(
                "MFA verification not yet implemented".to_string(),
            ),
        })
    }

    /// Validate a session
    ///
    /// This method validates a session ID and returns the associated user if valid.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session ID to validate
    ///
    /// # Returns
    ///
    /// The user associated with the session
    ///
    /// # Errors
    ///
    /// Returns an error if the session is invalid or expired
    async fn validate_session(&self, session_id: SessionId) -> Result<User> {
        debug!(
            session_id = %session_id,
            "Validating session"
        );

        // Get session from store
        let session = self
            .session_store
            .get_session(session_id)
            .await?
            .ok_or_else(|| {
                AuthencError::SessionNotFound(format!("Session {} not found", session_id))
            })?;

        // Check if session is expired
        if session.expires_at < chrono::Utc::now() {
            warn!(
                session_id = %session_id,
                "Session expired"
            );
            return Err(AuthencError::TokenExpired);
        }

        // Update last accessed time
        self.session_store.update_last_accessed(session_id).await?;

        // Get user
        let user = self
            .user_store
            .get_user(UserId::from_uuid(session.user_id))
            .await?;

        // Check if user is still enabled
        if !user.enabled {
            warn!(
                user_id = %user.id,
                session_id = %session_id,
                "User account is disabled"
            );
            return Err(AuthencError::UserDisabled);
        }

        debug!(
            user_id = %user.id,
            session_id = %session_id,
            "Session validated successfully"
        );

        Ok(user)
    }

    /// Logout (invalidate session)
    ///
    /// This method invalidates a session, effectively logging out the user.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session ID to invalidate
    async fn logout(&self, session_id: SessionId) -> Result<()> {
        info!(
            session_id = %session_id,
            "Logging out"
        );

        self.session_store.invalidate_session(session_id).await?;

        info!(
            session_id = %session_id,
            "Logout successful"
        );

        Ok(())
    }
}
