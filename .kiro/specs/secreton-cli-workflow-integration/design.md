# Design Document: Secreton CLI Workflow Integration

## Overview

This design document describes the architecture and implementation approach for completing the end-to-end workflow integration of Secreton CLI. The goal is to provide a complete CLI interface that mirrors HashiCorp Vault's user experience while leveraging Secreton's existing API infrastructure.

The implementation will extend the existing `secreton-cli` crate with new command modules for authentication, policy management, token management, and operator diagnostics. It will also add TOML policy file format support and persistent token storage.

Note: Secreton uses its own naming conventions distinct from other vault systems. Token prefixes use `stn.` instead of other common prefixes.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Secreton CLI                                 │
├─────────────────────────────────────────────────────────────────────┤
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │  login   │ │  policy  │ │  token   │ │ operator │ │  audit   │  │
│  │ command  │ │ commands │ │ commands │ │ commands │ │ commands │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘  │
│       │            │            │            │            │         │
│  ┌────┴────────────┴────────────┴────────────┴────────────┴────┐   │
│  │                    CLI Core Services                         │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │   │
│  │  │ TokenStore  │ │PolicyParser │ │ SealChecker │            │   │
│  │  │ (persistent)│ │(TOML)       │ │ (middleware)│            │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘            │   │
│  └──────────────────────────┬───────────────────────────────────┘   │
│                             │                                       │
│  ┌──────────────────────────┴───────────────────────────────────┐   │
│  │                    HTTP Client Layer                          │   │
│  │  - Token injection from TokenStore                            │   │
│  │  - Seal status pre-check                                      │   │
│  │  - Error handling and retry                                   │   │
│  └──────────────────────────┬───────────────────────────────────┘   │
└─────────────────────────────┼───────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Secreton API Server                             │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    ServiceContainer                           │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ │   │
│  │  │AuthSvc  │ │PolicySvc│ │TokenSvc │ │SealSvc  │ │AuditSvc │ │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘ │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

### CLI Module Structure

```
infra/secreton/crates/cli/src/
├── main.rs              # Entry point with command routing
├── lib.rs               # Library exports
├── config.rs            # Configuration management (existing)
├── seal.rs              # Seal/unseal commands (existing)
├── backup.rs            # Backup/restore commands (existing)
├── auth.rs              # NEW: Login/logout commands
├── policy.rs            # NEW: Policy management commands
├── token.rs             # NEW: Token management commands
├── operator.rs          # NEW: Operator diagnostics
├── audit.rs             # NEW: Audit log commands
├── middleware.rs        # NEW: Pre-operation checks
└── policy_parser/       # NEW: Policy file parsing
    ├── mod.rs
    ├── toml.rs          # TOML policy parser
    └── formatter.rs     # Policy pretty printer
```

## Components and Interfaces

### 1. TokenStore Component

Manages persistent token storage in the user's config directory.

```rust
pub struct TokenStore {
    config_dir: PathBuf,
    token_file: PathBuf,
}

impl TokenStore {
    pub fn new() -> Result<Self>;
    pub fn store_token(&self, token: &str, metadata: &TokenMetadata) -> Result<()>;
    pub fn load_token(&self) -> Result<Option<StoredToken>>;
    pub fn clear_token(&self) -> Result<()>;
    pub fn is_token_expired(&self) -> bool;
}

pub struct StoredToken {
    pub token: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub policies: Vec<String>,
    pub renewable: bool,
}
```

### 2. PolicyParser Component

Parses policy files in TOML formats.

```rust
pub trait PolicyParser {
    fn parse(&self, content: &str) -> Result<Policy, PolicyParseError>;
    fn format(&self, policy: &Policy) -> Result<String>;
    fn validate(&self, content: &str) -> Result<Vec<ValidationError>>;
}

pub struct TomlPolicyParser;

pub struct Policy {
    pub name: String,
    pub description: Option<String>,
    pub rules: Vec<PolicyRule>,
    pub namespace: String,
}

pub struct PolicyRule {
    pub effect: Effect,
    pub path: String,
    pub capabilities: Vec<Capability>,
    pub condition: Option<Condition>,
    pub mfa: Option<bool>,
}
```

### 3. SealChecker Middleware

Pre-operation middleware that checks vault status.

