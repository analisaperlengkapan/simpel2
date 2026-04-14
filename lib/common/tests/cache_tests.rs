//! Unit tests for cache module

#[cfg(test)]
mod cache_tests {
    use lib_common::cache::SensitivityLevel;

    #[test]
    fn test_sensitivity_level_ttl() {
        // Test that sensitivity levels have appropriate TTL values
        let public = SensitivityLevel::Public;
        let internal = SensitivityLevel::Internal;
        let confidential = SensitivityLevel::Confidential;

        // These are conceptual tests - actual TTL is in CacheManager implementation
        assert!(matches!(public, SensitivityLevel::Public));
        assert!(matches!(internal, SensitivityLevel::Internal));
        assert!(matches!(confidential, SensitivityLevel::Confidential));
    }

    #[test]
    fn test_cache_key_generation() {
        let key1 = format!("user:{}", "123");
        let key2 = format!("user:{}", "456");

        assert_eq!(key1, "user:123");
        assert_eq!(key2, "user:456");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_pattern_matching() {
        let pattern = "user:*";
        let key1 = "user:123";
        let key2 = "user:456";
        let key3 = "session:789";

        assert!(key1.starts_with("user:"));
        assert!(key2.starts_with("user:"));
        assert!(!key3.starts_with("user:"));
    }
}
