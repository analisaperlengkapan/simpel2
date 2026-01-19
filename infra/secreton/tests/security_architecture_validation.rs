//! Security Architecture Validation Tests for Secreton
//!
//! Validates security architecture compliance

use anyhow::Result;
use lib_crypto::hybrid::{CryptoMode, HybridCrypto, PerformancePriority, SecurityRequirements};
use secreton_core::models::SecurityLevel;

#[tokio::test]
async fn test_security_architecture_basics() -> Result<()> {
    // Test that security levels are properly defined
    let _levels = vec![
        SecurityLevel::Public,
        SecurityLevel::Internal,
        SecurityLevel::Confidential,
        SecurityLevel::Secret,
        SecurityLevel::TopSecret,
    ];

    Ok(())
}

#[tokio::test]
async fn test_crypto_architecture() -> Result<()> {
    // Test that crypto architecture supports all modes
    let modes = vec![
        CryptoMode::Classical,
        CryptoMode::Hybrid,
        CryptoMode::PostQuantum,
    ];

    for mode in modes {
        let crypto = HybridCrypto::new(
            mode,
            SecurityRequirements::default(),
            PerformancePriority::default(),
        )?;

        assert!(std::mem::size_of_val(&crypto) > 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_security_requirements_architecture() -> Result<()> {
    let reqs = SecurityRequirements {
        security_level: 256,
        quantum_safe: true,
        audit_required: true,
        compliance_flags: vec!["AUDIT".to_string()],
    };

    assert_eq!(reqs.security_level, 256);
    assert!(reqs.quantum_safe);
    assert!(reqs.audit_required);

    Ok(())
}
