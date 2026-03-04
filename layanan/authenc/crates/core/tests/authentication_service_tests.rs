//! Comprehensive unit tests for AuthenticationServiceImpl
//!
//! Tests cover:
//! - Successful authentication flow
//! - Failed authentication (invalid password, user not found, disabled user)
//! - MFA requirement checking
//! - Brute force protection integration
//! - Session creation and validation
//! - Logout functionality
//!
//! Target: >80% code coverage

use async_trait::async_trait;
use authenc_core::services::authentication_service::AuthenticationServiceImpl;
use authenc_types::{domain::*, domain_types::*, error::AuthencError, result::Result, traits::*};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// ============================================================================
// Mock Implementations
// ============================================================================

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
        users
            .values()
            .find(|u| u.id == id.0)
            .cloned()
            .ok_or_else(|| AuthencError::UserNotFound(format!("User {} not found", id)))
    }

    async fn get_user_by_username(&self, username: &str, _realm_id: RealmId) -> Result<User> {
        let users = self.users.lock().await;
        users
            .get(username)
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

    async fn list_users(
        &self,
        _realm_id: RealmId,
        _offset: usize,
        _limit: usize,
    ) -> Result<Vec<User>> {
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
        let now = chrono::Utc::now();
        let session = Session {
            id: uuid::Uuid::new_v4(),
            user_id: user_id.0,
            token: format!("test-token-{}", uuid::Uuid::new_v4()),
            refresh_token: None,
            created_at: now,
            expires_at: now + chrono::Duration::hours(8),
            last_accessed: now,
            ip_address: None,
            user_agent: None,
            revoked: false,
            mfa_verified: false,
            is_temp_session: false,
            mfa_verified_at: None,
        };
        let mut sessions = self.sessions.lock().await;
        sessions.insert(SessionId::from_uuid(session.id), session.clone());
        Ok(session)
    }

    async fn get_session(&self, id: SessionId) -> Result<Option<Session>> {
        let sessions = self.sessions.lock().await;
        Ok(sessions.get(&id).cloned())
    }

    async fn update_last_accessed(&self, id: SessionId) -> Result<()> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(&id) {
            session.last_accessed = chrono::Utc::now();
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

// ============================================================================
// Test Helper Functions
// ============================================================================

fn create_test_user(
    username: &str,
    password: &str,
    enabled: bool,
    mfa_enabled: bool,
    _realm_id: RealmId,
) -> User {
    let now = chrono::Utc::now();
    User {
        id: uuid::Uuid::new_v4(),
        username: username.to_string(),
        email: format!("{}@example.com", username),
        email_verified: true,
        first_name: None,
        last_name: None,
        nip: None,
        nama: None,
        jabatan: None,
        satker_code: "001".to_string(),
        phone_number: None,
        phone_verified: false,
        password_hash: Some(format!("hashed_{}", password)),
        totp_secret: None,
        totp_backup_codes: None,
        mfa_enabled,
        mfa_setup_at: None,
        mfa_last_used: None,
        webauthn_enabled: false,
        account_locked: false,
        account_locked_until: None,
        failed_login_attempts: 0,
        last_login_at: None,
        last_failed_login_at: None,
        password_changed_at: None,
        password_expires_at: None,
        require_password_change: false,
        realm_id: None,
        organization_id: None,
        roles: Vec::new(),
        permissions: Vec::new(),
        session_data: None,
        security_context: authenc_types::SecurityContext::default(),
        attributes: None,
        enabled,
        federated: false,
        created_at: now,
        updated_at: now,
        deleted_at: None,
        login_count: 0,
    }
}

fn create_auth_service() -> (
    AuthenticationServiceImpl,
    Arc<MockUserStore>,
    Arc<MockSessionStore>,
    Arc<MockBruteForceProtector>,
) {
    let user_store = Arc::new(MockUserStore::new());
    let session_store = Arc::new(MockSessionStore::new());
    let password_hasher = Arc::new(MockPasswordHasher);
    let brute_force_protector = Arc::new(MockBruteForceProtector::new());

    let auth_service = AuthenticationServiceImpl::new(
        user_store.clone(),
        session_store.clone(),
        password_hasher,
        brute_force_protector.clone(),
    );

    (
        auth_service,
        user_store,
        session_store,
        brute_force_protector,
    )
}

// ============================================================================
// Authentication Flow Tests
// ============================================================================

#[tokio::test]
async fn test_successful_authentication_without_mfa() {
    let (auth_service, user_store, _, _) = create_auth_service();
    let realm_id = RealmId::new();

    let user = create_test_user("testuser", "password123", true, false, realm_id);
    user_store.add_user(user.clone()).await;

    let credentials = Credentials {
        username: "testuser".to_string(),
        password: "password123".to_string(),
    };

    let result = auth_service
        .authenticate(credentials, realm_id)
        .await
        .unwrap();

    match result {
        AuthResult::Success {
            user_id,
            session_id,
        } => {
            assert_eq!(user_id.0, user.id);
            assert_ne!(session_id, SessionId::new());
        }
        _ => panic!("Expected AuthResult::Success"),
    }
}

#[tokio::test]
async fn test_successful_authentication_with_mfa_required() {
    let (auth_service, user_store, _, _) = create_auth_service();
    let realm_id = RealmId::new();

    let user = create_test_user("testuser", "password123", true, true, realm_id);
    user_store.add_user(user.clone()).await;

    let credentials = Credentials {
        username: "testuser".to_string(),
        password: "password123".to_string(),
    };

    let result = auth_service
        .authenticate(credentials, realm_id)
        .await
        .unwrap();

    match result {
        AuthResult::MfaRequired { user_id, mfa_token } => {
            assert_eq!(user_id.0, user.id);
            assert!(!mfa_token.is_empty());
            assert!(mfa_token.starts_with("mfa_"));
        }
        _ => panic!("Expected AuthResult::MfaRequired"),
    }
}

#[tokio::test]
async fn test_authentication_invalid_password() {
    let (auth_service, user_store, _, _) = create_auth_service();
    let realm_id = RealmId::new();

    let user = create_test_user("testuser", "password123", true, false, realm_id);
    user_store.add_user(user).await;

    let credentials = Credentials {
        username: "testuser".to_string(),
        password: "wrongpassword".to_string(),
    };

    let result = auth_service
        .authenticate(credentials, realm_id)
        .await
        .unwrap();

    match result {
        AuthResult::Failed { reason } => {
            assert!(matches!(reason, AuthFailureReason::InvalidCredentials));
        }
        _ => panic!("Expected AuthResult::Failed"),
    }
}

#[tokio::test]
async fn test_authentication_user_not_found() {
    let (auth_service, _, _, _) = create_auth_service();
    let realm_id = RealmId::new();

    let credentials = Credentials {
        username: "nonexistent".to_string(),
        password: "password123".to_string(),
    };

    let result = auth_service
        .authenticate(credentials, realm_id)
        .await
        .unwrap();

    match result {
        AuthResult::Failed { reason } => {
            assert!(matches!(reason, AuthFailureReason::InvalidCredentials));
        }
        _ => panic!("Expected AuthResult::Failed"),
    }
}

#[tokio::test]
async fn test_authentication_disabled_user() {
    let (auth_service, user_store, _, _) = create_auth_service();
    let realm_id = RealmId::new();

    let user = create_test_user("testuser", "password123", false, false, realm_id);
    user_store.add_user(user).await;

    let credentials = Credentials {
        username: "testuser".to_string(),
        password: "password123".to_string(),
    };

    let result = auth_service
        .authenticate(credentials, realm_id)
        .await
        .unwrap();

    match result {
        AuthResult::Failed { reason } => {
            assert!(matches!(reason, AuthFailureReason::UserDisabled));
        }
        _ => panic!("Expected AuthResult::Failed with UserDisabled"),
    }
}

// ============================================================================
// Brute Force Protection Tests
// ============================================================================

#[tokio::test]
async fn test_brute_force_protection_triggers_after_max_attempts() {
    let (auth_service, user_store, _, _) = create_auth_service();
    let realm_id = RealmId::new();

    let user = create_test_user("testuser", "password123", true, false, realm_id);
    user_store.add_user(user).await;

    // Fail 5 times
    for _ in 0..5 {
        let credentials = Credentials {
            username: "testuser".to_string(),
            password: "wrongpassword".to_string(),
        };
        let _ = auth_service.authenticate(credentials, realm_id).await;
    }

    // 6th attempt should be blocked even with correct password
    let credentials = Credentials {
        username: "testuser".to_string(),
        password: "password123".to_string(),
    };

    let result = auth_service
        .authenticate(credentials, realm_id)
        .await
        .unwrap();

    match result {
        AuthResult::Failed { reason } => {
            assert!(matches!(reason, AuthFailureReason::AccountLocked));
        }
        _ => panic!("Expected AuthResult::Failed with AccountLocked"),
    }
}

#[tokio::test]
async fn test_brute_force_protection_resets_on_success() {
    let (auth_service, user_store, _, brute_force_protector) = create_auth_service();
    let realm_id = RealmId::new();

    let user = create_test_user("testuser", "password123", true, false, realm_id);
    user_store.add_user(user).await;

    // Fail 3 times
    for _ in 0..3 {
        let credentials = Credentials {
            username: "testuser".to_string(),
            password: "wrongpassword".to_string(),
        };
        let _ = auth_service.authenticate(credentials, realm_id).await;
    }

    // Verify failures recorded
    let failures = brute_force_protector.failures.lock().await;
    assert_eq!(*failures.get("testuser").unwrap(), 3);
    drop(failures);

    // Successful authentication should reset
    let credentials = Credentials {
        username: "testuser".to_string(),
        password: "password123".to_string(),
    };
    let result = auth_service
        .authenticate(credentials, realm_id)
        .await
        .unwrap();
    assert!(matches!(result, AuthResult::Success { .. }));

    // Verify failures cleared
    let failures = brute_force_protector.failures.lock().await;
    assert!(!failures.contains_key("testuser"));
}

// ============================================================================
// Session Management Tests
// ============================================================================

#[tokio::test]
async fn test_validate_session_success() {
    let (auth_service, user_store, session_store, _) = create_auth_service();
    let realm_id = RealmId::new();

    let user = create_test_user("testuser", "password123", true, false, realm_id);
    user_store.add_user(user.clone()).await;

    // Create a session
    let session = session_store
        .create_session(UserId::from_uuid(user.id))
        .await
        .unwrap();

    // Validate session
    let validated_user = auth_service
        .validate_session(SessionId::from_uuid(session.id))
        .await
        .unwrap();
    assert_eq!(validated_user.id, user.id);
    assert_eq!(validated_user.username, user.username);
}

#[tokio::test]
async fn test_validate_session_not_found() {
    let (auth_service, _, _, _) = create_auth_service();

    let invalid_session_id = SessionId::new();
    let result = auth_service.validate_session(invalid_session_id).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AuthencError::SessionNotFound(_)
    ));
}

