use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User role enum shared across services
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub enum UserRole {
    /// System administrator
    Admin,
    /// Regular user
    #[default]
    User,
    /// Supervisor
    Supervisor,
    /// Guest (read-only)
    Guest,
    /// Custom role defined by external provider or dynamic configuration
    Custom(String),
}

impl UserRole {
    /// Get role display name in Indonesian
    pub fn display_name(&self) -> String {
        match self {
            Self::Admin => "Administrator".to_string(),
            Self::User => "Pengguna".to_string(),
            Self::Supervisor => "Supervisor".to_string(),
            Self::Guest => "Tamu".to_string(),
            Self::Custom(name) => name.clone(),
        }
    }

    /// Check if role has admin privileges
    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Admin)
    }

    /// Check if role can manage users
    pub fn can_manage_users(&self) -> bool {
        matches!(self, Self::Admin | Self::Supervisor)
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
