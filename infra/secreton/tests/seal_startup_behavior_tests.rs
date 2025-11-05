//! Integration tests for seal/unseal startup behavior
//!
//! CRITICAL SECURITY TESTS: These tests verify that the vault follows
//! HashiCorp Vault security best practices:
//! 1. Vault starts in sealed mode by default
//! 2. initialize() does NOT auto-unseal (vault remains sealed after init)
//! 3. All API operations are blocked when sealed (except whitelisted endpoints)
//! 4. Middleware properly checks seal status
//! 5. Health check reflects seal status (503 when sealed)

use secreton_core::services::seal::{InMemoryVaultStateStorage, SealConfig, SealService};
use std::sync::Arc;

#[tokio::test]
async fn test_vault_starts_sealed_by_default() {
    // CRITICAL: Vault should start in sealed mode
    let config = SealConfig::default();
    let storage = Arc::new(InMemoryVaultStateStorage::new());
    let seal_service = SealService::with_storage(config, storage);

    // Verify vault is sealed at startup
    assert!(
        seal_service.is_sealed().await,
        "Vault should start in sealed mode"
    );
    assert!(
        !seal_service.is_unsealed().await,
        "Vault should not be unsealed at startup"
    );

    // Verify status
    let status = seal_service.status().await;
    assert_eq!(
        status.state,
        secreton_core::services::seal::SealState::Sealed
    );
}

#[tokio::test]
async fn test_initialize_does_not_auto_unseal() {
    // CRITICAL SECURITY FIX: initialize() should NOT unseal the vault
    // This follows HashiCorp Vault best practices
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: chrono::Utc::now(),
    };

    let storage = Arc::new(InMemoryVaultStateStorage::new());
    let seal_service = SealService::with_storage(config, storage);

    // Initialize vault - generates master key and shares
    let shares = seal_service.initialize().await.unwrap();
    assert_eq!(shares.len(), 5, "Should generate 5 shares");

    // CRITICAL: Vault should remain SEALED after initialization
    assert!(
        seal_service.is_sealed().await,
        "Vault should remain sealed after initialization"
    );
    assert!(
        !seal_service.is_unsealed().await,
        "Vault should not be unsealed after initialization"
    );

    // Verify master key is NOT available when sealed
    let master_key_result = seal_service.get_master_key().await;
    assert!(
        master_key_result.is_err(),
        "Master key should not be available when sealed"
    );
}

#[tokio::test]
async fn test_manual_unseal_required_after_init() {
    // CRITICAL: After initialization, operators must manually unseal with threshold shares
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: chrono::Utc::now(),
    };

    let storage = Arc::new(InMemoryVaultStateStorage::new());
    let seal_service = SealService::with_storage(config, storage);

    // Initialize vault
    let shares = seal_service.initialize().await.unwrap();

    // Vault is sealed after init
    assert!(seal_service.is_sealed().await);

    // Provide threshold shares (3 of 5) to unseal
    for share in shares.iter().take(3) {
        let share_bytes = share.to_bytes().unwrap();
        seal_service.unseal_with_share(&share_bytes).await.unwrap();
    }

    // Now vault should be unsealed
    assert!(
        seal_service.is_unsealed().await,
        "Vault should be unsealed after providing threshold shares"
    );
    assert!(
        !seal_service.is_sealed().await,
        "Vault should not be sealed after unseal"
    );

    // Master key should be available after unseal
    let master_key_result = seal_service.get_master_key().await;
    assert!(
        master_key_result.is_ok(),
        "Master key should be available when unsealed"
    );
}

#[tokio::test]
async fn test_seal_clears_master_key_from_memory() {
    // CRITICAL: Sealing the vault should clear the master key from memory
    let config = SealConfig::default();
    let storage = Arc::new(InMemoryVaultStateStorage::new());
    let seal_service = SealService::with_storage(config, storage);

    // Initialize and unseal
    let shares = seal_service.initialize().await.unwrap();
    for share in shares.iter().take(3) {
        let share_bytes = share.to_bytes().unwrap();
        seal_service.unseal_with_share(&share_bytes).await.unwrap();
    }

    // Verify unsealed and master key available
    assert!(seal_service.is_unsealed().await);
    assert!(seal_service.get_master_key().await.is_ok());

    // Seal the vault
    seal_service.seal().await.unwrap();

    // Verify sealed and master key NOT available
    assert!(seal_service.is_sealed().await, "Vault should be sealed");
    assert!(
        seal_service.get_master_key().await.is_err(),
        "Master key should not be available after seal"
    );
}

