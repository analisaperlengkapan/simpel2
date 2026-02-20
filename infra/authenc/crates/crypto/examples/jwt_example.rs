//! Example demonstrating JWT token generation and validation with Ed25519
//!
//! Run with: cargo run --package authenc-crypto --example jwt_example

use authenc_crypto::jwt::{JwtService, TokenClaims};
use chrono::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== JWT Service Example with Ed25519 ===\n");

    // 1. Generate a new signing key (in production, load from Secreton)
    println!("1. Generating Ed25519 signing key...");
    let signing_key = JwtService::generate_signing_key();
    println!("   ✓ Generated 32-byte Ed25519 private key\n");

    // 2. Create JWT service
    println!("2. Creating JWT service...");
    let service = JwtService::new(
        &signing_key,
        "https://authenc.kejaksaan.go.id".to_string(),
        Duration::minutes(15),  // Access token: 15 minutes
        Duration::days(7),      // Refresh token: 7 days
    )?;
    println!("   ✓ JWT service initialized");
    println!("   Issuer: https://authenc.kejaksaan.go.id");
    println!("   Access token TTL: 15 minutes");
    println!("   Refresh token TTL: 7 days\n");

    // 3. Export public key (for verification by other services)
    println!("3. Exporting public key...");
    let public_key = service.get_public_key_base64();
    println!("   ✓ Public key (base64): {}\n", public_key);

    // 4. Generate access token
    println!("4. Generating access token...");
    let access_token = service.generate_access_token(
        "user-12345",
        Some("kejaksaan-ri".to_string()),
        Some("openid profile email".to_string()),
        Some("session-67890".to_string()),
    )?;
    println!("   ✓ Access token generated:");
    println!("   {}\n", access_token);

    // 5. Verify and decode access token
    println!("5. Verifying access token...");
    let claims = service.verify_token(&access_token)?;
    println!("   ✓ Token verified successfully!");
    println!("   Subject (user ID): {}", claims.sub);
    println!("   Issuer: {}", claims.iss);
    println!("   Realm: {}", claims.realm.as_ref().unwrap());
    println!("   Scope: {}", claims.scope.as_ref().unwrap());
    println!("   Session ID: {}", claims.sid.as_ref().unwrap());
    println!("   JWT ID: {}", claims.jti);
    println!("   Expires at: {}", chrono::DateTime::from_timestamp(claims.exp, 0).unwrap());
    println!("   Issued at: {}", chrono::DateTime::from_timestamp(claims.iat, 0).unwrap());
    println!("   Is expired: {}\n", claims.is_expired());

    // 6. Generate refresh token
    println!("6. Generating refresh token...");
    let refresh_token = service.generate_refresh_token("user-12345", "session-67890")?;
    println!("   ✓ Refresh token generated:");
    println!("   {}\n", refresh_token);

    // 7. Verify refresh token
    println!("7. Verifying refresh token...");
    let refresh_claims = service.verify_token(&refresh_token)?;
    println!("   ✓ Refresh token verified!");
    println!("   Subject: {}", refresh_claims.sub);
    println!("   Scope: {}", refresh_claims.scope.as_ref().unwrap());
    println!("   Session ID: {}\n", refresh_claims.sid.as_ref().unwrap());

    // 8. Generate token with custom claims
    println!("8. Generating token with custom claims...");
    let mut custom_claims = TokenClaims::new(
        "admin-user".to_string(),
        service.issuer().to_string(),
        Duration::minutes(15),
    );
    custom_claims = custom_claims
        .with_realm("kejaksaan-ri".to_string())
        .with_scope("admin:read admin:write".to_string())
        .with_custom_claim("role".to_string(), serde_json::json!("admin"))
        .with_custom_claim("department".to_string(), serde_json::json!("IT"))
        .with_custom_claim("permissions".to_string(), serde_json::json!(["users:read", "users:write", "realms:manage"]));

    let custom_token = service.generate_token(&custom_claims)?;
    println!("   ✓ Custom token generated");

    let decoded_custom = service.verify_token(&custom_token)?;
    println!("   Custom claims:");
    println!("   - role: {}", decoded_custom.custom.get("role").unwrap());
    println!("   - department: {}", decoded_custom.custom.get("department").unwrap());
    println!("   - permissions: {}\n", decoded_custom.custom.get("permissions").unwrap());

    // 9. Demonstrate signature verification failure
    println!("9. Testing signature verification...");
    let another_key = JwtService::generate_signing_key();
    let another_service = JwtService::new(
        &another_key,
        "https://authenc.kejaksaan.go.id".to_string(),
        Duration::minutes(15),
        Duration::days(7),
    )?;

    match another_service.verify_token(&access_token) {
        Ok(_) => println!("   ✗ Unexpected: Token verified with wrong key!"),
        Err(e) => println!("   ✓ Expected failure: {}\n", e),
    }

    // 10. Demonstrate issuer validation
    println!("10. Testing issuer validation...");
    let wrong_issuer_service = JwtService::new(
        &signing_key,
        "https://wrong-issuer.com".to_string(),
        Duration::minutes(15),
        Duration::days(7),
    )?;

    match wrong_issuer_service.verify_token(&access_token) {
        Ok(_) => println!("   ✗ Unexpected: Token verified with wrong issuer!"),
        Err(e) => println!("   ✓ Expected failure: {}\n", e),
    }

    println!("=== Example completed successfully! ===");

    Ok(())
}
