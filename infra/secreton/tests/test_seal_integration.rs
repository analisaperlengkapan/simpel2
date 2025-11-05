// Standalone test for Seal/Unseal with Shamir integration
// Run with: cargo test --test test_seal_integration

use chrono::Utc;
use secreton_core::services::seal::{SealConfig, SealService};

#[tokio::test]
async fn test_seal_unseal_with_shamir() {
    let config = SealConfig {
        seal_type: "shamir".to_string(),
        secret_shares: 5,
        secret_threshold: 3,
        created_at: Utc::now(),
    };

    let service = SealService::new(config);

    // Initialize vault - generates master key and shares
    println!("Initializing vault...");
    let shares = service.initialize().await.unwrap();
    assert_eq!(shares.len(), 5);
    println!("Generated {} shares", shares.len());

    // After initialization, vault is unsealed
    assert!(service.is_unsealed().await);
    println!("Vault is unsealed after initialization");

    // Get master key
    let master_key = service.get_master_key().await.unwrap();
    assert_eq!(master_key.len(), 32);
    println!("Master key length: {} bytes", master_key.len());

    // Seal the vault
    println!("Sealing vault...");
    service.seal().await.unwrap();
    assert!(service.is_sealed().await);
    println!("Vault is sealed");

    // Master key should not be available when sealed
    assert!(service.get_master_key().await.is_err());
    println!("Master key not available when sealed (as expected)");

    // Unseal with threshold shares (3 of 5)
    println!("Unsealing with 3 shares...");
    for (i, share) in shares.iter().take(3).enumerate() {
        let share_bytes = share.to_bytes().unwrap();
        println!("Providing share {} (x={})", i + 1, share.x());
        service.unseal_with_share(&share_bytes).await.unwrap();
    }

    // Should be unsealed now
    assert!(service.is_unsealed().await);
    println!("Vault is unsealed");

    // Master key should be available again
    let master_key_after = service.get_master_key().await.unwrap();
    assert_eq!(master_key, master_key_after);
    println!("Master key successfully reconstructed");

    println!("✅ All tests passed!");
}

#[tokio::test]
async fn test_share_verification() {
    let config = SealConfig::default();
    let service = SealService::new(config);

    // Initialize
    println!("Initializing vault...");
    let shares = service.initialize().await.unwrap();

    // Seal
    service.seal().await.unwrap();

    // Try to unseal with invalid share
    println!("Testing invalid share rejection...");
    let invalid_share = vec![0u8; 100];
    let result = service.unseal_with_share(&invalid_share).await;
    assert!(result.is_err());
    println!("Invalid share correctly rejected");

    // Valid shares should work
    println!("Testing valid shares...");
    for (i, share) in shares.iter().take(3).enumerate() {
        let share_bytes = share.to_bytes().unwrap();
        println!("Providing valid share {} (x={})", i + 1, share.x());
        service.unseal_with_share(&share_bytes).await.unwrap();
    }

    assert!(service.is_unsealed().await);
    println!("✅ Share verification test passed!");
}

#[tokio::test]
async fn test_insufficient_shares() {
    let config = SealConfig::default();
    let service = SealService::new(config);

    // Initialize
    let shares = service.initialize().await.unwrap();

    // Seal
    service.seal().await.unwrap();

    // Provide only 2 shares (need 3)
    println!("Providing only 2 shares (need 3)...");
    for (i, share) in shares.iter().take(2).enumerate() {
        let share_bytes = share.to_bytes().unwrap();
        println!("Providing share {} (x={})", i + 1, share.x());
        service.unseal_with_share(&share_bytes).await.unwrap();
    }

    // Should still be sealed
    assert!(service.is_sealed().await);
    println!("Vault still sealed with insufficient shares (as expected)");

    let status = service.status().await;
    assert_eq!(status.progress, 2);
    println!("Progress: {}/{}", status.progress, status.threshold);

    println!("✅ Insufficient shares test passed!");
}
