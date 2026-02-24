use authenc_storage::Database;
use authenc_storage::operations::protocol_mappers_ops;
use authenc_types::domain::protocol_mapper::{
    CreateProtocolMapperRequest, ProtocolMapper, ProtocolMapperConfiguration, ProtocolMapperType,
    UpdateProtocolMapperRequest,
};
use authenc_types::domain::user::User;
use authenc_types::{AuthencError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Protocol mapper service for managing and evaluating protocol mappers
#[derive(Clone)]
pub struct ProtocolMapperService {
    db: Arc<Database>,
}

impl ProtocolMapperService {
    /// Create a new protocol mapper service
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    // ==================== CRUD Operations ====================

    /// Create a new protocol mapper
    pub async fn create_mapper(
        &self,
        realm_id: Uuid,
        request: CreateProtocolMapperRequest,
    ) -> Result<ProtocolMapper> {
        // Validate configuration based on mapper type
        self.validate_mapper_config(&request.mapper_type, &request.config)?;

        protocol_mappers_ops::create(&self.db, realm_id, request).await
    }

    /// Get protocol mapper by ID
    pub async fn get_mapper(&self, id: Uuid) -> Result<Option<ProtocolMapper>> {
        protocol_mappers_ops::get_by_id(&self.db, id).await
    }

    /// Get protocol mapper by name
    pub async fn get_mapper_by_name(
        &self,
        realm_id: Uuid,
        name: &str,
    ) -> Result<Option<ProtocolMapper>> {
        protocol_mappers_ops::get_by_name(&self.db, realm_id, name).await
    }

    /// List all protocol mappers in a realm
    pub async fn list_mappers(&self, realm_id: Uuid) -> Result<Vec<ProtocolMapper>> {
        protocol_mappers_ops::list_by_realm(&self.db, realm_id).await
    }

    /// List protocol mappers by client
    pub async fn list_mappers_by_client(&self, client_id: Uuid) -> Result<Vec<ProtocolMapper>> {
        protocol_mappers_ops::list_by_client(&self.db, client_id).await
    }

    /// List protocol mappers by client scope
    pub async fn list_mappers_by_client_scope(
        &self,
        client_scope_id: Uuid,
    ) -> Result<Vec<ProtocolMapper>> {
        protocol_mappers_ops::list_by_client_scope(&self.db, client_scope_id).await
    }

    /// List protocol mappers by protocol
    pub async fn list_mappers_by_protocol(
        &self,
        realm_id: Uuid,
        protocol: &str,
    ) -> Result<Vec<ProtocolMapper>> {
        protocol_mappers_ops::list_by_protocol(&self.db, realm_id, protocol).await
    }

    /// List protocol mappers by type
    pub async fn list_mappers_by_type(
        &self,
        realm_id: Uuid,
        mapper_type: ProtocolMapperType,
    ) -> Result<Vec<ProtocolMapper>> {
        protocol_mappers_ops::list_by_type(&self.db, realm_id, mapper_type).await
    }

    /// Update protocol mapper
    pub async fn update_mapper(
        &self,
        id: Uuid,
        request: UpdateProtocolMapperRequest,
    ) -> Result<ProtocolMapper> {
        // Validate config if provided
        if let Some(ref config) = request.config {
            let mapper = self
                .get_mapper(id)
                .await?
                .ok_or_else(|| AuthencError::not_found("Protocol mapper not found"))?;
            self.validate_mapper_config(&mapper.mapper_type, config)?;
        }

        protocol_mappers_ops::update(&self.db, id, request).await
    }

    /// Delete protocol mapper
    pub async fn delete_mapper(&self, id: Uuid) -> Result<()> {
        protocol_mappers_ops::delete(&self.db, id).await
    }

    /// Get effective mappers for a client (includes client-level and scope-level)
    pub async fn get_effective_mappers(
        &self,
        client_id: Uuid,
        scopes: &[String],
    ) -> Result<Vec<ProtocolMapper>> {
        protocol_mappers_ops::get_effective_mappers_for_client(&self.db, client_id, scopes).await
    }

    /// Initialize standard protocol mappers for a realm
    pub async fn initialize_standard_mappers(&self, realm_id: Uuid) -> Result<Vec<ProtocolMapper>> {
        protocol_mappers_ops::initialize_standard_mappers(&self.db, realm_id).await
    }

    // ==================== Mapper Evaluation ====================

    /// Apply protocol mappers to generate token claims
    pub async fn apply_mappers(
        &self,
        mappers: &[ProtocolMapper],
        user: &User,
        client_id: &str,
        protocol: &str,
        token_type: TokenType,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut claims = HashMap::new();

        for mapper in mappers {
            // Skip if mapper doesn't match protocol
            if mapper.protocol != protocol {
                continue;
            }

            // Skip if not enabled
            if !mapper.enabled {
                continue;
            }

            // Check if should be included in this token type
            let should_include = match token_type {
                TokenType::AccessToken => mapper.config.include_in_access_token,
                TokenType::IdToken => mapper.config.include_in_id_token,
                TokenType::UserInfo => mapper.config.include_in_userinfo,
            };

            if !should_include {
                continue;
            }

            // Evaluate mapper and merge claims
            let mapper_claims = self.evaluate_mapper(mapper, user, client_id).await?;
            claims.extend(mapper_claims);
        }

        Ok(claims)
    }

    /// Evaluate a single mapper
    async fn evaluate_mapper(
        &self,
        mapper: &ProtocolMapper,
        user: &User,
        _client_id: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        match mapper.mapper_type {
            ProtocolMapperType::UserProperty => self.evaluate_user_property_mapper(mapper, user),
            ProtocolMapperType::UserAttribute => self.evaluate_user_attribute_mapper(mapper, user),
            ProtocolMapperType::UserRole => self.evaluate_user_role_mapper(mapper, user),
            ProtocolMapperType::UserRealmRole => self.evaluate_user_realm_role_mapper(mapper, user),
            ProtocolMapperType::UserClientRole => {
                self.evaluate_user_client_role_mapper(mapper, user)
            }
            ProtocolMapperType::UserGroup => self.evaluate_user_group_mapper(mapper, user),
            ProtocolMapperType::HardcodedClaim => self.evaluate_hardcoded_claim_mapper(mapper),
            ProtocolMapperType::FullName => self.evaluate_full_name_mapper(mapper, user),
            ProtocolMapperType::Audience => self.evaluate_audience_mapper(mapper),
            ProtocolMapperType::Script => {
                // Script mappers would require script evaluation engine
                // For now, return empty claims
                Ok(HashMap::new())
            }
        }
    }

    /// Evaluate user property mapper
    fn evaluate_user_property_mapper(
        &self,
        mapper: &ProtocolMapper,
        user: &User,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut claims = HashMap::new();

        let value = match mapper.config.user_property.as_deref() {
            Some("username") => Some(serde_json::Value::String(user.username.clone())),
            Some("email") => Some(serde_json::Value::String(user.email.clone())),
            Some("firstName") => user
                .first_name
                .as_ref()
                .map(|s| serde_json::Value::String(s.clone())),
            Some("lastName") => user
                .last_name
                .as_ref()
                .map(|s| serde_json::Value::String(s.clone())),
            _ => None,
        };

        if let Some(value) = value {
            claims.insert(mapper.config.claim_name.clone(), value);
        }

        Ok(claims)
    }

    /// Evaluate user attribute mapper
    fn evaluate_user_attribute_mapper(
        &self,
        mapper: &ProtocolMapper,
        user: &User,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut claims = HashMap::new();

        if let Some(ref attr_name) = mapper.config.user_attribute {
            // Check if user has this attribute
            if let Some(ref attributes) = user.attributes {
                if let Some(attr_value) = attributes.get(attr_name) {
                    // Handle multivalued attributes
                    if mapper.config.multivalued.unwrap_or(false) {
                        claims.insert(mapper.config.claim_name.clone(), attr_value.clone());
                    } else {
                        // For single-valued, extract first value if array
                        let value = if let Some(arr) = attr_value.as_array() {
                            arr.first().cloned().unwrap_or(attr_value.clone())
                        } else {
                            attr_value.clone()
                        };
                        claims.insert(mapper.config.claim_name.clone(), value);
                    }
                }
            }
        }

        Ok(claims)
    }

    /// Evaluate user role mapper
    fn evaluate_user_role_mapper(
        &self,
        mapper: &ProtocolMapper,
        user: &User,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut claims = HashMap::new();

        // Collect all role names
        let roles: Vec<String> = user.roles.iter().map(|r| r.name.clone()).collect();

        if !roles.is_empty() {
            let role_value = if mapper.config.multivalued.unwrap_or(true) {
                serde_json::Value::Array(roles.into_iter().map(serde_json::Value::String).collect())
            } else {
                serde_json::Value::String(roles.join(","))
            };

            claims.insert(mapper.config.claim_name.clone(), role_value);
        }

        Ok(claims)
    }

    /// Evaluate realm role mapper
    fn evaluate_user_realm_role_mapper(
        &self,
        mapper: &ProtocolMapper,
        user: &User,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // For now, same as user role mapper
        // In full implementation, would filter by realm-level roles
        self.evaluate_user_role_mapper(mapper, user)
    }

    /// Evaluate client role mapper
    fn evaluate_user_client_role_mapper(
        &self,
        mapper: &ProtocolMapper,
        user: &User,
    ) -> Result<HashMap<String, serde_json::Value>> {
        // For now, same as user role mapper
        // In full implementation, would filter by client-specific roles
        self.evaluate_user_role_mapper(mapper, user)
    }

    /// Evaluate user group mapper
    fn evaluate_user_group_mapper(
        &self,
        mapper: &ProtocolMapper,
        user: &User,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut claims = HashMap::new();

        // Get group memberships
        // TODO: User model doesn't have groups field yet - would need to fetch from database
        // For now, return empty groups
        let groups: Vec<String> = Vec::new();

        if !groups.is_empty() {
            let full_path = mapper.config.full_group_path.unwrap_or(false);

            let group_value = if mapper.config.multivalued.unwrap_or(true) {
                let group_strings: Vec<String> = if full_path {
                    // Include full path (e.g., "/parent/child")
                    groups
                        .into_iter()
                        .map(|g| format!("/{}", g.replace(' ', "_")))
                        .collect()
                } else {
                    groups
                };

                serde_json::Value::Array(
                    group_strings
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect(),
                )
            } else {
                serde_json::Value::String(groups.join(","))
            };

            claims.insert(mapper.config.claim_name.clone(), group_value);
        }

        Ok(claims)
    }

    /// Evaluate hardcoded claim mapper
    fn evaluate_hardcoded_claim_mapper(
        &self,
        mapper: &ProtocolMapper,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut claims = HashMap::new();

        if let Some(ref claim_value) = mapper.config.claim_value {
            // Parse value based on claim_value_type
            let value = match mapper.config.claim_value_type.as_deref() {
                Some("boolean") | Some("Boolean") => {
                    serde_json::Value::Bool(claim_value.parse().unwrap_or(false))
                }
                Some("int") | Some("Integer") => serde_json::Value::Number(
                    claim_value
                        .parse::<i64>()
                        .ok()
                        .and_then(|n| serde_json::Number::from_f64(n as f64))
                        .unwrap_or(serde_json::Number::from(0)),
                ),
                Some("long") | Some("Long") => serde_json::Value::Number(
                    claim_value
                        .parse::<i64>()
                        .ok()
                        .and_then(|n| serde_json::Number::from_f64(n as f64))
                        .unwrap_or(serde_json::Number::from(0)),
                ),
                Some("JSON") => serde_json::from_str(claim_value)
                    .unwrap_or_else(|_| serde_json::Value::String(claim_value.clone())),
                _ => serde_json::Value::String(claim_value.clone()),
            };

            claims.insert(mapper.config.claim_name.clone(), value);
        }

        Ok(claims)
    }

    /// Evaluate full name mapper
    fn evaluate_full_name_mapper(
        &self,
        mapper: &ProtocolMapper,
        user: &User,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut claims = HashMap::new();

        let full_name = format!(
            "{} {}",
            user.first_name.as_deref().unwrap_or(""),
            user.last_name.as_deref().unwrap_or("")
        )
        .trim()
        .to_string();

        if !full_name.is_empty() {
            claims.insert(
                mapper.config.claim_name.clone(),
                serde_json::Value::String(full_name),
            );
        }

        Ok(claims)
    }

    /// Evaluate audience mapper
    fn evaluate_audience_mapper(
        &self,
        mapper: &ProtocolMapper,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut claims = HashMap::new();

        // Check for custom audience
        if let Some(ref audience) = mapper.config.included_custom_audience {
            claims.insert(
                mapper.config.claim_name.clone(),
                serde_json::Value::String(audience.clone()),
            );
        } else if let Some(ref client_audience) = mapper.config.included_client_audience {
            // Use client audience
            claims.insert(
                mapper.config.claim_name.clone(),
                serde_json::Value::String(client_audience.clone()),
            );
        }

        Ok(claims)
    }

    // ==================== Validation ====================

    /// Validate mapper configuration based on type
    fn validate_mapper_config(
        &self,
        mapper_type: &ProtocolMapperType,
        config: &ProtocolMapperConfiguration,
    ) -> Result<()> {
        // Check claim_name is not empty
        if config.claim_name.is_empty() {
            return Err(AuthencError::validation("claim_name cannot be empty"));
        }

        // Type-specific validation
        match mapper_type {
            ProtocolMapperType::UserProperty => {
                if config.user_property.is_none() {
                    return Err(AuthencError::validation(
                        "user_property is required for UserProperty mapper",
                    ));
                }
            }
            ProtocolMapperType::UserAttribute => {
                if config.user_attribute.is_none() {
                    return Err(AuthencError::validation(
                        "user_attribute is required for UserAttribute mapper",
                    ));
                }
            }
            ProtocolMapperType::HardcodedClaim => {
                if config.claim_value.is_none() {
                    return Err(AuthencError::validation(
                        "claim_value is required for HardcodedClaim mapper",
                    ));
                }
            }
            ProtocolMapperType::Audience => {
                if config.included_custom_audience.is_none()
                    && config.included_client_audience.is_none()
                {
                    return Err(AuthencError::validation(
                        "Either included_custom_audience or included_client_audience is required for Audience mapper",
                    ));
                }
            }
            _ => {
                // Other mappers don't have strict requirements
            }
        }

        Ok(())
    }
}

/// Token type for mapper inclusion filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    AccessToken,
    IdToken,
    UserInfo,
}
