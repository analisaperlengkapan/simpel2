//! Post-Quantum Key Management Integration Tests
//!
//! Tests for ML-KEM key encapsulation, ML-DSA signatures, and hybrid key exchange
//! between authenc and secreton services.

use authenc::crypto::enhanced::{
    AdminLevel, EnhancedCryptoEngine, PegawaiClaims, PostQuantumMode, SecretonPermissions,
    SignerInfo,
};
use authenc::models::user::UserClaims;
use authenc::secreton_client::secreton_client::{PqAlgorithm, SecretonClient};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_mldsa_signature_support_in_authenc() {
    // Test ML-DSA signature generation in enhanced crypto engine
    let mut engine = EnhancedCryptoEngine::new();
    engine.set_pq_mode(PostQuantumMode::PostQuantum);

    let data = b"Authentication token data for SIMKARI";
    let signer = SignerInfo {
        user_id: Uuid::new_v4(),
        nip: "198501012010011001".to_string(),
        name: "Test Admin".to_string(),
        role: "admin".to_string(),
        satker_code: "A.01.01".to_string(),
        admin_level: Some(AdminLevel::AdminPusat),
    };

    // Generate audit signature with ML-DSA (if quantum feature is enabled)
    let signature_result = engine.generate_audit_signature(data, signer.clone()).await;

    #[cfg(feature = "quantum")]
    {
        assert!(signature_result.is_ok());
        let signature = signature_result.unwrap();
        assert!(
            signature.algorithm == "ML-DSA" || signature.algorithm == "Ed25519",
            "Expected ML-DSA or Ed25519 algorithm"
        );

        // Verify the signature
        let verification = engine.verify_audit_signature(data, &signature).await;
        assert!(verification.is_ok());
        assert!(verification.unwrap());
    }

    #[cfg(not(feature = "quantum"))]
    {
        // Without quantum feature, should fall back to classical
        assert!(signature_result.is_ok());
        let signature = signature_result.unwrap();
        assert_eq!(signature.algorithm, "Ed25519");
    }
}

#[tokio::test]
async fn test_hybrid_mode_signatures() {
    // Test hybrid mode (Ed25519 + ML-DSA)
    let mut engine = EnhancedCryptoEngine::new();
    engine.set_pq_mode(PostQuantumMode::Hybrid);

    let data = b"Hybrid signature test data";
    let signer = SignerInfo {
        user_id: Uuid::new_v4(),
        nip: "198501012010011001".to_string(),
        name: "Test Jaksa".to_string(),
        role: "jaksa".to_string(),
        satker_code: "A.01.01".to_string(),
        admin_level: None,
    };

    let signature_result = engine.generate_audit_signature(data, signer).await;

    #[cfg(feature = "quantum")]
    {
        assert!(signature_result.is_ok());
        let signature = signature_result.unwrap();
        assert_eq!(signature.algorithm, "Ed25519+ML-DSA");
        assert!(signature.pq_signature.is_some());

        // Verify hybrid signature
        let verification = engine.verify_audit_signature(data, &signature).await;
        assert!(verification.is_ok());
        assert!(verification.unwrap());
    }

    #[cfg(not(feature = "quantum"))]
    {
        // Without quantum feature, should fall back to classical
        assert!(signature_result.is_ok());
        let signature = signature_result.unwrap();
        assert_eq!(signature.algorithm, "Ed25519");
    }
}

