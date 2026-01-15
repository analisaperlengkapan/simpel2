//! Crypto Roundtrip Verification Tests for Secreton
//!
//! Tests encryption/decryption roundtrip

use anyhow::Result;
use lib_crypto::hybrid::{
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
    let dummy_public_key = vec![0u8; 32]; // Not used in classical mode

    let encrypted = crypto.encrypt(plaintext, &dummy_public_key)?;
    let decrypted = crypto.decrypt(&encrypted)?;

    assert_eq!(decrypted, plaintext);
    Ok(())
}

#[tokio::test]
async fn test_hybrid_crypto_roundtrip() -> Result<()> {
    // Sender and receiver with the same configuration
    let sender = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )?;

    let mut receiver = HybridCrypto::new(
        CryptoMode::Hybrid,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )?;

    // Receiver generates KEM keypair and exposes public key
    receiver.generate_kem_keypair()?;
    let receiver_keys = receiver.get_public_keys()?;
    let receiver_public_key = receiver_keys
        .pq_kem_public_key
        .expect("Receiver KEM public key should be present");

    let plaintext = b"hybrid test data";

    // Encrypt to receiver's public key
    let encrypted = sender.encrypt(plaintext, &receiver_public_key)?;
    let decrypted = receiver.decrypt(&encrypted)?;

    assert_eq!(decrypted, plaintext);
    Ok(())
}

#[tokio::test]
async fn test_post_quantum_crypto_roundtrip() -> Result<()> {
    // Sender and receiver in pure post-quantum mode
    let sender = HybridCrypto::new(
        CryptoMode::PostQuantum,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )?;

    let mut receiver = HybridCrypto::new(
        CryptoMode::PostQuantum,
        SecurityRequirements::default(),
        PerformancePriority::default(),
    )?;

    // Receiver generates KEM keypair and exposes public key
    receiver.generate_kem_keypair()?;
    let receiver_keys = receiver.get_public_keys()?;
    let receiver_public_key = receiver_keys
        .pq_kem_public_key
        .expect("Receiver KEM public key should be present");

    let plaintext = b"post-quantum test data";

    // Encrypt to receiver's public key
    let encrypted = sender.encrypt(plaintext, &receiver_public_key)?;
    let decrypted = receiver.decrypt(&encrypted)?;

    assert_eq!(decrypted, plaintext);
    Ok(())
}
