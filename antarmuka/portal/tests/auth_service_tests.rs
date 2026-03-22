//! Unit tests for AuthService
//!
//! Tests authentication service methods including login, logout,
//! session management, and token handling

// In integration tests, we need to use the crate name from Cargo.toml
extern crate portal_microfrontend;

use portal_microfrontend::features::auth::{
    AuthService, LoginCredentials, LoginResult, UserRole, UserSession,
};

#[cfg(test)]
mod auth_service_tests {
    use super::*;

    #[test]
    fn test_validate_token_structure_valid() {
        let valid_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        assert!(AuthService::validate_token_structure(valid_token));
    }

    #[test]
    fn test_validate_token_structure_invalid() {
        let invalid_token = "not.a.valid.jwt.token";
        assert!(!AuthService::validate_token_structure(invalid_token));

        let incomplete_token = "header.payload";
        assert!(!AuthService::validate_token_structure(incomplete_token));

        let empty_token = "";
        assert!(!AuthService::validate_token_structure(empty_token));
    }

    #[test]
    fn test_user_role_display_names() {
        assert_eq!(UserRole::Admin.display_name(), "Administrator Global");
        assert_eq!(UserRole::User.display_name(), "Pengguna");
        assert_eq!(UserRole::Supervisor.display_name(), "Supervisor");
        assert_eq!(UserRole::Guest.display_name(), "Tamu");
    }

    #[test]
    fn test_user_role_permissions() {
        assert!(UserRole::Admin.is_admin());
        assert!(!UserRole::User.is_admin());
        assert!(!UserRole::Supervisor.is_admin());
        assert!(!UserRole::Guest.is_admin());

        assert!(UserRole::Admin.can_manage_users());
        assert!(UserRole::Supervisor.can_manage_users());
        assert!(!UserRole::User.can_manage_users());
        assert!(!UserRole::Guest.can_manage_users());
    }

    #[test]
    fn test_session_validation_expired() {
        let expired_session = UserSession {
            id: "test-id".to_string(),
            username: "test@kejaksaan.go.id".to_string(),
            role: UserRole::User,
            name: "Test User".to_string(),
            email: "test@kejaksaan.go.id".to_string(),
            avatar: None,
            satuan_kerja: "Test Satuan Kerja".to_string(),
            captcha_validated: true,
            mfa_enabled: true,
            mfa_setup_required: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some("test_token".to_string()),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: Some(chrono::Utc::now().timestamp() - 3600), // Expired 1 hour ago
            permissions: vec![],
            ..Default::default()
        };

        assert!(!AuthService::is_session_valid(&expired_session));
    }

    #[test]
    fn test_session_validation_valid() {
        let valid_session = UserSession {
            id: "test-id".to_string(),
            username: "test@kejaksaan.go.id".to_string(),
            role: UserRole::User,
            name: "Test User".to_string(),
            email: "test@kejaksaan.go.id".to_string(),
            avatar: None,
            satuan_kerja: "Test Satuan Kerja".to_string(),
            captcha_validated: true,
            mfa_enabled: true,
            mfa_setup_required: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some("test_token".to_string()),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: Some(chrono::Utc::now().timestamp() + 3600), // Expires in 1 hour
            permissions: vec![],
            ..Default::default()
        };

        assert!(AuthService::is_session_valid(&valid_session));
    }

    #[test]
    fn test_session_validation_no_expiry() {
        let session_no_expiry = UserSession {
            id: "test-id".to_string(),
            username: "test@kejaksaan.go.id".to_string(),
            role: UserRole::User,
            name: "Test User".to_string(),
            email: "test@kejaksaan.go.id".to_string(),
            avatar: None,
            satuan_kerja: "Test Satuan Kerja".to_string(),
            captcha_validated: true,
            mfa_enabled: true,
            mfa_setup_required: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some("test_token".to_string()),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: None, // No expiration
            permissions: vec![],
            ..Default::default()
        };

        assert!(AuthService::is_session_valid(&session_no_expiry));
    }

