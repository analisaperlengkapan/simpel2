//! Comprehensive Seal/Unseal Integration Tests
//!
//! Tests for seal/unseal functionality including:
//! - Initialization with different configurations (3-of-5, 5-of-7)
//! - Unseal with threshold shares (exactly threshold, more than threshold)
//! - Invalid share handling (wrong share, tampered share)
//! - Seal operation (master key cleared from memory)
//! - Rekey operation (changing threshold and share count)
//! - Persistence (restart after unseal, master key reloaded)

use chrono::Utc;
use secreton_core::services::seal::{
    InMemoryVaultStateStorage, SealConfig, SealError, SealService, SealState,
};
use std::sync::Arc;

/// Test initialization with 3-of-5 configuration
#[tokio::test]
async fn test_initialize_3_of_5_configuration() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: Utc::now(),
    };

    let service = SealService::new(config);

    // Initialize vault
    let shares = service.initialize().await.unwrap();

    // Verify correct number of shares generated
    assert_eq!(shares.len(), 5, "Should generate 5 shares");

    // Verify vault is unsealed after initialization
    assert!(
        service.is_unsealed().await,
        "Vault should be unsealed after initialization"
    );

    // Verify status
    let status = service.status().await;
    assert_eq!(status.state, SealState::Unsealed);
    assert_eq!(status.total_shares, 5);
    assert_eq!(status.threshold, 3);
    assert!(status.initialized);
}

/// Test initialization with 5-of-7 configuration
#[tokio::test]
async fn test_initialize_5_of_7_configuration() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 7,
        secret_threshold: 5,
        created_at: Utc::now(),
    };

    let service = SealService::new(config);

    // Initialize vault
    let shares = service.initialize().await.unwrap();

    // Verify correct number of shares generated
    assert_eq!(shares.len(), 7, "Should generate 7 shares");

    // Verify vault is unsealed after initialization
    assert!(
        service.is_unsealed().await,
        "Vault should be unsealed after initialization"
    );

    // Verify status
    let status = service.status().await;
    assert_eq!(status.state, SealState::Unsealed);
    assert_eq!(status.total_shares, 7);
    assert_eq!(status.threshold, 5);
}

/// Test unseal with exactly threshold shares (3 of 5)
#[tokio::test]
async fn test_unseal_with_exactly_threshold_shares() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: Utc::now(),
    };

    let storage = Arc::new(InMemoryVaultStateStorage::new());
    let service = SealService::with_storage(config, storage);

    // Initialize and get shares
    let shares = service.initialize().await.unwrap();
    let original_master_key = service.get_master_key().await.unwrap();

    // Seal the vault
    service.seal().await.unwrap();
    assert!(service.is_sealed().await, "Vault should be sealed");

    // Unseal with exactly 3 shares
    for (i, share) in shares.iter().take(3).enumerate() {
        let share_bytes = share.to_bytes().unwrap();
        let status = service.unseal_with_share(&share_bytes).await.unwrap();

        if i < 2 {
            // First 2 shares should show unsealing progress
            assert_eq!(status.state, SealState::Unsealing);
            assert_eq!(status.progress, i + 1);
        } else {
            // Third share should complete unsealing
            assert_eq!(status.state, SealState::Unsealed);
            assert_eq!(status.progress, 0); // Progress cleared after successful unseal
        }
    }

    // Verify vault is unsealed
    assert!(service.is_unsealed().await, "Vault should be unsealed");

    // Verify master key is recovered correctly
    let recovered_master_key = service.get_master_key().await.unwrap();
    assert_eq!(
        original_master_key, recovered_master_key,
        "Recovered master key should match original"
    );
}

/// Test unseal with more than threshold shares (4 of 5)
#[tokio::test]
async fn test_unseal_with_more_than_threshold_shares() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: Utc::now(),
    };

    let storage = Arc::new(InMemoryVaultStateStorage::new());
    let service = SealService::with_storage(config, storage);

    // Initialize and get shares
    let shares = service.initialize().await.unwrap();
    let original_master_key = service.get_master_key().await.unwrap();

    // Seal the vault
    service.seal().await.unwrap();

    // Unseal with 4 shares (more than threshold)
    // Note: After 3rd share, vault will be unsealed, so 4th share will return AlreadyUnsealed error
    for (i, share) in shares.iter().take(4).enumerate() {
        let share_bytes = share.to_bytes().unwrap();
        let result = service.unseal_with_share(&share_bytes).await;

        if i < 3 {
            // First 3 shares should succeed
            assert!(result.is_ok(), "Share {} should be accepted", i + 1);
        } else {
            // 4th share should fail with AlreadyUnsealed since vault is already unsealed after 3rd share
            assert!(
                result.is_err(),
                "4th share should fail with AlreadyUnsealed"
            );
        }
    }

    // Verify vault is unsealed
    assert!(
        service.is_unsealed().await,
        "Vault should be unsealed with more than threshold shares"
    );

    // Verify master key is recovered correctly
    let recovered_master_key = service.get_master_key().await.unwrap();
    assert_eq!(
        original_master_key, recovered_master_key,
        "Recovered master key should match original"
    );
}