```rust
pub struct SealChecker {
    client: reqwest::Client,
    server_url: String,
    cache: Option<CachedStatus>,
    cache_ttl: Duration,
}

impl SealChecker {
    pub async fn check_ready(&self) -> Result<VaultStatus, VaultNotReadyError>;
    pub async fn require_unsealed(&self) -> Result<(), VaultNotReadyErr
    pub async fn require_initialized(&self) -> Result<(), VaultNotReadyError>;
}

pub enum VaultNotReadyError {
    NotInitialized,
    Sealed { progress: usize, threshold: usize },
    Unreachable(String),
}
```

### 4. CLI Command Interfaces

#### Auth Commands
```rust
pub enum AuthCommand {
    Login {
        #[arg(short, long)]
        method: Option<String>,  // "userpass", "token", "oidc"
        #[arg(short, long)]
        username: Option<String>,
    },
    Logout,
}
```

#### Policy Commands
```rust
pub enum PolicyCommand {
    List {
        #[arg(long)]
        namespace: Option<String>,
    },
    Read {
        name: String,
        #[arg(long, default_value = "table")]
        format: OutputFormat,
    },
    Write {
        name: String,
        file: PathBuf,
    },
    Delete {
        name: String,
        #[arg(long)]
        force: bool,
    },
    Fmt {
        file: PathBuf,
        #[arg(long)]
        check: bool,
    },
    Validate {
        file: PathBuf,
    },
    Test {
        name: String,
        #[arg(long)]
        path: String,
        #[arg(long)]
        action: String,
    },
}
```

#### Token Commands
```rust
pub enum TokenCommand {
    Create {
        #[arg(long)]
        policies: Vec<String>,
        #[arg(long)]
        ttl: Option<String>,
        #[arg(long)]
        renewable: bool,
    },
    Lookup {
        token: Option<String>,
    },
    Renew {
        #[arg(long)]
        increment: Option<String>,
    },
    Revoke {
        token: String,
    },
    Capabilities {
        path: String,
    },
}
```

## Data Models

### Policy TOML Format

```toml
# Policy definition in TOML format
name = "database-readonly"
description = "Read-only access to database secrets"
namespace = "default"

[[rules]]
effect = "allow"
path = "secret/data/database/*"
capabilities = ["read", "list"]

[[rules]]
effect = "deny"
path = "secret/data/database/admin/*"
capabilities = ["*"]

[[rules]]
effect = "allow"
path = "secret/data/database/credentials"
capabilities = ["read"]
mfa = true

[rules.condition]
time_range = { start = "2025-01-01T00:00:00Z", end = "2025-12-31T23:59:59Z" }
allowed_ips = ["192.168.1.0/24", "10.0.0.0/8"]
```

### CLI Configuration File

```toml
# ~/.secreton/config.toml
server_url = "https://secreton.example.com:8200"
default_namespace = "default"
output_format = "table"
timeout_seconds = 30

[tls]
ca_cert = "/path/to/ca.pem"
skip_verify = false
```

### Stored Token Format

```toml
# ~/.secreton/token
token = "stn.CAESIG..."
expires_at = "2025-12-03T10:00:00Z"
renewable = true
policies = ["default", "database-readonly"]
display_name = "user@example.com"
```



## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

Based on the prework analysis, the following correctness properties have been identified:

### Property 1: Login Authentication Consistency
*For any* valid credentials (username/password or token), authenticating via CLI should result in a token being stored locally, and that tfor subsequAPts.
**Validates: Requirements 1.1, 1.2**

### Property 2: Invalid Credentials Rejection
*For any* invalid credentials, the CLI should reject authentication and not store any token in the local configuration.
**Validates: Requirements 1.3**

### Property 3: Logout Token Removal
*For any* authenticated session, executing logout should remove the stored token, and subsequent operations should require re-authentication.
**Validates: Requirements 1.4**

### Property 4: Policy TOML Round-Trip Consistency
*For any* valid policy definition, parsing from TOML format and then printing back to the same format should produce a semantically equivalent policy.
**Validates: Requirements 3.1, 3.2, 3.3, 8.4**

### Property 5: Policy Write-Read Consistency
*For any* valid policy file, writing it via CLI and then reading itk should return the same policy rules.
**Validates: Requirements 2.3, 2.2**

### Property 6: Policy Validation Error Detection
*For any* policy file with syntax errors, the CLI should detect and report all errors with accurate line numbers.
**Validates: Requirements 2.6, 7.2**

### Property 7: Token Create-Lookup Consistency
*For any* token created with specific policies, looking up that token should return the same policies.
**Validates: Requirements 4.1, 1.5**

### Property 8: Token Revocation Effectiveness
*For any* revoked token, subsequent operations using that token should be rejected.
**Validates: Requirements 4.2**

