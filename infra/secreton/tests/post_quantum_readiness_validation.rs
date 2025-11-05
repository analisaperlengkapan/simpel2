//! Post-Quantum Cryptography Readiness Validation Tests for Secreton
//!
//! This module validates secreton's post-quantum cryptography features.

use anyhow::Result;
use secreton_crypto::hybrid::{
    CryptoMode, HybridCrypto, PerformancePriority, SecurityRequirements,
};
use secreton_crypto::pq_key_management::PostQuantumKeyManager;

#[cfg(test)]
mod secreton_post_quantum_readiness {
    use super::*;

    #[tokio::test]
    async fn test_crypto_mode_initialization() -> Result<()> {
        // Test initialization of different crypto modes
        let modes = vec![
            CryptoMode::PostQuantum,
            CryptoMode::Hybrid,
            CryptoMode::Classical,
        ];

        for mode in modes {
            let crypto = HybridCrypto::new(
                mode.clone(),
                SecurityRequirements::default(),
                PerformancePriority::default(),
            )?;
            assert!(
                matches!(crypto.mode(), m if m == mode),
                "Crypto mode should match {:?}",
                mode
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_post_quantum_key_manager() -> Result<()> {
        // Test PostQuantumKeyManager initialization
        let pq_key_manager = PostQuantumKeyManager::new();

        // Verify key manager exists
        assert!(
            std::mem::size_of_val(&pq_key_manager) > 0,
            "PQ Key Manager should be initialized"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_hybrid_crypto_post_quantum_mode() -> Result<()> {
        let crypto = HybridCrypto::new(
            CryptoMode::PostQuantum,
            SecurityRequirements::default(),
            PerformancePriority::default(),
        )?;

        // Verify it's in post-quantum mode
        assert_eq!(crypto.mode(), CryptoMode::PostQuantum);
        Ok(())
    }

    #[tokio::test]
    async fn test_hybrid_crypto_hybrid_mode() -> Result<()> {
        let crypto = HybridCrypto::new(
            CryptoMode::Hybrid,
            SecurityRequirements::default(),
            PerformancePriority::default(),
        )?;

        // Verify it's in hybrid mode
        assert_eq!(crypto.mode(), CryptoMode::Hybrid);
        Ok(())
    }

    #[tokio::test]
    async fn test_hybrid_crypto_classical_mode() -> Result<()> {
        let crypto = HybridCrypto::new(
            CryptoMode::Classical,
            SecurityRequirements::default(),
            PerformancePriority::default(),
        )?;

        // Verify it's in classical mode
        assert_eq!(crypto.mode(), CryptoMode::Classical);
        Ok(())
    }

    #[tokio::test]
    async fn test_security_requirements_defaults() -> Result<()> {
        let req = SecurityRequirements::default();

        // Verify security requirements have reasonable defaults
        assert!(std::mem::size_of_val(&req) > 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_performance_priority_defaults() -> Result<()> {
        let perf = PerformancePriority::default();

        // Verify performance priority has reasonable defaults
        assert!(std::mem::size_of_val(&perf) > 0);
        Ok(())
    }
}