#[tokio::test]
async fn test_validate_session_expired() {
    let (auth_service, user_store, session_store, _) = create_auth_service();
    let realm_id = RealmId::new();

    let user = create_test_user("testuser", "password123", true, false, realm_id);
    user_store.add_user(user.clone()).await;

    // Create an expired session
    let now = chrono::Utc::now();
    let session = Session {
        id: uuid::Uuid::new_v4(),
        user_id: user.id,
        token: "test-expired-token".to_string(),
        refresh_token: None,
        created_at: now - chrono::Duration::hours(10),
        expires_at: now - chrono::Duration::hours(2), // Expired
        last_accessed: now - chrono::Duration::hours(2),
        ip_address: None,
        user_agent: None,
        revoked: false,
        mfa_verified: false,
        is_temp_session: false,
        mfa_verified_at: None,
    };

    let mut sessions = session_store.sessions.lock().await;
    sessions.insert(SessionId::from_uuid(session.id), session.clone());
    drop(sessions);

    // Validate expired session
    let result = auth_service
        .validate_session(SessionId::from_uuid(session.id))
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthencError::TokenExpired));
}

#[tokio::test]
async fn test_validate_session_disabled_user() {
    let (auth_service, user_store, session_store, _) = create_auth_service();
    let realm_id = RealmId::new();

    let mut user = create_test_user("testuser", "password123", true, false, realm_id);
    user_store.add_user(user.clone()).await;

    // Create a session
    let session = session_store
        .create_session(UserId::from_uuid(user.id))
        .await
        .unwrap();

    // Disable user
    user.enabled = false;
    let mut users = user_store.users.lock().await;
    users.insert(user.username.clone(), user.clone());
    drop(users);

    // Validate session with disabled user
    let result = auth_service
        .validate_session(SessionId::from_uuid(session.id))
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AuthencError::UserDisabled));
}