    #[test]
    fn test_should_refresh_token_soon() {
        let session = UserSession {
            id: "test-id".to_string(),
            username: "test@kejaksaan.go.id".to_string(),
            role: UserRole::User,
            name: "Test User".to_string(),
            email: "test@kejaksaan.go.id".to_string(),
            avatar: None,
            satuan_kerja: "Test Satuan Kerja".to_string(),
            captcha_validated: true,
            mfa_enabled: true,
            mfa_setup_required: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some("test_token".to_string()),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: Some(chrono::Utc::now().timestamp() + 240), // Expires in 4 minutes
            permissions: vec![],
            ..Default::default()
        };

        assert!(AuthService::should_refresh_token(&session));
    }

    #[test]
    fn test_should_not_refresh_token_yet() {
        let session = UserSession {
            id: "test-id".to_string(),
            username: "test@kejaksaan.go.id".to_string(),
            role: UserRole::User,
            name: "Test User".to_string(),
            email: "test@kejaksaan.go.id".to_string(),
            avatar: None,
            satuan_kerja: "Test Satuan Kerja".to_string(),
            captcha_validated: true,
            mfa_enabled: true,
            mfa_setup_required: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some("test_token".to_string()),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: Some(chrono::Utc::now().timestamp() + 3600), // Expires in 1 hour
            permissions: vec![],
            ..Default::default()
        };

        assert!(!AuthService::should_refresh_token(&session));
    }

    #[test]
    fn test_has_permission_exact_match() {
        let session = UserSession {
            id: "test-id".to_string(),
            username: "test@kejaksaan.go.id".to_string(),
            role: UserRole::User,
            name: "Test User".to_string(),
            email: "test@kejaksaan.go.id".to_string(),
            avatar: None,
            satuan_kerja: "Test Satuan Kerja".to_string(),
            captcha_validated: true,
            mfa_enabled: true,
            mfa_setup_required: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some("test_token".to_string()),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: Some(chrono::Utc::now().timestamp() + 3600),
            permissions: vec!["user:read".to_string(), "user:write".to_string()],
            ..Default::default()
        };

        assert!(AuthService::has_permission(&session, "user:read"));
        assert!(AuthService::has_permission(&session, "user:write"));
        assert!(!AuthService::has_permission(&session, "admin:delete"));
    }

    #[test]
    fn test_has_permission_wildcard() {
        let session = UserSession {
            id: "test-id".to_string(),
            username: "test@kejaksaan.go.id".to_string(),
            role: UserRole::Admin,
            name: "Admin User".to_string(),
            email: "admin@kejaksaan.go.id".to_string(),
            avatar: None,
            satuan_kerja: "Admin Satuan Kerja".to_string(),
            captcha_validated: true,
            mfa_enabled: true,
            mfa_setup_required: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some("test_token".to_string()),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: Some(chrono::Utc::now().timestamp() + 3600),
            permissions: vec!["admin:*".to_string()],
            ..Default::default()
        };

        assert!(AuthService::has_permission(&session, "admin:read"));
        assert!(AuthService::has_permission(&session, "admin:write"));
        assert!(AuthService::has_permission(&session, "admin:delete"));
        assert!(!AuthService::has_permission(&session, "user:read"));
    }

    #[tokio::test]
    async fn test_login_empty_username() {
        let credentials = LoginCredentials {
            username: "".to_string(),
            password: "password123".to_string(),
            captcha_token: None,
        };

        match AuthService::login(credentials).await {
            LoginResult::Error(msg) => {
                assert!(msg.contains("Username tidak boleh kosong"));
            }
            LoginResult::Success(_) => {
                panic!("Login should fail with empty username");
            }
            LoginResult::MfaSetupRequired(_)
            | LoginResult::MfaVerificationRequired(_)
            | LoginResult::PasswordChangeRequired(_) => {
                panic!("Login should fail with empty username, not require MFA");
            }
        }
    }

