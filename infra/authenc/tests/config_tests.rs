use authenc::AppConfig;
use std::env;

#[test]
fn default_config_sane() {
    let cfg = AppConfig::default();
    assert!(cfg.server.port > 0);
    assert!(cfg.security.password_min_length >= 8);
}

#[test]
fn env_override_port() {
    // Use temp_env crate for safe environment variable manipulation in tests
    temp_env::with_var("PORT", Some("9090"), || {
        let cfg = AppConfig::from_env().unwrap();
        assert_eq!(cfg.server.port, 9090);
    });
}
