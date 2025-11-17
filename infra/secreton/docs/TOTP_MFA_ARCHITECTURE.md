# TOTP/MFA Architecture - Separation of Concerns

## Overview

Secreton implements a clear separation between TOTP secrets management and MFA authentication flow to avoid duplication and ensure proper RFC 6238 compliance.

## Architecture Components

### 1. TotpEngine (Secrets Engine)
**Location**: `crates/core/src/services/secrets/totp.rs`  
**API Endpoints**: `/v1/sys/totp/*`

**Responsibilities:**
- RFC 6238 compliant TOTP key generation and storage
- Cryptographically secure secret generation (20 bytes = 160 bits)
- Multiple algorithm support (SHA1, SHA256, SHA512)
- Configurable code length (6 or 8 digits)
- Configurable time period (default 30 seconds)
- TOTP code generation and validation with proper HMAC
- Replay attack prevention with validation history
- QR code URL generation for authenticator apps
- Key management (create, read, delete, list)
- Time window adjustment for clock skew (±1 period default)

**Use Cases:**
- Centralized TOTP key storage for all applications
- Authenc MFA secret storage
- Multi-application TOTP management
- Direct TOTP operations via API

**API Endpoints:**
```
POST   /v1/sys/totp/keys              - Create TOTP key
GET    /v1/sys/totp/keys/:key_name    - Get TOTP key details
DELETE /v1/sys/totp/keys/:key_name    - Delete TOTP key
GET    /v1/sys/totp/keys              - List all TOTP keys
POST   /v1/sys/totp/code/:key_name    - Generate TOTP code
POST   /v1/sys/totp/validate/:key_name - Validate TOTP code
```

### 2. MfaService (Authentication Service)
**Location**: `crates/core/src/services/mfa.rs`  
**API Endpoints**: `/v1/auth/mfa/*`

**Responsibilities:**
- User MFA configuration management
- Recovery codes generation and validation
- MFA method management (TOTP, Email, SMS, Push)
- User-facing MFA setup/disable operations
- Authentication flow integration
- User MFA status tracking
- Delegates TOTP operations to TotpEngine

**Use Cases:**
- User authentication with MFA
- MFA enrollment and setup
- Recovery code management
- Multi-method MFA support

**Integration with TotpEngine:**
```rust
// MfaService delegates TOTP operations to TotpEngine
pub struct MfaService {
    configs: Arc<RwLock<HashMap<String, MfaConfig>>>,
    totp_engine: Arc<TotpEngine>,  // Shared instance
}

// Enable TOTP for user
pub async fn enable_totp(&self, user_id: &str, issuer: String, account_name: String) 
    -> Result<TotpSetupResponse, MfaError> 
{
    // Create TOTP key in TotpEngine
    let key_name = format!("user:{}", user_id);
    let totp_response = self.totp_engine.create_key(request).await?;
    
    // Store reference in MFA config
    config.totp_key_name = Some(key_name);
    // ...
}

// Verify TOTP code
pub async fn verify_totp(&self, user_id: &str, code: &str) 
    -> Result<bool, MfaError> 
{
    // Get key reference
    let key_name = config.totp_key_name.as_ref()?;
    
    // Delegate validation to TotpEngine
    let response = self.totp_engine.validate_code(request).await?;
    // ...
}
```

## Data Flow

### MFA Setup Flow
```
User Request → MfaService.enable_totp()
    ↓
MfaService creates TotpKeyCreateRequest
    ↓
TotpEngine.create_key() → Generates RFC 6238 compliant key
    ↓
TotpEngine stores key with name "user:{user_id}"
    ↓
MfaService stores key reference in MfaConfig
    ↓
Returns TotpSetupResponse with secret, QR code, recovery codes
```

### TOTP Validation Flow
```
User Login → MfaService.verify_totp()
    ↓
MfaService gets key_name from MfaConfig
    ↓
TotpEngine.validate_code() → RFC 6238 validation
    ↓
TotpEngine checks replay attack
    ↓
TotpEngine validates with time window
    ↓
Returns validation result
    ↓
MfaService updates last_used_at
```