    #[tokio::test]
    async fn test_login_empty_password() {
        let credentials = LoginCredentials {
            username: "test@kejaksaan.go.id".to_string(),
            password: "".to_string(),
            captcha_token: None,
        };

        match AuthService::login(credentials).await {
            LoginResult::Error(msg) => {
                assert!(msg.contains("Password tidak boleh kosong"));
            }
            LoginResult::Success(_) => {
                panic!("Login should fail with empty password");
            }
            LoginResult::MfaSetupRequired(_)
            | LoginResult::MfaVerificationRequired(_)
            | LoginResult::PasswordChangeRequired(_) => {
                panic!("Login should fail with empty password, not require MFA");
            }
        }
    }

    #[tokio::test]
    async fn test_login_mock_success() {
        // Set mock mode
        unsafe {
            std::env::set_var("AUTHENC_API_URL", "mock");
        }

        let credentials = LoginCredentials {
            username: "admin".to_string(),
            password: "password123".to_string(),
            captcha_token: Some("mock_captcha".to_string()),
        };

        match AuthService::login(credentials).await {
            LoginResult::Success(session) => {
                assert_eq!(session.username, "admin");
                assert_eq!(session.role, UserRole::Admin);
                assert!(session.captcha_validated);
                // MFA setup is no longer required by default in mock mode
                assert!(!session.mfa_setup_required);
                assert!(session.access_token.is_some());
                assert!(session.refresh_token.is_some());
            }
            LoginResult::Error(msg) => {
                panic!("Login should succeed in mock mode: {}", msg);
            }
            LoginResult::MfaSetupRequired(_)
            | LoginResult::MfaVerificationRequired(_)
            | LoginResult::PasswordChangeRequired(_) => {
                panic!("Mock login should return Success for 'admin', not MFA required");
            }
        }

        // Clean up
        unsafe {
            std::env::remove_var("AUTHENC_API_URL");
        }
    }

    #[tokio::test]
    async fn test_login_mock_user_role() {
        // Set mock mode
        unsafe {
            std::env::set_var("AUTHENC_API_URL", "mock");
        }

        let credentials = LoginCredentials {
            username: "user@kejaksaan.go.id".to_string(),
            password: "password123".to_string(),
            captcha_token: Some("mock_captcha".to_string()),
        };

        match AuthService::login(credentials).await {
            LoginResult::Success(session) => {
                assert_eq!(session.role, UserRole::User);
                assert!(!session.role.is_admin());
                assert!(!session.role.can_manage_users());
            }
            LoginResult::Error(msg) => {
                panic!("Login should succeed in mock mode: {}", msg);
            }
            LoginResult::MfaSetupRequired(_)
            | LoginResult::MfaVerificationRequired(_)
            | LoginResult::PasswordChangeRequired(_) => {
                panic!("Mock login should return Success, not MFA required");
            }
        }

        // Clean up
        unsafe {
            std::env::remove_var("AUTHENC_API_URL");
        }
    }

    #[tokio::test]
    async fn test_login_mock_supervisor_role() {
        // Set mock mode
        unsafe {
            std::env::set_var("AUTHENC_API_URL", "mock");
        }

        let credentials = LoginCredentials {
            username: "supervisor@kejaksaan.go.id".to_string(),
            password: "password123".to_string(),
            captcha_token: Some("mock_captcha".to_string()),
        };

        match AuthService::login(credentials).await {
            LoginResult::Success(session) => {
                assert_eq!(session.role, UserRole::Supervisor);
                assert!(!session.role.is_admin());
                assert!(session.role.can_manage_users());
            }
            LoginResult::Error(msg) => {
                panic!("Login should succeed in mock mode: {}", msg);
            }
            LoginResult::MfaSetupRequired(_)
            | LoginResult::MfaVerificationRequired(_)
            | LoginResult::PasswordChangeRequired(_) => {
                panic!("Mock login should return Success, not MFA required");
            }
        }

        // Clean up
        unsafe {
            std::env::remove_var("AUTHENC_API_URL");
        }
    }

