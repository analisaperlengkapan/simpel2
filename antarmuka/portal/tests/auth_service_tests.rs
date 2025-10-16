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
        assert_eq!(UserRole::Admin.display_name(), "Administrator");
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
            division: "Test Division".to_string(),
            captcha_validated: true,
            mfa_enabled: true,
            mfa_setup_required: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some("test_token".to_string()),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: Some(chrono::Utc::now().timestamp() - 3600), // Expired 1 hour ago
            permissions: vec![],
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
            division: "Test Division".to_string(),
            captcha_validated: true,
            mfa_enabled: true,
            mfa_setup_required: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some("test_token".to_string()),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: Some(chrono::Utc::now().timestamp() + 3600), // Expires in 1 hour
            permissions: vec![],
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
            division: "Test Division".to_string(),
            captcha_validated: true,
            mfa_enabled: true,
            mfa_setup_required: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some("test_token".to_string()),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: None, // No expiration
            permissions: vec![],
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
            division: "Test Division".to_string(),
            captcha_validated: true,
            mfa_enabled: true,
            mfa_setup_required: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some("test_token".to_string()),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: Some(chrono::Utc::now().timestamp() + 240), // Expires in 4 minutes
            permissions: vec![],
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
            division: "Test Division".to_string(),
            captcha_validated: true,
            mfa_enabled: true,
            mfa_setup_required: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some("test_token".to_string()),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: Some(chrono::Utc::now().timestamp() + 3600), // Expires in 1 hour
            permissions: vec![],
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
            division: "Test Division".to_string(),
            captcha_validated: true,
            mfa_enabled: true,
            mfa_setup_required: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some("test_token".to_string()),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: Some(chrono::Utc::now().timestamp() + 3600),
            permissions: vec!["user:read".to_string(), "user:write".to_string()],
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
            division: "Admin Division".to_string(),
            captcha_validated: true,
            mfa_enabled: true,
            mfa_setup_required: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            access_token: Some("test_token".to_string()),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: Some(chrono::Utc::now().timestamp() + 3600),
            permissions: vec!["admin:*".to_string()],
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
                assert!(session.mfa_setup_required);
                assert!(session.access_token.is_some());
                assert!(session.refresh_token.is_some());
            }
            LoginResult::Error(msg) => {
                panic!("Login should succeed in mock mode: {}", msg);
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
        }

        // Clean up
        unsafe {
            std::env::remove_var("AUTHENC_API_URL");
        }
    }
}