#[tokio::test]
async fn test_pegawai_jwt_with_pq_mode() {
    // Test JWT signing with different PQ modes
    for mode in [
        PostQuantumMode::Classical,
        PostQuantumMode::Hybrid,
        PostQuantumMode::PostQuantum,
    ] {
        let mut engine = EnhancedCryptoEngine::new();
        engine.set_pq_mode(mode);

        let claims = PegawaiClaims {
            standard: UserClaims {
                sub: "user123".to_string(),
                username: "test.pegawai".to_string(),
                email: "test@kejaksaan.go.id".to_string(),
                realm_id: "kejaksaan".to_string(),
                roles: vec!["pegawai".to_string()],
                exp: (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                iat: Utc::now().timestamp() as usize,
                iss: "authenc".to_string(),
            },
            nip: "198501012010011001".to_string(),
            satker_code: "A.01.01".to_string(),
            jabatan: "Jaksa Muda".to_string(),
            eselon: Some("IV/a".to_string()),
            wilayah_code: Some("DKI".to_string()),
            admin_level: None,
            secreton_permissions: SecretonPermissions {
                read_secrets: vec!["config/*".to_string()],
                write_secrets: vec![],
                admin_operations: false,
                audit_access: false,
                satker_permissions: HashMap::new(),
            },
        };

        let jwt_result = engine.sign_jwt_for_pegawai(&claims).await;
        assert!(jwt_result.is_ok(), "JWT signing failed for mode {:?}", mode);

        let jwt = jwt_result.unwrap();
        assert!(!jwt.is_empty());
        assert!(jwt.contains('.'));
    }
}

#[tokio::test]
async fn test_secreton_client_pq_methods() {
    // Test SecretonClient post-quantum methods (mock server would be needed for full test)
    let client = SecretonClient::new(
        "https://secreton.test.local".to_string(),
        "test-token".to_string(),
    );

    // Test that methods are available and have correct signatures
    let key_id = "test-pq-key";

    // Test get_post_quantum_key method exists
    let pq_key_result = client
        .get_post_quantum_key(key_id, PqAlgorithm::MlDsa)
        .await;
    // Will fail without real server, but method should exist
    assert!(pq_key_result.is_err());

    // Test hybrid_key_exchange method exists
    let x25519_private = [0u8; 32];
    let hybrid_result = client
        .hybrid_key_exchange(key_id, &x25519_private, None)
        .await;
    // Will fail without real server, but method should exist
    assert!(hybrid_result.is_err());

    // Test mlkem_encapsulate_key method exists
    let symmetric_key = [0u8; 32];
    let encap_result = client.mlkem_encapsulate_key(key_id, &symmetric_key).await;
    // Will fail without real server, but method should exist
    assert!(encap_result.is_err());

    // Test mldsa_sign_token method exists
    let token_data = b"test token data";
    let sign_result = client.mldsa_sign_token(key_id, token_data).await;
    // Will fail without real server, but method should exist
    assert!(sign_result.is_err());
}

#[tokio::test]
async fn test_performance_metrics_with_pq_operations() {
    let mut engine = EnhancedCryptoEngine::new();
    engine.set_pq_mode(PostQuantumMode::Hybrid);

    // Perform multiple operations to generate metrics
    let data = b"Performance test data";
    let signer = SignerInfo {
        user_id: Uuid::new_v4(),
        nip: "198501012010011001".to_string(),
        name: "Test Admin".to_string(),
        role: "admin".to_string(),
        satker_code: "A.01.01".to_string(),
        admin_level: Some(AdminLevel::AdminPusat),
    };

    for _ in 0..5 {
        let _ = engine.generate_audit_signature(data, signer.clone()).await;
    }

    let metrics = engine.get_metrics().await;
    assert_eq!(metrics.audit_signature.count, 5);
    assert!(metrics.audit_signature.avg_time_ms > 0.0);
    assert_eq!(metrics.audit_signature.success_rate(), 100.0);
}

#[tokio::test]
async fn test_session_encryption_with_pq_context() {
    let engine = EnhancedCryptoEngine::with_pq_mode(PostQuantumMode::Hybrid);

    let session_data = serde_json::json!({
        "user_id": "123",
        "pq_mode": "hybrid",
        "security_level": "high"
    });

    let metadata = authenc::crypto::enhanced::SessionMetadata {
        session_id: "sess_pq_test".to_string(),
        user_id: Uuid::new_v4(),
        client_ip: Some("192.168.1.1".to_string()),
        user_agent: Some("SIMKARI/1.0".to_string()),
        created_at: Utc::now(),
        last_activity: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::hours(1),
    };

    let encrypted = engine
        .encrypt_session_data(&session_data, metadata.clone())
        .await;
    assert!(encrypted.is_ok());

    let encrypted_session = encrypted.unwrap();
    assert_eq!(encrypted_session.algorithm, "AES-256-GCM");

    let decrypted = engine.decrypt_session_data(&encrypted_session).await;
    assert!(decrypted.is_ok());
    assert_eq!(decrypted.unwrap(), session_data);
}

#[test]
fn test_pq_algorithm_enum() {
    // Test PqAlgorithm enum variants
    let algorithms = vec![
        PqAlgorithm::MlDsa,
        PqAlgorithm::MlKem,
        PqAlgorithm::SphincsPlusShake256,
        PqAlgorithm::Hybrid(Box::new(PqAlgorithm::MlDsa)),
    ];

    for algo in algorithms {
        // Verify enum can be cloned and serialized
        let _cloned = algo.clone();
        let serialized = serde_json::to_string(&algo);
        assert!(serialized.is_ok());
    }
}

#[tokio::test]
async fn test_batch_validation_with_pq_tokens() {
    use authenc::crypto::enhanced::BatchValidationRequest;

    let engine = EnhancedCryptoEngine::with_pq_mode(PostQuantumMode::Hybrid);

    // Create test tokens
    let mut tokens = Vec::new();
    for i in 0..3 {
        let claims = PegawaiClaims {
            standard: UserClaims {
                sub: format!("user{}", i),
                username: format!("test.user{}", i),
                email: format!("test{}@kejaksaan.go.id", i),
                realm_id: "kejaksaan".to_string(),
                roles: vec!["pegawai".to_string()],
                exp: (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
                iat: Utc::now().timestamp() as usize,
                iss: "authenc".to_string(),
            },
            nip: format!("19850101201001100{}", i),
            satker_code: "A.01.01".to_string(),
            jabatan: "Jaksa Muda".to_string(),
            eselon: None,
            wilayah_code: None,
            admin_level: None,
            secreton_permissions: SecretonPermissions {
                read_secrets: vec![],
                write_secrets: vec![],
                admin_operations: false,
                audit_access: false,
                satker_permissions: HashMap::new(),
            },
        };

        if let Ok(token) = engine.sign_jwt_for_pegawai(&claims).await {
            tokens.push(token);
        }
    }

    let request = BatchValidationRequest {
        tokens,
        use_cache: true,
        max_cache_age: Some(300),
    };

    let response = engine.validate_tokens_batch(request).await;
    assert!(response.is_ok());

    let batch_response = response.unwrap();
    assert_eq!(batch_response.results.len(), 3);
    assert!(batch_response.processing_time_ms > 0);
}
