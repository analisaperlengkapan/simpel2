use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User role enum shared across services.
///
/// NOTE: Specific admin hierarchy roles (AdminPusat, AdminEselonI, AdminWilayah,
/// AdminSatker, etc.) are now represented using Custom(String) for flexibility.
/// This allows roles to be defined in the database and integrated with external
/// identity providers like Keycloak or Vault without code changes.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub enum UserRole {
    /// System administrator (global)
    Admin,
    /// Regular user
    #[default]
    User,
    /// Supervisor
    Supervisor,
    /// Guest (read-only)
    Guest,
    /// Custom role defined by external provider or dynamic configuration.
    /// Examples: "admin_pusat", "admin_eselon_i", "admin_wilayah", "admin_satker",
    /// or any role from Keycloak/Vault.
    Custom(String),
}

impl UserRole {
    /// Get role display name in Indonesian.
    /// For Custom roles, returns a formatted display name based on the role string.
    pub fn display_name(&self) -> String {
        match self {
            Self::Admin => "Administrator Global".to_string(),
            Self::User => "Pengguna".to_string(),
            Self::Supervisor => "Supervisor".to_string(),
            Self::Guest => "Tamu".to_string(),
            Self::Custom(name) => Self::format_custom_role_name(name),
        }
    }

    /// Format a custom role name into a display name.
    /// Examples:
    /// - "admin_pusat" -> "Admin Pusat"
    /// - "admin_eselon_i" -> "Admin Eselon I"
    /// - "operator_keuangan" -> "Operator Keuangan"
    fn format_custom_role_name(name: &str) -> String {
        name.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Check if role has admin privileges.
    /// Returns true for Admin or any Custom role starting with "admin".
    pub fn is_admin(&self) -> bool {
        match self {
            Self::Admin => true,
            Self::Custom(name) => name.to_lowercase().starts_with("admin"),
            _ => false,
        }
    }

    /// Check if role can manage users.
    /// Admins and Supervisors can manage users.
    pub fn can_manage_users(&self) -> bool {
        self.is_admin() || matches!(self, Self::Supervisor)
    }

    /// Create a Custom role from a string.
    /// Useful for roles from JWT claims or database.
    pub fn from_string(role: &str) -> Self {
        match role.to_lowercase().as_str() {
            "admin" => Self::Admin,
            "user" => Self::User,
            "supervisor" => Self::Supervisor,
            "guest" => Self::Guest,
            _ => Self::Custom(role.to_string()),
        }
    }
}

/// SSO session data stored in the cookie and shared with frontend
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SsoSession {
    /// Session ID
    pub session_id: String,

    /// User ID
    pub user_id: String,

    /// Username
    pub username: String,

    /// User email
    pub email: Option<String>,

    /// User roles
    pub roles: Vec<String>,

    /// Session creation timestamp
    pub created_at: DateTime<Utc>,

    /// Session expiration timestamp
    pub expires_at: DateTime<Utc>,

    /// Client IP address
    pub ip_address: Option<String>,

    /// User agent
    pub user_agent: Option<String>,
}

impl SsoSession {
    /// Create a new SSO session
    pub fn new(
        user_id: String,
        username: String,
        email: Option<String>,
        roles: Vec<String>,
        max_age_seconds: i64,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(max_age_seconds);

        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            username,
            email,
            roles,
            created_at: now,
            expires_at,
            ip_address,
            user_agent,
        }
    }

    /// Check if the session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the session is valid
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// Check if user has specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Serialize session to JSON string
    pub fn to_json(&self) -> Result<String, crate::error::CommonError> {
        serde_json::to_string(self).map_err(|e| {
            crate::error::CommonError::Internal(format!("Failed to serialize SSO session: {}", e))
        })
    }

    /// Deserialize session from JSON string
    pub fn from_json(json: &str) -> Result<Self, crate::error::CommonError> {
        serde_json::from_str(json).map_err(|e| {
            crate::error::CommonError::Internal(format!("Failed to deserialize SSO session: {}", e))
        })
    }
}
