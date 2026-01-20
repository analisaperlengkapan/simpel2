//! Quick utility to generate Argon2 password hash
//! Usage: cargo run --example hash_password

use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version};
use rand::rngs::OsRng;

fn main() {
    let password = "admin123";
    let salt = SaltString::generate(&mut OsRng);

    // Configure Argon2id with same parameters as authenc
    let params = Params::new(
        65536,    // m_cost: 64 MB memory
        10,       // t_cost: 10 iterations
        4,        // p_cost: 4 parallel threads
        Some(32), // output length: 32 bytes
    )
    .expect("Invalid params");

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password");

    println!("Password: {}", password);
    println!("Hash: {}", password_hash.to_string());
}
