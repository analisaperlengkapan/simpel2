// Unit tests for portal authentication
// These run in non-WASM context for CI/CD

#[cfg(test)]
mod tests {
    use crate::features::auth::{LoginCredentials, UserRole, UserSession};

    #[test]
    fn test_user_role_display_names() {
        assert_eq!(UserRole::Admin.display_name(), "Administrator");
        assert_eq!(UserRole::User.display_name(), "Pengguna");
        assert_eq!(UserRole::Supervisor.display_name(), "Supervisor");
        assert_eq!(UserRole::Guest.display_name(), "Tamu");
    }

    #[test]
    fn test_user_role_permissions() {
        // Admin has all permissions
        assert!(UserRole::Admin.is_admin());
        assert!(UserRole::Admin.can_manage_users());

        // Supervisor can manage but not admin
        assert!(!UserRole::Supervisor.is_admin());
        assert!(UserRole::Supervisor.can_manage_users());

        // Regular user has no special permissions
        assert!(!UserRole::User.is_admin());
        assert!(!UserRole::User.can_manage_users());

        // Guest has no permissions
        assert!(!UserRole::Guest.is_admin());
        assert!(!UserRole::Guest.can_manage_users());
    }

    #[test]
    fn test_user_session_serialization() {
        let session = UserSession {
            id: "123".to_string(),
            username: "testuser".to_string(),
            role: UserRole::User,
            name: "Test User".to_string(),
            email: "test@kejaksaan.go.id".to_string(),
            avatar: None,
            division: "IT".to_string(),
            captcha_validated: true,
            mfa_enabled: false,
            mfa_setup_required: true,
            created_at: Some("2024-01-01T00:00:00Z".to_string()),
            access_token: Some("test_token".to_string()),
            refresh_token: Some("test_refresh".to_string()),
            expires_at: Some(1704067200), // 2024-01-01 00:00:00 UTC
            permissions: vec!["user:read".to_string()],
        };

        // Test serialization
        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("testuser"));
        assert!(json.contains("Test User"));

        // Test deserialization
        let deserialized: UserSession = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.username, session.username);
        assert_eq!(deserialized.role, session.role);
    }

    #[test]
    fn test_login_credentials_creation() {
        let credentials = LoginCredentials {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            captcha_token: None,
        };

        assert_eq!(credentials.username, "testuser");
        assert_eq!(credentials.password, "testpass");
    }

    #[test]
    fn test_user_session_default() {
        let session = UserSession::default();
        assert_eq!(session.username, "");
        assert_eq!(session.role, UserRole::User);
    }

    #[test]
    fn test_user_role_default() {
        let role = UserRole::default();
        assert_eq!(role, UserRole::User);
    }
}
