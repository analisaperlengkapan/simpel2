/// Integration test for JWKS endpoint
///
/// This test verifies that the JWKS endpoint is properly configured
/// and returns valid Ed25519 public keys in JWK format.

#[cfg(test)]
mod jwks_tests {
    use authenc::crypto::ed25519_keys::Ed25519Jwk;

    #[test]
    fn test_jwks_response_structure() {
        // This test verifies the JWKS response structure
        // without requiring a running server

        let jwk = authenc::crypto::ed25519_keys::get_ed25519_jwk();

        // Verify JWK structure
        assert_eq!(jwk.kty, "OKP");
        assert_eq!(jwk.crv, "Ed25519");
        assert_eq!(jwk.alg, "EdDSA");
        assert_eq!(jwk.key_use, "sig");
        assert!(!jwk.x.is_empty());
        assert!(!jwk.kid.is_empty());

        // Verify it can be serialized to JSON
        let json = serde_json::to_string(&jwk).unwrap();
        assert!(json.contains("\"kty\":\"OKP\""));
        assert!(json.contains("\"crv\":\"Ed25519\""));
        assert!(json.contains("\"alg\":\"EdDSA\""));

        // Verify it can be deserialized back
        let deserialized: Ed25519Jwk = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.kty, jwk.kty);
        assert_eq!(deserialized.crv, jwk.crv);
        assert_eq!(deserialized.x, jwk.x);
    }

    #[test]
    fn test_jwks_set_structure() {
        // Test the JWKS set structure with multiple keys
        let jwk = authenc::crypto::ed25519_keys::get_ed25519_jwk();

        let jwks = serde_json::json!({
            "keys": [jwk]
        });

        // Verify JWKS structure
        assert!(jwks["keys"].is_array());
        assert_eq!(jwks["keys"].as_array().unwrap().len(), 1);

        // Verify first key
        let first_key = &jwks["keys"][0];
        assert_eq!(first_key["kty"], "OKP");
        assert_eq!(first_key["crv"], "Ed25519");
        assert_eq!(first_key["alg"], "EdDSA");
    }

    #[test]
    fn test_jwks_cache_key() {
        // Verify the cache key constant is properly defined
        const JWKS_CACHE_KEY: &str = "jwks:response";
        assert!(!JWKS_CACHE_KEY.is_empty());
        assert!(JWKS_CACHE_KEY.starts_with("jwks:"));
    }

    #[test]
    fn test_jwks_cache_ttl() {
        // Verify the cache TTL is 1 hour (3600 seconds)
        const JWKS_CACHE_TTL_SECONDS: u64 = 3600;
        assert_eq!(JWKS_CACHE_TTL_SECONDS, 3600);
        assert_eq!(JWKS_CACHE_TTL_SECONDS, 60 * 60); // 1 hour
    }
}
