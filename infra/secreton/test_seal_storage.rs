// Simple test to verify seal service with storage works
use secreton_core::services::seal::{SealConfig, SealService, InMemoryVaultStateStorage};
use std::sync::Arc;
use chrono::Utc;

#[tokio::main]
async fn main() {
    println!("Testing Seal Service with Storage...\n");

    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: Utc::now(),
    };

    let storage = Arc::new(InMemoryVaultStateStorage::new());
    let service = SealService::with_storage(config, storage.clone());

    // Test 1: Initialize vault
    println!("1. Initializing vault...");
    let shares = service.initialize().await.expect("Failed to initialize");
    println!("   ✓ Generated {} shares", shares.len());
    assert_eq!(shares.len(), 5);

    // Test 2: Verify vault state was stored
    println!("\n2. Verifying vault state storage...");
    let vault_state = storage.load_vault_state().await.expect("Failed to load state");
    assert!(vault_state.is_some());
    println!("   ✓ Vault state stored successfully");

    // Test 3: Get master key
    println!("\n3. Getting master key...");
    let original_master_key = service.get_master_key().await.expect("Failed to get master key");
    println!("   ✓ Master key retrieved (length: {} bytes)", original_master_key.len());
    assert_eq!(original_master_key.len(), 32);

    // Test 4: Seal the vault
    println!("\n4. Sealing vault...");
    service.seal().await.expect("Failed to seal");
    assert!(service.is_sealed().await);
    println!("   ✓ Vault sealed");

    // Test 5: Verify master key is not accessible
    println!("\n5. Verifying master key is not accessible...");
    let result = service.get_master_key().await;
    assert!(result.is_err());
    println!("   ✓ Master key properly cleared from memory");

    // Test 6: Unseal with threshold shares
    println!("\n6. Unsealing with {} shares...", 3);
    for (i, share) in shares.iter().take(3).enumerate() {
        let share_bytes = share.to_bytes().expect("Failed to serialize share");
        service.unseal_with_share(&share_bytes).await.expect("Failed to unseal");
        println!("   - Share {} provided", i + 1);
    }
    assert!(service.is_unsealed().await);
    println!("   ✓ Vault unsealed");

    // Test 7: Verify master key recovered
    println!("\n7. Verifying master key recovery...");
    let recovered_master_key = service.get_master_key().await.expect("Failed to get master key");
    assert_eq!(original_master_key, recovered_master_key);
    println!("   ✓ Master key successfully recovered");

    // Test 8: Master key rotation
    println!("\n8. Testing master key rotation...");
    let new_shares = service.rotate_master_key().await.expect("Failed to rotate");
    let rotated_master_key = service.get_master_key().await.expect("Failed to get master key");
    assert_ne!(original_master_key, rotated_master_key);
    println!("   ✓ Master key rotated successfully");

    // Test 9: Seal and unseal with new shares
    println!("\n9. Testing seal/unseal with new shares...");
    service.seal().await.expect("Failed to seal");
    for share in new_shares.iter().take(3) {
        let share_bytes = share.to_bytes().expect("Failed to serialize share");
        service.unseal_with_share(&share_bytes).await.expect("Failed to unseal");
    }
    assert!(service.is_unsealed().await);
    let final_master_key = service.get_master_key().await.expect("Failed to get master key");
    assert_eq!(rotated_master_key, final_master_key);
    println!("   ✓ New shares work correctly");

    // Test 10: Load from storage
    println!("\n10. Testing load from storage...");
    service.seal().await.expect("Failed to seal");
    let service2 = SealService::with_storage(
        SealConfig {
            seal_type: "shamir".to_string(),
            secret_shares: 5,
            secret_threshold: 3,
            created_at: Utc::now(),
        },
        storage,
    );
    let loaded = service2.load_from_storage().await.expect("Failed to load");
    assert!(loaded);
    println!("   ✓ Vault state loaded from storage");

    // Unseal with new shares
    for share in new_shares.iter().take(3) {
        let share_bytes = share.to_bytes().expect("Failed to serialize share");
        service2.unseal_with_share(&share_bytes).await.expect("Failed to unseal");
    }
    let loaded_master_key = service2.get_master_key().await.expect("Failed to get master key");
    assert_eq!(rotated_master_key, loaded_master_key);
    println!("   ✓ Master key recovered from storage");

    println!("\n✅ All tests passed!");
}