    // ========== MFA Method Tests ==========

    #[test]
    fn test_mfa_session_state_update() {
        // This test verifies that MFA state can be tracked in session
        // Note: In WASM environment, this would interact with localStorage
        // For non-WASM tests, we just verify the method exists and compiles
        AuthService::update_session_mfa_state(true, false, false);
        AuthService::update_session_mfa_state(false, true, false);
        AuthService::update_session_mfa_state(false, false, true);
    }

    #[tokio::test]
    async fn test_login_mfa_setup_required() {
        // Set mock mode
        unsafe {
            std::env::set_var("AUTHENC_API_URL", "mock");
        }

        let credentials = LoginCredentials {
            username: "user_setup".to_string(),
            password: "password123".to_string(),
            captcha_token: Some("mock_captcha".to_string()),
        };

        match AuthService::login(credentials).await {
            LoginResult::MfaSetupRequired(temp_token) => {
                assert_eq!(temp_token, "mock_temp_token_setup");
            }
            LoginResult::Success(_) => {
                panic!("Login should require MFA setup for user_setup");
            }
            LoginResult::MfaVerificationRequired(_) => {
                panic!("Login should require MFA setup, not verification");
            }
            LoginResult::PasswordChangeRequired(_) => {
                panic!("Login should require MFA setup, not password change");
            }
            LoginResult::Error(msg) => {
                panic!("Login should require MFA setup: {}", msg);
            }
        }

        // Clean up
        unsafe {
            std::env::remove_var("AUTHENC_API_URL");
        }
    }

    #[tokio::test]
    async fn test_login_mfa_verification_required() {
        // Set mock mode
        unsafe {
            std::env::set_var("AUTHENC_API_URL", "mock");
        }

        let credentials = LoginCredentials {
            username: "user_verify".to_string(),
            password: "password123".to_string(),
            captcha_token: Some("mock_captcha".to_string()),
        };

        match AuthService::login(credentials).await {
            LoginResult::MfaVerificationRequired(temp_token) => {
                assert_eq!(temp_token, "mock_temp_token_verify");
            }
            LoginResult::Success(_) => {
                panic!("Login should require MFA verification for user_verify");
            }
            LoginResult::MfaSetupRequired(_) => {
                panic!("Login should require MFA verification, not setup");
            }
            LoginResult::PasswordChangeRequired(_) => {
                panic!("Login should require MFA verification, not password change");
            }
            LoginResult::Error(msg) => {
                panic!("Login should require MFA verification: {}", msg);
            }
        }

        // Clean up
        unsafe {
            std::env::remove_var("AUTHENC_API_URL");
        }
    }

    #[tokio::test]
    async fn test_login_no_mfa_required() {
        // Set mock mode
        unsafe {
            std::env::set_var("AUTHENC_API_URL", "mock");
        }

        let credentials = LoginCredentials {
            username: "user_nomfa".to_string(),
            password: "password123".to_string(),
            captcha_token: Some("mock_captcha".to_string()),
        };

        match AuthService::login(credentials).await {
            LoginResult::Success(session) => {
                assert_eq!(session.username, "user_nomfa");
                assert!(!session.mfa_enabled);
                assert!(!session.mfa_setup_required);
            }
            LoginResult::MfaSetupRequired(_)
            | LoginResult::MfaVerificationRequired(_)
            | LoginResult::PasswordChangeRequired(_) => {
                panic!("Login should not require MFA for user_nomfa");
            }
            LoginResult::Error(msg) => {
                panic!("Login should succeed: {}", msg);
            }
        }

        // Clean up
        unsafe {
            std::env::remove_var("AUTHENC_API_URL");
        }
    }

    #[test]
    fn test_temp_token_storage() {
        // Test temp token save and retrieval
        // Note: In WASM environment, this would interact with localStorage
        // For non-WASM tests, we just verify the methods exist and compile
        AuthService::save_temp_token("test_temp_token");
        let _token = AuthService::get_temp_token();
        AuthService::clear_temp_token();
    }
}