// ============================================================================
// Logout Tests
// ============================================================================

#[tokio::test]
async fn test_logout_success() {
    let (auth_service, user_store, session_store, _) = create_auth_service();
    let realm_id = RealmId::new();

    let user = create_test_user("testuser", "password123", true, false, realm_id);
    user_store.add_user(user.clone()).await;

    // Create a session
    let session = session_store
        .create_session(UserId::from_uuid(user.id))
        .await
        .unwrap();

    // Verify session exists
    let sessions = session_store.sessions.lock().await;
    assert!(sessions.contains_key(&SessionId::from_uuid(session.id)));
    drop(sessions);

    // Logout
    auth_service
        .logout(SessionId::from_uuid(session.id))
        .await
        .unwrap();

    // Verify session removed
    let sessions = session_store.sessions.lock().await;
    assert!(!sessions.contains_key(&SessionId::from_uuid(session.id)));
}

#[tokio::test]
async fn test_logout_nonexistent_session() {
    let (auth_service, _, _, _) = create_auth_service();

    let invalid_session_id = SessionId::new();

    // Logout should succeed even if session doesn't exist (idempotent)
    let result = auth_service.logout(invalid_session_id).await;
    assert!(result.is_ok());
}

// ============================================================================
// Edge Cases and Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_authentication_empty_username() {
    let (auth_service, _, _, _) = create_auth_service();
    let realm_id = RealmId::new();

    let credentials = Credentials {
        username: "".to_string(),
        password: "password123".to_string(),
    };

    let result = auth_service
        .authenticate(credentials, realm_id)
        .await
        .unwrap();

    match result {
        AuthResult::Failed { reason } => {
            assert!(matches!(reason, AuthFailureReason::InvalidCredentials));
        }
        _ => panic!("Expected AuthResult::Failed"),
    }
}