## Key Differences

| Aspect | TotpEngine | MfaService |
|--------|-----------|------------|
| **Purpose** | TOTP key storage & validation | User MFA management |
| **Scope** | Generic secrets engine | User authentication |
| **TOTP Implementation** | RFC 6238 compliant (totp-lite) | Delegates to TotpEngine |
| **Storage** | TOTP keys by name | User MFA configs |
| **API Level** | System API (`/v1/sys/totp/`) | Auth API (`/v1/auth/mfa/`) |
| **Use Case** | Centralized TOTP management | User authentication flow |
| **Recovery Codes** | No | Yes (managed by MfaService) |
| **Multi-Method MFA** | No | Yes (TOTP, Email, SMS, Push) |

## Removed Duplication

### Before Refactoring
**Problem**: MfaService had its own simplified TOTP implementation:
```rust
// OLD: Simplified, non-RFC compliant
impl TotpConfig {
    fn generate_code(&self, counter: u64) -> String {
        // Simplified TOTP generation (NOT RFC 6238 compliant)
        let hash = (counter ^ 0x123456789ABCDEF).to_string();
        // ... simplified logic
    }
}
```

### After Refactoring
**Solution**: MfaService delegates to TotpEngine:
```rust
// NEW: Delegates to RFC 6238 compliant TotpEngine
impl MfaService {
    pub async fn verify_totp(&self, user_id: &str, code: &str) -> Result<bool, MfaError> {
        let key_name = self.get_totp_key_name(user_id)?;
        let response = self.totp_engine.validate_code(request).await?;
        Ok(response.valid)
    }
}
```

## Shared Instance

Both MfaService and API handlers share the same TotpEngine instance via ServiceContainer:

```rust
// ServiceContainer initialization
let totp_engine = Arc::new(TotpEngine::new());

// Shared with MfaService
let mfa = Arc::new(MfaService::with_totp_engine(totp_engine.clone()));

// Shared with API handlers
ServiceContainer {
    totp_engine,  // For direct API access
    mfa,          // For authentication flow
    // ...
}
```

## Benefits

1. **No Duplication**: Single RFC 6238 compliant TOTP implementation
2. **Clear Separation**: TotpEngine for storage, MfaService for auth flow
3. **Reusability**: TotpEngine can be used by any service
4. **Maintainability**: Changes to TOTP logic only in one place
5. **Testability**: Each component can be tested independently
6. **Compliance**: Proper RFC 6238 implementation with totp-lite
7. **Security**: Replay attack prevention, proper HMAC, secure random generation

## Testing

### TotpEngine Tests
```rust
// Test RFC 6238 compliance
#[tokio::test]
async fn test_totp_code_generation() { /* ... */ }

#[tokio::test]
async fn test_replay_prevention() { /* ... */ }
```

### MfaService Tests
```rust
// Test MFA flow integration
#[tokio::test]
async fn test_enable_totp() { /* ... */ }

#[tokio::test]
async fn test_recovery_codes() { /* ... */ }
```

## Migration Notes

### For Existing Code
If you have code using the old `TotpConfig`:
```rust
// OLD
let totp_config = TotpConfig::new(issuer, account);
let code = totp_config.generate_code(counter);

// NEW - Use TotpEngine
let request = TotpKeyCreateRequest { /* ... */ };
let response = totp_engine.create_key(request).await?;
let code_response = totp_engine.generate_code(&key_name).await?;
```

### For Authentication Flow
Use MfaService for user authentication:
```rust
// Enable TOTP for user
let response = mfa_service.enable_totp(user_id, issuer, account).await?;

// Verify TOTP code
let valid = mfa_service.verify_totp(user_id, code).await?;
```

## Conclusion

The refactored architecture provides:
- ✅ No duplication of TOTP functionality
- ✅ RFC 6238 compliant implementation
- ✅ Clear separation of concerns
- ✅ Shared TotpEngine instance
- ✅ Proper integration between components
- ✅ Production-ready security features
