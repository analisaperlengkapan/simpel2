use argon2::{Algorithm, Argon2, ParamsBuilder, PasswordHasher, PasswordVerifier, Version, password_hash::{PasswordHash, SaltString}};
use rand::rngs::OsRng;

fn main() {
    let password = "199203142014031001";

    // Use PRODUCTION params (same as Argon2PasswordHasher::new())
    let params = ParamsBuilder::new()
        .m_cost(65536)
        .t_cost(3)   // Production uses 3, not 10!
        .p_cost(4)
        .build()
        .expect("Invalid params");
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    // Generate hash
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).expect("Failed to hash");
    println!("New hash (t=3): {}", password_hash.to_string());

    // Also verify the admin hash (t=10) works
    let admin_hash_str = "$argon2id$v=19$m=65536,t=10,p=4$8MuAQnFGugABncdYFSRtbQ$8NwOF/oRK5AOJkJpQXFvyXidQgCr8C1V7fH4KWS5YiE";
    let admin_hash = PasswordHash::new(admin_hash_str).expect("Failed to parse admin hash");
    match argon2.verify_password(password.as_bytes(), &admin_hash) {
        Ok(()) => println!("Admin hash verification with t=3 instance: OK"),
        Err(e) => println!("Admin hash verification with t=3 instance: FAILED - {}", e),
    }

    // Verify with t=10 instance too
    let params10 = ParamsBuilder::new()
        .m_cost(65536)
        .t_cost(10)
        .p_cost(4)
        .build()
        .expect("Invalid params");
    let argon2_10 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params10);
    match argon2_10.verify_password(password.as_bytes(), &admin_hash) {
        Ok(()) => println!("Admin hash verification with t=10 instance: OK"),
        Err(e) => println!("Admin hash verification with t=10 instance: FAILED - {}", e),
    }
}