#[tokio::test]
async fn test_authentication_empty_password() {
    let (auth_service, user_store, _, _) = create_auth_service();
    let realm_id = RealmId::new();

    let user = create_test_user("testuser", "password123", true, false, realm_id);
    user_store.add_user(user).await;

    let credentials = Credentials {
        username: "testuser".to_string(),
        password: "".to_string(),
    };

    let result = auth_service
        .authenticate(credentials, realm_id)
        .await
        .unwrap();

    match result {
        AuthResult::Failed { reason } => {
            assert!(matches!(reason, AuthFailureReason::InvalidCredentials));
        }
        _ => panic!("Expected AuthResult::Failed"),
    }
}

#[tokio::test]
async fn test_authentication_case_sensitive_username() {
    let (auth_service, user_store, _, _) = create_auth_service();
    let realm_id = RealmId::new();

    let user = create_test_user("TestUser", "password123", true, false, realm_id);
    user_store.add_user(user).await;

    // Try with different case
    let credentials = Credentials {
        username: "testuser".to_string(),
        password: "password123".to_string(),
    };

    let result = auth_service
        .authenticate(credentials, realm_id)
        .await
        .unwrap();

    match result {
        AuthResult::Failed { reason } => {
            assert!(matches!(reason, AuthFailureReason::InvalidCredentials));
        }
        _ => panic!("Expected AuthResult::Failed (username should be case-sensitive)"),
    }
}

#[tokio::test]
async fn test_multiple_sequential_authentications() {
    let (auth_service, user_store, _, _) = create_auth_service();
    let realm_id = RealmId::new();

    // Create multiple users
    for i in 1..=5 {
        let user = create_test_user(&format!("user{}", i), "password123", true, false, realm_id);
        user_store.add_user(user).await;
    }

    // Authenticate sequentially
    for i in 1..=5 {
        let credentials = Credentials {
            username: format!("user{}", i),
            password: "password123".to_string(),
        };
        let result = auth_service
            .authenticate(credentials, realm_id)
            .await
            .unwrap();
        assert!(matches!(result, AuthResult::Success { .. }));
    }
}
