//! Integration tests for namespace management API

use secreton_core::namespace::{
    AdminLevel, JwtClaims, Namespace, NamespaceAccessControl, NamespaceHierarchy, NamespaceService,
    NamespaceType,
};
use std::collections::HashMap;

#[test]
fn test_namespace_hierarchy_creation() {
    let hierarchy = NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

    assert_eq!(hierarchy.root.id, "pusat");
    assert_eq!(hierarchy.root.name, "Kejaksaan Agung RI");
    assert_eq!(hierarchy.root.namespace_type, NamespaceType::Pusat);
}

#[test]
fn test_add_wilayah_namespace() {
    let mut hierarchy =
        NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

    let wilayah = hierarchy
        .add_wilayah(
            "wilayah-sumut".to_string(),
            "Kejaksaan Tinggi Sumatera Utara".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add wilayah");

    assert_eq!(wilayah.id, "wilayah-sumut");
    assert_eq!(wilayah.namespace_type, NamespaceType::Wilayah);
    assert_eq!(wilayah.parent, Some("pusat".to_string()));
    assert_eq!(hierarchy.wilayah_map.len(), 1);
}

#[test]
fn test_add_satker_namespace() {
    let mut hierarchy =
        NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

    hierarchy
        .add_wilayah(
            "wilayah-sumut".to_string(),
            "Kejaksaan Tinggi Sumatera Utara".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add wilayah");

    let satker = hierarchy
        .add_satker(
            "satker-kja001".to_string(),
            "Kejaksaan Negeri Medan".to_string(),
            "wilayah-sumut".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add satker");

    assert_eq!(satker.id, "satker-kja001");
    assert_eq!(satker.namespace_type, NamespaceType::Satker);
    assert_eq!(satker.parent, Some("wilayah-sumut".to_string()));
    assert_eq!(hierarchy.satker_map.len(), 1);
}

#[test]
fn test_namespace_access_control_pusat() {
    let mut hierarchy =
        NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

    hierarchy
        .add_wilayah(
            "wilayah-sumut".to_string(),
            "Kejaksaan Tinggi Sumatera Utara".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add wilayah");

    let access_control = NamespaceAccessControl::new(hierarchy.clone());

    let pusat_claims = JwtClaims {
        sub: "admin".to_string(),
        name: "Admin Pusat".to_string(),
        email: "admin@kejaksaan.go.id".to_string(),
        satker_code: None,
        wilayah_code: None,
        admin_level: AdminLevel::Pusat,
        roles: vec!["admin".to_string()],
        permissions: vec!["*".to_string()],
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        iss: "authenc".to_string(),
        metadata: HashMap::new(),
    };

    let accessible = access_control.get_accessible_namespaces(&pusat_claims);
    assert!(accessible.contains(&"pusat".to_string()));
    assert!(accessible.contains(&"wilayah-sumut".to_string()));
}

#[test]
fn test_namespace_access_control_wilayah() {
    let mut hierarchy =
        NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

    hierarchy
        .add_wilayah(
            "wilayah-sumut".to_string(),
            "Kejaksaan Tinggi Sumatera Utara".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add wilayah");

    hierarchy
        .add_satker(
            "satker-kja001".to_string(),
            "Kejaksaan Negeri Medan".to_string(),
            "wilayah-sumut".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add satker");

    let access_control = NamespaceAccessControl::new(hierarchy.clone());

    let wilayah_claims = JwtClaims {
        sub: "wilayah_admin".to_string(),
        name: "Admin Wilayah Sumut".to_string(),
        email: "admin.sumut@kejaksaan.go.id".to_string(),
        satker_code: None,
        wilayah_code: Some("SUMUT".to_string()),
        admin_level: AdminLevel::Wilayah,
        roles: vec!["admin".to_string()],
        permissions: vec!["namespace:*".to_string()],
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        iss: "authenc".to_string(),
        metadata: HashMap::new(),
    };

    let accessible = access_control.get_accessible_namespaces(&wilayah_claims);
    assert!(accessible.contains(&"wilayah-sumut".to_string()));
    assert!(accessible.contains(&"satker-kja001".to_string()));
    assert!(!accessible.contains(&"pusat".to_string()));
}

#[test]
fn test_namespace_access_control_satker() {
    let mut hierarchy =
        NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

    hierarchy
        .add_wilayah(
            "wilayah-sumut".to_string(),
            "Kejaksaan Tinggi Sumatera Utara".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add wilayah");

    hierarchy
        .add_satker(
            "satker-kja001".to_string(),
            "Kejaksaan Negeri Medan".to_string(),
            "wilayah-sumut".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add satker");

    let access_control = NamespaceAccessControl::new(hierarchy.clone());

    let satker_claims = JwtClaims {
        sub: "satker_admin".to_string(),
        name: "Admin Satker Medan".to_string(),
        email: "admin.medan@kejaksaan.go.id".to_string(),
        satker_code: Some("KJA001".to_string()),
        wilayah_code: Some("SUMUT".to_string()),
        admin_level: AdminLevel::Satker,
        roles: vec!["admin".to_string()],
        permissions: vec!["namespace:read".to_string()],
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        iss: "authenc".to_string(),
        metadata: HashMap::new(),
    };

    let accessible = access_control.get_accessible_namespaces(&satker_claims);
    assert!(accessible.contains(&"satker-kja001".to_string()));
    assert!(!accessible.contains(&"wilayah-sumut".to_string()));
    assert!(!accessible.contains(&"pusat".to_string()));
}

#[test]
fn test_namespace_service_thread_safety() {
    let service = NamespaceService::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

    // Add wilayah
    service.with_hierarchy_mut(|h| {
        h.add_wilayah(
            "wilayah-sumut".to_string(),
            "Kejaksaan Tinggi Sumatera Utara".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add wilayah");
    });

    // Verify
    let hierarchy = service.hierarchy();
    assert_eq!(hierarchy.wilayah_map.len(), 1);
    assert!(hierarchy.wilayah_map.contains_key("wilayah-sumut"));
}

#[test]
fn test_namespace_delete_with_children() {
    let mut hierarchy =
        NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

    hierarchy
        .add_wilayah(
            "wilayah-sumut".to_string(),
            "Kejaksaan Tinggi Sumatera Utara".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add wilayah");

    hierarchy
        .add_satker(
            "satker-kja001".to_string(),
            "Kejaksaan Negeri Medan".to_string(),
            "wilayah-sumut".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add satker");

    // Cannot delete wilayah with children
    let result = hierarchy.delete_namespace("wilayah-sumut");
    assert!(result.is_err());

    // Can delete satker (no children)
    let result = hierarchy.delete_namespace("satker-kja001");
    assert!(result.is_ok());
    assert_eq!(hierarchy.satker_map.len(), 0);

    // Now can delete wilayah
    let result = hierarchy.delete_namespace("wilayah-sumut");
    assert!(result.is_ok());
    assert_eq!(hierarchy.wilayah_map.len(), 0);
}

#[test]
fn test_namespace_count_descendants() {
    let mut hierarchy =
        NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

    hierarchy
        .add_wilayah(
            "wilayah-sumut".to_string(),
            "Kejaksaan Tinggi Sumatera Utara".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add wilayah");

    hierarchy
        .add_satker(
            "satker-kja001".to_string(),
            "Kejaksaan Negeri Medan".to_string(),
            "wilayah-sumut".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add satker");

    hierarchy
        .add_satker(
            "satker-kja002".to_string(),
            "Kejaksaan Negeri Binjai".to_string(),
            "wilayah-sumut".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add satker");

    // Pusat has 3 descendants (1 wilayah + 2 satker)
    assert_eq!(hierarchy.count_all_descendants("pusat"), 3);

    // Wilayah has 2 descendants (2 satker)
    assert_eq!(hierarchy.count_all_descendants("wilayah-sumut"), 2);

    // Satker has 0 descendants
    assert_eq!(hierarchy.count_all_descendants("satker-kja001"), 0);
}

#[test]
fn test_cross_namespace_access_denial() {
    let mut hierarchy =
        NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

    // Create two wilayah with satkers
    hierarchy
        .add_wilayah(
            "wilayah-sumut".to_string(),
            "Kejaksaan Tinggi Sumatera Utara".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add wilayah sumut");

    hierarchy
        .add_wilayah(
            "wilayah-jatim".to_string(),
            "Kejaksaan Tinggi Jawa Timur".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add wilayah jatim");

    hierarchy
        .add_satker(
            "satker-kja001".to_string(),
            "Kejaksaan Negeri Medan".to_string(),
            "wilayah-sumut".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add satker medan");

    hierarchy
        .add_satker(
            "satker-kja101".to_string(),
            "Kejaksaan Negeri Surabaya".to_string(),
            "wilayah-jatim".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add satker surabaya");

    let access_control = NamespaceAccessControl::new(hierarchy.clone());

    // Satker A (Medan) user
    let satker_a_claims = JwtClaims {
        sub: "user_medan".to_string(),
        name: "User Medan".to_string(),
        email: "user.medan@kejaksaan.go.id".to_string(),
        satker_code: Some("KJA001".to_string()),
        wilayah_code: Some("SUMUT".to_string()),
        admin_level: AdminLevel::Satker,
        roles: vec!["user".to_string()],
        permissions: vec!["namespace:read".to_string()],
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        iss: "authenc".to_string(),
        metadata: HashMap::new(),
    };

    let accessible_a = access_control.get_accessible_namespaces(&satker_a_claims);

    // Satker A can only access its own namespace
    assert!(accessible_a.contains(&"satker-kja001".to_string()));
    // Satker A cannot access Satker B
    assert!(!accessible_a.contains(&"satker-kja101".to_string()));
    // Satker A cannot access other wilayah
    assert!(!accessible_a.contains(&"wilayah-jatim".to_string()));
    // Satker A cannot access pusat
    assert!(!accessible_a.contains(&"pusat".to_string()));
}

#[test]
fn test_namespace_quota_enforcement() {
    let mut hierarchy =
        NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

    hierarchy
        .add_wilayah(
            "wilayah-sumut".to_string(),
            "Kejaksaan Tinggi Sumatera Utara".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add wilayah");

    hierarchy
        .add_satker(
            "satker-kja001".to_string(),
            "Kejaksaan Negeri Medan".to_string(),
            "wilayah-sumut".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add satker");

    // Get mutable reference to satker
    let satker = hierarchy
        .get_namespace_mut("satker-kja001")
        .expect("Satker not found");

    // Set low quota for testing
    satker.quotas.max_secrets = Some(10);
    satker.quotas.current_usage.secrets_count = 0;

    // Initially not exceeded
    assert!(!satker.is_quota_exceeded());

    // Increase usage to limit
    satker.quotas.current_usage.secrets_count = 10;
    assert!(satker.is_quota_exceeded());

    // Test storage quota
    satker.quotas.max_storage_bytes = Some(1024 * 1024); // 1 MB
    satker.quotas.current_usage.storage_bytes = 0;
    satker.quotas.current_usage.secrets_count = 5; // Below secret limit

    assert!(!satker.is_quota_exceeded());

    satker.quotas.current_usage.storage_bytes = 1024 * 1024; // At limit
    assert!(satker.is_quota_exceeded());

    // Test lease quota
    satker.quotas.max_leases = Some(100);
    satker.quotas.current_usage.leases_count = 0;
    satker.quotas.current_usage.storage_bytes = 512 * 1024; // Below storage limit
    satker.quotas.current_usage.secrets_count = 5; // Below secret limit

    assert!(!satker.is_quota_exceeded());

    satker.quotas.current_usage.leases_count = 100;
    assert!(satker.is_quota_exceeded());

    // Test policy quota
    satker.quotas.max_policies = Some(10);
    satker.quotas.current_usage.policies_count = 0;
    satker.quotas.current_usage.leases_count = 50; // Below lease limit
    satker.quotas.current_usage.storage_bytes = 512 * 1024; // Below storage limit
    satker.quotas.current_usage.secrets_count = 5; // Below secret limit

    assert!(!satker.is_quota_exceeded());

    satker.quotas.current_usage.policies_count = 10;
    assert!(satker.is_quota_exceeded());
}

#[test]
fn test_namespace_creation_with_invalid_parent() {
    let mut hierarchy =
        NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

    // Try to add satker without creating wilayah first
    let result = hierarchy.add_satker(
        "satker-kja001".to_string(),
        "Kejaksaan Negeri Medan".to_string(),
        "wilayah-nonexistent".to_string(),
        "admin".to_string(),
    );

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("wilayah namespace: wilayah-nonexistent")
    );

    // Try to add wilayah with invalid ID format
    let result = hierarchy.add_wilayah(
        "invalid-id".to_string(),
        "Invalid Wilayah".to_string(),
        "admin".to_string(),
    );

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Wilayah ID must start with 'wilayah-'")
    );

    // Try to add satker with invalid ID format
    hierarchy
        .add_wilayah(
            "wilayah-sumut".to_string(),
            "Kejaksaan Tinggi Sumatera Utara".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add wilayah");

    let result = hierarchy.add_satker(
        "invalid-id".to_string(),
        "Invalid Satker".to_string(),
        "wilayah-sumut".to_string(),
        "admin".to_string(),
    );

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Satker ID must start with 'satker-'")
    );
}

#[test]
fn test_jwt_claims_extraction_and_validation() {
    let hierarchy = NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

    let access_control = NamespaceAccessControl::new(hierarchy.clone());

    // Test valid Pusat claims
    let pusat_claims = JwtClaims {
        sub: "admin_pusat".to_string(),
        name: "Admin Pusat".to_string(),
        email: "admin@kejaksaan.go.id".to_string(),
        satker_code: None,
        wilayah_code: None,
        admin_level: AdminLevel::Pusat,
        roles: vec!["admin".to_string()],
        permissions: vec!["*".to_string()],
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        iss: "authenc".to_string(),
        metadata: HashMap::new(),
    };

    // Verify claims structure
    assert_eq!(pusat_claims.admin_level, AdminLevel::Pusat);
    assert!(pusat_claims.satker_code.is_none());
    assert!(pusat_claims.wilayah_code.is_none());
    assert!(pusat_claims.permissions.contains(&"*".to_string()));

    // Test valid Wilayah claims
    let wilayah_claims = JwtClaims {
        sub: "admin_wilayah".to_string(),
        name: "Admin Wilayah".to_string(),
        email: "admin.wilayah@kejaksaan.go.id".to_string(),
        satker_code: None,
        wilayah_code: Some("SUMUT".to_string()),
        admin_level: AdminLevel::Wilayah,
        roles: vec!["admin".to_string()],
        permissions: vec!["namespace:*".to_string()],
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        iss: "authenc".to_string(),
        metadata: HashMap::new(),
    };

    assert_eq!(wilayah_claims.admin_level, AdminLevel::Wilayah);
    assert!(wilayah_claims.satker_code.is_none());
    assert_eq!(wilayah_claims.wilayah_code, Some("SUMUT".to_string()));

    // Test valid Satker claims
    let satker_claims = JwtClaims {
        sub: "user_satker".to_string(),
        name: "User Satker".to_string(),
        email: "user.satker@kejaksaan.go.id".to_string(),
        satker_code: Some("KJA001".to_string()),
        wilayah_code: Some("SUMUT".to_string()),
        admin_level: AdminLevel::Satker,
        roles: vec!["user".to_string()],
        permissions: vec!["namespace:read".to_string()],
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        iss: "authenc".to_string(),
        metadata: HashMap::new(),
    };

    assert_eq!(satker_claims.admin_level, AdminLevel::Satker);
    assert_eq!(satker_claims.satker_code, Some("KJA001".to_string()));
    assert_eq!(satker_claims.wilayah_code, Some("SUMUT".to_string()));

    // Test token expiration validation
    let expired_claims = JwtClaims {
        sub: "expired_user".to_string(),
        name: "Expired User".to_string(),
        email: "expired@kejaksaan.go.id".to_string(),
        satker_code: None,
        wilayah_code: None,
        admin_level: AdminLevel::Satker,
        roles: vec!["user".to_string()],
        permissions: vec!["namespace:read".to_string()],
        exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp(), // Expired
        iat: (chrono::Utc::now() - chrono::Duration::hours(2)).timestamp(),
        iss: "authenc".to_string(),
        metadata: HashMap::new(),
    };

    // Verify expiration is in the past
    let now = chrono::Utc::now().timestamp();
    assert!(expired_claims.exp < now, "Token should be expired");

    // Test issuer validation
    assert_eq!(pusat_claims.iss, "authenc");
    assert_eq!(wilayah_claims.iss, "authenc");
    assert_eq!(satker_claims.iss, "authenc");

    // Test metadata extraction
    let mut metadata = HashMap::new();
    metadata.insert("department".to_string(), "IT".to_string());
    metadata.insert("location".to_string(), "Jakarta".to_string());

    let claims_with_metadata = JwtClaims {
        sub: "user_with_metadata".to_string(),
        name: "User With Metadata".to_string(),
        email: "user@kejaksaan.go.id".to_string(),
        satker_code: Some("KJA001".to_string()),
        wilayah_code: Some("SUMUT".to_string()),
        admin_level: AdminLevel::Satker,
        roles: vec!["user".to_string()],
        permissions: vec!["namespace:read".to_string()],
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        iss: "authenc".to_string(),
        metadata: metadata.clone(),
    };

    assert_eq!(claims_with_metadata.metadata.len(), 2);
    assert_eq!(
        claims_with_metadata.metadata.get("department"),
        Some(&"IT".to_string())
    );
    assert_eq!(
        claims_with_metadata.metadata.get("location"),
        Some(&"Jakarta".to_string())
    );
}

#[test]
fn test_namespace_quota_usage_percentage() {
    let mut hierarchy =
        NamespaceHierarchy::new("Kejaksaan Agung RI".to_string(), "admin".to_string());

    hierarchy
        .add_wilayah(
            "wilayah-sumut".to_string(),
            "Kejaksaan Tinggi Sumatera Utara".to_string(),
            "admin".to_string(),
        )
        .expect("Failed to add wilayah");

    let wilayah = hierarchy
        .get_namespace_mut("wilayah-sumut")
        .expect("Wilayah not found");

    // Set quotas
    wilayah.quotas.max_secrets = Some(100);
    wilayah.quotas.max_storage_bytes = Some(1024 * 1024); // 1 MB

    // Test 0% usage
    wilayah.quotas.current_usage.secrets_count = 0;
    wilayah.quotas.current_usage.storage_bytes = 0;
    assert_eq!(wilayah.get_quota_usage_percentage(), 0.0);

    // Test 50% usage
    wilayah.quotas.current_usage.secrets_count = 50;
    wilayah.quotas.current_usage.storage_bytes = 512 * 1024; // 0.5 MB
    let percentage = wilayah.get_quota_usage_percentage();
    assert!((percentage - 50.0).abs() < 0.1);

    // Test 100% usage
    wilayah.quotas.current_usage.secrets_count = 100;
    wilayah.quotas.current_usage.storage_bytes = 1024 * 1024; // 1 MB
    let percentage = wilayah.get_quota_usage_percentage();
    assert!((percentage - 100.0).abs() < 0.1);

    // Test over 100% usage
    wilayah.quotas.current_usage.secrets_count = 150;
    wilayah.quotas.current_usage.storage_bytes = 2 * 1024 * 1024; // 2 MB
    let percentage = wilayah.get_quota_usage_percentage();
    assert!(percentage > 100.0);
}
