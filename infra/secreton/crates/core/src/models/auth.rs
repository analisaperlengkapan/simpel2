use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Canonical LoginRequest - use this throughout the project
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `LoginRequest`.
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub mfa_code: Option<String>,
    pub remember_me: Option<bool>,
}

/// Canonical LoginResponse - use this throughout the project
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `LoginResponse`.
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserInfo,
    pub mfa_required: bool,
}

/// Canonical RefreshTokenRequest
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `RefreshTokenRequest`.
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `AuthRequest`.
pub enum AuthRequest {
    Token { token: String },
    UserPass { username: String, password: String },
    Ldap { username: String, password: String },
    Oidc { code: String, state: String },
    Okta { username: String, password: String },
    Github { token: String },
    Radius { username: String, password: String },
    AppRole { role_id: String, secret_id: String },
    Kubernetes { jwt: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `AuthResponse`.
pub struct AuthResponse {
    pub authenticated: bool,
    pub user_info: UserInfo,
    pub policies: Vec<String>,
    pub lease_duration: i64,
    pub renewable: bool,
    pub token: String,
    pub accessor: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `UserInfo`.
pub struct UserInfo {
    pub username: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub groups: Vec<String>,
    pub policies: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `AuthMethod`.
pub struct AuthMethod {
    pub name: String,
    pub method_type: AuthMethodType,
    pub config: HashMap<String, String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Mewakili pub `AuthMethodType`.
pub enum AuthMethodType {
    Token,
    UserPass,
    Ldap,
    Oidc,
    Okta,
    Github,
    Radius,
    AppRole,
    Kubernetes,
}

impl AuthMethod {
    /// Mewakili pub `new(name`.
    pub fn new(name: String, method_type: AuthMethodType) -> Self {
        Self {
            name,
            method_type,
            config: HashMap::new(),
            enabled: true,
        }
    }

    /// Mewakili pub `with_config(mut`.
    pub fn with_config(mut self, key: String, value: String) -> Self {
        self.config.insert(key, value);
        self
    }

    /// Mewakili pub `enable(mut`.
    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Mewakili pub `disable(mut`.
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }
}