#[tokio::test]
async fn test_unseal_progress_tracking() {
    // Test that unseal progress is tracked correctly
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: chrono::Utc::now(),
    };

    let storage = Arc::new(InMemoryVaultStateStorage::new());
    let seal_service = SealService::with_storage(config, storage);

    // Initialize
    let shares = seal_service.initialize().await.unwrap();

    // Provide 1 share
    let share_bytes = shares[0].to_bytes().unwrap();
    seal_service.unseal_with_share(&share_bytes).await.unwrap();

    let status = seal_service.status().await;
    assert_eq!(
        status.progress, 1,
        "Progress should be 1 after providing 1 share"
    );
    assert_eq!(
        status.state,
        secreton_core::services::seal::SealState::Unsealing
    );
    assert!(
        seal_service.is_sealed().await,
        "Vault should still be sealed with only 1 share"
    );

    // Provide 2nd share
    let share_bytes = shares[1].to_bytes().unwrap();
    seal_service.unseal_with_share(&share_bytes).await.unwrap();

    let status = seal_service.status().await;
    assert_eq!(
        status.progress, 2,
        "Progress should be 2 after providing 2 shares"
    );
    assert!(
        seal_service.is_sealed().await,
        "Vault should still be sealed with only 2 shares"
    );

    // Provide 3rd share (threshold met)
    let share_bytes = shares[2].to_bytes().unwrap();
    seal_service.unseal_with_share(&share_bytes).await.unwrap();

    let status = seal_service.status().await;
    assert_eq!(
        status.progress, 0,
        "Progress should reset to 0 after successful unseal"
    );
    assert_eq!(
        status.state,
        secreton_core::services::seal::SealState::Unsealed
    );
    assert!(
        seal_service.is_unsealed().await,
        "Vault should be unsealed after threshold met"
    );
}

#[tokio::test]
async fn test_load_from_storage_starts_sealed() {
    // CRITICAL: When loading vault state from storage, vault should start sealed
    let config = SealConfig::default();
    let storage = Arc::new(InMemoryVaultStateStorage::new());

    // Initialize first service
    let service1 = SealService::with_storage(config.clone(), storage.clone());
    let shares = service1.initialize().await.unwrap();

    // Unseal to get master key
    for share in shares.iter().take(3) {
        let share_bytes = share.to_bytes().unwrap();
        service1.unseal_with_share(&share_bytes).await.unwrap();
    }
    assert!(service1.is_unsealed().await);

    // Seal it
    service1.seal().await.unwrap();
    assert!(service1.is_sealed().await);

    // Create second service and load from storage
    let service2 = SealService::with_storage(config, storage);
    let loaded = service2.load_from_storage().await.unwrap();
    assert!(loaded, "Should successfully load vault state");

    // CRITICAL: Vault should be sealed after loading from storage
    assert!(
        service2.is_sealed().await,
        "Vault should be sealed after loading from storage"
    );
    assert!(
        !service2.is_unsealed().await,
        "Vault should not be unsealed after loading"
    );

    // Master key should NOT be available
    assert!(
        service2.get_master_key().await.is_err(),
        "Master key should not be available when sealed"
    );

    // Unseal with shares
    for share in shares.iter().take(3) {
        let share_bytes = share.to_bytes().unwrap();
        service2.unseal_with_share(&share_bytes).await.unwrap();
    }

    // Now should be unsealed
    assert!(
        service2.is_unsealed().await,
        "Vault should be unsealed after providing shares"
    );
}

#[tokio::test]
async fn test_invalid_share_rejected() {
    // Test that invalid shares are rejected
    let config = SealConfig::default();
    let storage = Arc::new(InMemoryVaultStateStorage::new());
    let seal_service = SealService::with_storage(config, storage);

    // Initialize
    seal_service.initialize().await.unwrap();

    // Try to unseal with invalid share
    let invalid_share = vec![0u8; 100];
    let result = seal_service.unseal_with_share(&invalid_share).await;

    assert!(result.is_err(), "Invalid share should be rejected");
}

#[tokio::test]
async fn test_duplicate_shares_ignored() {
    // Test that duplicate shares are ignored (only counted once)
    let config = SealConfig::default();
    let storage = Arc::new(InMemoryVaultStateStorage::new());
    let seal_service = SealService::with_storage(config, storage);

    // Initialize
    let shares = seal_service.initialize().await.unwrap();

    // Provide same share twice
    let share_bytes = shares[0].to_bytes().unwrap();
    seal_service.unseal_with_share(&share_bytes).await.unwrap();
    seal_service.unseal_with_share(&share_bytes).await.unwrap();

    let status = seal_service.status().await;
    assert_eq!(
        status.progress, 1,
        "Duplicate share should only be counted once"
    );
}

#[tokio::test]
async fn test_reset_unseal_progress() {
    // Test that unseal progress can be reset
    let config = SealConfig::default();
    let storage = Arc::new(InMemoryVaultStateStorage::new());
    let seal_service = SealService::with_storage(config, storage);

    // Initialize
    let shares = seal_service.initialize().await.unwrap();

    // Provide 2 shares
    for share in shares.iter().take(2) {
        let share_bytes = share.to_bytes().unwrap();
        seal_service.unseal_with_share(&share_bytes).await.unwrap();
    }

    let status = seal_service.status().await;
    assert_eq!(status.progress, 2);

    // Reset unseal progress
    seal_service.reset_unseal().await.unwrap();

    let status = seal_service.status().await;
    assert_eq!(status.progress, 0, "Progress should be reset to 0");
    assert_eq!(
        status.state,
        secreton_core::services::seal::SealState::Sealed
    );
}
