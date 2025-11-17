use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Protocol mapper entity - maps user/client attributes to protocol-specific claims/attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMapper {
    /// Unique identifier
    pub id: Uuid,
    /// Client ID this mapper belongs to (optional - can be realm-level)
    pub client_id: Option<Uuid>,
    /// Client scope ID this mapper belongs to (optional)
    pub client_scope_id: Option<Uuid>,
    /// Realm ID
    pub realm_id: Uuid,
    /// Mapper name
    pub name: String,
    /// Protocol (openid-connect, saml)
    pub protocol: String,
    /// Mapper type
    pub mapper_type: ProtocolMapperType,
    /// Configuration
    pub config: ProtocolMapperConfiguration,
    /// Whether mapper is enabled
    pub enabled: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Protocol mapper type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProtocolMapperType {
    /// Maps user properties to token claims
    UserProperty,
    /// Maps user roles to token claims
    UserRole,
    /// Maps realm roles to token claims
    UserRealmRole,
    /// Maps client roles to token claims
    UserClientRole,
    /// Maps user groups to token claims
    UserGroup,
    /// Maps user attributes to token claims
    UserAttribute,
    /// Maps hardcoded values to token claims
    HardcodedClaim,
    /// Maps full name (firstName + lastName)
    FullName,
    /// Maps audience to token
    Audience,
    /// Custom script-based mapping
    Script,
}

impl std::fmt::Display for ProtocolMapperType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserProperty => write!(f, "user-property"),
            Self::UserRole => write!(f, "user-role"),
            Self::UserRealmRole => write!(f, "user-realm-role"),
            Self::UserClientRole => write!(f, "user-client-role"),
            Self::UserGroup => write!(f, "user-group"),
            Self::UserAttribute => write!(f, "user-attribute"),
            Self::HardcodedClaim => write!(f, "hardcoded-claim"),
            Self::FullName => write!(f, "full-name"),
            Self::Audience => write!(f, "audience"),
            Self::Script => write!(f, "script"),
        }
    }
}

impl std::str::FromStr for ProtocolMapperType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user-property" => Ok(Self::UserProperty),
            "user-role" => Ok(Self::UserRole),
            "user-realm-role" => Ok(Self::UserRealmRole),
            "user-client-role" => Ok(Self::UserClientRole),
            "user-group" => Ok(Self::UserGroup),
            "user-attribute" => Ok(Self::UserAttribute),
            "hardcoded-claim" => Ok(Self::HardcodedClaim),
            "full-name" => Ok(Self::FullName),
            "audience" => Ok(Self::Audience),
            "script" => Ok(Self::Script),
            _ => Err(format!("Unknown protocol mapper type: {}", s)),
        }
    }
}

/// Protocol mapper configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMapperConfiguration {
    /// Claim name in the token
    pub claim_name: String,
    /// Claim value type (String, long, int, boolean, JSON)
    pub claim_value_type: Option<String>,
    /// User attribute name (for user-attribute mapper)
    pub user_attribute: Option<String>,
    /// User property name (for user-property mapper)
    pub user_property: Option<String>,
    /// Hardcoded claim value (for hardcoded-claim mapper)
    pub claim_value: Option<String>,
    /// Role name prefix (for role mappers)
    pub role_prefix: Option<String>,
    /// Whether to include client roles (for role mappers)
    pub include_client_roles: Option<bool>,
    /// Whether to include realm roles (for role mappers)
    pub include_realm_roles: Option<bool>,
    /// Client ID for client role mapping
    pub client_id_for_role_mappings: Option<String>,
    /// Group path for group mapping
    pub group_path: Option<String>,
    /// Whether to include full group path
    pub full_group_path: Option<bool>,
    /// Whether this claim should be included in access tokens
    pub include_in_access_token: bool,
    /// Whether this claim should be included in identity tokens
    pub include_in_id_token: bool,
    /// Whether this claim should be included in user info
    pub include_in_userinfo: bool,
    /// Multivalued attribute flag
    pub multivalued: Option<bool>,
    /// Aggregate attribute values flag
    pub aggregate_attrs: Option<bool>,
    /// JSON type label for complex claims
    pub json_type_label: Option<String>,
    /// Audience value (for audience mapper)
    pub included_client_audience: Option<String>,
    /// Custom audience value
    pub included_custom_audience: Option<String>,
    /// Script text (for script mapper)
    pub script: Option<String>,
    /// Additional custom configuration
    #[serde(flatten)]
    pub additional_config: HashMap<String, serde_json::Value>,
}

