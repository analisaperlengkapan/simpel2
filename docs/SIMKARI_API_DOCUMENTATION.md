# SIMKARI API Documentation

## Overview

This document provides comprehensive API documentation for the enhanced cryptographic functionality and role-based access control system in the SIMKARI (Sistem Informasi Manajemen Kejaksaan Republik Indonesia) super app platform.

## Table of Contents

1. [Authentication & Authorization APIs](#authentication--authorization-apis)
2. [Secret Management APIs](#secret-management-apis)
3. [Role-Based Access Control](#role-based-access-control)
4. [Post-Quantum Cryptography](#post-quantum-cryptography)
5. [Integration Examples](#integration-examples)
6. [Error Handling](#error-handling)

## Authentication & Authorization APIs

### Authenc Enhanced APIs

#### Enhanced User Management

```rust
use authenc::models::{User, Role, AdminLevel, RoleScope};
use authenc::crypto::CryptoEngine;

/// Enhanced user model for SIMKARI operations
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub nip: String,                    // Nomor Induk Pegawai
    pub nama: String,
    pub email: String,
    pub satker_code: String,            // Kode satuan kerja
    pub jabatan: String,                // Jabatan pegawai
    pub roles: Vec<Role>,               // Roles managed by admin
    pub permissions: Vec<Permission>,   // Derived from roles
    pub session_data: EncryptedSessionData,
    pub secreton_access_policy: SecretonAccessPolicy,
    pub last_auth: DateTime<Utc>,
    pub security_context: SecurityContext,
}

impl User {
    /// Create a new user with basic information
    ///
    /// # Arguments
    /// * `nip` - Nomor Induk Pegawai (Employee ID)
    /// * `nama` - Full name of the employee
    /// * `email` - Email address
    /// * `satker_code` - Work unit code
    /// * `jabatan` - Position/title
    ///
    /// # Example
    /// ```rust
    /// let user = User::new(
    ///     "198501012010011001",
    ///     "John Doe",
    ///     "john.doe@kejaksaan.go.id",
    ///     "KJA001",
    ///     "Jaksa Muda"
    /// )?;
    /// ```
    pub fn new(
        nip: &str,
        nama: &str,
        email: &str,
        satker_code: &str,
        jabatan: &str,
    ) -> Result<Self, AuthencError>;

    /// Check if user has permission for specific operation
    ///
    /// # Arguments
    /// * `operation` - The operation to check permission for
    /// * `resource` - Optional resource identifier
    ///
    /// # Example
    /// ```rust
    /// if user.has_permission("read_secrets", Some("KJA001/config"))? {
    ///     // User can read secrets for KJA001 config
    /// }
    /// ```
    pub fn has_permission(&self, operation: &str, resource: Option<&str>) -> Result<bool, AuthencError>;

    /// Get user's accessible satker codes based on roles
    ///
    /// # Returns
    /// Vector of satker codes the user can access
    ///
    /// # Example
    /// ```rust
    /// let accessible_satker = user.get_accessible_satker();
    /// for satker in accessible_satker {
    ///     println!("User can access: {}", satker);
    /// }
    /// ```
    pub fn get_accessible_satker(&self) -> Vec<String>;
}
```

#### Enhanced Cryptographic Engine

```rust
use authenc::crypto::{CryptoEngine, Claims, SessionData, AuditData};

impl CryptoEngine {
    /// Sign JWT token for government employee authentication
    ///
    /// # Arguments
    /// * `claims` - JWT claims including NIP, satker_code, and roles
    ///
    /// # Returns
    /// Signed JWT token string
    ///
    /// # Example
    /// ```rust
    /// let claims = Claims {
    ///     sub: user.nip.clone(),
    ///     satker_code: user.satker_code.clone(),
    ///     roles: user.roles.iter().map(|r| r.name.clone()).collect(),
    ///     exp: Utc::now() + Duration::hours(8),
    /// };
    ///
    /// let token = crypto_engine.sign_jwt_for_government(&claims).await?;
    /// ```
    pub async fn sign_jwt_for_government(&self, claims: &Claims) -> Result<String, CryptoError>;

    /// Validate multiple tokens in batch for performance
    ///
    /// # Arguments
    /// * `tokens` - Array of JWT token strings to validate
    ///
    /// # Returns
    /// Vector of boolean results corresponding to each token
    ///
    /// # Example
    /// ```rust
    /// let tokens = vec!["token1", "token2", "token3"];
    /// let results = crypto_engine.verify_token_batch(&tokens).await?;
    ///
    /// for (i, valid) in results.iter().enumerate() {
    ///     println!("Token {}: {}", i, if *valid { "Valid" } else { "Invalid" });
    /// }
    /// ```
    pub async fn verify_token_batch(&self, tokens: &[String]) -> Result<Vec<bool>, CryptoError>;

    /// Encrypt session data for SIMKARI users
    ///
    /// # Arguments
    /// * `data` - Session data to encrypt
    ///
    /// # Returns
    /// Encrypted session data
    ///
    /// # Example
    /// ```rust
    /// let session_data = SessionData {
    ///     user_id: user.id,
    ///     satker_code: user.satker_code.clone(),
    ///     permissions: user.permissions.clone(),
    ///     expires_at: Utc::now() + Duration::hours(8),
    /// };
    ///
    /// let encrypted = crypto_engine.encrypt_session_data(&session_data).await?;
    /// ```
    pub async fn encrypt_session_data(&self, data: &SessionData) -> Result<EncryptedData, CryptoError>;

    /// Generate audit signature for compliance requirements
    ///
    /// # Arguments
    /// * `data` - Audit data to sign
    ///
    /// # Returns
    /// Digital signature for audit trail
    ///
    /// # Example
    /// ```rust
    /// let audit_data = AuditData {
    ///     event_type: "secret_access",
    ///     nip: user.nip.clone(),
    ///     satker_code: user.satker_code.clone(),
    ///     resource: "KJA001/database_config",
    ///     timestamp: Utc::now(),
    /// };
    ///
    /// let signature = crypto_engine.generate_government_audit_signature(&audit_data).await?;
    /// ```
    pub async fn generate_government_audit_signature(&self, data: &AuditData) -> Result<Signature, CryptoError>;
}
```

#### Secreton Integration Client

```rust
use authenc::vault::{SecretonClient, SecurityContext, ApplicationConfig};

impl SecretonClient {
    /// Get signing key for JWT operations
    ///
    /// # Arguments
    /// * `key_id` - Identifier for the signing key
    /// * `context` - Security context including user and satker information
    ///
    /// # Returns
    /// Signing key for cryptographic operations
    ///
    /// # Example
    /// ```rust
    /// let context = SecurityContext {
    ///     user_nip: user.nip.clone(),
    ///     satker_code: user.satker_code.clone(),
    ///     operation: "jwt_signing",
    /// };
    ///
    /// let signing_key = secreton_client.get_signing_key("jwt_key_2024", &context).await?;
    /// ```
    pub async fn get_signing_key(&self, key_id: &str, context: &SecurityContext) -> Result<SigningKey, SecretonError>;

    /// Get encryption key for data protection
    ///
    /// # Arguments
    /// * `resource_id` - Resource identifier requiring encryption
    /// * `context` - Security context for access control
    ///
    /// # Returns
    /// Encryption key for the specified resource
    ///
    /// # Example
    /// ```rust
    /// let context = SecurityContext {
    ///     user_nip: user.nip.clone(),
    ///     satker_code: user.satker_code.clone(),
    ///     operation: "data_encryption",
    /// };
    ///
    /// let enc_key = secreton_client.get_encryption_key("database_config", &context).await?;
    /// ```
    pub async fn get_encryption_key(&self, resource_id: &str, context: &SecurityContext) -> Result<EncryptionKey, SecretonError>;

    /// Validate user access to specific secret
    ///
    /// # Arguments
    /// * `user_id` - User identifier (NIP)
    /// * `secret_path` - Path to the secret in secreton
    ///
    /// # Returns
    /// Boolean indicating if access is allowed
    ///
    /// # Example
    /// ```rust
    /// let has_access = secreton_client
    ///     .validate_user_secret_access(&user.nip, "KJA001/database_password")
    ///     .await?;
    ///
    /// if has_access {
    ///     // Proceed with secret retrieval
    /// }
    /// ```
    pub async fn validate_user_secret_access(&self, user_id: &str, secret_path: &str) -> Result<bool, SecretonError>;

    /// Get application configuration from secreton
    ///
    /// # Arguments
    /// * `app_id` - Application identifier
    ///
    /// # Returns
    /// Application configuration including secrets and settings
    ///
    /// # Example
    /// ```rust
    /// let config = secreton_client.get_application_config("simkari_portal").await?;
    ///
    /// println!("Database URL: {}", config.database_url);
    /// println!("API Keys: {:?}", config.api_keys);
    /// ```
    pub async fn get_application_config(&self, app_id: &str) -> Result<ApplicationConfig, SecretonError>;

    /// Get post-quantum cryptographic key
    ///
    /// # Arguments
    /// * `key_id` - Key identifier
    /// * `algorithm` - Post-quantum algorithm type
    ///
    /// # Returns
    /// Post-quantum cryptographic key
    ///
    /// # Example
    /// ```rust
    /// use authenc::crypto::PqAlgorithm;
    ///
    /// let pq_key = secreton_client
    ///     .get_post_quantum_key("pq_signing_2024", PqAlgorithm::MlDsa65)
    ///     .await?;
    /// ```
    pub async fn get_post_quantum_key(&self, key_id: &str, algorithm: PqAlgorithm) -> Result<PqKey, SecretonError>;
}
```

## Secret Management APIs

### Secreton Enhanced APIs

#### Enhanced Secret Engine

```rust
use secreton::engines::{EnhancedSecretEngine, Secret, UserCredentials};

impl EnhancedSecretEngine {
    /// Get application-specific secret
    ///
    /// # Arguments
    /// * `app_id` - Application identifier
    /// * `path` - Secret path within the application
    ///
    /// # Returns
    /// Secret value with metadata
    ///
    /// # Example
    /// ```rust
    /// let secret = engine
    ///     .get_application_secret("simkari_portal", "database/password")
    ///     .await?;
    ///
    /// println!("Secret value: {}", secret.value);
    /// println!("Created by: {}", secret.metadata.created_by_nip.unwrap_or_default());
    /// ```
    pub async fn get_application_secret(&self, app_id: &str, path: &str) -> Result<Secret, SecretError>;

    /// Get user credentials for authentication
    ///
    /// # Arguments
    /// * `user_id` - User identifier (NIP)
    ///
    /// # Returns
    /// User credentials including hashed password and metadata
    ///
    /// # Example
    /// ```rust
    /// let credentials = engine.get_user_credentials("198501012010011001").await?;
    ///
    /// if credentials.verify_password("user_input_password")? {
    ///     // Password is correct
    /// }
    /// ```
    pub async fn get_user_credentials(&self, user_id: &str) -> Result<UserCredentials, SecretError>;

    /// Retrieve multiple secrets in batch operation
    ///
    /// # Arguments
    /// * `resource_ids` - Array of resource identifiers
    ///
    /// # Returns
    /// Vector of secrets corresponding to each resource ID
    ///
    /// # Example
    /// ```rust
    /// let resource_ids = vec![
    ///     "KJA001/database_config",
    ///     "KJA001/api_keys",
    ///     "KJA001/encryption_keys"
    /// ];
    ///
    /// let secrets = engine.batch_get_secrets(&resource_ids).await?;
    ///
    /// for (id, secret) in resource_ids.iter().zip(secrets.iter()) {
    ///     println!("Resource {}: Retrieved secret", id);
    /// }
    /// ```
    pub async fn batch_get_secrets(&self, resource_ids: &[String]) -> Result<Vec<Secret>, SecretError>;

    /// Validate application token for API access
    ///
    /// # Arguments
    /// * `token` - Application token to validate
    /// * `app_context` - Application context for validation
    ///
    /// # Returns
    /// Boolean indicating if token is valid
    ///
    /// # Example
    /// ```rust
    /// let is_valid = engine
    ///     .validate_application_token("app_token_123", "simkari_portal")
    ///     .await?;
    ///
    /// if is_valid {
    ///     // Token is valid, proceed with operation
    /// }
    /// ```
    pub async fn validate_application_token(&self, token: &str, app_context: &str) -> Result<bool, SecretError>;

    /// Get post-quantum encrypted secret
    ///
    /// # Arguments
    /// * `path` - Secret path
    /// * `algorithm` - Post-quantum algorithm for encryption
    ///
    /// # Returns
    /// Secret encrypted with post-quantum cryptography
    ///
    /// # Example
    /// ```rust
    /// use secreton::crypto::PqAlgorithm;
    ///
    /// let pq_secret = engine
    ///     .get_pq_encrypted_secret("sensitive/long_term_key", PqAlgorithm::MlKem768)
    ///     .await?;
    /// ```
    pub async fn get_pq_encrypted_secret(&self, path: &str, algorithm: PqAlgorithm) -> Result<Secret, SecretError>;
}
```

#### Authenc Authentication Provider

```rust
use secreton::auth::{AuthencAuthProvider, AuthResult, TokenValidation};

impl AuthencAuthProvider {
    /// Authenticate user with flexible credentials
    ///
    /// # Arguments
    /// * `user_id` - User identifier (NIP)
    /// * `credentials` - Authentication credentials
    ///
    /// # Returns
    /// Authentication result with user information
    ///
    /// # Example
    /// ```rust
    /// use secreton::auth::Credentials;
    ///
    /// let credentials = Credentials::Password {
    ///     nip: "198501012010011001",
    ///     password: "secure_password",
    /// };
    ///
    /// let auth_result = provider
    ///     .authenticate_user("198501012010011001", &credentials)
    ///     .await?;
    ///
    /// if auth_result.success {
    ///     println!("User authenticated: {}", auth_result.user.nama);
    /// }
    /// ```
    pub async fn authenticate_user(&self, user_id: &str, credentials: &Credentials) -> Result<AuthResult, AuthError>;

    /// Validate JWT token with post-quantum signature support
    ///
    /// # Arguments
    /// * `token` - JWT token to validate
    ///
    /// # Returns
    /// Token validation result with claims
    ///
    /// # Example
    /// ```rust
    /// let validation = provider.validate_token("jwt_token_here").await?;
    ///
    /// if validation.valid {
    ///     println!("Token valid for user: {}", validation.claims.sub);
    ///     println!("Satker: {}", validation.claims.satker_code);
    /// }
    /// ```
    pub async fn validate_token(&self, token: &str) -> Result<TokenValidation, AuthError>;

    /// Check resource permissions based on user roles
    ///
    /// # Arguments
    /// * `user` - User object with roles and permissions
    /// * `resource_id` - Resource identifier to check access for
    ///
    /// # Returns
    /// Boolean indicating if access is allowed
    ///
    /// # Example
    /// ```rust
    /// let has_permission = provider
    ///     .check_resource_permissions(&user, "KJA001/sensitive_documents")
    ///     .await?;
    ///
    /// if has_permission {
    ///     // User can access the resource
    /// }
    /// ```
    pub async fn check_resource_permissions(&self, user: &User, resource_id: &str) -> Result<bool, AuthError>;

    /// Validate application access to resources
    ///
    /// # Arguments
    /// * `app_id` - Application identifier
    /// * `resource` - Resource path
    ///
    /// # Returns
    /// Boolean indicating if application can access resource
    ///
    /// # Example
    /// ```rust
    /// let app_access = provider
    ///     .validate_application_access("simkari_portal", "user_management")
    ///     .await?;
    ///
    /// if app_access {
    ///     // Application has access to user management
    /// }
    /// ```
    pub async fn validate_application_access(&self, app_id: &str, resource: &str) -> Result<bool, AuthError>;

    /// Validate post-quantum digital signature
    ///
    /// # Arguments
    /// * `signature` - Post-quantum signature to validate
    /// * `data` - Original data that was signed
    ///
    /// # Returns
    /// Boolean indicating if signature is valid
    ///
    /// # Example
    /// ```rust
    /// use secreton::crypto::PqSignature;
    ///
    /// let signature = PqSignature::MlDsa65(signature_bytes);
    /// let data = b"important document content";
    ///
    /// let is_valid = provider.validate_pq_signature(&signature, data).await?;
    ///
    /// if is_valid {
    ///     // Signature is valid and document is authentic
    /// }
    /// ```
    pub async fn validate_pq_signature(&self, signature: &PqSignature, data: &[u8]) -> Result<bool, AuthError>;
}
```

## Role-Based Access Control

### Role Hierarchy System

The SIMKARI system implements a hierarchical role-based access control system designed for the Indonesian Attorney General's Office structure:

#### Admin Levels

```rust
use authenc::models::{AdminLevel, RoleScope, Role};

/// Administrative levels in the Attorney General's Office hierarchy
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AdminLevel {
    /// Administrator for a specific satker (work unit)
    AdminSatker(String),

    /// Administrator for a wilayah (regional area - kejaksaan tinggi)
    AdminWilayah(String),

    /// Administrator for Eselon I level at kejaksaan agung
    AdminEselonI,

    /// Central administrator with highest privileges
    AdminPusat,
}

impl AdminLevel {
    /// Check if this admin level can manage the specified role scope
    ///
    /// # Arguments
    /// * `scope` - Role scope to check management rights for
    ///
    /// # Returns
    /// Boolean indicating if management is allowed
    ///
    /// # Example
    /// ```rust
    /// let admin = AdminLevel::AdminWilayah("SUMUT".to_string());
    /// let scope = RoleScope::Satker("KJA001".to_string());
    ///
    /// if admin.can_manage_scope(&scope) {
    ///     // Admin can manage roles for this satker
    /// }
    /// ```
    pub fn can_manage_scope(&self, scope: &RoleScope) -> bool;

    /// Get all satker codes this admin level can manage
    ///
    /// # Returns
    /// Vector of satker codes under this admin's jurisdiction
    ///
    /// # Example
    /// ```rust
    /// let admin = AdminLevel::AdminWilayah("SUMUT".to_string());
    /// let manageable_satker = admin.get_manageable_satker();
    ///
    /// for satker in manageable_satker {
    ///     println!("Can manage: {}", satker);
    /// }
    /// ```
    pub fn get_manageable_satker(&self) -> Vec<String>;

    /// Check if this admin level is higher than another
    ///
    /// # Arguments
    /// * `other` - Other admin level to compare with
    ///
    /// # Returns
    /// Boolean indicating if this level is higher
    ///
    /// # Example
    /// ```rust
    /// let admin_pusat = AdminLevel::AdminPusat;
    /// let admin_satker = AdminLevel::AdminSatker("KJA001".to_string());
    ///
    /// assert!(admin_pusat.is_higher_than(&admin_satker));
    /// ```
    pub fn is_higher_than(&self, other: &AdminLevel) -> bool;
}
```

#### Role Scopes

```rust
/// Scope of role application within the organization
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RoleScope {
    /// Role applies to a specific satker
    Satker(String),

    /// Role applies to a wilayah (multiple satker)
    Wilayah(String),

    /// Role applies at central/national level
    Pusat,
}

impl RoleScope {
    /// Check if this scope includes the specified satker
    ///
    /// # Arguments
    /// * `satker_code` - Satker code to check inclusion for
    ///
    /// # Returns
    /// Boolean indicating if satker is included in this scope
    ///
    /// # Example
    /// ```rust
    /// let scope = RoleScope::Wilayah("SUMUT".to_string());
    ///
    /// if scope.includes_satker("KJA001") {
    ///     // This role applies to KJA001
    /// }
    /// ```
    pub fn includes_satker(&self, satker_code: &str) -> bool;

    /// Get all satker codes included in this scope
    ///
    /// # Returns
    /// Vector of satker codes within this scope
    ///
    /// # Example
    /// ```rust
    /// let scope = RoleScope::Wilayah("SUMUT".to_string());
    /// let included_satker = scope.get_included_satker();
    ///
    /// for satker in included_satker {
    ///     println!("Scope includes: {}", satker);
    /// }
    /// ```
    pub fn get_included_satker(&self) -> Vec<String>;

    /// Check if this scope is broader than another scope
    ///
    /// # Arguments
    /// * `other` - Other scope to compare with
    ///
    /// # Returns
    /// Boolean indicating if this scope is broader
    ///
    /// # Example
    /// ```rust
    /// let pusat_scope = RoleScope::Pusat;
    /// let satker_scope = RoleScope::Satker("KJA001".to_string());
    ///
    /// assert!(pusat_scope.is_broader_than(&satker_scope));
    /// ```
    pub fn is_broader_than(&self, other: &RoleScope) -> bool;
}
```

#### Role Management

```rust
use authenels::{Role, Permission};

impl Role {
    /// Create a new role with specified scope and permissions
    ///
    /// # Arguments
    /// * `name` - Role name
    /// * `scope` - Role scope (Satker, Wilayah, or Pusat)
    /// * `permissions` - List of permissions for this role
    /// * `managed_by` - Admin level that manages this role
    ///
    /// # Returns
    /// New role instance
    ///
    /// # Example
    /// ```rust
    /// let role = Role::new(
    ///     "Jaksa Penuntut",
    ///     RoleScope::Satker("KJA001".to_string()),
    ///     vec![
    ///         Permission::ReadCases,
    ///         Permission::WriteCaseNotes,
    ///         Permission::AccessEvidence,
    ///     ],
    ///     AdminLevel::AdminSatker("KJA001".to_string()),
    /// )?;
    /// ```
    pub fn new(
        name: &str,
        scope: RoleScope,
        permissions: Vec<Permission>,
        managed_by: AdminLevel,
    ) -> Result<Self, RoleError>;

    /// Check if role grants specific permission for a resource
    ///
    /// # Arguments
    /// * `permission` - Permission to check
    /// * `resource_satker` - Satker code of the resource
    ///
    /// # Returns
    /// Boolean indicating if permission is granted
    ///
    /// # Example
    /// ```rust
    /// let role = Role::new(/* ... */)?;
    ///
    /// if role.grants_permission(&Permission::ReadSecrets, "KJA001") {
    ///     // Role grants read access to KJA001 secrets
    /// }
    /// ```
    pub fn grants_permission(&self, permission: &Permission, resource_satker: &str) -> bool;

    /// Get effective permissions for a specific satker
    ///
    /// # Arguments
    /// * `satker_code` - Satker code to get permissions for
    ///
    /// # Returns
    /// Vector of effective permissions for the satker
    ///
    /// # Example
    /// ```rust
    /// let role = Role::new(/* ... */)?;
    /// let effective_perms = role.get_effective_permissions("KJA001");
    ///
    /// for perm in effective_perms {
    ///     println!("Effective permission: {:?}", perm);
    /// }
    /// ```
    pub fn get_effective_permissions(&self, satker_code: &str) -> Vec<Permission>;

    /// Check if role can be managed by specified admin level
    ///
    /// # Arguments
    /// * `admin_level` - Admin level attempting to manage this role
    ///
    /// # Returns
    /// Boolean indicating if management is allowed
    ///
    /// # Example
    /// ```rust
    /// let role = Role::new(/* ... */)?;
    /// let admin = AdminLevel::AdminWilayah("SUMUT".to_string());
    ///
    /// if role.can_be_managed_by(&admin) {
    ///     // Admin can modify this role
    /// }
    /// ```
    pub fn can_be_managed_by(&self, admin_level: &AdminLevel) -> bool;
}
```

### Permission System

```rust
use authenc::models::Permission;

/// Permissions available in the SIMKARI system
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Permission {
    // Secret Management Permissions
    ReadSecrets,
    WriteSecrets,
    DeleteSecrets,
    ManageSecrets,

    // User Management Permissions
    ReadUsers,
    WriteUsers,
    DeleteUsers,
    ManageUsers,

    // Role Management Permissions
    ReadRoles,
    WriteRoles,
    DeleteRoles,
    ManageRoles,

    // Case Management Permissions (SIMKARI specific)
    ReadCases,
    WriteCases,
    DeleteCases,
    ManageCases,

    // Evidence Management Permissions
    ReadEvidence,
    WriteEvidence,
    DeleteEvidence,
    ManageEvidence,

    // Audit and Compliance Permissions
    ReadAuditLogs,
    WriteAuditLogs,
    ManageCompliance,

    // Administrative Permissions
    SystemAdmin,
    SecurityAdmin,
    AuditAdmin,

    // Application-specific Permissions
    AccessPortal,
    AccessBadiklat,
    AccessIntel,
    AccessPidmil,
    AccessPidsus,
    AccessPidum,
    AccessPengawasan,
    AccessPemulihanAset,
}

impl Permission {
    /// Check if permission applies to specific resource type
    ///
    /// # Arguments
    /// * `resource_type` - Type of resource to check
    ///
    /// # Returns
    /// Boolean indicating if permission applies
    ///
    /// # Example
    /// ```rust
    /// let perm = Permission::ReadSecrets;
    ///
    /// if perm.applies_to_resource("secrets") {
    ///     // Permission applies to secrets
    /// }
    /// ```
    pub fn applies_to_resource(&self, resource_type: &str) -> bool;

    /// Get permission level (Read, Write, Delete, Manage)
    ///
    /// # Returns
    /// Permission level enum
    ///
    /// # Example
    /// ```rust
    /// let perm = Permission::WriteSecrets;
    /// let level = perm.get_level(); // Returns PermissionLevel::Write
    /// ```
    pub fn get_level(&self) -> PermissionLevel;

    /// Check if this permission implies another permission
    ///
    /// # Arguments
    /// * `other` - Other permission to check implication for
    ///
    /// # Returns
    /// Boolean indicating if this permission implies the other
    ///
    /// # Example
    /// ```rust
    /// let manage_perm = Permission::ManageSecrets;
    /// let read_perm = Permission::ReadSecrets;
    ///
    /// assert!(manage_perm.implies(&read_perm));
    /// ```
    pub fn implies(&self, other: &Permission) -> bool;
}
```

## Post-Quantum Cryptography

### Hybrid Cryptographic System

```rust
use secreton::crypto::{HybridCrypto, CryptoMode, PqAlgorithm};

/// Hybrid cryptographic system supporting classical and post-quantum algorithms
pub struct HybridCrypto {
    classical: ClassicalCrypto,
    post_quantum: PostQuantumCrypto,
    mode: CryptoMode,
}

impl HybridCrypto {
    /// Create new hybrid crypto instance
    ///
    /// # Arguments
    /// * `mode` - Cryptographic mode (Classical, Hybrid, or PostQuantum)
    ///
    /// # Returns
    /// New hybrid crypto instance
    ///
    /// # Example
    /// ```rust
    /// let hybrid_crypto = HybridCrypto::new(CryptoMode::Hybrid)?;
    /// ```
    pub fn new(mode: CryptoMode) -> Result<Self, CryptoError>;

    /// Sign data with hybrid approach
    ///
    /// # Arguments
    /// * `data` - Data to sign
    /// * `key_id` - Key identifier for signing
    ///
    /// # Returns
    /// Hybrid signature (classical + post-quantum)
    ///
    /// # Example
    /// ```rust
    /// let data = b"important document";
    /// let signature = hybrid_crypto.sign_hybrid(data, "signing_key_2024").await?;
    ///
    /// // Signature contains both Ed25519 and ML-DSA signatures
    /// println!("Classical signature: {:?}", signature.classical);
    /// println!("Post-quantum signature: {:?}", signature.post_quantum);
    /// ```
    pub async fn sign_hybrid(&self, data: &[u8], key_id: &str) -> Result<HybridSignature, CryptoError>;

    /// Verify hybrid signature
    ///
    /// # Arguments
    /// * `data` - Original data
    /// * `signature` - Hybrid signature to verify
    /// * `public_key_id` - Public key identifier
    ///
    /// # Returns
    /// Boolean indicating if signature is valid
    ///
    /// # Example
    /// ```rust
    /// let is_valid = hybrid_crypto
    ///     .verify_hybrid(data, &signature, "public_key_2024")
    ///     .await?;
    ///
    /// if is_valid {
    ///     // Both classical and post-quantum signatures are valid
    /// }
    /// ```
    pub async fn verify_hybrid(&self, data: &[u8], signature: &HybridSignature, public_key_id: &str) -> Result<bool, CryptoError>;

    /// Encrypt data with hybrid key encapsulation
    ///
    /// # Arguments
    /// * `data` - Data to encrypt
    /// * `recipient_key_id` - Recipient's key identifier
    ///
    /// # Returns
    /// Hybrid encrypted data with dual key encapsulation
    ///
    /// # Example
    /// ```rust
    /// let sensitive_data = b"classified information";
    /// let encrypted = hybrid_crypto
    ///     .encrypt_hybrid(sensitive_data, "recipient_key_2024")
    ///     .await?;
    ///
    /// // Encrypted with both X25519 and ML-KEM
    /// println!("Classical KEM: {:?}", encrypted.classical_kem);
    /// println!("Post-quantum KEM: {:?}", encrypted.pq_kem);
    /// ```
    pub async fn encrypt_hybrid(&self, data: &[u8], recipient_key_id: &str) -> Result<HybridEncrypted, CryptoError>;

    /// Decrypt hybrid encrypted data
    ///
    /// # Arguments
    /// * `encrypted` - Hybrid encrypted data
    /// * `private_key_id` - Private key identifier for decryption
    ///
    /// # Returns
    /// Decrypted plaintext data
    ///
    /// # Example
    /// ```rust
    /// let decrypted = hybrid_crypto
    ///     .decrypt_hybrid(&encrypted, "private_key_2024")
    ///     .await?;
    ///
    /// println!("Decrypted data: {:?}", decrypted);
    /// ```
    pub async fn decrypt_hybrid(&self, encrypted: &HybridEncrypted, private_key_id: &str) -> Result<Vec<u8>, CryptoError>;

    /// Migrate from classical to post-quantum mode
    ///
    /// # Arguments
    /// * `target_mode` - Target cryptographic mode
    ///
    /// # Returns
    /// Migration result with updated keys
    ///
    /// # Example
    /// ```rust
    /// let migration_result = hybrid_crypto
    ///     .migrate_to_mode(CryptoMode::PostQuantum)
    ///     .await?;
    ///
    /// println!("Migrated {} keys", migration_result.migrated_keys.len());
    /// ```
    pub async fn migrate_to_mode(&mut self, target_mode: CryptoMode) -> Result<MigrationResult, CryptoError>;
}
```

### Post-Quantum Algorithms

```rust
use secreton::crypto::{PqAlgorithm, MlDsaKey, MlKemKey};

/// Supported post-quantum algorithms
#[derive(Debug, Clone, PartialEq)]
pub enum PqAlgorithm {
    /// ML-DSA (Module-Lattice-Based Digital Signature Algorithm)
    MlDsa44,    // NIST Level 2 security
    MlDsa65,    // NIST Level 3 security
    MlDsa87,    // NIST Level 5 security

    /// ML-KEM (Module-Lattice-Based Key Encapsulation Mechanism)
    MlKem512,   // NIST Level 1 security
    MlKem768,   // NIST Level 3 security
    MlKem1024,  // NIST Level 5 security
}

impl PqAlgorithm {
    /// Get security level for the algorithm
    ///
    /// # Returns
    /// NIST security level (1-5)
    ///
    /// # Example
    /// ```rust
    /// let algorithm = PqAlgorithm::MlDsa65;
    /// let level = algorithm.security_level(); // Returns 3
    /// ```
    pub fn security_level(&self) -> u8;

    /// Check if algorithm is suitable for long-term use
    ///
    /// # Returns
    /// Boolean indicating long-term suitability
    ///
    /// # Example
    /// ```rust
    /// let algorithm = PqAlgorithm::MlKem1024;
    ///
    /// if algorithm.is_long_term_secure() {
    ///     // Suitable for long-term secret protection
    /// }
    /// ```
    pub fn is_long_term_secure(&self) -> bool;

    /// Get recommended use cases for the algorithm
    ///
    /// # Returns
    /// Vector of recommended use cases
    ///
    /// # Example
    /// ```rust
    /// let algorithm = PqAlgorithm::MlDsa65;
    /// let use_cases = algorithm.recommended_use_cases();
    ///
    /// for use_case in use_cases {
    ///     println!("Recommended for: {}", use_case);
    /// }
    /// ```
    pub fn recommended_use_cases(&self) -> Vec<String>;
}
```

### Key Management

```rust
use secreton::crypto::{PqKeyManager, KeyRotationPolicy};

impl PqKeyManager {
    /// Generate new post-quantum key pair
    ///
    /// # Arguments
    /// * `algorithm` - Post-quantum algorithm to use
    /// * `key_id` - Identifier for the new key
    /// * `metadata` - Key metadata including usage and expiration
    ///
    /// # Returns
    /// Generated key pair
    ///
    /// # Example
    /// ```rust
    /// let key_pair = pq_manager
    ///     .generate_key_pair(
    ///         PqAlgorithm::MlDsa65,
    ///         "signing_key_2024_q4",
    ///         KeyMetadata {
    ///             usage: KeyUsage::Signing,
    ///             expires_at: Utc::now() + Duration::days(365),
    ///             satker_code: Some("KJA001".to_string()),
    ///         }
    ///     )
    ///     .await?;
    /// ```
    pub async fn generate_key_pair(
        &self,
        algorithm: PqAlgorithm,
        key_id: &str,
        metadata: KeyMetadata,
    ) -> Result<PqKeyPair, CryptoError>;

    /// Rotate keys according to policy
    ///
    /// # Arguments
    /// * `policy` - Key rotation policy
    ///
    /// # Returns
    /// Rotation result with old and new keys
    ///
    /// # Example
    /// ```rust
    /// let policy = KeyRotationPolicy {
    ///     rotation_interval: Duration::days(90),
    ///     overlap_period: Duration::days(7),
    ///     algorithm_upgrade: Some(PqAlgorithm::MlDsa87),
    /// };
    ///
    /// let rotation_result = pq_manager.rotate_keys(&policy).await?;
    ///
    /// println!("Rotated {} keys", rotation_result.rotated_count);
    /// ```
    pub async fn rotate_keys(&self, policy: &KeyRotationPolicy) -> Result<RotationResult, CryptoError>;

    /// Archive old keys securely
    ///
    /// # Arguments
    /// * `key_ids` - Keys to archive
    /// * `archive_policy` - Archival policy
    ///
    /// # Returns
    /// Archive result
    ///
    /// # Example
    /// ```rust
    /// let archive_policy = ArchivePolicy {
    ///     encryption_algorithm: PqAlgorithm::MlKem1024,
    ///     retention_period: Duration::days(2555), // 7 years
    ///     compliance_flags: vec!["KEJAKSAAN_ARCHIVE".to_string()],
    /// };
    ///
    /// let result = pq_manager
    ///     .archive_keys(&["old_key_1", "old_key_2"], &archive_policy)
    ///     .await?;
    /// ```
    pub async fn archive_keys(&self, key_ids: &[&str], archive_policy: &ArchivePolicy) -> Result<ArchiveResult, CryptoError>;
}
```
## Integration Examples

### Complete Authentication Flow

```rust
use authenc::{AuthencService, models::{User, Credentials}};
use secreton::{SecretonService, auth::AuthencAuthProvider};

/// Complete example of user authentication and secret access
async fn authenticate_and_access_secret() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize services
    let authenc = AuthencService::new().await?;
    let secreton = SecretonService::new().await?;

    // Step 1: Authenticate user
    let credentials = Credentials::Password {
        nip: "198501012010011001",
        password: "secure_password",
    };

    let auth_result = authenc
        .authenticate_user("198501012010011001", &credentials)
        .await?;

    if !auth_result.success {
        return Err("Authentication failed".into());
    }

    // Step 2: Generate JWT token
    let claims = Claims {
        sub: auth_result.user.nip.clone(),
        satker_code: auth_result.user.satker_code.clone(),
        roles: auth_result.user.roles.iter().map(|r| r.name.clone()).collect(),
        exp: Utc::now() + Duration::hours(8),
    };

    let token = authenc.crypto_engine()
        .sign_jwt_for_government(&claims)
        .await?;

    // Step 3: Access secret using token
    let secret_path = format!("{}/database_config", auth_result.user.satker_code);

    // Validate token in secreton
    let auth_provider = AuthencAuthProvider::new("https://authenc.internal").await?;
    let token_validation = auth_provider.validate_token(&token).await?;

    if !token_validation.valid {
        return Err("Token validation failed".into());
    }

    // Check permissions
    let has_permission = auth_provider
        .check_resource_permissions(&auth_result.user, &secret_path)
        .await?;

    if !has_permission {
        return Err("Insufficient permissions".into());
    }

    // Retrieve secret
    let secret = secreton.enhanced_engine()
        .get_application_secret("simkari_portal", "database_config")
        .await?;

    println!("Successfully retrieved secret: {}", secret.path);
    println!("Secret created by NIP: {}", secret.metadata.created_by_nip.unwrap_or_default());

    Ok(())
}
```

### Batch Operations Example

```rust
use authenc::vault::SecretonClient;
use secreton::engines::EnhancedSecretEngine;

/// Example of batch operations for performance
async fn batch_operations_example() -> Result<(), Box<dyn std::error::Error>> {
    let secreton_client = SecretonClient::new("https://secreton.internal").await?;
    let secret_engine = EnhancedSecretEngine::new().await?;

    // Batch token validation
    let tokens = vec![
        "jwt_tokento_string(),
        "jwt_token_2".to_string(),
        "jwt_token_3".to_string(),
    ];

    let validation_results = authenc.crypto_engine()
        .verify_token_batch(&tokens)
        .await?;

    // Batch secret retrieval
    let resource_ids = vec![
        "KJA001/database_config".to_string(),
        "KJA001/api_keys".to_string(),
        "KJA001/encryption_keys".to_string(),
    ];

    let secrets = secret_engine
        .batch_get_secrets(&resource_ids)
        .await?;

    // Process results
    for (i, (token_valid, secret)) in validation_results.iter().zip(secrets.iter()).enumerate() {
        if *token_valid {
            println!("Token {} valid, secret retrieved: {}", i, secret.path);
        } else {
            println!("Token {} invalid, skipping secret access", i);
        }
    }

    Ok(())
}
```

### Post-Quantum Migration Example

```rust
use secreton::crypto::{HybridCrypto, CryptoMode, PqAlgorithm};

/// Example of migrating to post-quantum cryptography
async fn post_quantum_migration_example() -> Result<(), Box<dyn std::error::Error>> {
    // Start with hybrid mode
    let mut hybrid_crypto = HybridCrypto::new(CryptoMode::Hybrid)?;

    // Sign document with hybrid approach
    let document = b"important legal document";
    let hybrid_signature = hybrid_crypto
        .sign_hybrid(document, "legal_signing_key")
        .await?;

    println!("Document signed with hybrid cryptography");
    println!("Classical signature length: {}", hybrid_signature.classical.len());
    printlnst-quantum signature length: {}", hybrid_signature.post_quantum.len());

    // Encrypt sensitive data with hybrid KEM
    let sensitive_data = b"classified case inftion";
    let hybrid_encrypted = hybrid_crypto
        .encrypt_hybrid(sensitive_data, "case_encryption_key")
        .await?;

    println!("Data encrypted with hybrid KEM");

    // Migrate to pure post-quantum mode
    let migration_result = hybrid_crypto
        .migrate_to_mode(CryptoMode::PostQuantum)
        .await?;

    println!("Migration completed:");
    println!("- Migrated {} keys", migration_result.migrated_keys.len());
    println!("- Archived {} old keys", migration_result.archived_keys.len());

    // Verify signature still works after migration
    let is_valid = hybrid_crypto
        .verify_hybrid(document, &hybrid_signature, "legal_public_key")
        .await?;

    if is_valid {
        println!("Signature verification successful after migration");
    }

    Ok(())
}
```

### Role Management Example

```rust
use authenc::models::{Role, RoleScope, AdminLevel, Permission};

/// Example of hierarchical role management
async fn role_management_example() -> Result<(), Box<dyn std::error::Error>> {
    let authenc = AuthencService::new().await?;

    // Admin Pusat creates wilayah-level role
    let admin_pusat = AdminLevel::AdminPusat;

    let wilayah_admin_role = Role::new(
        "Admin Wilayah SUMUT",
        RoleScope::Wilayah("SUMUT".to_string()),
        vec![
            Permission::ManageUsers,
            Permission::ManageRoles,
            Permission::ReadAuditLogs,
            Permission::AccessPortal,
        ],
        admin_pusat.clone(),
    )?;

    // Admin Wilayah creates satker-level role
    let admin_wilayah = AdminLevel::AdminWilayah("SUMUT".to_string());

    let jaksa_role = Role::new(
        "Jaksa Penuntut KJA001",
        RoleScope::Satker("KJA001".to_string()),
        vec![
            Permission::ReadCases,
            Permission::WriteCases,
            Permission::ReadEvidence,
            Permission::AccessPidsus,
        ],
        admin_wilayah.clone(),
    )?;

    // Check role hierarchy
    if wilayah_admin_role.can_be_managed_by(&admin_pusat) {
        println!("Admin Pusat can manage wilayah role");
    }

    if jaksa_role.can_be_managed_by(&admin_wilayah) {
        println!("Admin Wilayah can manage jaksa role");
    }

    // Check permissions
    if jaksa_role.grants_permission(&Permission::ReadCases, "KJA001") {
        println!("Jaksa can read cases in their satker");
    }

    if !jaksa_role.grants_permission(&Permission::ReadCases, "KJA002") {
        println!("Jaksa cannot read cases in other satker");
    }

    Ok(())
}
```

### Application Integration Example

```rust
use authenc::vault::SecretonClient;
use secreton::engines::EnhancedSecretEngine;

/// Example of SIMKARI application integration
async fn simkari_application_integration() -> Result<(), Box<dyn std::error::Error>> {
    let secreton_client = SecretonClient::new("https://secreton.internal").await?;
    let secret_engine = EnhancedSecretEngine::new().await?;

    // Get application configuration
    let portal_config = secreton_client
        .get_application_config("simkari_portal")
        .await?;

    println!("Portal configuration loaded:");
    println!("- Database URL: {}", portal_config.database_url);
    println!("- Redis URL: {}", portal_config.redis_url);
    println!("- API Keys: {} configured", portal_config.api_keys.len());

    // Get satker-specific secrets
    let satker_secrets = vec![
        "KJA001/database_password",
        "KJA001/encryption_key",
        "KJA001/signing_certificate",
    ];

    for secret_path in satker_secrets {
        let secret = secret_engine
            .get_application_secret("simkari_portal", secret_path)
            .await?;

        println!("Retrieved secret: {}", secret.path);
        println!("- Owner satker: {}", secret.satker_owner);
        println!("- Audit required: {}", secret.access_control.audit_required);
    }

    // Validate application token
    let app_token = "application_jwt_token_here";
    let is_valid = secret_engine
        .validate_application_token(app_token, "simkari_portal")
        .await?;

    if is_valid {
        println!("Application token is valid");
    }

    Ok(())
}
```

## Error Handling

### Authenc Error Types

```rust
use authenc::error::{AuthencError, SecretonIntegrationError};

/// Comprehensive error handling for authenc operations
#[derive(Error, Debug)]
pub enum AuthencError {
    // Authentication errors
    #[error("Invalid credentials for NIP: {nip}")]
    InvalidCredentials { nip: String },

    #[error("User not found: {nip}")]
    UserNotFound { nip: String },

    #[error("Account locked for NIP: {nip}")]
    AccountLocked { nip: String },

    // Authorization errors
    #[error("Insufficient permissions for operation: {operation}")]
    InsufficientPermissions { operation: String },

    #[error("Access denied to resource: {resource}")]
    AccessDenied { resource: String },

    #[error("Role not found: {role_name}")]
    RoleNotFound { role_name: String },

    // Token errors
    #[error("Invalid JWT token")]
    InvalidToken,

    #[error("Token expired at: {expired_at}")]
    TokenExpired { expired_at: DateTime<Utc> },

    #[error("Token signature verification failed")]
    TokenSignatureInvalid,

    // Secreton integration errors
    #[error("Secreton communication failed: {message}")]
    SecretonCommunicationError { message: String },

    #[error("Secret access denied: {path}")]
    SecretAccessDenied { path: String },

    #[error("Secreton authentication failed")]
    SecretonAuthenticationFailed,

    #[error("Secret not found in secreton: {path}")]
    SecretNotFound { path: String },

    // Cryptographic errors
    #[error(phic operation failed: {operation}")]
    CryptographicError { operation: String },

    #[error("Key not found: {key_id}")]
    KeyNotFound { key_id: String },

    #[error("Post-quantum operation failed: {algorithm:?}")]
    PostQuantumError { algorithm: PqAlgorithm },

    // System errors
    #[error("Database error: {message}")]
    DatabaseError { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },
}

impl AuthencError {
    /// Check if error is related to secreton integration
    pub fn is_secreton_related(&self) -> bool {
        matches!(self,
            AuthencError::SecretonCommunicationError { .. } |
            AuthencError::SecretAccessDenied { .. } |
            AuthencError::SecretonAuthenticationFailed |
            AuthencError::SecretNotFound { .. }
        )
    }

    /// Check if operation should be retried
    pub fn should_retry(&self) -> bool {
        matches!(self,
            AuthencError::SecretonCommunicationError { .. } |
            AuthencError::NetworkError { .. } |
            AuthencError::DatabaseError { .. }
        )
    }

    /// Get retry delay for retryable errors
    pub fn retry_delay(&self) -> Option<Duration> {
        match self {
            AuthencError::SecretonCommunicationError { .. } => Some(Duration::seconds(5)),
            AuthencError::NetworkError { .. } => Some(Duration::seconds(2)),
            AuthencError::DatabaseError { .. } =>n::seconds(1)),
            _ => None,
        }
    }

    /// Check if error requires user reauthentication
    pub fn requires_reauthentication(&self) -> bool {
        matches!(self,
            AuthencError::InvalidToken |
            AuthencError::TokenExpired { .. } |
            AuthencError::TokenSignatureInvalid |
            AuthencError::SecretonAuthenticationFailed
        )
    }
}
```

### Secreton Error Types

```rust
use secreton::error::{SecretonError, AuthencIntegrationError};

/// Comprehensive error handling for secreton operations
#[derive(Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecretonError {
    // Secret management errors
    #[error("Secret not found: {path}")]
    SecretNotFound { path: String },

    #[error("Secret already exists: {path}")]
    SecretAlreadyExists { path: String },

    #[error("Invalid secret path: {path}")]
    InvalidSecretPath { path: String },

    #[error("Secret value too large: {size} bytes")]
    SecretTooLarge { size: usize },

    // Access control errors
    #[error("Access denied for NIP {nip} to resource: {resource}")]
    AccessDenied { nip: String, resource: String },

    #[error("Insufficient role permissions: {required_role}")]
    InsufficientRolePermissions { required_role: String },

    #[error("Satker access denied: {satker_code}")]
    SatkerAccessDenied { satker_code: String },

    #[error("Time-based access restriction violated")]
    TimeBasedAccessDenied,

    // Authenc integration errors
    #[error("Authenc token validation failed: {reason}")]
    AuthencTokenValidationFailed { reason: String },

    #[error("Authenc authentication failed: {message}")]
    AuthencAuthenticationFailed { message: String },

    #[error("IAM permission denied for operation: {operation}")]
    IamPermissionDenied { operation: String },

    #[error("Authenc communication timeout")]
    AuthencCommunicationTimeout,

    // Cryptographic errors
    #[error("Encryption failed: {algorithm}")]
    EncryptionFailed { algorithm: String },

    #[error("Decryption failed: {algorithm}")]
    DecryptionFailed { algorithm: String },

    #[error("Key derivation failed")]
    KeyDerivationFailed,

    #[error("Post-quantum operation failed: {algorithm:?}")]
    PostQuantumOperationFailed { algorithm: PqAlgorithm },

    // Storage errors
    #[error("Storage backend error: {message}")]
    StorageError { message: String },

    #[error("Database connection failed")]
    DatabaseConnectionFailed,

    #[error("Transaction failed: {reason}")]
    TransactionFailed { reason: String },

    // Audit errors
    #[error("Audit logging failed: {event_type}")]
    AuditLoggingFailed { event_type: String },

    #[error("Compliance violation: {violation}")]
    ComplianceViolation { violation: String },

    // System errors
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },
}

impl SecretonError {
    /// Check if error is related to authenc integra
b fn is_authenc_related(&self) -> bool {
        matches!(self,
            SecretonError::AuthencTokenValidationFailed { .. } |
            SecretonError::AhencAuthenticationFailed { .. } |
            SecretonError::IamPermissionDenied { .. } |
            SecretonError::AuthencCommunicationTimeout
        )
    }

    /// Check if error requires reauthentication
    pub fn requires_reauthentication(&self) -> bool {
        matches!(self,
            SecretonError::AuthencTokenValidationFailed { .. } |
            SecretonError::AuthencAuthenticationFailed { .. }
        )
    }

    /// Check if operation should be retried
    pub fn should_retry(&self) -> bool {
        matches!(self,
            SecretonError::AuthencCommunicationTimeout |
            SecretonError::DatabaseConnectionFailed |
            SecretonError::ServiceUnavailable { .. }
        )
    }

    /// Get security severity level
    pub fn security_severity(&self) -> SecuritySeverity {
        match self {
            SecretonError::AccessDenied { .. } |
            SecretonError::InsufficientRolePermissions { .. } |
            SecretonError::SatkerAccessDenied { .. } => SecuritySeverity::High,

            SecretonError::ComplianceViolation { .. } |
            SecretonError::AuditLoggingFailed { .. } => SecuritySeverity::Critical,

            SecretonError::PostQuantumOperationFailed { .. } |
            SecretonError::EncryptionFailed { .. } |
            SecretonError::DecryptionFailed { .. } => SecuritySeverity::Medium,

            _ => SecuritySeverity::Low,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}
```

### Error Recovery Patterns

```rust
use std::time::Duration;
use tokio::time::sleep;

/// Retry pattern for recoverable errors
pub async fn retry_with_backoff<T, E, F, Fut>(
    operation: F,
    max_retries: usize,
    base_delay: Duration,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut retries = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if retries >= max_retries {
                    return Err(error);
                }

                let delay = base_delay * 2_u32.pow(retries as u32);
                sleep(delay).await;
                retries += 1;
            }
        }
    }
}

