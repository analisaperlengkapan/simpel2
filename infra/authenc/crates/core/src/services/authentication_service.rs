//! Authentication service implementation
//!
//! This module implements the core authentication logic, including:
//! - Username/password authentication
//! - Brute force protection integration
//! - MFA requirement checking
//! - Session creation on successful authentication

use std::sync::Arc;
use async_trait::async_trait;
use tracing::{debug, info, warn};

use authenc_types::{
    domain::*,
    traits::*,
    result::Result,
    error::AuthencError,
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
    async fn authenticate(&self, credentials: Credentials, realm_id: RealmId) -> Result<AuthResult> {
        debug!(
            username = %credentials.username,
            realm_id = %realm_id,
            "Starting authentication"
        );

        // Step 1: Check brute force protection
        if let Err(e) = self.brute_force_protector.check(&credentials.username).await {
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
        let user = match self.user_store.get_user_by_username(&credentials.username, realm_id).await {
            Ok(user) => user,
            Err(AuthencError::UserNotFound(_)) => {
                // User not found - record failure and return invalid credentials
                self.brute_force_protector.record_failure(&credentials.username).await?;
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
        let password_valid = match self.password_hasher.verify(&credentials.password, &user.password_hash) {
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
            self.brute_force_protector.record_failure(&credentials.username).await?;
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
        self.brute_force_protector.record_success(&credentials.username).await?;

        // Step 6: Check MFA requirement
        if user.mfa_enabled {
            let mfa_token = self.generate_mfa_token(&user).await?;
            info!(
                user_id = %user.id,
                username = %credentials.username,
                "Authentication succeeded, MFA required"
            );
            return Ok(AuthResult::MfaRequired {
                user_id: user.id,
                mfa_token,
            });
        }

        // Step 7: Create session
        let session = match self.session_store.create_session(user.id).await {
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
            user_id: user.id,
            session_id: session.id,
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
                "MFA verification not yet implemented".to_string()
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
        let session = self.session_store.get_session(session_id).await?
            .ok_or_else(|| AuthencError::SessionNotFound(format!("Session {} not found", session_id)))?;

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
        let user = self.user_store.get_user(session.user_id).await?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::Mutex;

    // Mock implementations for testing

    struct MockUserStore {
        users: Arc<Mutex<HashMap<String, User>>>,
    }

    impl MockUserStore {
        fn new() -> Self {
            Self {
                users: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        async fn add_user(&self, user: User) {
            let mut users = self.users.lock().await;
            users.insert(user.username.clone(), user);
        }
    }

    #[async_trait]
    impl UserStore for MockUserStore {
        async fn get_user(&self, id: UserId) -> Result<User> {
            let users = self.users.lock().await;
            users.values()
                .find(|u| u.id == id)
                .cloned()
                .ok_or_else(|| AuthencError::UserNotFound(format!("User {} not found", id)))
        }

        async fn get_user_by_username(&self, username: &str, _realm_id: RealmId) -> Result<User> {
            let users = self.users.lock().await;
            users.get(username)
                .cloned()
                .ok_or_else(|| AuthencError::UserNotFound(format!("User {} not found", username)))
        }

        async fn get_user_by_email(&self, _email: &str, _realm_id: RealmId) -> Result<User> {
            unimplemented!()
        }

        async fn create_user(&self, _req: CreateUserRequest) -> Result<User> {
            unimplemented!()
        }

        async fn update_user(&self, _id: UserId, _req: UpdateUserRequest) -> Result<User> {
            unimplemented!()
        }

        async fn delete_user(&self, _id: UserId) -> Result<()> {
            unimplemented!()
        }

        async fn list_users(&self, _realm_id: RealmId, _offset: usize, _limit: usize) -> Result<Vec<User>> {
            unimplemented!()
        }

        async fn username_exists(&self, _username: &str, _realm_id: RealmId) -> Result<bool> {
            unimplemented!()
        }

        async fn email_exists(&self, _email: &str, _realm_id: RealmId) -> Result<bool> {
            unimplemented!()
        }
    }

    struct MockSessionStore {
        sessions: Arc<Mutex<HashMap<SessionId, Session>>>,
    }

    impl MockSessionStore {
        fn new() -> Self {
            Self {
                sessions: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl SessionStore for MockSessionStore {
        async fn create_session(&self, user_id: UserId) -> Result<Session> {
            let session = Session {
                id: SessionId::new(),
                user_id,
                created_at: chrono::Utc::now(),
                expires_at: chrono::Utc::now() + chrono::Duration::hours(8),
                last_accessed_at: chrono::Utc::now(),
            };
            let mut sessions = self.sessions.lock().await;
            sessions.insert(session.id, session.clone());
            Ok(session)
        }

        async fn get_session(&self, id: SessionId) -> Result<Option<Session>> {
            let sessions = self.sessions.lock().await;
            Ok(sessions.get(&id).cloned())
        }

        async fn update_last_accessed(&self, id: SessionId) -> Result<()> {
            let mut sessions = self.sessions.lock().await;
            if let Some(session) = sessions.get_mut(&id) {
                session.last_accessed_at = chrono::Utc::now();
            }
            Ok(())
        }

        async fn invalidate_session(&self, id: SessionId) -> Result<()> {
            let mut sessions = self.sessions.lock().await;
            sessions.remove(&id);
            Ok(())
        }

        async fn invalidate_user_sessions(&self, _user_id: UserId) -> Result<()> {
            unimplemented!()
        }

        async fn list_user_sessions(&self, _user_id: UserId) -> Result<Vec<Session>> {
            unimplemented!()
        }

        async fn cleanup_expired_sessions(&self) -> Result<usize> {
            unimplemented!()
        }
    }

    struct MockPasswordHasher;

    impl PasswordHasher for MockPasswordHasher {
        fn hash(&self, password: &str) -> Result<String> {
            // Simple mock: just prefix with "hashed_"
            Ok(format!("hashed_{}", password))
        }

        fn verify(&self, password: &str, hash: &str) -> Result<bool> {
            Ok(hash == format!("hashed_{}", password))
        }
    }

    struct MockBruteForceProtector {
        failures: Arc<Mutex<HashMap<String, usize>>>,
    }

    impl MockBruteForceProtector {
        fn new() -> Self {
            Self {
                failures: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl BruteForceProtector for MockBruteForceProtector {
        async fn check(&self, username: &str) -> Result<()> {
            let failures = self.failures.lock().await;
            let count = failures.get(username).unwrap_or(&0);
            if *count >= 5 {
                return Err(AuthencError::AccountLocked {
                    username: username.to_string(),
                    locked_until: None,
                });
            }
            Ok(())
        }

        async fn record_failure(&self, username: &str) -> Result<()> {
            let mut failures = self.failures.lock().await;
            let count = failures.entry(username.to_string()).or_insert(0);
            *count += 1;
            Ok(())
        }

        async fn record_success(&self, username: &str) -> Result<()> {
            let mut failures = self.failures.lock().await;
            failures.remove(username);
            Ok(())
        }

        async fn unlock(&self, username: &str) -> Result<()> {
            let mut failures = self.failures.lock().await;
            failures.remove(username);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_successful_authentication() {
        // Setup
        let user_store = Arc::new(MockUserStore::new());
        let session_store = Arc::new(MockSessionStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let brute_force_protector = Arc::new(MockBruteForceProtector::new());

        let realm_id = RealmId::new();
        let user = User {
            id: UserId::new(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password123".to_string(),
            enabled: true,
            email_verified: true,
            mfa_enabled: false,
            realm_id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        user_store.add_user(user.clone()).await;

        let auth_service = AuthenticationServiceImpl::new(
            user_store,
            session_store,
            password_hasher,
            brute_force_protector,
        );

        // Test
        let credentials = Credentials {
            username: "testuser".to_string(),
            password: "password123".to_string(),
        };

        let result = auth_service.authenticate(credentials, realm_id).await.unwrap();

        // Assert
        match result {
            AuthResult::Success { user_id, session_id } => {
                assert_eq!(user_id, user.id);
                assert_ne!(session_id, SessionId::new()); // Should have a valid session ID
            }
            _ => panic!("Expected AuthResult::Success"),
        }
    }

    #[tokio::test]
    async fn test_invalid_password() {
        // Setup
        let user_store = Arc::new(MockUserStore::new());
        let session_store = Arc::new(MockSessionStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let brute_force_protector = Arc::new(MockBruteForceProtector::new());

        let realm_id = RealmId::new();
        let user = User {
            id: UserId::new(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password123".to_string(),
            enabled: true,
            email_verified: true,
            mfa_enabled: false,
            realm_id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        user_store.add_user(user.clone()).await;

        let auth_service = AuthenticationServiceImpl::new(
            user_store,
            session_store,
            password_hasher,
            brute_force_protector,
        );

        // Test
        let credentials = Credentials {
            username: "testuser".to_string(),
            password: "wrongpassword".to_string(),
        };

        let result = auth_service.authenticate(credentials, realm_id).await.unwrap();

        // Assert
        match result {
            AuthResult::Failed { reason } => {
                assert!(matches!(reason, AuthFailureReason::InvalidCredentials));
            }
            _ => panic!("Expected AuthResult::Failed"),
        }
    }

    #[tokio::test]
    async fn test_user_not_found() {
        // Setup
        let user_store = Arc::new(MockUserStore::new());
        let session_store = Arc::new(MockSessionStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let brute_force_protector = Arc::new(MockBruteForceProtector::new());

        let auth_service = AuthenticationServiceImpl::new(
            user_store,
            session_store,
            password_hasher,
            brute_force_protector,
        );

        // Test
        let credentials = Credentials {
            username: "nonexistent".to_string(),
            password: "password123".to_string(),
        };

        let result = auth_service.authenticate(credentials, RealmId::new()).await.unwrap();

        // Assert
        match result {
            AuthResult::Failed { reason } => {
                assert!(matches!(reason, AuthFailureReason::InvalidCredentials));
            }
            _ => panic!("Expected AuthResult::Failed"),
        }
    }

    #[tokio::test]
    async fn test_disabled_user() {
        // Setup
        let user_store = Arc::new(MockUserStore::new());
        let session_store = Arc::new(MockSessionStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let brute_force_protector = Arc::new(MockBruteForceProtector::new());

        let realm_id = RealmId::new();
        let user = User {
            id: UserId::new(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password123".to_string(),
            enabled: false, // Disabled user
            email_verified: true,
            mfa_enabled: false,
            realm_id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        user_store.add_user(user.clone()).await;

        let auth_service = AuthenticationServiceImpl::new(
            user_store,
            session_store,
            password_hasher,
            brute_force_protector,
        );

        // Test
        let credentials = Credentials {
            username: "testuser".to_string(),
            password: "password123".to_string(),
        };

        let result = auth_service.authenticate(credentials, realm_id).await.unwrap();

        // Assert
        match result {
            AuthResult::Failed { reason } => {
                assert!(matches!(reason, AuthFailureReason::UserDisabled));
            }
            _ => panic!("Expected AuthResult::Failed with UserDisabled"),
        }
    }

    #[tokio::test]
    async fn test_mfa_required() {
        // Setup
        let user_store = Arc::new(MockUserStore::new());
        let session_store = Arc::new(MockSessionStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let brute_force_protector = Arc::new(MockBruteForceProtector::new());

        let realm_id = RealmId::new();
        let user = User {
            id: UserId::new(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password123".to_string(),
            enabled: true,
            email_verified: true,
            mfa_enabled: true, // MFA enabled
            realm_id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        user_store.add_user(user.clone()).await;

        let auth_service = AuthenticationServiceImpl::new(
            user_store,
            session_store,
            password_hasher,
            brute_force_protector,
        );

        // Test
        let credentials = Credentials {
            username: "testuser".to_string(),
            password: "password123".to_string(),
        };

        let result = auth_service.authenticate(credentials, realm_id).await.unwrap();

        // Assert
        match result {
            AuthResult::MfaRequired { user_id, mfa_token } => {
                assert_eq!(user_id, user.id);
                assert!(!mfa_token.is_empty());
            }
            _ => panic!("Expected AuthResult::MfaRequired"),
        }
    }

    #[tokio::test]
    async fn test_brute_force_protection() {
        // Setup
        let user_store = Arc::new(MockUserStore::new());
        let session_store = Arc::new(MockSessionStore::new());
        let password_hasher = Arc::new(MockPasswordHasher);
        let brute_force_protector = Arc::new(MockBruteForceProtector::new());

        let realm_id = RealmId::new();
        let user = User {
            id: UserId::new(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password123".to_string(),
            enabled: true,
            email_verified: true,
            mfa_enabled: false,
            realm_id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        user_store.add_user(user.clone()).await;

        let auth_service = AuthenticationServiceImpl::new(
            user_store,
            session_store,
            password_hasher,
            brute_force_protector,
        );

        // Test: Fail 5 times
        for _ in 0..5 {
            let credentials = Credentials {
                username: "testuser".to_string(),
                password: "wrongpassword".to_string(),
            };
            let _ = auth_service.authenticate(credentials, realm_id).await;
        }

        // Test: 6th attempt should be blocked
        let credentials = Credentials {
            username: "testuser".to_string(),
            password: "password123".to_string(), // Even with correct password
        };

        let result = auth_service.authenticate(credentials, realm_id).await.unwrap();

        // Assert
        match result {
            AuthResult::Failed { reason } => {
                assert!(matches!(reason, AuthFailureReason::AccountLocked));
            }
            _ => panic!("Expected AuthResult::Failed with AccountLocked"),
        }
    }
}
