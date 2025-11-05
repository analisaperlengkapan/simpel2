//! Crypto Roundtrip Verification Tests for Secreton
//!
//! Tests encryption/decryption roundtrip

use anyhow::Result;
use secreton_crypto::hybrid::{
    CryptoMode, HybridCrypto, PerformancePriority, SecurityRequirements,
};

#[tokio::test]
async fn test_classical_crypto_roundtrip() -> Result<()> {
    let crypto = HybridCrypto::new(
        CryptoMode::Classical,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )?;

    let plaintext = b"test data for roundtrip";
    let aad = b"associated data";

    let encrypted = crypto.encrypt(plaintext, aad)?;
    let decrypted = crypto.decrypt(&encrypted)?;

    assert_eq!(decrypted, plaintext);
    Ok(())
}

#[tokio::test]
async fn test_hybrid_crypto_roundtrip() -> Result<()> {
    let crypto = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )?;

    let plaintext = b"hybrid test data";
    let aad = b"hybrid aad";

    let encrypted = crypto.encrypt(plaintext, aad)?;
    let decrypted = crypto.decrypt(&encrypted)?;

    assert_eq!(decrypted, plaintext);
    Ok(())
}

#[tokio::test]
async fn test_post_quantum_crypto_roundtrip() -> Result<()> {
    let crypto = HybridCrypto::new(
        CryptoMode::PostQuantum,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )?;

    let plaintext = b"post-quantum test data";
    let aad = b"pq aad";

    let encrypted = crypto.encrypt(plaintext, aad)?;
    let decrypted = crypto.decrypt(&encrypted)?;

    assert_eq!(decrypted, plaintext);
    Ok(())
}