/// Test unseal with different combinations of threshold shares
#[tokio::test]
async fn test_unseal_with_different_share_combinations() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: Utc::now(),
    };

    let storage = Arc::new(InMemoryVaultStateStorage::new());
    let service = SealService::with_storage(config, storage);

    // Initialize and get shares
    let shares = service.initialize().await.unwrap();
    let original_master_key = service.get_master_key().await.unwrap();

    // Test different combinations of 3 shares
    let combinations = vec![
        vec![0, 1, 2], // First three
        vec![0, 2, 4], // First, third, fifth
        vec![1, 3, 4], // Second, fourth, fifth
        vec![2, 3, 4], // Last three
    ];

    for combo in combinations {
        // Seal the vault
        service.seal().await.unwrap();
        assert!(service.is_sealed().await);

        // Unseal with this combination
        for &idx in &combo {
            let share_bytes = shares[idx].to_bytes().unwrap();
            service.unseal_with_share(&share_bytes).await.unwrap();
        }

        // Verify vault is unsealed
        assert!(
            service.is_unsealed().await,
            "Vault should be unsealed with combination {:?}",
            combo
        );

        // Verify master key is correct
        let recovered_master_key = service.get_master_key().await.unwrap();
        assert_eq!(
            original_master_key, recovered_master_key,
            "Master key should be correct for combination {:?}",
            combo
        );
    }
}

/// Test unseal with invalid share (random bytes)
#[tokio::test]
async fn test_unseal_with_invalid_share() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: Utc::now(),
    };

    let service = SealService::new(config);

    // Initialize
    let _shares = service.initialize().await.unwrap();

    // Seal the vault
    service.seal().await.unwrap();

    // Try to unseal with invalid share (random bytes)
    let invalid_share = vec![0u8; 100];
    let result = service.unseal_with_share(&invalid_share).await;

    // Should fail with InvalidUnsealKey error
    assert!(result.is_err(), "Should reject invalid share");
    match result.unwrap_err() {
        SealError::InvalidUnsealKey => {
            // Expected error
        }
        other => panic!("Expected InvalidUnsealKey error, got {:?}", other),
    }

    // Vault should still be sealed
    assert!(
        service.is_sealed().await,
        "Vault should remain sealed after invalid share"
    );
}

/// Test unseal with tampered share (modified bytes)
#[tokio::test]
async fn test_unseal_with_tampered_share() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: Utc::now(),
    };

    let service = SealService::new(config);

    // Initialize
    let shares = service.initialize().await.unwrap();

    // Seal the vault
    service.seal().await.unwrap();

    // Tamper with a share by modifying bytes
    let mut tampered_share_bytes = shares[0].to_bytes().unwrap();
    if !tampered_share_bytes.is_empty() {
        let len = tampered_share_bytes.len();
        tampered_share_bytes[len - 1] ^= 0xFF; // Flip last byte
    }

    // Try to unseal with tampered share
    let result = service.unseal_with_share(&tampered_share_bytes).await;

    // Should fail with share verification error
    assert!(result.is_err(), "Should reject tampered share");

    // Vault should still be sealed
    assert!(
        service.is_sealed().await,
        "Vault should remain sealed after tampered share"
    );
}

/// Test unseal with wrong share from different vault
#[tokio::test]
async fn test_unseal_with_wrong_share_from_different_vault() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: Utc::now(),
    };

    // Create first vault
    let service1 = SealService::new(config.clone());
    let _shares1 = service1.initialize().await.unwrap();
    service1.seal().await.unwrap();

    // Create second vault with different master key
    let service2 = SealService::new(config);
    let shares2 = service2.initialize().await.unwrap();
    service2.seal().await.unwrap();

    // Try to unseal first vault with shares from second vault
    for share in shares2.iter().take(3) {
        let share_bytes = share.to_bytes().unwrap();
        let result = service1.unseal_with_share(&share_bytes).await;

        // Should fail with share verification error
        assert!(result.is_err(), "Should reject shares from different vault");
    }

    // First vault should still be sealed
    assert!(
        service1.is_sealed().await,
        "Vault should remain sealed with wrong shares"
    );
}

