//! Example demonstrating PostgresRealmStore usage
//!
//! This example shows how to use the PostgresRealmStore to manage realms.
//!
//! Run with:
//! ```bash
//! cargo run --example realm_store_example
//! ```

use authenc_storage::{Database, PostgresRealmStore};
use authenc_types::{traits::RealmStore, RealmId};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Database connection string (use environment variable in production)
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://authenc:password@localhost:5432/authenc".to_string());

    println!("Connecting to database: {}", database_url);

    // Create database connection pool
    let db = Arc::new(Database::new(&database_url, 20).await?);

    // Create realm store
    let realm_store = PostgresRealmStore::new(db.clone());

    println!("\n=== Creating Realm ===");
    let realm = realm_store
        .create_realm(
            "kejaksaan-ri".to_string(),
            "Kejaksaan Republik Indonesia".to_string(),
        )
        .await?;

    println!("Created realm: {:?}", realm);
    println!("  ID: {}", realm.id);
    println!("  Name: {}", realm.name);
    println!("  Display Name: {}", realm.display_name);
    println!("  Enabled: {}", realm.enabled);

    println!("\n=== Getting Realm by ID ===");
    let fetched_realm = realm_store.get_realm(realm.id).await?;
    println!("Fetched realm: {:?}", fetched_realm);

    println!("\n=== Getting Realm by Name ===");
    let fetched_by_name = realm_store.get_realm_by_name("kejaksaan-ri").await?;
    println!("Fetched by name: {:?}", fetched_by_name);

    println!("\n=== Updating Realm ===");
    let updated_realm = realm_store
        .update_realm(
            realm.id,
            Some("Kejaksaan RI - Updated".to_string()),
            None,
        )
        .await?;
    println!("Updated realm: {:?}", updated_realm);
    println!("  New Display Name: {}", updated_realm.display_name);

    println!("\n=== Creating Another Realm ===");
    let realm2 = realm_store
        .create_realm(
            "kejaksaan-agung".to_string(),
            "Kejaksaan Agung".to_string(),
        )
        .await?;
    println!("Created second realm: {:?}", realm2);

    println!("\n=== Listing All Realms ===");
    let realms = realm_store.list_realms().await?;
    println!("Total realms: {}", realms.len());
    for (i, r) in realms.iter().enumerate() {
        println!("  {}. {} - {} (enabled: {})", i + 1, r.name, r.display_name, r.enabled);
    }

    println!("\n=== Checking Realm Name Existence ===");
    let exists = realm_store.realm_name_exists("kejaksaan-ri").await?;
    println!("Realm 'kejaksaan-ri' exists: {}", exists);

    let not_exists = realm_store.realm_name_exists("non-existent").await?;
    println!("Realm 'non-existent' exists: {}", not_exists);

    println!("\n=== Disabling Realm ===");
    let disabled_realm = realm_store
        .update_realm(realm.id, None, Some(false))
        .await?;
    println!("Disabled realm: {:?}", disabled_realm);
    println!("  Enabled: {}", disabled_realm.enabled);

    println!("\n=== Deleting Realms ===");
    realm_store.delete_realm(realm.id).await?;
    println!("Deleted realm: {}", realm.id);

    realm_store.delete_realm(realm2.id).await?;
    println!("Deleted realm: {}", realm2.id);

    println!("\n=== Verifying Deletion ===");
    let remaining_realms = realm_store.list_realms().await?;
    println!("Remaining realms: {}", remaining_realms.len());

    println!("\n=== Testing Error Cases ===");

    // Try to get non-existent realm
    println!("Attempting to get non-existent realm...");
    match realm_store.get_realm(RealmId::new()).await {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Expected error: {}", e),
    }

    // Try to create duplicate realm
    println!("Creating realm for duplicate test...");
    let test_realm = realm_store
        .create_realm("test-realm".to_string(), "Test Realm".to_string())
        .await?;

    println!("Attempting to create duplicate realm...");
    match realm_store
        .create_realm("test-realm".to_string(), "Test Realm Duplicate".to_string())
        .await
    {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Expected error: {}", e),
    }

    // Cleanup
    realm_store.delete_realm(test_realm.id).await?;
    println!("Cleaned up test realm");

    println!("\n=== Example Complete ===");

    Ok(())
}
