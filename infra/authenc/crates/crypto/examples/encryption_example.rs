//! Example demonstrating EncryptionService usage
//!
//! Run with: cargo run --example encryption_example

use authenc_crypto::encryption::{key_derivation, EncryptionService};

fn main() {
    println!("=== ChaCha20-Poly1305 Encryption Example ===\n");

    // Example 1: Basic encryption/decryption
    println!("1. Basic Encryption/Decryption:");
    let service = EncryptionService::new_with_random_key();
    let plaintext = b"This is a secret message!";

    let encrypted = service.encrypt(plaintext).expect("Encryption failed");
    println!("   Plaintext: {:?}", String::from_utf8_lossy(plaintext));
    println!("   Ciphertext length: {} bytes", encrypted.ciphertext.len());
    println!("   Nonce length: {} bytes", encrypted.nonce.len());

    let decrypted = service.decrypt(&encrypted).expect("Decryption failed");
    println!("   Decrypted: {:?}", String::from_utf8_lossy(&decrypted));
    assert_eq!(plaintext, decrypted.as_slice());
    println!("   ✓ Roundtrip successful!\n");

    // Example 2: Base64 encoding
    println!("2. Base64 Encoding:");
    let encoded = service
        .encrypt_to_base64(plaintext)
        .expect("Encryption failed");
    println!("   Base64 encoded: {}", encoded);

    let decoded = service
        .decrypt_from_base64(&encoded)
        .expect("Decryption failed");
    println!("   Decoded: {:?}", String::from_utf8_lossy(&decoded));
    assert_eq!(plaintext, decoded.as_slice());
    println!("   ✓ Base64 roundtrip successful!\n");

    // Example 3: Key derivation from password
    println!("3. Key Derivation from Password:");
    let password = "my-secure-password-123";
    let salt = key_derivation::generate_salt();
    println!("   Password: {}", password);
    println!("   Salt length: {} bytes", salt.len());

    let derived_key = key_derivation::derive_key_from_password(password, &salt)
        .expect("Key derivation failed");
    println!("   Derived key length: {} bytes", derived_key.len());

    // Use derived key for encryption
    let service_with_derived_key =
        EncryptionService::new(&derived_key).expect("Failed to create service");
    let encrypted = service_with_derived_key
        .encrypt(plaintext)
        .expect("Encryption failed");
    let decrypted = service_with_derived_key
        .decrypt(&encrypted)
        .expect("Decryption failed");
    assert_eq!(plaintext, decrypted.as_slice());
    println!("   ✓ Encryption with derived key successful!\n");

    // Example 4: Key derivation consistency
    println!("4. Key Derivation Consistency:");
    let key1 = key_derivation::derive_key_from_password(password, &salt)
        .expect("Key derivation failed");
    let key2 = key_derivation::derive_key_from_password(password, &salt)
        .expect("Key derivation failed");
    assert_eq!(key1, key2);
    println!("   ✓ Same password + salt = same key\n");

    // Example 5: Different salts produce different keys
    println!("5. Different Salts:");
    let salt2 = key_derivation::generate_salt();
    let key3 = key_derivation::derive_key_from_password(password, &salt2)
        .expect("Key derivation failed");
    assert_ne!(key1, key3);
    println!("   ✓ Different salt = different key\n");

    println!("=== All examples completed successfully! ===");
}