impl Default for ProtocolMapperConfiguration {
    fn default() -> Self {
        Self {
            claim_name: String::new(),
            claim_value_type: Some("String".to_string()),
            user_attribute: None,
            user_property: None,
            claim_value: None,
            role_prefix: None,
            include_client_roles: Some(false),
            include_realm_roles: Some(true),
            client_id_for_role_mappings: None,
            group_path: None,
            full_group_path: Some(false),
            include_in_access_token: true,
            include_in_id_token: true,
            include_in_userinfo: true,
            multivalued: Some(false),
            aggregate_attrs: Some(false),
            json_type_label: None,
            included_client_audience: None,
            included_custom_audience: None,
            script: None,
            additional_config: HashMap::new(),
        }
    }
}

/// Request to create a protocol mapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProtocolMapperRequest {
    /// Mapper name
    pub name: String,
    /// Protocol (openid-connect, saml)
    pub protocol: String,
    /// Mapper type
    pub mapper_type: ProtocolMapperType,
    /// Configuration
    pub config: ProtocolMapperConfiguration,
    /// Client ID (optional)
    pub client_id: Option<Uuid>,
    /// Client scope ID (optional)
    pub client_scope_id: Option<Uuid>,
}

/// Request to update a protocol mapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProtocolMapperRequest {
    /// Mapper name (optional)
    pub name: Option<String>,
    /// Configuration (optional)
    pub config: Option<ProtocolMapperConfiguration>,
    /// Enabled status (optional)
    pub enabled: Option<bool>,
}

/// Protocol mapper response DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMapperResponse {
    pub id: Uuid,
    pub name: String,
    pub protocol: String,
    pub mapper_type: ProtocolMapperType,
    pub config: ProtocolMapperConfiguration,
    pub client_id: Option<Uuid>,
    pub client_scope_id: Option<Uuid>,
    pub realm_id: Uuid,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProtocolMapper> for ProtocolMapperResponse {
    fn from(mapper: ProtocolMapper) -> Self {
        Self {
            id: mapper.id,
            name: mapper.name,
            protocol: mapper.protocol,
            mapper_type: mapper.mapper_type,
            config: mapper.config,
            client_id: mapper.client_id,
            client_scope_id: mapper.client_scope_id,
            realm_id: mapper.realm_id,
            enabled: mapper.enabled,
            created_at: mapper.created_at,
            updated_at: mapper.updated_at,
        }
    }
}

/// Standard OIDC protocol mappers
pub mod standard_mappers {
    use super::*;

