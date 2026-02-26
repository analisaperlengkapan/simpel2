//! SSH Enhancement Demo
//!
//! This example demonstrates the new SSH engine enhancements:
//! - TTL validation with min/max bounds
//! - Principal embedding in certificates
//! - Extension support
//! - Host certificate signing
//! - Certificate audit

use secreton_core::services::secrets::ssh::{
    SshCertificateRequest, SshEngine, SshHostCertificateRequest, SshKeyType, SshRole,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SSH Engine Enhancement Demo ===\n");

    // Create SSH engine
    let engine = SshEngine::new();

    // 1. Create SSH CA
    println!("1. Creating SSH CA...");
    engine
        .create_ca("demo-ca".to_string(), SshKeyType::Ed25519)
        .await?;
    println!("   ✓ CA created\n");

    // 2. Create role with TTL bounds
    println!("2. Creating SSH role with TTL bounds...");
    let mut role = SshRole::new(
        "demo-role".to_string(),
        SshKeyType::Ed25519,
        "demouser".to_string(),
    );
    role.min_ttl = 60; // 1 minute
    role.max_ttl = 86400; // 24 hours
    role.default_ttl = 28800; // 8 hours
    role.allowed_users = vec!["alice".to_string(), "bob".to_string()];
    role.allow_host_certificates = true;

    // Add allowed extensions
    role.allowed_extensions
        .insert("permit-pty".to_string(), "".to_string());
    role.allowed_extensions
        .insert("permit-port-forwarding".to_string(), "".to_string());

    engine.create_role(role).await?;
    println!("   ✓ Role created with:");
    println!("     - min_ttl: 60s");
    println!("     - max_ttl: 86400s");
    println!("     - default_ttl: 28800s");
    println!("     - Extensions: permit-pty, permit-port-forwarding\n");

    // 3. Test TTL validation
    println!("3. Testing TTL validation...");
    let role = engine.get_role("demo-role").await.unwrap();

    // Valid TTL
    match role.validate_ttl(3600) {
        Ok(_) => println!("   ✓ Valid TTL (3600s) accepted"),
        Err(e) => println!("   ✗ Error: {}", e),
    }

    // Below minimum
    match role.validate_ttl(30) {
        Ok(_) => println!("   ✗ TTL below minimum should be rejected"),
        Err(_) => println!("   ✓ TTL below minimum (30s) rejected"),
    }

    // Above maximum
    match role.validate_ttl(100000) {
        Ok(_) => println!("   ✗ TTL above maximum should be rejected"),
        Err(_) => println!("   ✓ TTL above maximum (100000s) rejected"),
    }
    println!();

    // 4. Generate keypair and sign user certificate with principals
    println!("4. Signing user certificate with principals...");
    let keypair = engine.generate_keypair("demo-role").await?;

    let mut extensions = HashMap::new();
    extensions.insert("permit-pty".to_string(), "".to_string());

    let request = SshCertificateRequest {
        public_key: keypair.public_key,
        cert_type: "user".to_string(),
        valid_principals: vec!["alice".to_string(), "bob".to_string()],
        ttl: Some(3600),
        extensions: Some(extensions),
    };

    let cert = engine
        .sign_certificate("demo-ca", "demo-role", request)
        .await?;
    println!("   ✓ User certificate signed");
    println!("     - Serial: {}", cert.serial_number);
    println!("     - Type: {}", cert.cert_type);
    println!();

    // 5. Sign host certificate
    println!("5. Signing host certificate...");
    let host_keypair = engine.generate_keypair("demo-role").await?;

    let host_request = SshHostCertificateRequest {
        public_key: host_keypair.public_key,
        hostnames: vec![
            "server1.example.com".to_string(),
            "server2.example.com".to_string(),
        ],
        ttl: Some(7200),
    };

    let host_cert = engine
        .sign_host_certificate("demo-ca", "demo-role", host_request)
        .await?;
    println!("   ✓ Host certificate signed");
    println!("     - Serial: {}", host_cert.serial_number);
    println!("     - Type: {}", host_cert.cert_type);
    println!();

    // 6. Get certificate audit
    println!("6. Retrieving certificate audit...");
    let audit = engine.get_certificate_audit().await;
    println!("   ✓ Found {} active certificates:", audit.len());

    for (i, cert_meta) in audit.iter().enumerate() {
        println!("\n   Certificate {}:", i + 1);
        println!("     - Serial: {}", cert_meta.serial_number);
        println!("     - Type: {}", cert_meta.cert_type);
        println!("     - Principals: {:?}", cert_meta.principals);
        println!("     - CA: {}", cert_meta.ca_name);
        println!("     - Role: {}", cert_meta.role_name);
        println!("     - Valid from: {}", cert_meta.valid_after);
        println!("     - Valid until: {}", cert_meta.valid_before);
        println!("     - Extensions: {:?}", cert_meta.extensions);
    }

    println!("\n=== Demo Complete ===");
    Ok(())
}