/// Circuit breaker pattern for service protection
pub struct CircuitBreaker {
    failure_threshold: usize,
    recovery_timeout: Duration,
    failure_count: usize,
    last_failure_time: Option<Instant>,
    state: CircuitBreakerState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,     // Normal operation
    Open,       // Failing, reject requests
    HalfOpen,   // Testing recovery
}

impl CircuitBreaker {
    /// Execute operation with circuit breaker protection
    pub async fn execute<T, E, F, Fut>(&mut self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        match self.state {
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() > self.recovery_timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                    } else {
                        return Err(CircuitBreakerError::CircuitOpen);
                    }
                }
            }
            CircuitBreakerState::HalfOpen | CircuitBreakerState::Closed => {}
        }

        match operation().await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(error) => {
                self.on_failure();
                Err(CircuitBreakerError::OperationFailed(error))
            }
        }
    }

    fn on_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitBreakerState::Closed;
    }

    fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitBreakerState::Open;
        }
    }
}

#[derive(Error, Debug)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit breaker is open")]
    CircuitOpen,

    #[error("Operation failed: {0:?}")]
    OperationFailed(E),
}
```

This comprehensive API documentation covers all the enhanced functionality, role-based access control system, post-quantum cryptography integration, and proper error handling patterns for the SIMKARI super app platform.