    /// Create username mapper
    pub fn username_mapper(realm_id: Uuid) -> ProtocolMapper {
        ProtocolMapper {
            id: Uuid::new_v4(),
            client_id: None,
            client_scope_id: None,
            realm_id,
            name: "username".to_string(),
            protocol: "openid-connect".to_string(),
            mapper_type: ProtocolMapperType::UserProperty,
            config: ProtocolMapperConfiguration {
                claim_name: "preferred_username".to_string(),
                claim_value_type: Some("String".to_string()),
                user_property: Some("username".to_string()),
                include_in_access_token: true,
                include_in_id_token: true,
                include_in_userinfo: true,
                ..Default::default()
            },
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Create email mapper
    pub fn email_mapper(realm_id: Uuid) -> ProtocolMapper {
        ProtocolMapper {
            id: Uuid::new_v4(),
            client_id: None,
            client_scope_id: None,
            realm_id,
            name: "email".to_string(),
            protocol: "openid-connect".to_string(),
            mapper_type: ProtocolMapperType::UserProperty,
            config: ProtocolMapperConfiguration {
                claim_name: "email".to_string(),
                claim_value_type: Some("String".to_string()),
                user_property: Some("email".to_string()),
                include_in_access_token: true,
                include_in_id_token: true,
                include_in_userinfo: true,
                ..Default::default()
            },
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Create full name mapper
    pub fn full_name_mapper(realm_id: Uuid) -> ProtocolMapper {
        ProtocolMapper {
            id: Uuid::new_v4(),
            client_id: None,
            client_scope_id: None,
            realm_id,
            name: "full name".to_string(),
            protocol: "openid-connect".to_string(),
            mapper_type: ProtocolMapperType::FullName,
            config: ProtocolMapperConfiguration {
                claim_name: "name".to_string(),
                claim_value_type: Some("String".to_string()),
                include_in_access_token: false,
                include_in_id_token: true,
                include_in_userinfo: true,
                ..Default::default()
            },
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Create realm roles mapper
    pub fn realm_roles_mapper(realm_id: Uuid) -> ProtocolMapper {
        ProtocolMapper {
            id: Uuid::new_v4(),
            client_id: None,
            client_scope_id: None,
            realm_id,
            name: "realm roles".to_string(),
            protocol: "openid-connect".to_string(),
            mapper_type: ProtocolMapperType::UserRealmRole,
            config: ProtocolMapperConfiguration {
                claim_name: "realm_access.roles".to_string(),
                claim_value_type: Some("JSON".to_string()),
                multivalued: Some(true),
                include_in_access_token: true,
                include_in_id_token: false,
                include_in_userinfo: false,
                ..Default::default()
            },
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Create groups mapper
    pub fn groups_mapper(realm_id: Uuid) -> ProtocolMapper {
        ProtocolMapper {
            id: Uuid::new_v4(),
            client_id: None,
            client_scope_id: None,
            realm_id,
            name: "groups".to_string(),
            protocol: "openid-connect".to_string(),
            mapper_type: ProtocolMapperType::UserGroup,
            config: ProtocolMapperConfiguration {
                claim_name: "groups".to_string(),
                claim_value_type: Some("JSON".to_string()),
                full_group_path: Some(true),
                multivalued: Some(true),
                include_in_access_token: true,
                include_in_id_token: true,
                include_in_userinfo: true,
                ..Default::default()
            },
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// Conversion from database row
impl TryFrom<tokio_postgres::Row> for ProtocolMapper {
    type Error = crate::error::AuthencError;

    fn try_from(row: tokio_postgres::Row) -> Result<Self, Self::Error> {
        let config_json: serde_json::Value = row.try_get("config").map_err(|e| {
            crate::error::AuthencError::database(format!("Failed to get config: {}", e))
        })?;

        let config: ProtocolMapperConfiguration =
            serde_json::from_value(config_json).map_err(|e| {
                crate::error::AuthencError::internal(&format!(
                    "Failed to parse mapper config: {}",
                    e
                ))
            })?;

        let mapper_type_str: String = row.try_get("mapper_type").map_err(|e| {
            crate::error::AuthencError::database(format!("Failed to get mapper_type: {}", e))
        })?;

        let mapper_type = mapper_type_str
            .parse()
            .map_err(|e: String| crate::error::AuthencError::internal(&e))?;

        Ok(Self {
            id: row.try_get("id").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get id: {}", e))
            })?,
            client_id: row.try_get("client_id").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get client_id: {}", e))
            })?,
            client_scope_id: row.try_get("client_scope_id").map_err(|e| {
                crate::error::AuthencError::database(format!(
                    "Failed to get client_scope_id: {}",
                    e
                ))
            })?,
            realm_id: row.try_get("realm_id").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get realm_id: {}", e))
            })?,
            name: row.try_get("name").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get name: {}", e))
            })?,
            protocol: row.try_get("protocol").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get protocol: {}", e))
            })?,
            mapper_type,
            config,
            enabled: row.try_get("enabled").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get enabled: {}", e))
            })?,
            created_at: row.try_get("created_at").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get created_at: {}", e))
            })?,
            updated_at: row.try_get("updated_at").map_err(|e| {
                crate::error::AuthencError::database(format!("Failed to get updated_at: {}", e))
            })?,
        })
    }
}
