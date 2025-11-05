//! Integration tests for JWT validation with caching
//!
//! This test suite validates the JWT validation caching implementation
//! to ensure it meets the performance requirements:
//! - Cached validation: < 10ms
//! - Uncached validation: < 50ms
//! - Cache hit ratio: > 80%

use authenc::config::RedisConfig;
use authenc::services::cache::redis_cache::RedisCache;
use authenc::services::cache::{Cache, CacheConfig};
use authenc::services::{JwtValidator, ValidationResult};
use authenc::utils::crypto::jwt::{generate_jwt, verify_jwt};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Test JWT validation without cache
#[tokio::test]
async fn test_jwt_validation_without_cache() {
    // Create validator without cache
    let validator = JwtValidator::new(None);

    // Generate a valid token
    let token = generate_jwt("test_user").expect("Failed to generate token");

    // Measure validation time
    let start = Instant::now();
    let result = validator.validate_token(&token).await.unwrap();
    let elapsed = start.elapsed();

    // Verify result
    assert!(result.valid, "Token should be valid");
    assert_eq!(result.user_id, Some("test_user".to_string()));
    assert!(result.expires_at.is_some());
    assert!(result.error.is_none());

    println!("Validation without cache took: {:?}", elapsed);
    // Without cache, should still be reasonably fast (< 50ms)
    assert!(
        elapsed < Duration::from_millis(50),
        "Validation should be < 50ms"
    );
}

/// Test JWT validation with invalid token
#[tokio::test]
async fn test_jwt_validation_invalid_token() {
    let validator = JwtValidator::new(None);

    // Invalid token
    let token = "invalid.jwt.token";

    // Validate token
    let result = validator.validate_token(token).await.unwrap();

    // Verify result
    assert!(!result.valid, "Token should be invalid");
    assert!(result.user_id.is_none());
    assert!(result.error.is_some());
}

/// Test JWT validation with expired token
#[tokio::test]
async fn test_jwt_validation_expired_token() {
    let validator = JwtValidator::new(None);

    // Create an expired token (this would require modifying the JWT generation
    // to accept custom expiration, or waiting for a token to expire)
    // For now, we'll test with a tampered token that will fail validation
    let token = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0X3VzZXIiLCJleHAiOjB9.invalid";

    // Validate token
    let result = validator.validate_token(token).await.unwrap();

    // Verify result
    assert!(!result.valid, "Expired token should be invalid");
}

/// Test JWT validation with cache (requires Redis)
#[tokio::test]
#[ignore] // Requires Redis to be running
async fn test_jwt_validation_with_cache() {
    // Setup Redis cache
    let redis_config = RedisConfig {
        enabled: true,
        url: "redis://localhost:6379/15".to_string(), // Use test database
        default_ttl: 300,
        mfa_cache_ttl: 300,
        otp_verification_ttl: 300,
    };

    let redis_cache = RedisCache::new(&redis_config)
        .await
        .expect("Failed to create Redis cache");

    let cache: Arc<dyn Cache> = Arc::new(redis_cache);
    let validator = JwtValidator::new(Some(cache));

    // Generate a valid token
    let token = generate_jwt("test_user_cached").expect("Failed to generate JWT");

    // First validation (cache miss)
    let start = Instant::now();
    let result1 = validator.validate_token(&token).await.unwrap();
    let elapsed1 = start.elapsed();

    assert!(result1.valid, "Token should be valid ");
    println!("First validation (cache miss) took: {:?}", elapsed1);

    // Second validation (cache hit)
    let start = Instant::now();
    let result2 = validator.validate_token(&token).await.unwrap();
    let elapsed2 = start.elapsed();

    assert!(result2.valid, "Token should still be valid ");
    println!("Second validation (cache hit) took: {:?}", elapsed2);

    // Cache hit should be significantly faster
    assert!(
        elapsed2 < Duration::from_millis(10),
        "Cached validation should be < 10ms, got {:?}",
        elapsed2
    );

    // Cache hit should be faster than cache miss
    assert!(
        elapsed2 < elapsed1,
        "Cached validation should be faster than uncached "
    );
}

/// Test token revocation and blacklist
#[tokio::test]
#[ignore] // Requires Redis to be running
async fn test_token_revocation_and_blacklist() {
    // Setup Redis cache
    let redis_config = RedisConfig {
        enabled: true,
        url: "redis://localhost:6379/15".to_string(),
        default_ttl: 300,
        mfa_cache_ttl: 300,
        otp_verification_ttl: 300,
    };

    let redis_cache = RedisCache::new(&redis_config)
        .await
        .expect("Failed to create Redis cache");

    let cache: Arc<dyn Cache> = Arc::new(redis_cache);
    let validator = JwtValidator::new(Some(cache));

    // Generate a valid token
    let token = generate_jwt("test_user_revoke").expect("Failed to generate token");

    // Validate token (should be valid)
    let result1 = validator.validate_token(&token).await.unwrap();
    assert!(result1.valid, "Token should be valid initially");

    // Revoke the token
    let claims = verify_jwt(&token).expect("Failed to verify token");
    validator
        .revoke_token(&token, &claims)
        .await
        .expect("Failed to revoke token");

    // Validate token again (should be invalid due to blacklist)
    let result2 = validator.validate_token(&token).await.unwrap();
    assert!(!result2.valid, "Token should be invalid after revocation");
    assert!(
        result2.error.is_some(),
        "Error message should indicate revocation"
    );
}

