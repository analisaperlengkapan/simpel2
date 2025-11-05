//! Security Validation Tests for Secreton
//!
//! Tests security features and compliance

use anyhow::Result;
use secreton_core::models::SecurityLevel;
use secreton_crypto::hybrid::{
    CryptoMode, HybridCrypto, PerformancePriority, SecurityRequirements,
};

#[tokio::test]
async fn test_security_levels_exist() -> Result<()> {
    // Test that all security levels are accessible
    let levels = vec![
        SecurityLevel::Public,
        SecurityLevel::Internal,
        SecurityLevel::Confidential,
        SecurityLevel::Secret,
        SecurityLevel::TopSecret,
    ];

    assert_eq!(levels.len(), 5);
    Ok(())
}

#[tokio::test]
async fn test_crypto_with_high_security() -> Result<()> {
    let security_reqs = SecurityRequirements {
        security_level: 256,
        quantum_safe: true,
        audit_required: true,
        compliance_flags: vec!["HIGH_SECURITY".to_string()],
    };

    let crypto = HybridCrypto::new(
        CryptoMode::PostQuantum,
        security_reqs,
        PerformancePriority::Security,
    )?;

    assert_eq!(crypto.mode(), CryptoMode::PostQuantum);
    Ok(())
}

#[tokio::test]
async fn test_encryption_with_security_context() -> Result<()> {
    let security_reqs = SecurityRequirements {
        security_level: 192,
        quantum_safe: false,
        audit_required: true,
        compliance_flags: vec![],
    };

    let crypto = HybridCrypto::new(
        CryptoMode::Classical,
        security_reqs,
        PerformancePriority::Balanced,
    )?;

    let plaintext = b"sensitive data";
    let aad = b"security context";

    let encrypted = crypto.encrypt(plaintext, aad)?;
    let decrypted = crypto.decrypt(&encrypted)?;

    assert_eq!(decrypted, plaintext);
    Ok(())
}

#[tokio::test]
async fn test_hybrid_mode_security() -> Result<()> {
    let security_reqs = SecurityRequirements {
        security_level: 256,
        quantum_safe: true,
        audit_required: true,
        compliance_flags: vec!["HYBRID_MODE".to_string()],
    };

    let crypto = HybridCrypto::new(
        CryptoMode::Hybrid,
        security_reqs,
        PerformancePriority::Balanced,
    )?;

    assert_eq!(crypto.mode(), CryptoMode::Hybrid);
    Ok(())
}