### Property 9: Sealed Vault Operation Rejection
*For any* secret operation attempted on a sealed vault, the CLI should reject the operation with a clear message indicating unseal is required.
**Validates: Requirements 6.2, 9.1, 10.1**

### Property 10: Uninitialized Vault Operation Rejection
*For any* operation attempted on an uninitialized vault, the CLI should reject the operation with a clear message indicating initialization is required.
**Validates: Requirements 6.1, 9.2**

### Property 11: Policy Enforcement Consistency
*For any* token with specific policies, operations should be allowed or denied consistently with the policy rules.
**Validates: Requirements 7.1, 10.2**

### Property 12: Namespace Isolation
*For any* secret or policy created in a namespace, it should only be visible to users with access to that namespace.
**Validates: Requirements 11.1, 11.3**

### Property 13: Audit Log Completeness
*For any* CLI operation (success or failure), an audit log entry should be created with the operation details.
**Validates: Requirements 12.1, 12.2**

### Property 14: Server Address Configuration Precedence
*For any* combination of environment variable, config file, and command-line flag for server address, the precedence should be: flag > env var > config file > default.
**Validates: Requirements 5.1, 5.2, 5.3, 5.4**

### Property 15: Example Policy Validity
*For any* example policy file provided, it should parse successfully and be applicable to the vault without modification.
**Validates: Requirements 13.2, 13.4**

## Error Handling

### CLI Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Vault not initialized. Run 'secreton seal init' first.")]
    VaultNotInitialized,

    #[error("Vault is sealed. Run 'secreton seal unseal' to unseal.")]
    VaultSealed { progress: usize, threshold: usize },

    #[error("Authentication required. Run 'secreton login' first.")]
    AuthenticationRequired,

    #[error("Token expired. Run 'secreton login' to re-authenticate.")]
    TokenExpired,

    #[error("Permission denied: {operation} on {path}")]
    PermissionDenied { operation: String, path: String },

    #[error("Policy parse error at line {line}: {message}")]
    PolicyParseError { line: usize, message: String },

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },
}
```

### Error Recovery Strategies

1. **Connection Errors**: Retry with exponential backoff (max 3 attempts)
2. **Token Expired**: Prompt for re-authentication
3. **Vault Sealed**: Display unseal progress and instructions
4. **Permission Denied**: Display required capabilities and suggest policy changes

## Testing Strategy

### Dual Testing Approach

This implementation uses both unit tests and property-based tests:

- **Unit tests**: Verify specific examples, edge cases, and error conditions
- **Property-based tests**: Verify universal properties that should hold across all inputs

### Property-Based Testing Framework

The implementation will use `proptest` (already in workspace dependencies) for property-based testing.

```rust
// Example property test structure
proptest! {
    #[test]
    fn test_policy_roundtrip(policy in arb_policy()) {
        // Feature: secreton-cli-workflow-integration, Property 4: Policy TOML Round-Trip Consistency
        // Validates: Requirements 3.1, 3.2, 3.4, 8.4
        let toml_str = TomlPolicyParser::format(&policy)?;
        let parsed = TomlPolicyParser::parse(&toml_str)?;
        prop_assert_eq!(policy.rules, parsed.rules);
    }
}
```

### Test Categories

1. **Policy Parser Tests**
   - TOML parsing and formatting
   - Round-trip consistency
   - Error detection with line numbers

2. **Token Store Tests**
   - Token persistence
   - Token expiration detection
   - Token clearing

3. **Seal Checker Tests**
   - Status caching
   - Error message formatting
   - State transition handling

4. **Integration Tests**
   - Full workflow: init → unseal → login → operations
   - Policy enforcement end-to-end
   - Namespace isolation

### Test Configuration

Property-based tests should run with a minimum of 100 iterations:

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    // tests here
}
```

## Implementation Notes

### Dependencies

New dependencies required:
- `toml` (already in workspace) for TOML policy parsing
- `dirs` (already in workspace) for config directory
- `proptest` (already in workspace) for property-based testing

### Backward Compatibility

- Existing CLI commands (seal, backup, transit, secret) remain unchanged
- New commands are additive
- Config file format is backward compatible (new fields are optional)

### Security Considerations

1. **Token Storage**: Tokens stored with file permissions 0600
2. **Password Input**: Use `rpassword` for secure password entry (already in workspace)
3. **TLS Verification**: Enabled by default, can be disabled for development
4. **Audit Logging**: All operations logged with user context