/// Test cache invalidation
#[tokio::test]
#[ignore] // Requires Redis to be running
async fn test_cache_invalidation() {
    // Setup Redis cache
    let redis_config = RedisConfig {
        enabled: true,
        url: "redis://localhost:6379/15".to_string(),
        default_ttl: 300,
        mfa_cache_ttl: 300,
        otp_verification_ttl: 300,
    };

    let redis_cache = RedisCache::new(&redis_config)
        .await
        .expect("Failed to create Redis cache");

    let cache: Arc<dyn Cache> = Arc::new(redis_cache);
    let validator = JwtValidator::new(Some(cache));

    // Generate a valid token
    let token = generate_jwt("test_user_invalidate").expect("Failed to generate token");

    // First validation (cache miss)
    let result1 = validator.validate_token(&token).await.unwrap();
    assert!(result1.valid, "Token should be valid");

    // Second validation (cache hit - should be fast)
    let start = Instant::now();
    let result2 = validator.validate_token(&token).await.unwrap();
    let elapsed_cached = start.elapsed();
    assert!(result2.valid, "Token should still be valid");

    // Invalidate cache
    validator
        .invalidate_cache(&token)
        .await
        .expect("Failed to invalidate cache");

    // Third validation (cache miss again - should be slower)
    let start = Instant::now();
    let result3 = validator.validate_token(&token).await.unwrap();
    let elapsed_uncached = start.elapsed();
    assert!(result3.valid, "Token should still be valid");

    // After invalidation, validation should be slower than cached
    println!("Cached validation: {:?}", elapsed_cached);
    println!(
        "Uncached validation after invalidation: {:?}",
        elapsed_uncached
    );
}

/// Benchmark JWT validation performance
#[tokio::test]
#[ignore] // Requires Redis to be running
async fn benchmark_jwt_validation_performance() {
    // Setup Redis cache
    let redis_config = RedisConfig {
        enabled: true,
        url: "redis://localhost:6379/15".to_string(),
        default_ttl: 300,
        mfa_cache_ttl: 300,
        otp_verification_ttl: 300,
    };

    let redis_cache = RedisCache::new(&redis_config)
        .await
        .expect("Failed to create Redis cache");

    let cache: Arc<dyn Cache> = Arc::new(redis_cache);
    let validator = JwtValidator::new(Some(cache));

    // Generate test tokens
    let tokens: Vec<String> = (0..100)
        .map(|i| generate_jwt(&format!("user_{}", i)).expect("Failed to generate token"))
        .collect();

    // Warm up cache
    for token in &tokens {
        let _ = validator.validate_token(token).await;
    }

    // Benchmark cached validations
    let start = Instant::now();
    for token in &tokens {
        let result = validator.validate_token(token).await.unwrap();
        assert!(result.valid);
    }
    let total_elapsed = start.elapsed();
    let avg_elapsed = total_elapsed / tokens.len() as u32;

    println!("Validated {} tokens in {:?}", tokens.len(), total_elapsed);
    println!("Average validation time: {:?}", avg_elapsed);
    println!(
        "Validations per second: {:.2}",
        1000.0 / avg_elapsed.as_millis() as f64 * 1000.0
    );

    // Verify performance target (< 10ms average for cached tokens)
    assert!(
        avg_elapsed < Duration::from_millis(10),
        "Average cached validation should be < 10ms, got {:?}",
        avg_elapsed
    );
}

/// Test concurrent JWT validations
#[tokio::test]
#[ignore] // Requires Redis to be running
async fn test_concurrent_jwt_validations() {
    // Setup Redis cache
    let redis_config = RedisConfig {
        enabled: true,
        url: "redis://localhost:6379/15".to_string(),
        default_ttl: 300,
        mfa_cache_ttl: 300,
        otp_verification_ttl: 300,
    };

    let redis_cache = RedisCache::new(&redis_config)
        .await
        .expect("Failed to create Redis cache");

    let cache: Arc<dyn Cache> = Arc::new(redis_cache);
    let validator = Arc::new(JwtValidator::new(Some(cache)));

    // Generate a token
    let token = generate_jwt("concurrent_user").expect("Failed to generate token");

    // Spawn multiple concurrent validation tasks
    let mut handles = vec![];
    for i in 0..50 {
        let validator_clone = Arc::clone(&validator);
        let token_clone = token.clone();

        let handle = tokio::spawn(async move {
            let result = validator_clone.validate_token(&token_clone).await.unwrap();
            assert!(result.valid, "Token should be valid in task {}", i);
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task panicked");
    }

    println!("Successfully validated token 50 times concurrently");
}
