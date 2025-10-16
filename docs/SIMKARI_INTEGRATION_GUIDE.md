# SIMKARI Integration Guide

## Overview

This guide provides comprehensive instructions for integrating with the SIMKARI (Sistem Informasi Manajemen Kejaksaan Republik Indonesia) super app platform's enhanced authentication and secret management systems.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Authentication Integration](#authentication-integration)
3. [Secret Management Integration](#secret-management-integration)
4. [Role-Based Access Control](#role-based-access-control)
5. [Post-Quantum Cryptography](#post-quantum-cryptography)
6. [Microfrontend Integration](#microfrontend-integration)
7. [Best Practices](#best-practices)
8. [Troubleshooting](#troubleshooting)

## Quick Start

### Prerequisites

- Rust 1.70+ with async/await support
- Access to SIMKARI authenc and secreton services
- Valid NIP (Nomor Induk Pegawai) and satker code
- mTLS certificates for service communication

### Basic Setup

```rust
// Cargo.toml
[dependencies]
authenc = { path = "../../infra/authenc" }
secreton = { path = "../../infra/secreton" }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.0", features = ["v4"] }

// main.rs
use authenc::{AuthencService, models::{User, Credentials}};
use secreton::{SecretonService, auth::AuthencAuthProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize services
    let authenc = AuthencService::new().await?;
    let secreton = SecretonService::new().await?;

    println!("SIMKARI services initialized successfully");
    Ok(())
}
```

## Authentication Integration

### User Authentication

```rust
use authenc::{AuthencService, models::{Credentials, Claims}};
use chrono::{Utc, Duration};

/// Authenticate a government employee
async fn authenticate_pegawai(
    authenc: &AuthencService,
    nip: &str,
    password: &str,
) -> Result<String, authenc::error::AuthencError> {
    // Create credentials
    let credentials = Credentials::Password {
        nip: nip.to_string(),
        password: password.to_string(),
    };

    // Authenticate user
    let auth_result = authenc
        .authenticate_user(nip, &credentials)
        .await?;

    if !auth_result.success {
        return Err(authenc::error::AuthencError::InvalidCredentials {
            nip: nip.to_string(),
        });
    }

    // Generate JWT token
    let claims = Claims {
        sub: auth_result.user.nip.clone(),
        satker_code: auth_result.user.satker_code.clone(),
        roles: auth_result.user.roles.iter().map(|r| r.name.clone()).collect(),
        exp: Utc::now() + Duration::hours(8),
        iat: Utc::now(),
        iss: "simkari-authenc".to_string(),
        aud: "simkari-services".to_string(),
    };

    let token = authenc.crypto_engine()
        .sign_jwt_for_government(&claims)
        .await?;

    println!("User authenticated: {}", auth_result.user.nama);
    println!("Satker: {}", auth_result.user.satker_code);
    println!("Roles: {:?}", auth_result.user.roles.iter().map(|r| &r.name).collect::<Vec<_>>());

    Ok(token)
}
```

### Token Validation

```rust
use secreton::auth::AuthencAuthProvider;

/// Validate JWT token in secreton
async fn validate_token_in_secreton(
    token: &str,
) -> Result<bool, secreton::error::SecretonError> {
    let auth_provider = AuthencAuthProvider::new("https://authenc.internal").await?;

    let validation = auth_provider.validate_token(token).await?;

    if validation.valid {
        println!("Token valid for user: {}", validation.claims.sub);
        println!("Expires at: {}", validation.claims.exp);
        println!("Satker: {}", validation.claims.satker_code);
    }

    Ok(validation.valid)
}
```

### Session Management

```rust
use authenc::models::{SessionData, EncryptedSessionData};

/// Manage user sessions with encryption
async fn manage_user_session(
    authenc: &AuthencService,
    user: &User,
) -> Result<EncryptedSessionData, authenc::error::AuthencError> {
    let session_data = SessionData {
        user_id: user.id,
        nip: user.nip.clone(),
        satker_code: user.satker_code.clone(),
        permissions: user.permissions.clone(),
        roles: user.roles.clone(),
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(8),
        last_activity: Utc::now(),
    };

    let encrypted_session = authenc.crypto_engine()
        .encrypt_session_data(&session_data)
        .await?;

    println!("Session created for user: {}", user.nip);
    println!("Session expires at: {}", session_data.expires_at);

    Ok(encrypted_session)
}
```

## Secret Management Integration

### Application Configuration

```rust
use secreton::engines::EnhancedSecretEngine;

/// Get application configuration from secreton
async fn get_application_config(
    app_id: &str,
) -> Result<ApplicationConfig, secreton::error::SecretonError> {
    let secret_engine = EnhancedSecretEngine::new().await?;

    let config = secret_engine
        .get_application_config(app_id)
        .await?;

    println!("Configuration loaded for app: {}", app_id);
    println!("Database URL: {}", config.database_url);
    println!("Redis URL: {}", config.redis_url);
    println!("API Keys: {} configured", config.api_keys.len());

    Ok(config)
}
```

### Satker-Specific Secrets

```rust
use secreton::models::{Secret, AccessControl};

/// Access satker-specific secrets
async fn access_satker_secrets(
    secret_engine: &EnhancedSecretEngine,
    satker_code: &str,
    user_nip: &str,
) -> Result<Vec<Secret>, secreton::error::SecretonError> {
    let secret_paths = vec![
        format!("{}/database_password", satker_code),
        format!("{}/encryption_key", satker_code),
        format!("{}/signing_certificate", satker_code),
    ];

    let mut secrets = Vec::new();

    for path in secret_paths {
        // Check access permissions first
        let has_access = secret_engine
            .validate_user_secret_access(user_nip, &path)
            .await?;

        if has_access {
            let secret = secret_engine
                .get_application_secret("simkari_portal", &path)
                .await?;

            println!("Retrieved secret: {}", secret.path);
            println!("Owner satker: {}", secret.satker_owner);

            secrets.push(secret);
        } else {
            println!("Access denied to secret: {}", path);
        }
    }

    Ok(secrets)
}
```

### Batch Operations

```rust
/// Perform batch secret operations for performance
async fn batch_secret_operations(
    secret_engine: &EnhancedSecretEngine,
    resource_ids: &[String],
) -> Result<Vec<Secret>, secreton::error::SecretonError> {
    // Batch retrieval is more efficient than individual calls
    let secrets = secret_engine
        .batch_get_secrets(resource_ids)
        .await?;

    println!("Retrieved {} secrets in batch operation", secrets.len());

    for (id, secret) in resource_ids.iter().zip(secrets.iter()) {
        println!("Resource {}: {}", id, secret.path);
        println!("  Created by: {}", secret.metadata.created_by_nip.unwrap_or_default());
        println!("  Audit required: {}", secret.access_control.audit_required);
    }

    Ok(secrets)
}
```

## Role-Based Access Control

### Role Assignment

```rust
use authenc::models::{Role, RoleScope, AdminLevel, Permission};

/// Assign roles to users based on organizational hierarchy
async fn assign_user_roles(
    authenc: &AuthencService,
    user_nip: &str,
    admin_level: AdminLevel,
) -> Result<(), authenc::error::AuthencError> {
    // Only admins can assign roles
    match admin_level {
        AdminLevel::AdminSatker(satker_code) => {
            // Satker admin can only assign satker-level roles
            let role = Role::new(
                "Jaksa Penuntut",
                RoleScope::Satker(satker_code.clone()),
                vec![
                    Permission::ReadCases,
                    Permission::WriteCases,
                    Permission::ReadEvidence,
                    Permission::AccessPidsus,
                ],
                admin_level,
            )?;

            authenc.assign_role_to_user(user_nip, &role).await?;
            println!("Assigned satker role to user: {}", user_nip);
        }

        AdminLevel::AdminWilayah(wilayah_code) => {
            // Wilayah admin can assign wilayah and satker roles
            let role = Role::new(
                "Supervisor Wilayah",
                RoleScope::Wilayah(wilayah_code.clone()),
                vec![
                    Permission::ManageUsers,
                    Permission::ReadAuditLogs,
                    Permission::AccessPortal,
                    Permission::ReadCases,
                ],
                admin_level,
            )?;

            authenc.assign_role_to_user(user_nip, &role).await?;
            println!("Assigned wilayah role to user: {}", user_nip);
        }

        AdminLevel::AdminPusat => {
            // Central admin can assign any role
            let role = Role::new(
                "Administrator Pusat",
                RoleScope::Pusat,
                vec![
                    Permission::SystemAdmin,
                    Permission::ManageUsers,
                    Permission::ManageRoles,
                    Permission::ReadAuditLogs,
                ],
                admin_level,
            )?;

            authenc.assign_role_to_user(user_nip, &role).await?;
            println!("Assigned central admin role to user: {}", user_nip);
        }

        _ => {
            return Err(authenc::error::AuthencError::InsufficientPermissions {
                operation: "assign_roles".to_string(),
            });
        }
    }

    Ok(())
}
```

### Permission Checking

```rust
/// Check user permissions for specific operations
async fn check_user_permissions(
    user: &User,
    operation: &str,
    resource: &str,
) -> Result<bool, authenc::error::AuthencError> {
    // Check if user has required permission
    let has_permission = user.has_permission(operation, Some(resource))?;

    if has_permission {
        println!("User {} has permission for {} on {}", user.nip, operation, resource);
    } else {
        println!("User {} lacks permission for {} on {}", user.nip, operation, resource);
    }

    // Additional checks for satker-specific resources
    if resource.contains('/') {
        let parts: Vec<&str> = resource.split('/').collect();
        if let Some(resource_satker) = parts.first() {
            let accessible_satker = user.get_accessible_satker();

            if !accessible_satker.contains(&resource_satker.to_string()) {
                println!("User cannot access resources from satker: {}", resource_satker);
                return Ok(false);
            }
        }
    }

    Ok(has_permission)
}
```

### Hierarchical Access Control

```rust
/// Implement hierarchical access control
async fn hierarchical_access_example(
    user: &User,
    target_satker: &str,
) -> Result<bool, authenc::error::AuthencError> {
    for role in &user.roles {
        match &role.scope {
            RoleScope::Pusat => {
                // Central roles can access any satker
                println!("Central role grants access to all satker");
                return Ok(true);
            }

            RoleScope::Wilayah(wilayah_code) => {
                // Check if target satker is in this wilayah
                if is_satker_in_wilayah(target_satker, wilayah_code) {
                    println!("Wilayah role grants access to satker: {}", target_satker);
                    return Ok(true);
                }
            }

            RoleScope::Satker(satker_code) => {
                // Direct satker match
                if satker_code == target_satker {
                    println!("Satker role grants access to own satker: {}", target_satker);
                    return Ok(true);
                }
            }
        }
    }

    println!("No role grants access to satker: {}", target_satker);
    Ok(false)
}

/// Helper function to check if satker belongs to wilayah
fn is_satker_in_wilayah(satker_code: &str, wilayah_code: &str) -> bool {
    // Implementation would check organizational hierarchy
    // This is a simplified example
    match wilayah_code {
        "SUMUT" => satker_code.starts_with("KJA0"), // North Sumatra
        "JABAR" => satker_code.starts_with("KJA1"), // West Java
        "JATENG" => satker_code.starts_with("KJA2"), // Central Java
        _ => false,
    }
}
```

## Post-Quantum Cryptography

### Hybrid Cryptography Setup

```rust
use secreton::crypto::{HybridCrypto, CryptoMode, PqAlgorithm};

/// Initialize hybrid cryptographic system
async fn setup_hybrid_crypto() -> Result<HybridCrypto, secreton::error::SecretonError> {
    // Start with hybrid mode for gradual transition
    let hybrid_crypto = HybridCrypto::new(CryptoMode::Hybrid)?;

    println!("Hybrid cryptography initialized");
    println!("Mode: Classical + Post-Quantum");

    Ok(hybrid_crypto)
}
```

### Document Signing with Post-Quantum

```rust
/// Sign legal documents with hybrid cryptography
async fn sign_legal_document(
    hybrid_crypto: &HybridCrypto,
    document: &[u8],
    signer_nip: &str,
) -> Result<HybridSignature, secreton::error::SecretonError> {
    let key_id = format!("legal_signing_key_{}", signer_nip);

    let signature = hybrid_crypto
        .sign_hybrid(document, &key_id)
        .await?;

    println!("Document signed with hybrid cryptography");
    println!("Signer NIP: {}", signer_nip);
    println!("Classical signature length: {}", signature.classical.len());
    println!("Post-quantum signature length: {}", signature.post_quantum.len());

    Ok(signature)
}
```

### Key Management

```rust
use secreton::crypto::{PqKeyManager, KeyMetadata, KeyUsage};

/// Manage post-quantum keys
async fn manage_pq_keys(
    pq_manager: &PqKeyManager,
    satker_code: &str,
) -> Result<(), secreton::error::SecretonError> {
    // Generate signing key for the satker
    let signing_key = pq_manager
        .generate_key_pair(
            PqAlgorithm::MlDsa65,
            &format!("signing_key_{}_{}", satker_code, Utc::now().year()),
            KeyMetadata {
                usage: KeyUsage::Signing,
                expires_at: Utc::now() + Duration::days(365),
                satker_code: Some(satker_code.to_string()),
                compliance_flags: vec!["KEJAKSAAN_LEGAL".to_string()],
            }
        )
        .await?;

    println!("Generated signing key for satker: {}", satker_code);

    // Generate encryption key for sensitive data
    let encryption_key = pq_manager
        .generate_key_pair(
            PqAlgorithm::MlKem768,
            &format!("encryption_key_{}_{}", satker_code, Utc::now().year()),
            KeyMetadata {
                usage: KeyUsage::Encryption,
                expires_at: Utc::now() + Duration::days(365),
                satker_code: Some(satker_code.to_string()),
                compliance_flags: vec!["KEJAKSAAN_SENSITIVE".to_string()],
            }
        )
        .await?;

    println!("Generated encryption key for satker: {}", satker_code);

    Ok(())
}
```

## Microfrontend Integration

### Portal Integration

```rust
use serde::{Deserialize, Serialize};

/// Configuration for microfrontend integration
#[derive(Debug, Serialize, Deserialize)]
pub struct MicrofrontendConfig {
    pub app_id: String,
    pub base_url: String,
    pub auth_endpoint: String,
    pub secret_endpoint: String,
    pub allowed_origins: Vec<String>,
}

/// Initialize microfrontend with SIMKARI services
async fn initialize_microfrontend(
    config: &MicrofrontendConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get application configuration
    let secret_engine = EnhancedSecretEngine::new().await?;
    let app_config = secret_engine
        .get_application_config(&config.app_id)
        .await?;

    println!("Microfrontend initialized: {}", config.app_id);
    println!("Base URL: {}", config.base_url);
    println!("Database configured: {}", !app_config.database_url.is_empty());

    Ok(())
}
```

### Frontend Authentication

```javascript
// Frontend JavaScript integration
class SimkariAuth {
    constructor(authEndpoint) {
        this.authEndpoint = authEndpoint;
        this.token = localStorage.getItem('simkari_token');
    }

    async authenticate(nip, password) {
        const response = await fetch(`${this.authEndpoint}/authenticate`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ nip, password }),
        });

        if (response.ok) {
            const data = await response.json();
            this.token = data.token;
            localStorage.setItem('simkari_token', this.token);

            console.log('Authentication successful');
            console.log('User:', data.user.nama);
            console.log('Satker:', data.user.satker_code);

            return data;
        } else {
            throw new Error('Authentication failed');
        }
    }

    async getSecrets(paths) {
        if (!this.token) {
            throw new Error('Not authenticated');
        }

        const response = await fetch(`${this.authEndpoint}/secrets/batch`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${this.token}`,
            },
            body: JSON.stringify({ paths }),
        });

        if (response.ok) {
            return await response.json();
        } else {
            throw new Error('Failed to retrieve secrets');
        }
    }

    isAuthenticated() {
        return !!this.token;
    }

    logout() {
        this.token = null;
        localStorage.removeItem('simkari_token');
    }
}

// Usage example
const auth = new SimkariAuth('https://authenc.internal');

// Authenticate user
auth.authenticate('198501012010011001', 'password')
    .then(data => {
        console.log('Logged in as:', data.user.nama);

        // Get application secrets
        return auth.getSecrets([
            'KJA001/database_config',
            'KJA001/api_keys'
        ]);
    })
    .then(secrets => {
        console.log('Retrieved secrets:', secrets);
    })
    .catch(error => {
        console.error('Error:', error);
    });
```

## Best Practices

### Security Best Practices

1. **Always Use mTLS**: Ensure all service-to-service communication uses mutual TLS authentication.

```rust
use rustls::{ClientConfig, Certificate, PrivateKey};

async fn setup_mtls_client() -> Result<reqwest::Client, Box<dyn std::error::Error>> {
    let cert_pem = std::fs::read("client.crt")?;
    let key_pem = std::fs::read("client.key")?;
    let ca_pem = std::fs::read("ca.crt")?;

    let cert = Certificate(cert_pem);
    let key = PrivateKey(key_pem);
    let ca = Certificate(ca_pem);

    let mut config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(vec![ca])
        .with_single_cert(vec![cert], key)?;

    let client = reqwest::Client::builder()
        .use_preconfigured_tls(config)
        .build()?;

    Ok(client)
}
```

2. **Implement Circuit Breakers**: Protect services from cascading failures.

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

struct ServiceClient {
    circuit_breaker: Arc<Mutex<CircuitBreaker>>,
    client: reqwest::Client,
}

impl ServiceClient {
    async fn call_service<T>(&self, request: T) -> Result<Response, ServiceError>
    where
        T: Serialize,
    {
        let mut cb = self.circuit_breaker.lock().await;

        cb.execute(|| async {
            self.client
                .post("https://service.internal/api")
                .json(&request)
                .send()
                .await
        }).await
    }
}
```

3. **Use Proper Error Handling**: Always handle errors gracefully and provide meaningful messages.

```rust
async fn handle_service_call() -> Result<(), ApplicationError> {
    match call_authenc_service().await {
        Ok(result) => Ok(result),
        Err(AuthencError::SecretonCommunicationError { message }) => {
            log::warn!("Secreton communication failed: {}", message);
            // Implement fallback logic
            use_cached_credentials().await
        }
        Err(AuthencError::TokenExpired { expired_at }) => {
            log::info!("Token expired at: {}", expired_at);
            // Redirect to login
            Err(ApplicationError::RequiresReauthentication)
        }
        Err(error) => {
            log::error!("Unexpected error: {:?}", error);
            Err(ApplicationError::InternalError(error.to_string()))
        }
    }
}
```

### Performance Best Practices

1. **Use Batch Operations**: Minimize network calls by batching requests.

```rust
// Good: Batch operation
let secrets = secret_engine.batch_get_secrets(&resource_ids).await?;

// Avoid: Individual calls
let mut secrets = Vec::new();
for id in resource_ids {
    let secret = secret_engine.get_secret(&id).await?;
    secrets.push(secret);
}
```

2. **Implement Caching**: Cache frequently accessed data with appropriate TTL.

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct CachedSecrets {
    cache: HashMap<String, (Secret, Instant)>,
    ttl: Duration,
}

impl CachedSecrets {
    async fn get_secret(&mut self, path: &str) -> Result<Secret, SecretonError> {
        if let Some((secret, cached_at)) = self.cache.get(path) {
            if cached_at.elapsed() < self.ttl {
                return Ok(secret.clone());
            }
        }

        let secret = fetch_secret_from_service(path).await?;
        self.cache.insert(path.to_string(), (secret.clone(), Instant::now()));

        Ok(secret)
    }
}
```

3. **Use Connection Pooling**: Reuse HTTP connections for better performance.

```rust
use reqwest::Client;
use std::sync::Arc;

#[derive(Clone)]
struct ServiceConnector {
    client: Arc<Client>,
}

impl ServiceConnector {
    fn new() -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client: Arc::new(client),
        }
    }
}
```

## Troubleshooting

### Common Issues

1. **Authentication Failures**

```rust
// Check token validity
async fn debug_token_issues(token: &str) -> Result<(), Box<dyn std::error::Error>> {
    match validate_token(token).await {
        Ok(validation) => {
            println!("Token validation successful");
            println!("User: {}", validation.claims.sub);
            println!("Expires: {}", validation.claims.exp);
        }
        Err(AuthencError::TokenExpired { expired_at }) => {
            println!("Token expired at: {}", expired_at);
            println!("Current time: {}", Utc::now());
        }
        Err(AuthencError::TokenSignatureInvalid) => {
            println!("Token signature is invalid");
            println!("Check signing key configuration");
        }
        Err(error) => {
            println!("Token validation failed: {:?}", error);
        }
    }

    Ok(())
}
```

2. **Permission Denied Errors**

```rust
// Debug permission issues
async fn debug_permission_issues(
    user: &User,
    resource: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Debugging permissions for user: {}", user.nip);
    println!("Resource: {}", resource);

    println!("User roles:");
    for role in &user.roles {
        println!("  - {} (scope: {:?})", role.name, role.scope);
        println!("    Permissions: {:?}", role.permissions);
    }

    let accessible_satker = user.get_accessible_satker();
    println!("Accessible satker: {:?}", accessible_satker);

    if let Some(resource_satker) = resource.split('/').next() {
        if !accessible_satker.contains(&resource_satker.to_string()) {
            println!("ERROR: User cannot access satker: {}", resource_satker);
        }
    }

    Ok(())
}
```

3. **Service Communication Issues**

```rust
// Debug service connectivity
async fn debug_service_connectivity() -> Result<(), Box<dyn std::error::Error>> {
    // Test authenc connectivity
    match reqwest::get("https://authenc.internah").await {
        Ok(response) => {
          println!("Authenc health check: {}", response.status());
        }
        Err(error) => {
            println!("Authenc connectivity failed:error);
        }
    }

    // Test secreton connectivity
    match reqwest::get("https://secreton.internal/health").await {
        Ok(response) => {
            println!("Secreton health check: {}", response.status());
        }
        Err(error) => {
            println!("Secreton connectivity failed: {:?}", error);
        }
    }

    Ok(())
}
```

### Logging and Monitoring

```rust
use tracing::{info, warn, error, debug};

// Structured logging for debugging
async fn log_authentication_attempt(nip: &str, result: &Result<AuthResult, AuthencError>) {
    match result {
        Ok(auth_result) => {
            info!(
                nip = nip,
                success = auth_result.success,
                satker = auth_result.user.satker_code,
                "Authentication attempt"
            );
        }
        Err(error) => {
            warn!(
                nip = nip,
                error = %error,
                "Authentication failed"
            );
        }
    }
}

// Performance monitoring
async fn monitor_secret_access(path: &str, duration: Duration) {
    if duration > Duration::from_millis(1000) {
        warn!(
            path = path,
            duration_ms = duration.as_millis(),
            "Slow secret access detected"
        );
    } else {
        debug!(
            path = path,
            duration_ms = duration.as_millis(),
            "Secret access completed"
        );
    }
}
```

This integration guide provides comprehensive examples and best practices for integrating with the SIMKARI platform's enhanced authentication and secret management systems.
