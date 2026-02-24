//! Integration tests for JWT token generation in gRPC responses
//!
//! Tests task 10.6: Implement JWT token generation for gRPC responses

use authenc_crypto::jwt::JwtService;
use authenc_types::config::JwtConfig;
use chrono::Duration;

#[test]
fn test_jwt_service_initialization() {
    // Generate a signing key
    let signing_key = JwtService::generate_signing_key();

    // Create JWT service with configuration
    let config = JwtConfig::default();
    let jwt_service = JwtService::new(
        &signing_key,
        config.issuer.clone(),
        config.access_token_ttl(),
        config.refresh_token_ttl(),
    )
    .expect("Failed to create JWT service");

    // Verify issuer
    assert_eq!(jwt_service.issuer(), &config.issuer);
}

#[test]
fn test_access_token_generation() {
    let signing_key = JwtService::generate_signing_key();
    let config = JwtConfig::default();
    let jwt_service = JwtService::new(
        &signing_key,
        config.issuer.clone(),
        config.access_token_ttl(),
        config.refresh_token_ttl(),
    )
    .unwrap();

    // Generate access token
    let user_id = "user-123";
    let realm = Some("master".to_string());
    let scope = Some("openid profile".to_string());
    let session_id = Some("session-456".to_string());

    let token = jwt_service
        .generate_access_token(user_id, realm.clone(), scope.clone(), session_id.clone())
        .expect("Failed to generate access token");

    // Verify token is not empty
    assert!(!token.is_empty());

    // Verify token format (JWT has 3 parts separated by dots)
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3, "JWT should have 3 parts");
}

#[test]
fn test_access_token_validation() {
    let signing_key = JwtService::generate_signing_key();
    let config = JwtConfig::default();
    let jwt_service = JwtService::new(
        &signing_key,
        config.issuer.clone(),
        config.access_token_ttl(),
        config.refresh_token_ttl(),
    )
    .unwrap();

    // Generate and validate access token
    let user_id = "user-123";
    let token = jwt_service
        .generate_access_token(user_id, None, None, None)
        .unwrap();

    let claims = jwt_service
        .verify_token(&token)
        .expect("Failed to verify token");

    // Verify claims
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.iss, config.issuer);
    assert!(!claims.is_expired());
}

#[test]
fn test_refresh_token_generation() {
    let signing_key = JwtService::generate_signing_key();
    let config = JwtConfig::default();
    let jwt_service = JwtService::new(
        &signing_key,
        config.issuer.clone(),
        config.access_token_ttl(),
        config.refresh_token_ttl(),
    )
    .unwrap();

    // Generate refresh token
    let user_id = "user-123";
    let session_id = "session-456";

    let token = jwt_service
        .generate_refresh_token(user_id, session_id)
        .expect("Failed to generate refresh token");

    // Verify token is not empty
    assert!(!token.is_empty());

    // Verify token
    let claims = jwt_service
        .verify_token(&token)
        .expect("Failed to verify refresh token");

    // Verify it's a refresh token
    assert_eq!(claims.scope, Some("refresh_token".to_string()));
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.sid, Some(session_id.to_string()));
}

#[test]
fn test_token_expiration_configuration() {
    let signing_key = JwtService::generate_signing_key();

    // Custom configuration with shorter TTL
    let custom_config = JwtConfig {
        issuer: "https://test.example.com".to_string(),
        access_token_ttl_seconds: 60,    // 1 minute
        refresh_token_ttl_seconds: 3600, // 1 hour
        signing_key_path: None,
    };

    let jwt_service = JwtService::new(
        &signing_key,
        custom_config.issuer.clone(),
        custom_config.access_token_ttl(),
        custom_config.refresh_token_ttl(),
    )
    .unwrap();

    // Generate token
    let token = jwt_service
        .generate_access_token("user-123", None, None, None)
        .unwrap();

    let claims = jwt_service.verify_token(&token).unwrap();

    // Verify expiration is approximately 1 minute from now
    let now = chrono::Utc::now().timestamp();
    let exp_diff = claims.exp - now;
    assert!(
        exp_diff >= 55 && exp_diff <= 65,
        "Expiration should be ~60 seconds"
    );
}

#[test]
fn test_token_rotation() {
    let signing_key = JwtService::generate_signing_key();
    let config = JwtConfig::default();
    let jwt_service = JwtService::new(
        &signing_key,
        config.issuer.clone(),
        config.access_token_ttl(),
        config.refresh_token_ttl(),
    )
    .unwrap();

    // Generate first refresh token
    let token1 = jwt_service
        .generate_refresh_token("user-123", "session-456")
        .unwrap();

    // Generate second refresh token (rotation)
    let token2 = jwt_service
        .generate_refresh_token("user-123", "session-456")
        .unwrap();

    // Tokens should be different (different JTI)
    assert_ne!(token1, token2);

    // Both should be valid
    let claims1 = jwt_service.verify_token(&token1).unwrap();
    let claims2 = jwt_service.verify_token(&token2).unwrap();

    assert_eq!(claims1.sub, claims2.sub);
    assert_ne!(claims1.jti, claims2.jti);
}