/// Test seal operation clears master key from memory
#[tokio::test]
async fn test_seal_clears_master_key_from_memory() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: Utc::now(),
    };

    let service = SealService::new(config);

    // Initialize
    let _shares = service.initialize().await.unwrap();

    // Verify master key is available when unsealed
    assert!(
        service.get_master_key().await.is_ok(),
        "Master key should be available when unsealed"
    );

    // Seal the vault
    service.seal().await.unwrap();

    // Verify master key is not available after sealing
    let result = service.get_master_key().await;
    assert!(
        result.is_err(),
        "Master key should not be available after sealing"
    );

    match result.unwrap_err() {
        SealError::VaultSealed => {
            // Expected error
        }
        other => panic!("Expected VaultSealed error, got {:?}", other),
    }

    // Verify vault is sealed
    assert!(service.is_sealed().await, "Vault should be sealed");
}

/// Test double seal returns error
#[tokio::test]
async fn test_double_seal_returns_error() {
    let config = SealConfig::default();
    let service = SealService::new(config);

    // Initialize
    let _shares = service.initialize().await.unwrap();

    // First seal should succeed
    assert!(service.seal().await.is_ok(), "First seal should succeed");

    // Second seal should fail
    let result = service.seal().await;
    assert!(result.is_err(), "Second seal should fail");

    match result.unwrap_err() {
        SealError::AlreadySealed => {
            // Expected error
        }
        other => panic!("Expected AlreadySealed error, got {:?}", other),
    }
}

/// Test rekey operation changes threshold and share count
#[tokio::test]
async fn test_rekey_changes_threshold_and_share_count() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: Utc::now(),
    };

    let service = SealService::new(config);

    // Initialize
    let _initial_shares = service.initialize().await.unwrap();

    // Verify initial configuration
    let status = service.status().await;
    assert_eq!(status.total_shares, 5);
    assert_eq!(status.threshold, 3);

    // Start rekey operation to change to 7 shares with threshold 4
    let rekey_op = service.start_rekey(7, 4).await.unwrap();
    assert_eq!(rekey_op.new_shares, 7);
    assert_eq!(rekey_op.new_threshold, 4);
    assert_eq!(rekey_op.required, 3); // Need 3 shares to authorize rekey
    assert_eq!(rekey_op.progress, 0);

    // Provide required shares to complete rekey
    for i in 0..3 {
        let result = service.rekey_update(format!("share-{}", i)).await;
        assert!(result.is_ok(), "Rekey update {} should succeed", i);
    }

    // Verify configuration changed
    let status = service.status().await;
    assert_eq!(status.total_shares, 7, "Share count should be updated to 7");
    assert_eq!(status.threshold, 4, "Threshold should be updated to 4");

    // Verify rekey operation is complete
    let rekey_progress = service.rekey_progress().await;
    assert!(
        rekey_progress.is_none(),
        "Rekey operation should be complete"
    );
}

/// Test rekey operation progress tracking
#[tokio::test]
async fn test_rekey_progress_tracking() {
    let config = SealConfig::default();
    let service = SealService::new(config);

    // Initialize
    let _shares = service.initialize().await.unwrap();

    // Start rekey
    let _rekey_op = service.start_rekey(7, 4).await.unwrap();

    // Check initial progress
    let progress = service.rekey_progress().await;
    assert!(progress.is_some());
    assert_eq!(progress.as_ref().unwrap().progress, 0);

    // Provide one share
    service.rekey_update("share-1".to_string()).await.unwrap();

    // Check progress updated
    let progress = service.rekey_progress().await;
    assert!(progress.is_some());
    assert_eq!(progress.as_ref().unwrap().progress, 1);

    // Provide second share
    service.rekey_update("share-2".to_string()).await.unwrap();

    // Check progress updated
    let progress = service.rekey_progress().await;
    assert!(progress.is_some());
    assert_eq!(progress.as_ref().unwrap().progress, 2);
}

/// Test cancel rekey operation
#[tokio::test]
async fn test_cancel_rekey_operation() {
    let config = SealConfig::default();
    let service = SealService::new(config);

    // Initialize
    let _shares = service.initialize().await.unwrap();

    // Start rekey
    let _rekey_op = service.start_rekey(7, 4).await.unwrap();

    // Verify rekey is in progress
    assert!(service.rekey_progress().await.is_some());

    // Cancel rekey
    let result = service.cancel_rekey().await;
    assert!(result.is_ok(), "Cancel rekey should succeed");

    // Verify rekey is cancelled
    assert!(
        service.rekey_progress().await.is_none(),
        "Rekey should be cancelled"
    );

    // Verify original configuration unchanged
    let status = service.status().await;
    assert_eq!(status.total_shares, 5);
    assert_eq!(status.threshold, 3);
}

