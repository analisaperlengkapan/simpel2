use layanan_ai::models::{ModelMetadata, ModelRegistry};

#[test]
fn test_registry_logic() {
    let mut registry = ModelRegistry::new();
    let id = "model-1";
    let meta = ModelMetadata {
        id: id.to_string(),
        name: "test".to_string(),
        version: "1.0".to_string(),
        path: "path".to_string(),
        status: "draft".to_string(),
        created_at: chrono::Utc::now(),
        approved_by: None,
    };

    registry.add_model(meta);

    let m = registry.get_model(id).unwrap();
    assert_eq!(m.status, "draft");
    assert_eq!(m.approved_by, None);

    let approved = registry.approve_model(id, "admin");
    assert!(approved);

    let m2 = registry.get_model(id).unwrap();
    assert_eq!(m2.status, "approved");
    assert_eq!(m2.approved_by, Some("admin".to_string()));

    let not_found = registry.approve_model("non-existent", "admin");
    assert!(!not_found);
}
