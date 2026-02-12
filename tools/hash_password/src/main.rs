use argon2::{
    Algorithm, Argon2, Params, PasswordHasher, Version,
    password_hash::SaltString,
};
use rand_core::OsRng;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "captcha" {
        // Generate CAPTCHA token
        generate_captcha_token();
    } else {
        // Generate password hash
        let password = "SIMPelv2@2026!";
        let salt = SaltString::generate(&mut OsRng);
        let params = Params::new(65536, 10, 4, Some(32)).unwrap();
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
        println!("{}", hash.to_string());
    }
}

fn generate_captcha_token() {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use hkdf::Hkdf;

    let secret = "default_jwt_secret_change_in_production";
    let challenge_id = "test-challenge-001";
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let payload = format!("{}:{}", challenge_id, timestamp);

    // HKDF derive key
    let hk = Hkdf::<Sha256>::new(None, secret.as_bytes());
    let mut captcha_key = [0u8; 32];
    hk.expand(b"captcha-v1", &mut captcha_key).unwrap();

    // HMAC-SHA256
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(&captcha_key).unwrap();
    mac.update(payload.as_bytes());
    let result = mac.finalize();
    let signature = hex::encode(result.into_bytes());

    let token = format!("{}:{}:{}", challenge_id, timestamp, signature);
    println!("{}", token);
}
