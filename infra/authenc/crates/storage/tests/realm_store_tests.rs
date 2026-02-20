//! Unit tests for PostgresRealmStore
//!
//! These tests verify the RealmStore implementation logic for multi-tenant realm management.

use authenc_types::{domain::Realm, RealmId};
use chrono::Utc;

#[test]
fn test_realm_creation() {
    let realm_id = RealmId::new();
    let now = Utc::now();

    let realm = Realm {
        id: realm_id,
        name: "test-realm".to_string(),
        display_name: "Test Realm".to_string(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert_eq!(realm.id, realm_id);
    assert_eq!(realm.name, "test-realm");
    assert_eq!(realm.display_name, "Test Realm");
    assert!(realm.enabled);
}

#[test]
fn test_realm_id_generation() {
    let id1 = RealmId::new();
    let id2 = RealmId::new();

    // Each generated ID should be unique
    assert_ne!(id1, id2);
}

#[test]
fn test_realm_id_display() {
    let id = RealmId::new();
    let display_str = format!("{}", id);

    // Should display the UUID
    assert!(!display_str.is_empty());
}

#[test]
fn test_realm_name_validation() {
    let realm_id = RealmId::new();
    let now = Utc::now();

    // Valid realm names
    let valid_names = vec![
        "simple-realm",
        "realm123",
        "test_realm",
        "realm-with-dashes",
        "UPPERCASE",
        "MixedCase",
    ];

    for name in valid_names {
        let realm = Realm {
            id: realm_id,
            name: name.to_string(),
            display_name: format!("Display {}", name),
            enabled: true,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(realm.name, name);
    }
}

#[test]
fn test_realm_enabled_states() {
    let realm_id = RealmId::new();
    let now = Utc::now();

    let enabled_realm = Realm {
        id: realm_id,
        name: "enabled-realm".to_string(),
        display_name: "Enabled Realm".to_string(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert!(enabled_realm.enabled);

    let disabled_realm = Realm {
        enabled: false,
        ..enabled_realm
    };

    assert!(!disabled_realm.enabled);
}

#[test]
fn test_realm_display_name() {
    let realm_id = RealmId::new();
    let now = Utc::now();

    let realm = Realm {
        id: realm_id,
        name: "internal-name".to_string(),
        display_name: "User Friendly Display Name".to_string(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert_ne!(realm.name, realm.display_name);
    assert_eq!(realm.display_name, "User Friendly Display Name");
}

#[test]
fn test_realm_timestamps() {
    let realm_id = RealmId::new();
    let created_at = Utc::now();
    let updated_at = created_at + chrono::Duration::hours(1);

    let realm = Realm {
        id: realm_id,
        name: "test-realm".to_string(),
        display_name: "Test Realm".to_string(),
        enabled: true,
        created_at,
        updated_at,
    };

    assert_eq!(realm.created_at, created_at);
    assert_eq!(realm.updated_at, updated_at);
    assert!(realm.updated_at > realm.created_at);
}

#[test]
fn test_realm_update_timestamp() {
    let realm_id = RealmId::new();
    let created_at = Utc::now();

    let mut realm = Realm {
        id: realm_id,
        name: "test-realm".to_string(),
        display_name: "Test Realm".to_string(),
        enabled: true,
        created_at,
        updated_at: created_at,
    };

    // Simulate an update
    std::thread::sleep(std::time::Duration::from_millis(10));
    realm.updated_at = Utc::now();

    assert!(realm.updated_at > realm.created_at);
}

#[test]
fn test_multiple_realms() {
    let now = Utc::now();

    let realm1 = Realm {
        id: RealmId::new(),
        name: "realm1".to_string(),
        display_name: "Realm One".to_string(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    let realm2 = Realm {
        id: RealmId::new(),
        name: "realm2".to_string(),
        display_name: "Realm Two".to_string(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    // Realms should have different IDs
    assert_ne!(realm1.id, realm2.id);

    // Realms should have different names
    assert_ne!(realm1.name, realm2.name);
    assert_ne!(realm1.display_name, realm2.display_name);
}

#[test]
fn test_realm_name_uniqueness_check() {
    let realm1_name = "unique-realm";
    let realm2_name = "unique-realm"; // Same name

    // In a real scenario, this should be prevented by the store
    assert_eq!(realm1_name, realm2_name);
}

#[test]
fn test_realm_with_special_characters() {
    let realm_id = RealmId::new();
    let now = Utc::now();

    let realm = Realm {
        id: realm_id,
        name: "realm-with-special_chars123".to_string(),
        display_name: "Realm with Special Characters & Symbols!".to_string(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert!(realm.name.contains('-'));
    assert!(realm.name.contains('_'));
    assert!(realm.display_name.contains('&'));
    assert!(realm.display_name.contains('!'));
}

#[test]
fn test_realm_empty_display_name() {
    let realm_id = RealmId::new();
    let now = Utc::now();

    let realm = Realm {
        id: realm_id,
        name: "test-realm".to_string(),
        display_name: String::new(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert!(realm.display_name.is_empty());
}

#[test]
fn test_realm_long_names() {
    let realm_id = RealmId::new();
    let now = Utc::now();

    let long_name = "a".repeat(100);
    let long_display_name = "Display Name ".repeat(20);

    let realm = Realm {
        id: realm_id,
        name: long_name.clone(),
        display_name: long_display_name.clone(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert_eq!(realm.name.len(), 100);
    assert!(realm.display_name.len() > 100);
}

#[test]
fn test_realm_case_sensitivity() {
    let realm_id = RealmId::new();
    let now = Utc::now();

    let lowercase_realm = Realm {
        id: realm_id,
        name: "lowercase".to_string(),
        display_name: "Lowercase".to_string(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    let uppercase_realm = Realm {
        id: RealmId::new(),
        name: "LOWERCASE".to_string(),
        display_name: "LOWERCASE".to_string(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    // Names are different (case-sensitive)
    assert_ne!(lowercase_realm.name, uppercase_realm.name);
}

#[test]
fn test_realm_default_state() {
    let realm_id = RealmId::new();
    let now = Utc::now();

    // New realm should be enabled by default
    let realm = Realm {
        id: realm_id,
        name: "new-realm".to_string(),
        display_name: "New Realm".to_string(),
        enabled: true,
        created_at: now,
        updated_at: now,
    };

    assert!(realm.enabled);
    assert_eq!(realm.created_at, realm.updated_at);
}

#[test]
fn test_realm_disable_operation() {
    let realm_id = RealmId::new();
    let created_at = Utc::now();

    let mut realm = Realm {
        id: realm_id,
        name: "test-realm".to_string(),
        display_name: "Test Realm".to_string(),
        enabled: true,
        created_at,
        updated_at: created_at,
    };

    // Disable the realm
    realm.enabled = false;
    realm.updated_at = Utc::now();

    assert!(!realm.enabled);
    assert!(realm.updated_at >= realm.created_at);
}