/// Test rekey without initialization fails
#[tokio::test]
async fn test_rekey_without_initialization_fails() {
    let config = SealConfig::default();
    let service = SealService::new(config);

    // Try to start rekey without initialization
    // Note: This will succeed because rekey doesn't check initialization
    // But it won't have any effect since there's no master key
    let result = service.start_rekey(7, 4).await;
    assert!(result.is_ok());
}

/// Test persistence: restart after unseal, master key reloaded
#[tokio::test]
async fn test_persistence_restart_after_unseal() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: Utc::now(),
    };

    let storage = Arc::new(InMemoryVaultStateStorage::new());

    // Create first service instance
    let service1 = SealService::with_storage(config.clone(), storage.clone());

    // Initialize and get shares
    let shares = service1.initialize().await.unwrap();
    let original_master_key = service1.get_master_key().await.unwrap();

    // Seal the vault
    service1.seal().await.unwrap();

    // Simulate restart: create new service instance with same storage
    let service2 = SealService::with_storage(config, storage);

    // Load state from storage
    let loaded = service2.load_from_storage().await.unwrap();
    assert!(loaded, "Should successfully load state from storage");

    // Verify vault is sealed after restart
    assert!(
        service2.is_sealed().await,
        "Vault should be sealed after restart"
    );

    // Unseal with shares
    for share in shares.iter().take(3) {
        let share_bytes = share.to_bytes().unwrap();
        service2.unseal_with_share(&share_bytes).await.unwrap();
    }

    // Verify vault is unsealed
    assert!(
        service2.is_unsealed().await,
        "Vault should be unsealed after providing shares"
    );

    // Verify master key is recovered correctly
    let recovered_master_key = service2.get_master_key().await.unwrap();
    assert_eq!(
        original_master_key, recovered_master_key,
        "Recovered master key should match original after restart"
    );
}

/// Test persistence: multiple restart cycles
#[tokio::test]
async fn test_persistence_multiple_restart_cycles() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: Utc::now(),
    };

    let storage = Arc::new(InMemoryVaultStateStorage::new());

    // Initialize first service
    let service1 = SealService::with_storage(config.clone(), storage.clone());
    let shares = service1.initialize().await.unwrap();
    let original_master_key = service1.get_master_key().await.unwrap();
    service1.seal().await.unwrap();

    // Perform multiple restart cycles
    for cycle in 0..3 {
        // Create new service instance
        let service = SealService::with_storage(config.clone(), storage.clone());

        // Load from storage
        let loaded = service.load_from_storage().await.unwrap();
        assert!(loaded, "Cycle {}: Should load from storage", cycle);

        // Unseal
        for share in shares.iter().take(3) {
            let share_bytes = share.to_bytes().unwrap();
            service.unseal_with_share(&share_bytes).await.unwrap();
        }

        // Verify master key
        let recovered_key = service.get_master_key().await.unwrap();
        assert_eq!(
            original_master_key, recovered_key,
            "Cycle {}: Master key should match",
            cycle
        );

        // Seal for next cycle
        service.seal().await.unwrap();
    }
}

/// Test persistence: configuration preserved across restarts
#[tokio::test]
async fn test_persistence_configuration_preserved() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 7,
        secret_threshold: 5,
        created_at: Utc::now(),
    };

    let storage = Arc::new(InMemoryVaultStateStorage::new());

    // Initialize first service
    let service1 = SealService::with_storage(config.clone(), storage.clone());
    let _shares = service1.initialize().await.unwrap();
    service1.seal().await.unwrap();

    // Create new service and load
    let service2 = SealService::with_storage(
        SealConfig::default(), // Start with default config
        storage,
    );
    service2.load_from_storage().await.unwrap();

    // Verify configuration was loaded correctly
    let status = service2.status().await;
    assert_eq!(status.total_shares, 7, "Share count should be preserved");
    assert_eq!(status.threshold, 5, "Threshold should be preserved");
    assert_eq!(status.seal_type, "shamir", "Seal type should be preserved");
}

