//! Session management tests
//!
//! Tests session validation, expiration, and token refresh logic
//! Note: These tests focus on business logic that doesn't require WASM/localStorage

// In integration tests, we need to use the crate name from Cargo.toml
extern crate portal_microfrontend;

use portal_microfrontend::features::auth::{AuthService, TokenResponse, UserRole, UserSession};

#[cfg(test)]
mod session_management_tests {
    use super::*;

    /// Helper function to create a test session
    fn create_test_session(expires_in_seconds: i64) -> UserSession {
        let now = chrono::Utc::now();
        let expires_at = now + chrono::Duration::seconds(expires_in_seconds);

        UserSession {
            id: uuid::Uuid::new_v4().to_string(),
            username: "test@kejaksaan.go.id".to_string(),
            role: UserRole::User,
            name: "Test User".to_string(),
            email: "test@kejaksaan.go.id".to_string(),
            avatar: None,
            satuan_kerja: "Test Satuan Kerja".to_string(),
            captcha_validated: true,
            mfa_enabled: true,
            mfa_setup_required: false,
            created_at: Some(now.to_rfc3339()),
            access_token: Some("test_access_token".to_string()),
            refresh_token: Some("test_refresh_token".to_string()),
            expires_at: Some(expires_at.timestamp()),
            ..Default::default()
        }
    }

    /// Test session expiration detection
    #[test]
    fn test_session_expiration_detection() {
        // Create expired session (expired 1 hour ago)
        let expired_session = create_test_session(-3600);
        assert!(!AuthService::is_session_valid(&expired_session));

        // Create valid session (expires in 1 hour)
        let valid_session = create_test_session(3600);
        assert!(AuthService::is_session_valid(&valid_session));

        // Create session expiring soon (expires in 2 minutes)
        let expiring_soon_session = create_test_session(120);
        assert!(AuthService::is_session_valid(&expiring_soon_session));
    }

    /// Test token refresh requirement detection
    #[test]
    fn test_token_refresh_requirement() {
        // Session expiring in 10 minutes - should not need refresh yet
        let session_10min = create_test_session(600);
        assert!(!AuthService::should_refresh_token(&session_10min));

        // Session expiring in 4 minutes - should need refresh
        let session_4min = create_test_session(240);
        assert!(AuthService::should_refresh_token(&session_4min));

        // Session expiring in 1 minute - should need refresh
        let session_1min = create_test_session(60);
        assert!(AuthService::should_refresh_token(&session_1min));

        // Already expired session - should need refresh
        let expired_session = create_test_session(-60);
        assert!(!AuthService::is_session_valid(&expired_session));
    }

    /// Test session without expiration
    #[test]
    fn test_session_without_expiration() {
        let mut session = create_test_session(3600);
        session.expires_at = None; // No expiration set

        // Session without expiration should be considered valid
        assert!(AuthService::is_session_valid(&session));

        // Session without expiration should not need refresh
        assert!(!AuthService::should_refresh_token(&session));
    }

    /// Test token structure validation
    #[test]
    fn test_token_structure_validation() {
        // Valid JWT structure (3 parts separated by dots)
        let valid_jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        assert!(AuthService::validate_token_structure(valid_jwt));

        // Invalid structures
        assert!(!AuthService::validate_token_structure("invalid"));
        assert!(!AuthService::validate_token_structure("header.payload"));
        assert!(!AuthService::validate_token_structure(""));
        assert!(!AuthService::validate_token_structure("a.b.c.d")); // Too many parts
    }

    /// Test token refresh response handling
    #[test]
    fn test_token_refresh_response() {
        let token_response = TokenResponse {
            access_token: "new_access_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 28800, // 8 hours
            refresh_token: Some("new_refresh_token".to_string()),
        };

        // Verify token response structure
        assert_eq!(token_response.access_token, "new_access_token");
        assert_eq!(token_response.token_type, "Bearer");
        assert_eq!(token_response.expires_in, 28800);
        assert_eq!(
            token_response.refresh_token,
            Some("new_refresh_token".to_string())
        );
    }

    /// Test session validity edge cases
    #[test]
    fn test_session_validity_edge_cases() {
        // Session expiring in exactly 5 minutes (300 seconds)
        let session_5min = create_test_session(300);
        assert!(AuthService::is_session_valid(&session_5min));
        assert!(!AuthService::should_refresh_token(&session_5min)); // Exactly at threshold

        // Session expiring in 299 seconds (just under 5 minutes)
        let session_299sec = create_test_session(299);
        assert!(AuthService::is_session_valid(&session_299sec));
        assert!(AuthService::should_refresh_token(&session_299sec)); // Should refresh

        // Session expiring in 1 second
        let session_1sec = create_test_session(1);
        assert!(AuthService::is_session_valid(&session_1sec));
        assert!(AuthService::should_refresh_token(&session_1sec));

        // Session expired 1 second ago
        let session_expired_1sec = create_test_session(-1);
        assert!(!AuthService::is_session_valid(&session_expired_1sec));
    }

