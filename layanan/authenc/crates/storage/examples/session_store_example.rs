//! Example demonstrating PostgresSessionStore usage
//!
//! This example shows how to:
//! - Create a session for a user
//! - Retrieve a session by ID
//! - Update session last accessed time
//! - List all active sessions for a user
//! - Invalidate a session
//! - Clean up expired sessions
//!
//! Run with: cargo run --example session_store_example

use authenc_storage::{Database, PostgresSessionStore};
use authenc_types::{SessionId, UserId, traits::SessionStore};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== PostgresSessionStore Example ===\n");

    // Database connection string (adjust as needed)
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://authenc:password@localhost:5432/authenc".to_string());

    println!("Connecting to database...");
    let db = Arc::new(Database::new(&database_url, 20).await?);
    println!("✓ Connected to database\n");

    // Create session store with default timeouts (15min idle, 8h absolute)
    let session_store = PostgresSessionStore::new(Arc::clone(&db));

    // Example user ID (in real usage, this would come from user authentication)
    let user_id = UserId::new();
    println!("Using example user ID: {}\n", user_id);

    // 1. Create a session
    println!("1. Creating session for user...");
    let session = session_store.create_session(user_id).await?;
    println!("✓ Session created:");
    println!("   - Session ID: {}", session.id);
    println!("   - User ID: {}", session.user_id);
    println!("   - Created at: {}", session.created_at);
    println!("   - Expires at: {}", session.expires_at);
    println!("   - Last accessed: {}\n", session.last_accessed);

    // 2. Retrieve the session
    println!("2. Retrieving session by ID...");
    let retrieved_session = session_store.get_session(SessionId(session.id)).await?;
    match retrieved_session {
        Some(s) => {
            println!("✓ Session retrieved:");
            println!("   - Session ID: {}", s.id);
            println!("   - User ID: {}", s.user_id);
        }
        None => println!("✗ Session not found or expired"),
    }
    println!();

    // 3. Update last accessed time
    println!("3. Updating session last accessed time...");
    session_store.update_last_accessed(SessionId(session.id)).await?;
    println!("✓ Session last accessed time updated\n");

    // 4. List all active sessions for the user
    println!("4. Listing all active sessions for user...");
    let user_sessions: Vec<_> = session_store.list_user_sessions(user_id).await?;
    println!("✓ Found {} active session(s):", user_sessions.len());
    for (i, s) in user_sessions.iter().enumerate() {
        println!(
            "   {}. Session ID: {} (last accessed: {})",
            i + 1,
            s.id,
            s.last_accessed
        );
    }
    println!();

    // 5. Create another session for the same user
    println!("5. Creating a second session for the same user...");
    let session2 = session_store.create_session(user_id).await?;
    println!("✓ Second session created: {}\n", session2.id);

    // 6. List sessions again
    println!("6. Listing sessions again...");
    let user_sessions = session_store.list_user_sessions(user_id).await?;
    println!("✓ Found {} active session(s)\n", user_sessions.len());

    // 7. Invalidate a specific session
    println!("7. Invalidating first session...");
    session_store.invalidate_session(SessionId(session.id)).await?;
    println!("✓ Session invalidated\n");

    // 8. Verify session is gone
    println!("8. Verifying session is invalidated...");
    let retrieved_session = session_store.get_session(SessionId(session.id)).await?;
    match retrieved_session {
        Some(_) => println!("✗ Session still exists (unexpected)"),
        None => println!("✓ Session successfully invalidated"),
    }
    println!();

    // 9. Invalidate all sessions for the user
    println!("9. Invalidating all sessions for user...");
    session_store.invalidate_user_sessions(user_id).await?;
    println!("✓ All user sessions invalidated\n");

    // 10. Verify all sessions are gone
    println!("10. Verifying all sessions are invalidated...");
    let user_sessions = session_store.list_user_sessions(user_id).await?;
    println!(
        "✓ Found {} active session(s) (should be 0)\n",
        user_sessions.len()
    );

    // 11. Cleanup expired sessions
    println!("11. Running expired session cleanup...");
    let cleaned_count = session_store.cleanup_expired_sessions().await?;
    println!("✓ Cleaned up {} expired session(s)\n", cleaned_count);

    // 12. Custom timeouts example
    println!("12. Creating session store with custom timeouts...");
    let custom_session_store = PostgresSessionStore::with_timeouts(
        Arc::clone(&db),
        30, // 30 minutes idle timeout
        12, // 12 hours absolute timeout
    );
    let custom_session = custom_session_store.create_session(user_id).await?;
    println!("✓ Session created with custom timeouts:");
    println!("   - Session ID: {}", custom_session.id);
    println!(
        "   - Expires at: {} (12 hours from now)\n",
        custom_session.expires_at
    );

    // Cleanup
    custom_session_store
        .invalidate_session(SessionId(custom_session.id))
        .await?;

    println!("=== Example completed successfully ===");

    Ok(())
}
