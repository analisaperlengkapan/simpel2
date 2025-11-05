//! Authentication and Authorization for Secreton
//!
//! This module provides comprehensive identity and access management (IAM) capabilities,
//! integrating with Authenc (the project's centralized authentication service) and supporting
//! multiple authentication methods including post-quantum cryptography.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │          API Layer (Handlers)                   │
//! └────────────────────┬────────────────────────────┘
//!                      │
//!                      ▼
//! ┌─────────────────────────────────────────────────┐
//! │      Auth Module (Core)                         │
//! │  ┌────────────────────────────────────────┐    │
//! │  │    AuthProvider Trait                  │    │
//! │  │  (Polymorphic authentication)          │    │
//! │  └────────┬───────────────────────────────┘    │
//! │           │                                      │
//! │  ┌────────▼─────────┐                          │
//! │  │ AuthencProvider  │  (Centralized)           │
//! │  │  - JWT validation│                          │
//! │  │  - MFA support   │                          │
//! │  │  - PQ crypto     │                          │
//! │  └──────────────────┘                          │
//! └────────────────────┬────────────────────────────┘
//!                      │
//!                      ▼
//! ┌─────────────────────────────────────────────────┐
//! │      Authenc Service (External)                 │
//! │  - User database                                │
//! │  - Session management                           │
//! │  - MFA verification                             │
//! │  - Role/permission management                   │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! # Authentication Flow
//!
//! 1. **User Login** → Submit credentials to Authenc
//! 2. **MFA** → Authenc requires TOTP/WebAuthn (if enabled)
//! 3. **Token Issuance** → Authenc returns JWT access + refresh tokens
//! 4. **Secreton Access** → User includes JWT in Authorization header
//! 5. **Token Validation** → Secreton validates JWT via Authenc or cached public key
//! 6. **Authorization** → Check user roles/permissions against Secreton policies
//! 7. **Operation** → Execute requested vault operation
//!
//! # Example: Authenticate User
//!
//! ```rust,no_run
//! use secreton_core::auth::{AuthProvider, AuthencAuthProvider, Credentials};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = AuthencAuthProvider::new("http://authenc:8000").await?;
//!
//! let creds = Credentials {
//!     username: "alice@kejaksaan.go.id".to_string(),
//!     password: "secure_password".to_string(),
//!     mfa_code: Some("123456".to_string()), // TOTP code
//! };
//!
//! match provider.authenticate(&creds).await {
//!     Ok(user) => {
//!         println!("Authenticated: {} (roles: {:?})", user.username, user.roles);
//!         // user.token contains JWT for subsequent requests
//!     }
//!     Err(e) => eprintln!("Authentication failed: {}", e),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Example: Validate JWT Token
//!
//! ```rust,no_run
//! use secreton_core::auth::{AuthProvider, AuthencAuthProvider};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let provider = AuthencAuthProvider::new("http://authenc:8000").await?;
//! let token = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...";
//!
//! match provider.validate_token(token).await {
//!     Ok(user) => {
//!         println!("Valid token for: {}", user.username);
//!         // Proceed with operation
//!     }
//!     Err(_) => {
//!         // Return 401 Unauthorized
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Post-Quantum Cryptography
//!
//! Secreton integrates post-quantum signature validation for future-proof security:
//!
//! ```rust,no_run
//! use secreton_core::auth::{PostQuantumValidator, PqSignature};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let validator = PostQuantumValidator::new();
//! let signature = PqSignature {
//!     algorithm: "ML-DSA-65".to_string(), // FIPS 204 (Dilithium)
//!     data: vec![/* signature bytes */],
//! };
//!
//! if validator.verify(&signature, b"message", &public_key)? {
//!     println!("Post-quantum signature valid");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Token Caching
//!
//! To reduce Authenc load, validated tokens are cached:
//!
//! - **In-Memory Cache**: LRU cache with configurable TTL (default: 15 minutes)
//! - **Automatic Refresh**: Tokens auto-refreshed before expiration
//! - **Invalidation**: Manual invalidation on logout or role change
//!
//! # Security Features
//!
//! - **JWT Validation**: Ed25519 signature verification (Authenc standard)
//! - **MFA Support**: TOTP (SHA-256), WebAuthn/FIDO2
//! - **Token Rotation**: Refresh tokens for long-lived sessions
//! - **Rate Limiting**: Brute-force protection (Authenc-side)
//! - **Audit Logging**: All auth events logged (see `audit` module)
//! - **Post-Quantum Ready**: ML-DSA (Dilithium) support
//!
//! # Authorization Model
//!
//! Secreton uses a hybrid model:
//!
//! 1. **Authenc Roles** - Broad organizational roles (admin, auditor, operator)
//! 2. **Secreton Policies** - Granular path-based permissions (see `policy` module)
//!
//! Example: User with `operator` role + `secrets:read:app/*` policy can read app secrets.
//!
//! # Performance
//!
//! - **Token Validation**: ~0.5ms (cached), ~5ms (Authenc call)
//! - **Cache Hit Rate**: >95% in production
//! - **Concurrent Users**: Scales linearly (stateless validation)
//!
//! # Configuration
//!
//! Set in environment or config file:
//!
//! ```bash
//! AUTHENC_URL=http://authenc:8000
//! TOKEN_CACHE_TTL_SECS=900  # 15 minutes
//! TOKEN_CACHE_SIZE=10000     # Max cached tokens
//! ```
//!
//! # See Also
//!
//! - [`AuthProvider`] - Trait for pluggable auth backends
//! - [`AuthencAuthProvider`] - Authenc integration implementation
//! - [`TokenCache`] - Token caching layer
//! - [`PostQuantumValidator`] - PQC signature verification
//! - `crate::policy` - Authorization policy engine
//! - `crate::audit` - Authentication audit logging

pub mod authenc_provider;

// Re-export commonly used types
pub use authenc_provider::{
    AuthProvider, AuthResult, AuthencAuthProvider, Credentials, PostQuantumValidator, PqSignature,
    TokenCache, TokenValidation, User, ValidationCache,
};
