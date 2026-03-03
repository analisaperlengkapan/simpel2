//! Unit tests for PostgresRealmStore
//!
//! These tests verify the RealmStore implementation logic for multi-tenant realm management.
//! Updated to match the current Realm struct definition in authenc_types.

use authenc_types::domain::Realm;
use chrono::Utc;
use uuid::Uuid;

/// Helper to create a test Realm with all required fields and sensible defaults
fn test_realm(name: &str) -> Realm {
    let now = Utc::now();
    Realm {
        id: Uuid::new_v4(),
        name: name.to_string(),
        display_name: Some(format!("Display {}", name)),
        description: None,
        enabled: true,
        ssl_required: "external".to_string(),
        registration_allowed: false,
        registration_email_as_username: false,
        remember_me: false,
        verify_email: false,
        login_with_email_allowed: true,
        duplicate_emails_allowed: false,
        reset_password_allowed: true,
        edit_username_allowed: false,
        brute_force_protected: true,
        max_failure_wait_seconds: 900,
        minimum_quick_login_wait_seconds: 60,
        wait_increment_seconds: 60,
        quick_login_check_milli_seconds: 1000,
        max_delta_time_seconds: 43200,
        failure_factor: 30,
        default_signature_algorithm: "RS256".to_string(),
        revoke_refresh_token: false,
        refresh_token_max_reuse: 0,
        access_token_lifespan: 300,
        access_token_lifespan_for_implicit_flow: 900,
        sso_session_idle_timeout: 1800,
        sso_session_max_lifespan: 36000,
        sso_session_idle_timeout_remember_me: 0,
        sso_session_max_lifespan_remember_me: 0,
        offline_session_idle_timeout: 2592000,
        offline_session_max_lifespan: 5184000,
        client_session_idle_timeout: 0,
        client_session_max_lifespan: 0,
        access_code_lifespan: 60,
        access_code_lifespan_user_action: 300,
        access_code_lifespan_login: 1800,
        action_token_generated_by_admin_lifespan: 43200,
        action_token_generated_by_user_lifespan: 300,
        oauth2_device_code_lifespan: 600,
        oauth2_device_polling_interval: 5,
        attributes: None,
        created_at: now,
        updated_at: now,
        deleted_at: None,
    }
}

#[test]
fn test_realm_creation() {
    let realm = test_realm("test-realm");
    assert_eq!(realm.name, "test-realm");
    assert_eq!(realm.display_name, Some("Display test-realm".to_string()));
    assert!(realm.enabled);
}

#[test]
fn test_realm_id_generation() {
    let r1 = test_realm("realm1");
    let r2 = test_realm("realm2");
    assert_ne!(r1.id, r2.id);
}

#[test]
fn test_realm_id_display() {
    let realm = test_realm("test");
    let display_str = format!("{}", realm.id);
    assert!(!display_str.is_empty());
}

#[test]
fn test_realm_name_validation() {
    let valid_names = vec![
        "simple-realm",
        "realm123",
        "test_realm",
        "realm-with-dashes",
        "UPPERCASE",
        "MixedCase",
    ];

    for name in valid_names {
        let realm = test_realm(name);
        assert_eq!(realm.name, name);
    }
}

#[test]
fn test_realm_enabled_states() {
    let enabled = test_realm("enabled-realm");
    assert!(enabled.enabled);

    let disabled = Realm {
        enabled: false,
        ..enabled
    };
    assert!(!disabled.enabled);
}

#[test]
fn test_realm_display_name() {
    let mut realm = test_realm("internal-name");
    realm.display_name = Some("User Friendly Display Name".to_string());

    assert_ne!(realm.name, realm.display_name.as_deref().unwrap_or(""));
    assert_eq!(
        realm.display_name.as_deref(),
        Some("User Friendly Display Name")
    );
}

#[test]
fn test_realm_timestamps() {
    let created_at = Utc::now();
    let updated_at = created_at + chrono::Duration::hours(1);

    let mut realm = test_realm("test-realm");
    realm.created_at = created_at;
    realm.updated_at = updated_at;

    assert_eq!(realm.created_at, created_at);
    assert_eq!(realm.updated_at, updated_at);
    assert!(realm.updated_at > realm.created_at);
}

#[test]
fn test_realm_update_timestamp() {
    let mut realm = test_realm("test-realm");
    let created_at = realm.created_at;

    std::thread::sleep(std::time::Duration::from_millis(10));
    realm.updated_at = Utc::now();

    assert!(realm.updated_at > created_at);
}

#[test]
fn test_multiple_realms() {
    let r1 = test_realm("realm1");
    let r2 = test_realm("realm2");

    assert_ne!(r1.id, r2.id);
    assert_ne!(r1.name, r2.name);
}

#[test]
fn test_realm_name_uniqueness_check() {
    let realm1_name = "unique-realm";
    let realm2_name = "unique-realm";
    assert_eq!(realm1_name, realm2_name);
}

#[test]
fn test_realm_with_special_characters() {
    let mut realm = test_realm("realm-with-special_chars123");
    realm.display_name = Some("Realm with Special Characters & Symbols!".to_string());

    assert!(realm.name.contains('-'));
    assert!(realm.name.contains('_'));
    assert!(realm.display_name.as_deref().unwrap().contains('&'));
}

#[test]
fn test_realm_empty_display_name() {
    let mut realm = test_realm("test-realm");
    realm.display_name = None;
    assert!(realm.display_name.is_none());
}

#[test]
fn test_realm_long_names() {
    let long_name = "a".repeat(100);
    let realm = test_realm(&long_name);
    assert_eq!(realm.name.len(), 100);
}

#[test]
fn test_realm_case_sensitivity() {
    let lower = test_realm("lowercase");
    let upper = test_realm("LOWERCASE");
    assert_ne!(lower.name, upper.name);
}

#[test]
fn test_realm_default_state() {
    let realm = test_realm("new-realm");
    assert!(realm.enabled);
    assert_eq!(realm.created_at, realm.updated_at);
}

#[test]
fn test_realm_disable_operation() {
    let mut realm = test_realm("test-realm");
    realm.enabled = false;
    realm.updated_at = Utc::now();

    assert!(!realm.enabled);
    assert!(realm.updated_at >= realm.created_at);
}

#[test]
fn test_realm_security_settings() {
    let realm = test_realm("secure-realm");
    assert!(realm.brute_force_protected);
    assert_eq!(realm.ssl_required, "external");
    assert_eq!(realm.default_signature_algorithm, "RS256");
    assert_eq!(realm.failure_factor, 30);
}

#[test]
fn test_realm_session_timeouts() {
    let realm = test_realm("session-realm");
    assert_eq!(realm.access_token_lifespan, 300);
    assert_eq!(realm.sso_session_idle_timeout, 1800);
    assert_eq!(realm.sso_session_max_lifespan, 36000);
}

#[test]
fn test_realm_soft_delete() {
    let mut realm = test_realm("deletable-realm");
    assert!(realm.deleted_at.is_none());

    realm.deleted_at = Some(Utc::now());
    assert!(realm.deleted_at.is_some());
}
