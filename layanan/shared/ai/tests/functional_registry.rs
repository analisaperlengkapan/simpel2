use layanan_ai::models::{ModelRegistry, ModelMetadata};
use once_cell::sync::Lazy;

static REGISTRY: Lazy<ModelRegistry> = Lazy::new(|| ModelRegistry::new());

#[test]
fn test_registry_logic() {
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

    REGISTRY.add_model(meta);

    let m = REGISTRY.get_model(id).unwrap();
    assert_eq!(m.status, "draft");
    assert_eq!(m.approved_by, None);

    let approved = REGISTRY.approve_model(id, "admin");
    assert!(approved);

    let m2 = REGISTRY.get_model(id).unwrap();
    assert_eq!(m2.status, "approved");
    assert_eq!(m2.approved_by, Some("admin".to_string()));

    let not_found = REGISTRY.approve_model("non-existent", "admin");
    assert!(!not_found);
}
