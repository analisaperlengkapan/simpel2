//! Example demonstrating PostgresClientStore usage
//!
//! This example shows how to use the PostgresClientStore to manage OAuth2/OIDC clients.
//!
//! Run with: cargo run --example client_store_example

use authenc_storage::{Database, PostgresClientStore};
use authenc_types::{RealmId, traits::ClientStore};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Database connection string (use environment variable in production)
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://authenc:password@localhost:5432/authenc".to_string());

    // Create database connection pool
    let db = Arc::new(Database::new(&database_url, 20).await?);

    // Create client store
    let client_store = PostgresClientStore::new(db.clone());

    // Example realm ID (in practice, this would come from a realm lookup)
    let realm_id = RealmId::new();

    println!("=== OAuth2 Client Store Example ===\n");

    // 1. Create a public client (for SPAs, mobile apps)
    println!("1. Creating a public client...");
    let public_client = client_store
        .create_client(
            "portal-spa".to_string(),
            "Portal Single Page Application".to_string(),
            true, // is_public
            realm_id,
        )
        .await?;

    println!("   Created public client:");
    println!("   - ID: {}", public_client.id);
    println!("   - Client ID: {}", public_client.client_id);
    println!("   - Name: {}", public_client.name);
    println!("   - Is Public: {}", public_client.is_public);
    println!(
        "   - Has Secret: {}\n",
        public_client.client_secret_hash.is_some()
    );

    // 2. Create a confidential client (for backend services)
    println!("2. Creating a confidential client...");
    let confidential_client = client_store
        .create_client(
            "backend-service".to_string(),
            "Backend Service Client".to_string(),
            false, // is_public
            realm_id,
        )
        .await?;

    println!("   Created confidential client:");
    println!("   - ID: {}", confidential_client.id);
    println!("   - Client ID: {}", confidential_client.client_id);
    println!("   - Name: {}", confidential_client.name);
    println!("   - Is Public: {}", confidential_client.is_public);
    println!(
        "   - Has Secret: {}\n",
        confidential_client.client_secret_hash.is_some()
    );

    // 3. Update client with redirect URIs and scopes
    println!("3. Updating public client with redirect URIs and scopes...");
    let updated_client = client_store
        .update_client(
            public_client.id,
            None, // name unchanged
            Some(vec![
                "https://portal.kejaksaan.go.id/callback".to_string(),
                "https://portal.kejaksaan.go.id/silent-refresh".to_string(),
            ]),
            Some(vec![
                "openid".to_string(),
                "profile".to_string(),
                "email".to_string(),
            ]),
            None, // enabled unchanged
        )
        .await?;

    println!("   Updated client:");
    println!("   - Redirect URIs: {:?}", updated_client.redirect_uris);
    println!("   - Allowed Scopes: {:?}\n", updated_client.allowed_scopes);

    // 4. Get client by ID
    println!("4. Retrieving client by ID...");
    let retrieved_client = client_store.get_client(public_client.id).await?;
    println!(
        "   Retrieved: {} ({})\n",
        retrieved_client.name, retrieved_client.client_id
    );

    // 5. Get client by client_id string
    println!("5. Retrieving client by client_id string...");
    let retrieved_by_client_id = client_store
        .get_client_by_client_id("portal-spa", realm_id)
        .await?;
    println!(
        "   Retrieved: {} (ID: {})\n",
        retrieved_by_client_id.name, retrieved_by_client_id.id
    );

    // 6. List all clients in realm
    println!("6. Listing all clients in realm...");
    let clients = client_store.list_clients(realm_id).await?;
    println!("   Found {} client(s):", clients.len());
    for client in &clients {
        println!(
            "   - {} ({}) - {}",
            client.name,
            client.client_id,
            if client.is_public {
                "Public"
            } else {
                "Confidential"
            }
        );
    }
    println!();

    // 7. Update client secret (for confidential clients)
    println!("7. Updating client secret...");
    let new_secret_hash = "hashed_secret_value_here"; // In practice, hash with Argon2
    client_store
        .update_client_secret(confidential_client.id, new_secret_hash.to_string())
        .await?;
    println!("   Client secret updated successfully\n");

    // 8. Disable a client
    println!("8. Disabling a client...");
    let disabled_client = client_store
        .update_client(
            confidential_client.id,
            None,
            None,
            None,
            Some(false), // enabled = false
        )
        .await?;
    println!("   Client disabled: {}\n", !disabled_client.enabled);

    // 9. Delete a client (soft delete)
    println!("9. Deleting a client...");
    client_store.delete_client(public_client.id).await?;
    println!("   Client deleted successfully\n");

    // 10. Verify deletion
    println!("10. Verifying deletion...");
    match client_store.get_client(public_client.id).await {
        Ok(_) => println!("   ERROR: Client still exists!"),
        Err(e) => println!("   Confirmed deleted: {}\n", e),
    }

    println!("=== Example completed successfully ===");

    Ok(())
}