    /// Test session with different user roles
    #[test]
    fn test_session_with_different_roles() {
        // Admin session
        let mut admin_session = create_test_session(3600);
        admin_session.role = UserRole::Admin;
        assert!(AuthService::is_session_valid(&admin_session));
        assert!(admin_session.role.is_admin());

        // Supervisor session
        let mut supervisor_session = create_test_session(3600);
        supervisor_session.role = UserRole::Supervisor;
        assert!(AuthService::is_session_valid(&supervisor_session));
        assert!(supervisor_session.role.can_manage_users());

        // Regular user session
        let mut user_session = create_test_session(3600);
        user_session.role = UserRole::User;
        assert!(AuthService::is_session_valid(&user_session));
        assert!(!user_session.role.is_admin());

        // Guest session
        let mut guest_session = create_test_session(3600);
        guest_session.role = UserRole::Guest;
        assert!(AuthService::is_session_valid(&guest_session));
        assert!(!guest_session.role.can_manage_users());
    }

    /// The IAM console gate: EXACT `admin`, nothing that merely looks like it.
    ///
    /// These replace six tests that pinned a permission vocabulary — `"admin:*"`,
    /// `"user:read"`, wildcard namespaces — which no issuer in this system ever
    /// minted. #930 removed `has_permission` for exactly that reason (it could
    /// never match a real token), and the tests went unnoticed because CI never
    /// compiled this crate's host test targets. What replaced the vocabulary is
    /// `lib_core::authz`, and `can_administer_iam` is where this type uses it.
    #[test]
    fn test_iam_console_needs_the_exact_admin_role() {
        let with_roles = |roles: &[&str]| {
            let mut s = create_test_session(3600);
            s.roles = roles.iter().map(|r| r.to_string()).collect();
            s
        };

        assert!(with_roles(&["admin"]).can_administer_iam());
        assert!(
            with_roles(&["ADMIN"]).can_administer_iam(),
            "role names are compared case-insensitively"
        );
        assert!(with_roles(&["validator_pusat", "admin"]).can_administer_iam());

        // The bug #930 fixed was a prefix match: `starts_with("admin")` let
        // these through. Each must be refused.
        for lookalike in [
            "admin_master_read_only",
            "administrative",
            "admin_audit_log",
            "admin_pusat",
            "superadmin",
        ] {
            assert!(
                !with_roles(&[lookalike]).can_administer_iam(),
                "{lookalike} is not the IAM admin role"
            );
        }
    }

    /// `roles` is the authority whenever the token carried any; the single
    /// `role` field is only a fallback for sessions persisted before `roles`
    /// existed.
    #[test]
    fn test_iam_console_falls_back_to_role_only_when_roles_is_empty() {
        let mut legacy_admin = create_test_session(3600);
        legacy_admin.role = UserRole::Admin;
        assert!(
            legacy_admin.can_administer_iam(),
            "a pre-`roles` admin session must not be locked out by the upgrade"
        );

        let mut legacy_user = create_test_session(3600);
        legacy_user.role = UserRole::User;
        assert!(!legacy_user.can_administer_iam());

        // Once the token names roles, the old field no longer decides: an
        // `Admin` role field next to roles without `admin` grants nothing.
        let mut stale = create_test_session(3600);
        stale.role = UserRole::Admin;
        stale.roles = vec!["validator_pusat".to_string()];
        assert!(
            !stale.can_administer_iam(),
            "the server-derived roles outrank a stale role field"
        );
    }

    /// Test session serialization and deserialization
    #[test]
    fn test_session_serialization() {
        let session = create_test_session(3600);

        // Serialize to JSON
        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("test@kejaksaan.go.id"));
        assert!(json.contains("Test User"));

        // Deserialize from JSON
        let deserialized: UserSession = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.username, session.username);
        assert_eq!(deserialized.email, session.email);
        assert_eq!(deserialized.role, session.role);
        assert_eq!(deserialized.mfa_enabled, session.mfa_enabled);
    }

    /// Test token response serialization
    #[test]
    fn test_token_response_serialization() {
        let token_response = TokenResponse {
            access_token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            refresh_token: Some("test_refresh".to_string()),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&token_response).unwrap();
        assert!(json.contains("test_token"));
        assert!(json.contains("Bearer"));

        // Deserialize from JSON
        let deserialized: TokenResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.access_token, token_response.access_token);
        assert_eq!(deserialized.token_type, token_response.token_type);
        assert_eq!(deserialized.expires_in, token_response.expires_in);
    }
}
