//! Example: Password hashing with Argon2id
//!
//! This example demonstrates how to use the Argon2PasswordHasher
//! for secure password hashing and verification.
//!
//! Run with:
//! ```bash
//! cargo run --example password_hashing
//! ```

use authenc_crypto::password::Argon2PasswordHasher;
use authenc_types::traits::PasswordHasher;

fn main() {
    println!("=== Argon2id Password Hashing Example ===\n");

    // Create a password hasher with default parameters
    let hasher = Argon2PasswordHasher::new();
    println!("Created Argon2PasswordHasher with default parameters:");
    println!("  - Memory cost: 64 MB (65536 KiB)");
    println!("  - Time cost: 3 iterations");
    println!("  - Parallelism: 4 threads");
    println!("  - Algorithm: Argon2id (hybrid mode)\n");

    // Example 1: Hash a password
    let password = "my-secure-password-123";
    println!("1. Hashing password: '{}'", password);

    let hash = hasher.hash(password).expect("Failed to hash password");
    println!("   Hash: {}\n", hash);

    // Example 2: Verify correct password
    println!("2. Verifying correct password...");
    let is_valid = hasher
        .verify(password, &hash)
        .expect("Failed to verify password");
    println!("   Result: {} ✓\n", is_valid);

    // Example 3: Verify incorrect password
    println!("3. Verifying incorrect password...");
    let is_valid = hasher
        .verify("wrong-password", &hash)
        .expect("Failed to verify password");
    println!("   Result: {} ✗\n", is_valid);

    // Example 4: Same password, different salts
    println!("4. Hashing the same password twice (different salts):");
    let hash1 = hasher.hash(password).expect("Failed to hash password");
    let hash2 = hasher.hash(password).expect("Failed to hash password");
    println!("   Hash 1: {}", hash1);
    println!("   Hash 2: {}", hash2);
    println!("   Are they equal? {}", hash1 == hash2);
    println!(
        "   Both verify correctly? {}\n",
        hasher.verify(password, &hash1).unwrap() && hasher.verify(password, &hash2).unwrap()
    );

    // Example 5: Custom parameters (higher security)
    println!("5. Using custom parameters (higher security):");
    let high_security_hasher = Argon2PasswordHasher::with_params(
        131072, // 128 MB memory
        4,      // 4 iterations
        8,      // 8 threads
    )
    .expect("Failed to create hasher with custom params");

    println!("   Memory cost: 128 MB (131072 KiB)");
    println!("   Time cost: 4 iterations");
    println!("   Parallelism: 8 threads");

    let high_security_hash = high_security_hasher
        .hash(password)
        .expect("Failed to hash password");
    println!("   Hash: {}\n", high_security_hash);

    // Example 6: Unicode password
    println!("6. Hashing Unicode password:");
    let unicode_password = "パスワード🔐";
    let unicode_hash = hasher
        .hash(unicode_password)
        .expect("Failed to hash Unicode password");
    println!("   Password: '{}'", unicode_password);
    println!("   Hash: {}", unicode_hash);
    println!(
        "   Verifies: {}\n",
        hasher.verify(unicode_password, &unicode_hash).unwrap()
    );

    // Example 7: Performance comparison
    println!("7. Performance comparison:");
    use std::time::Instant;

    let start = Instant::now();
    let _ = hasher.hash("test-password").unwrap();
    let default_duration = start.elapsed();
    println!("   Default params: {:?}", default_duration);

    let start = Instant::now();
    let _ = high_security_hasher.hash("test-password").unwrap();
    let high_security_duration = start.elapsed();
    println!("   High security params: {:?}", high_security_duration);
    println!(
        "   Slowdown factor: {:.2}x\n",
        high_security_duration.as_secs_f64() / default_duration.as_secs_f64()
    );

    println!("=== Example Complete ===");
}
