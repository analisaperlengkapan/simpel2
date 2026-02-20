//! Unit tests for domain types

#[cfg(test)]
mod tests {
    use super::super::domain::*;

    #[test]
    fn test_user_id_creation() {
        let id1 = UserId::new();
        let id2 = UserId::new();
        assert_ne!(id1, id2, "Each UserId should be unique");
    }

    #[test]
    fn test_user_id_display() {
        let id = UserId::new();
        let display = format!("{}", id);
        assert!(!display.is_empty());
        assert_eq!(display.len(), 36); // UUID string length
    }

    #[test]
    fn test_user_id_from_uuid() {
        let uuid = uuid::Uuid::new_v4();
        let user_id = UserId::from(uuid);
        assert_eq!(user_id.as_uuid(), &uuid);
    }

    #[test]
    fn test_realm_id_creation() {
        let id1 = RealmId::new();
        let id2 = RealmId::new();
        assert_ne!(id1, id2, "Each RealmId should be unique");
    }

    #[test]
    fn test_client_id_creation() {
        let id1 = ClientId::new();
        let id2 = ClientId::new();
        assert_ne!(id1, id2, "Each ClientId should be unique");
    }

    #[test]
    fn test_session_id_creation() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2, "Each SessionId should be unique");
    }

    #[test]
    fn test_auth_result_success_serialization() {
        let result = AuthResult::Success {
            user_id: UserId::new(),
            session_id: SessionId::new(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"type\":\"success\""));
    }

    #[test]
    fn test_auth_result_mfa_required_serialization() {
        let result = AuthResult::MfaRequired {
            user_id: UserId::new(),
            mfa_token: "test-token".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"type\":\"mfa_required\""));
        assert!(json.contains("\"mfa_token\":\"test-token\""));
    }

    #[test]
    fn test_auth_result_failed_serialization() {
        let result = AuthResult::Failed {
            reason: AuthFailureReason::InvalidCredentials,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"type\":\"failed\""));
    }

    #[test]
    fn test_auth_failure_reason_display() {
        assert_eq!(
            format!("{}", AuthFailureReason::InvalidCredentials),
            "Invalid credentials"
        );
        assert_eq!(
            format!("{}", AuthFailureReason::UserDisabled),
            "User account is disabled"
        );
        assert_eq!(
            format!("{}", AuthFailureReason::AccountLocked),
            "Account is locked"
        );
    }

    #[test]
    fn test_credentials_creation() {
        let creds = Credentials {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };

        assert_eq!(creds.username, "testuser");
        assert_eq!(creds.password, "testpass");
    }

    #[test]
    fn test_create_user_request() {
        let req = CreateUserRequest {
            username: "newuser".to_string(),
            email: "newuser@example.com".to_string(),
            password: "password123".to_string(),
            realm_id: RealmId::new(),
        };

        assert_eq!(req.username, "newuser");
        assert_eq!(req.email, "newuser@example.com");
    }

    #[test]
    fn test_update_user_request_partial() {
        let req = UpdateUserRequest {
            email: Some("newemail@example.com".to_string()),
            password: None,
            enabled: Some(false),
            email_verified: None,
            mfa_enabled: None,
        };

        assert_eq!(req.email, Some("newemail@example.com".to_string()));
        assert_eq!(req.password, None);
        assert_eq!(req.enabled, Some(false));
    }
}
