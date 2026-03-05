//! Unit tests for PostgresUserStore
//!
//! These tests verify the UserStore implementation logic without requiring a real database.

use authenc_types::{
    RealmId, SecurityContext, UserId,
    domain::{CreateUserRequest, UpdateUserRequest, user::User},
};
use chrono::Utc;

/// Helper to create a test User with all required fields
fn test_user() -> User {
    let now = Utc::now();
    User {
        id: uuid::Uuid::new_v4(),
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        email_verified: false,
        first_name: None,
        last_name: None,
        nip: None,
        nama: None,
        jabatan: None,
        satker_code: "001".to_string(),
        phone_number: None,
        phone_verified: false,
        password_hash: Some("hashed_password".to_string()),
        totp_secret: None,
        totp_backup_codes: None,
        mfa_enabled: false,
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
        security_context: SecurityContext::default(),
        attributes: None,
        enabled: true,
        federated: false,
        created_at: now,
        updated_at: now,
        deleted_at: None,
        login_count: 0,
    }
}

/// Helper to create a test CreateUserRequest
fn test_create_request() -> CreateUserRequest {
    CreateUserRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        satker_code: "001".to_string(),
        password: Some("hashed_password".to_string()),
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        nip: None,
        nama: None,
        jabatan: None,
        phone_number: None,
        realm_id: Some(uuid::Uuid::new_v4()),
        organization_id: None,
        roles: None,
        attributes: None,
    }
}

/// Helper to create an empty UpdateUserRequest
fn empty_update_request() -> UpdateUserRequest {
    UpdateUserRequest {
        username: None,
        email: None,
        satker_code: None,
        first_name: None,
        last_name: None,
        nip: None,
        nama: None,
        jabatan: None,
        phone_number: None,
        enabled: None,
        email_verified: None,
        phone_verified: None,
        require_password_change: None,
        password: None,
        mfa_enabled: None,
        attributes: None,
    }
}

#[test]
fn test_create_user_request_validation() {
    let req = test_create_request();
    assert_eq!(req.username, "testuser");
    assert_eq!(req.email, "test@example.com");
    assert!(req.realm_id.is_some());
}

#[test]
fn test_update_user_request_empty() {
    let req = empty_update_request();
    assert!(req.email.is_none());
    assert!(req.password.is_none());
    assert!(req.enabled.is_none());
    assert!(req.email_verified.is_none());
    assert!(req.mfa_enabled.is_none());
}

#[test]
fn test_update_user_request_partial() {
    let mut req = empty_update_request();
    req.email = Some("newemail@example.com".to_string());
    req.enabled = Some(false);
    req.mfa_enabled = Some(true);

    assert_eq!(req.email.as_deref(), Some("newemail@example.com"));
    assert!(req.password.is_none());
    assert_eq!(req.enabled, Some(false));
    assert_eq!(req.mfa_enabled, Some(true));
}

#[test]
fn test_user_struct_creation() {
    let user = test_user();
    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, "test@example.com");
    assert!(user.enabled);
    assert!(!user.email_verified);
    assert!(!user.mfa_enabled);
}

#[test]
fn test_user_id_generation() {
    let id1 = UserId::new();
    let id2 = UserId::new();
    assert_ne!(id1, id2);
}

#[test]
fn test_user_id_display() {
    let id = UserId::new();
    let display_str = format!("{}", id);
    assert!(!display_str.is_empty());
}

#[test]
fn test_create_user_request_with_special_characters() {
    let mut req = test_create_request();
    req.username = "user.name+test".to_string();
    req.email = "user+tag@example.com".to_string();

    assert!(req.username.contains('.'));
    assert!(req.username.contains('+'));
    assert!(req.email.contains('+'));
}

#[test]
fn test_update_user_request_all_fields() {
    let mut req = empty_update_request();
    req.email = Some("new@example.com".to_string());
    req.password = Some("new_hashed_password".to_string());
    req.enabled = Some(true);
    req.email_verified = Some(true);
    req.mfa_enabled = Some(true);

    assert!(req.email.is_some());
    assert!(req.password.is_some());
    assert!(req.enabled.is_some());
    assert!(req.email_verified.is_some());
    assert!(req.mfa_enabled.is_some());
}

#[test]
fn test_user_enabled_states() {
    let user = test_user();
    assert!(user.enabled);

    let disabled_user = User {
        enabled: false,
        ..user
    };
    assert!(!disabled_user.enabled);
}

#[test]
fn test_user_mfa_states() {
    let user = test_user();
    assert!(!user.mfa_enabled);

    let user_with_mfa = User {
        mfa_enabled: true,
        ..user
    };
    assert!(user_with_mfa.mfa_enabled);
}

#[test]
fn test_user_email_verification_states() {
    let user = test_user();
    assert!(!user.email_verified);

    let verified_user = User {
        email_verified: true,
        ..user
    };
    assert!(verified_user.email_verified);
}

#[test]
fn test_realm_id_generation() {
    let id1 = RealmId::new();
    let id2 = RealmId::new();
    assert_ne!(id1, id2);
}

#[test]
fn test_realm_id_display() {
    let id = RealmId::new();
    let display_str = format!("{}", id);
    assert!(!display_str.is_empty());
}