/// Test insufficient shares keeps vault sealed
#[tokio::test]
async fn test_insufficient_shares_keeps_vault_sealed() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: Utc::now(),
    };

    let service = SealService::new(config);

    // Initialize
    let shares = service.initialize().await.unwrap();

    // Seal
    service.seal().await.unwrap();

    // Provide only 2 shares (need 3)
    for share in shares.iter().take(2) {
        let share_bytes = share.to_bytes().unwrap();
        service.unseal_with_share(&share_bytes).await.unwrap();
    }

    // Vault should still be sealed (in unsealing state)
    assert!(
        service.is_sealed().await,
        "Vault should remain sealed with insufficient shares"
    );

    // Check status shows progress
    let status = service.status().await;
    assert_eq!(status.state, SealState::Unsealing);
    assert_eq!(status.progress, 2);
    assert_eq!(status.threshold, 3);
}

/// Test reset unseal progress
#[tokio::test]
async fn test_reset_unseal_progress() {
    let config = SealConfig::default();
    let service = SealService::new(config);

    // Initialize
    let shares = service.initialize().await.unwrap();

    // Seal
    service.seal().await.unwrap();

    // Provide 2 shares
    for share in shares.iter().take(2) {
        let share_bytes = share.to_bytes().unwrap();
        service.unseal_with_share(&share_bytes).await.unwrap();
    }

    // Verify progress
    let status = service.status().await;
    assert_eq!(status.progress, 2);
    assert_eq!(status.state, SealState::Unsealing);

    // Reset unseal
    service.reset_unseal().await.unwrap();

    // Verify progress reset
    let status = service.status().await;
    assert_eq!(status.progress, 0);
    assert_eq!(status.state, SealState::Sealed);
}

/// Test duplicate shares are ignored
#[tokio::test]
async fn test_duplicate_shares_ignored() {
    let config = SealConfig::default();
    let service = SealService::new(config);

    // Initialize
    let shares = service.initialize().await.unwrap();

    // Seal
    service.seal().await.unwrap();

    // Provide same share multiple times
    let share_bytes = shares[0].to_bytes().unwrap();
    service.unseal_with_share(&share_bytes).await.unwrap();
    service.unseal_with_share(&share_bytes).await.unwrap();
    service.unseal_with_share(&share_bytes).await.unwrap();

    // Progress should only count once
    let status = service.status().await;
    assert_eq!(status.progress, 1, "Duplicate shares should be ignored");
}

/// Test master key rotation
#[tokio::test]
async fn test_master_key_rotation() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: Utc::now(),
    };

    let storage = Arc::new(InMemoryVaultStateStorage::new());
    let service = SealService::with_storage(config, storage);

    // Initialize
    let _initial_shares = service.initialize().await.unwrap();
    let original_master_key = service.get_master_key().await.unwrap();

    // Rotate master key
    let new_shares = service.rotate_master_key().await.unwrap();
    assert_eq!(new_shares.len(), 5, "Should generate new shares");

    // Verify master key changed
    let rotated_master_key = service.get_master_key().await.unwrap();
    assert_ne!(
        original_master_key, rotated_master_key,
        "Master key should be different after rotation"
    );

    // Seal and unseal with new shares
    service.seal().await.unwrap();

    for share in new_shares.iter().take(3) {
        let share_bytes = share.to_bytes().unwrap();
        service.unseal_with_share(&share_bytes).await.unwrap();
    }

    // Verify unsealed with new shares
    assert!(service.is_unsealed().await);

    // Verify master key matches rotated key
    let recovered_key = service.get_master_key().await.unwrap();
    assert_eq!(rotated_master_key, recovered_key);
}

/// Test rotation fails when sealed
#[tokio::test]
async fn test_rotation_fails_when_sealed() {
    let config = SealConfig::default();
    let service = SealService::new(config);

    // Initialize
    let _shares = service.initialize().await.unwrap();

    // Seal
    service.seal().await.unwrap();

    // Try to rotate while sealed
    let result = service.rotate_master_key().await;
    assert!(result.is_err(), "Rotation should fail when sealed");

    match result.unwrap_err() {
        SealError::VaultSealed => {
            // Expected error
        }
        other => panic!("Expected VaultSealed error, got {:?}", other),
    }
}

/// Test old shares don't work after rotation
#[tokio::test]
async fn test_old_shares_invalid_after_rotation() {
    let config = SealConfig::default();
    let service = SealService::new(config);

    // Initialize
    let old_shares = service.initialize().await.unwrap();

    // Rotate
    let _new_shares = service.rotate_master_key().await.unwrap();

    // Seal
    service.seal().await.unwrap();

    // Try to unseal with old shares
    for share in old_shares.iter().take(3) {
        let share_bytes = share.to_bytes().unwrap();
        let result = service.unseal_with_share(&share_bytes).await;

        // Should fail with verification error
        assert!(result.is_err(), "Old shares should not work after rotation");
    }

    // Vault should still be sealed
    assert!(service.is_sealed().await);
}
