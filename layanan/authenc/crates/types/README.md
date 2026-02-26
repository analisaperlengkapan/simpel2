# authenc-types

Shared types, traits, and interfaces for the Authenc authentication system.

## Overview

This crate provides the foundational types and trait definitions used across all Authenc crates, enabling dependency injection and modular architecture.

## Features

- **Strongly-typed IDs**: `UserId`, `RealmId`, `ClientId`, `SessionId`
- **Domain Models**: `User`, `Session`, `Realm`, `OidcClient`
- **Authentication Results**: `AuthResult` enum with success, MFA required, and failure variants
- **Service Traits**: Interfaces for dependency injection
  - `UserStore` - User storage operations
  - `SessionStore` - Session management
  - `RealmStore` - Realm management
  - `ClientStore` - OAuth2 client management
  - `AuthenticationService` - Authentication logic
  - `PasswordHasher` - Password hashing
  - `BruteForceProtector` - Brute force protection
  - `TokenGenerator` - JWT token generation
- **Error Types**: Comprehensive error handling with `AuthencError`

## Usage

```rust
use authenc_types::{
    UserId, RealmId, AuthResult, Credentials,
    UserStore, AuthenticationService,
};

// Create strongly-typed IDs
let user_id = UserId::new();
let realm_id = RealmId::new();

// Use authentication result
match auth_result {
    AuthResult::Success { user_id, session_id } => {
        println!("User {} authenticated successfully", user_id);
    }
    AuthResult::MfaRequired { user_id, mfa_token } => {
        println!("MFA required for user {}", user_id);
    }
    AuthResult::Failed { reason } => {
        println!("Authentication failed: {}", reason);
    }
}

// Implement service traits for dependency injection
struct MyUserStore { /* ... */ }

#[async_trait]
impl UserStore for MyUserStore {
    async fn get_user(&self, id: UserId) -> Result<User> {
        // Implementation
    }
    // ... other methods
}
```

## Design Principles

1. **No Implementation Logic**: This crate contains only types and trait definitions
2. **Dependency Injection**: All services are defined as traits for testability
3. **Type Safety**: Strongly-typed IDs prevent mixing different ID types
4. **Serialization**: All domain types support serde serialization

## Dependencies

- `uuid` - Unique identifier generation
- `serde` - Serialization/deserialization
- `chrono` - Date and time handling
- `thiserror` - Error type derivation
- `async-trait` - Async trait support

## Related Crates

- `authenc-core` - Business logic implementations
- `authenc-storage` - Database layer implementations
- `authenc-crypto` - Cryptographic operations
- `authenc-api` - REST API layer
- `authenc-grpc` - gRPC service layer

## Requirements

This crate implements:
- REQ-ARCH-001: Multi-crate architecture
- REQ-ARCH-003: Shared type definitions
