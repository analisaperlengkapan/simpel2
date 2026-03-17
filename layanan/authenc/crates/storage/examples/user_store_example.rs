//! Example demonstrating PostgresUserStore usage
//!
//! This example shows how to:
//! - Create a database connection pool
//! - Initialize PostgresUserStore
//! - Perform CRUD operations on users
//!
//! To run this example:
//! ```bash
//! # Set up PostgreSQL database first
//! cargo run --example user_store_example
//! ```

use authenc_storage::{Database, PostgresUserStore};
use authenc_types::{
    RealmId, UserId,
    domain::{CreateUserRequest, UpdateUserRequest},
    traits::UserStore,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    println!("=== PostgresUserStore Example ===\n");

    // 1. Create database connection pool
    println!("1. Connecting to database...");
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://authenc:password@localhost:5432/authenc".to_string());

    let db = Arc::new(Database::new(&database_url, 20).await?);
    println!("   ✓ Connected to database\n");

    // 2. Initialize PostgresUserStore
    println!("2. Initializing PostgresUserStore...");
    let user_store = PostgresUserStore::new(Arc::clone(&db));
    println!("   ✓ PostgresUserStore initialized\n");

    // 3. Create a test realm (assuming it exists)
    let realm_id = RealmId::new();
    println!("3. Using realm ID: {}\n", realm_id);

    // 4. Create a new user
    println!("4. Creating a new user...");
    let create_request = CreateUserRequest {
        username: "john_doe".to_string(),
        email: "john.doe@example.com".to_string(),
        password: Some("$argon2id$v=19$m=65536,t=3,p=4$...".to_string()), // Pre-hashed password
        satker_code: "123".to_string(),
        first_name: None,
        last_name: None,
        nip: None,
        nama: None,
        jabatan: None,
        phone_number: None,
        realm_id: Some(realm_id.0),
        organization_id: None,
        roles: None,
        attributes: None,
        enabled: None,
    };

    match user_store.create_user(create_request).await {
        Ok(user) => {
            println!("   ✓ User created successfully:");
            println!("     - ID: {}", user.id);
            println!("     - Username: {}", user.username);
            println!("     - Email: {}", user.email);
            println!("     - Enabled: {}", user.enabled);
            println!("     - MFA Enabled: {}", user.mfa_enabled);
            println!();

            // 5. Get user by ID
            println!("5. Retrieving user by ID...");
            let retrieved_user = user_store.get_user(UserId(user.id)).await?;
            println!("   ✓ User retrieved: {}", retrieved_user.username);
            println!();

            // 6. Get user by username
            println!("6. Retrieving user by username...");
            let user_by_username = user_store
                .get_user_by_username(&user.username, realm_id)
                .await?;
            println!("   ✓ User found: {}", user_by_username.email);
            println!();

            // 7. Update user
            println!("7. Updating user email...");
            let update_request = UpdateUserRequest {
                email: Some("john.doe.updated@example.com".to_string()),
                username: None,
                satker_code: None,
                first_name: None,
                last_name: None,
                nip: None,
                nama: None,
                jabatan: None,
                phone_number: None,
                password: None,
                enabled: None,
                email_verified: Some(true),
                mfa_enabled: None,
                require_password_change: None,
                phone_verified: None,
                attributes: None,
            };

            let updated_user = user_store
                .update_user(UserId(user.id), update_request)
                .await?;
            println!("   ✓ User updated:");
            println!("     - New email: {}", updated_user.email);
            println!("     - Email verified: {}", updated_user.email_verified);
            println!();

            // 8. List users
            println!("8. Listing users in realm...");
            let users = user_store.list_users(realm_id, 0, 10).await?;
            println!("   ✓ Found {} user(s)", users.len());
            println!();

            // 9. Check if username exists
            println!("9. Checking if username exists...");
            let exists = user_store.username_exists("john_doe", realm_id).await?;
            println!("   ✓ Username 'john_doe' exists: {}", exists);
            println!();

            // 10. Delete user (soft delete)
            println!("10. Deleting user...");
            user_store.delete_user(UserId(user.id)).await?;
            println!("   ✓ User deleted (soft delete - account disabled)");
            println!();

            // 11. Verify user is disabled
            println!("11. Verifying user is disabled...");
            let deleted_user = user_store.get_user(UserId(user.id)).await?;
            println!("   ✓ User enabled status: {}", deleted_user.enabled);
            println!();
        }
        Err(e) => {
            println!("   ✗ Failed to create user: {}", e);
            println!("   Note: This example requires a running PostgreSQL database");
            println!("   with the users table created.");
        }
    }

    // 12. Check pool status
    println!("12. Database pool status:");
    let pool_status = db.pool_status();
    println!("   - Total connections: {}", pool_status.size);
    println!("   - Available connections: {}", pool_status.available);
    println!("   - Waiting tasks: {}", pool_status.waiting);
    println!();

    println!("=== Example completed ===");

    Ok(())
}
