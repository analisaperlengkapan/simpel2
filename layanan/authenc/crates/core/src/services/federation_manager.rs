//! Federation Manager - Central orchestration for user federation
//!
//! This module provides the main FederationManager that coordinates:
//! - LDAP/Active Directory federation
//! - Social login providers (OAuth2/OIDC)
//! - Just-In-Time (JIT) user provisioning
//! - User linking between local and external identities
//! - Federated authentication flows

use crate::database::Database;
use crate::error::{AuthencError, Result};
use authenc_types::User;

// TODO: SPI module doesn't exist yet - needs implementation
// use crate::spi::ldap_federation::{
//     DefaultLdapFederationProvider, LdapFederationConfig, LdapFederationProvider,
// };
// use crate::spi::social::{DefaultSocialProvider, SocialProvider, SocialProviderConfig};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use uuid::Uuid;

/// Federation provider type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FederationProviderType {
    /// LDAP directory server
    Ldap,
    /// Active Directory (Microsoft AD)
    ActiveDirectory,
    /// Social login provider (OAuth2/OIDC)
    Social,
}

/// Federated identity link information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedIdentityLink {
    pub id: Uuid,
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub identity_provider_alias: String,
    pub federated_user_id: String,
    pub federated_username: Option<String>,
    pub token: Option<String>,
    pub token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub refresh_token: Option<String>,
    pub federated_attributes: Option<serde_json::Value>,
    pub linked_at: chrono::DateTime<chrono::Utc>,
    pub last_authenticated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub authentication_count: i32,
}

/// Identity provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProviderConfig {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub alias: String,
    pub display_name: String,
    pub enabled: bool,
    pub provider_type: FederationProviderType,
    pub trust_email: bool,
    pub store_token: bool,
    pub link_only: bool,
    pub config: serde_json::Value,
}

/// Federation authentication result
#[derive(Debug, Clone)]
pub struct FederationAuthResult {
    /// Whether authentication succeeded
    pub success: bool,
    /// Local user (either existing or newly created)
    pub user: Option<User>,
    /// Federated identity link
    pub identity_link: Option<FederatedIdentityLink>,
    /// Access token from provider (if store_token is true)
    pub provider_token: Option<String>,
    /// Error message if authentication failed
    pub error: Option<String>,
}

// TODO: SPI trait placeholders - needs implementation
// These traits will be defined in the SPI module
pub trait LdapFederationProvider: Send + Sync {
    // Placeholder for LDAP provider trait
}

pub trait SocialProvider: Send + Sync {
    // Placeholder for social provider trait
}

/// Federation Manager - main service for user federation
pub struct FederationManager {
    db: Arc<Database>,
    // TODO: SPI-dependent fields commented out - needs implementation
    // ldap_providers: Arc<RwLock<HashMap<String, Arc<dyn LdapFederationProvider>>>>,
    // social_providers: Arc<RwLock<HashMap<String, Arc<dyn SocialProvider>>>>,
    config_cache: Arc<RwLock<HashMap<Uuid, IdentityProviderConfig>>>,
}

impl FederationManager {
    /// Create a new FederationManager
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            // TODO: SPI-dependent fields commented out - needs implementation
            // ldap_providers: Arc::new(RwLock::new(HashMap::new())),
            // social_providers: Arc::new(RwLock::new(HashMap::new())),
            config_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize federation manager by loading all configured providers
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing Federation Manager...");

        // Load all identity provider configurations from database
        let configs = self.load_identity_provider_configs().await?;

        info!("Found {} identity provider configurations", configs.len());

        for config in configs {
            if config.enabled {
                match self.register_provider_from_config(&config).await {
                    Ok(_) => info!(
                        "Successfully registered provider: {} ({})",
                        config.alias, config.display_name
                    ),
                    Err(e) => error!("Failed to register provider {}: {}", config.alias, e),
                }
            }
        }

        info!("Federation Manager initialization complete");
        Ok(())
    }

    /// Load identity provider configurations from database
    async fn load_identity_provider_configs(&self) -> Result<Vec<IdentityProviderConfig>> {
        let client = self.db.get_connection().await?;

        let rows = client
            .query(
                "SELECT id, realm_id, alias, display_name, enabled, provider_type,
                        trust_email, store_token, link_only, config
                 FROM identity_broker_configs
                 WHERE enabled = true
                 ORDER BY alias",
                &[],
            )
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to load identity provider configs: {}", e))
            })?;

        let mut configs = Vec::new();

        for row in rows {
            let provider_type_str: String = row.get(5);
            let provider_type = match provider_type_str.as_str() {
                "ldap" => FederationProviderType::Ldap,
                "active_directory" => FederationProviderType::ActiveDirectory,
                "social" | "oidc" | "oauth2" => FederationProviderType::Social,
                _ => continue, // Skip unknown types
            };

            let config = IdentityProviderConfig {
                id: row.get(0),
                realm_id: row.get(1),
                alias: row.get(2),
                display_name: row.get(3),
                enabled: row.get(4),
                provider_type,
                trust_email: row.get(6),
                store_token: row.get(7),
                link_only: row.get(8),
                config: row.get(9),
            };

            configs.push(config);
        }

        Ok(configs)
    }

    /// Register a provider from configuration
    async fn register_provider_from_config(&self, config: &IdentityProviderConfig) -> Result<()> {
        // Cache the configuration
        {
            let mut cache = self.config_cache.write().await;
            cache.insert(config.id, config.clone());
        }

        // TODO: SPI-dependent code commented out - needs implementation
        // match config.provider_type {
        //     FederationProviderType::Ldap | FederationProviderType::ActiveDirectory => {
        //         self.register_ldap_provider(config).await?;
        //     }
        //     FederationProviderType::Social => {
        //         self.register_social_provider(config).await?;
        //     }
        // }

        Ok(())
    }

    // TODO: SPI-dependent methods commented out - needs implementation
    /*
    /// Register LDAP/AD provider
    async fn register_ldap_provider(&self, config: &IdentityProviderConfig) -> Result<()> {
        // Parse LDAP configuration from JSON
        let ldap_config: LdapFederationConfig = serde_json::from_value(config.config.clone())
            .map_err(|e| AuthencError::validation(format!("Invalid LDAP config: {}", e)))?;

        // Create LDAP provider
        let provider = Arc::new(DefaultLdapFederationProvider::new(ldap_config));

        // Register provider
        let mut providers = self.ldap_providers.write().await;
        providers.insert(config.alias.clone(), provider);

        Ok(())
    }

    /// Register social login provider
    async fn register_social_provider(&self, config: &IdentityProviderConfig) -> Result<()> {
        // Parse social provider configuration from JSON
        let social_config: SocialProviderConfig = serde_json::from_value(config.config.clone())
            .map_err(|e| AuthencError::validation(format!("Invalid social config: {}", e)))?;

        // Create social provider
        let provider = Arc::new(DefaultSocialProvider::new(social_config));

        // Register provider
        let mut providers = self.social_providers.write().await;
        providers.insert(config.alias.clone(), provider);

        Ok(())
    }
    */

    /// Authenticate user via LDAP/AD federation
    pub async fn authenticate_ldap(
        &self,
        _provider_alias: &str,
        _username: &str,
        _password: &str,
        _realm_id: Uuid,
    ) -> Result<FederationAuthResult> {
        // TODO: SPI-dependent code - needs implementation
        Err(AuthencError::not_implemented(
            "LDAP authentication requires SPI implementation",
        ))
    }

    /// Authenticate user via social login
    pub async fn authenticate_social(
        &self,
        provider_alias: &str,
        auth_code: &str,
        redirect_uri: &str,
        realm_id: Uuid,
    ) -> Result<FederationAuthResult> {
        // 1. Get provider configuration
        let config = self.get_provider_config_by_alias(provider_alias, realm_id).await?;

        // 2. Parse social provider config from stored JSON
        let social_config: crate::spi::social::SocialProviderConfig =
            serde_json::from_value(config.config.clone()).map_err(|e| {
                AuthencError::config(format!("Invalid social provider config: {}", e))
            })?;

        // 3. Create a social provider instance
        let provider = crate::spi::social::DefaultSocialProvider::new(social_config);

        // 4. Exchange authorization code for token
        let token = crate::spi::social::SocialProvider::exchange_code(
            &provider,
            auth_code,
            redirect_uri,
        )
        .await?;

        // 5. Fetch user profile from provider
        let profile = crate::spi::social::SocialProvider::get_user_profile(
            &provider,
            &token,
        )
        .await?;

        // 6. Find or create identity link
        let existing_link = self
            .find_identity_link(
                provider_alias,
                &profile.provider_user_id,
                realm_id,
            )
            .await?;

        if let Some(link) = existing_link {
            // Existing user - update auth stats and token
            self.update_authentication_stats(&link.id).await?;

            if config.store_token {
                let expires_at = token.expires_in.map(|e| {
                    chrono::Utc::now() + chrono::Duration::seconds(e as i64)
                });
                self.update_identity_link_token(
                    &link.id,
                    &token.access_token,
                    expires_at,
                    token.refresh_token.as_deref(),
                )
                .await?;
            }

            Ok(FederationAuthResult {
                success: true,
                user: None, // Caller should load user by link.user_id
                identity_link: Some(link),
                provider_token: if config.store_token {
                    Some(token.access_token)
                } else {
                    None
                },
                error: None,
            })
        } else {
            // New user - provision via social profile
            let user = self
                .provision_user_from_social(
                    &serde_json::to_value(&profile).unwrap_or_default(),
                    realm_id,
                    &config,
                )
                .await?;

            // Create identity link
            let expires_at = token.expires_in.map(|e| {
                chrono::Utc::now() + chrono::Duration::seconds(e as i64)
            });
            let link = self
                .create_identity_link(
                    user.id,
                    realm_id,
                    provider_alias,
                    &profile.provider_user_id,
                    profile.username.as_deref(),
                    if config.store_token {
                        Some(token.access_token.as_str())
                    } else {
                        None
                    },
                    expires_at,
                    token.refresh_token.as_deref(),
                    Some(Some(serde_json::to_value(&profile.attributes).unwrap_or_default())),
                )
                .await?;

            Ok(FederationAuthResult {
                success: true,
                user: Some(user),
                identity_link: Some(link),
                provider_token: if config.store_token {
                    Some(token.access_token)
                } else {
                    None
                },
                error: None,
            })
        }
    }

    /// Find identity link by provider alias and federated user ID
    async fn find_identity_link(
        &self,
        provider_alias: &str,
        federated_user_id: &str,
        realm_id: Uuid,
    ) -> Result<Option<FederatedIdentityLink>> {
        let client = self.db.get_connection().await?;

        let row = client
            .query_opt(
                "SELECT id, user_id, realm_id, identity_provider_alias, federated_user_id,
                        federated_username, token, token_expires_at, refresh_token,
                        federated_attributes, linked_at, last_authenticated_at, authentication_count
                 FROM federated_identity_links
                 WHERE identity_provider_alias = $1 AND federated_user_id = $2 AND realm_id = $3",
                &[&provider_alias, &federated_user_id, &realm_id],
            )
            .await
            .map_err(|e| AuthencError::database(format!("Failed to find identity link: {}", e)))?;

        match row {
            Some(row) => Ok(Some(FederatedIdentityLink {
                id: row.get(0),
                user_id: row.get(1),
                realm_id: row.get(2),
                identity_provider_alias: row.get(3),
                federated_user_id: row.get(4),
                federated_username: row.get(5),
                token: row.get(6),
                token_expires_at: row.get(7),
                refresh_token: row.get(8),
                federated_attributes: row.get(9),
                linked_at: row.get(10),
                last_authenticated_at: row.get(11),
                authentication_count: row.get(12),
            })),
            None => Ok(None),
        }
    }

    /// Get identity link by ID
    async fn get_identity_link(&self, link_id: &Uuid) -> Result<Option<FederatedIdentityLink>> {
        let client = self.db.get_connection().await?;

        let row = client
            .query_opt(
                "SELECT id, user_id, realm_id, identity_provider_alias, federated_user_id,
                        federated_username, token, token_expires_at, refresh_token,
                        federated_attributes, linked_at, last_authenticated_at, authentication_count
                 FROM federated_identity_links
                 WHERE id = $1",
                &[link_id],
            )
            .await
            .map_err(|e| AuthencError::database(format!("Failed to get identity link: {}", e)))?;

        match row {
            Some(row) => Ok(Some(FederatedIdentityLink {
                id: row.get(0),
                user_id: row.get(1),
                realm_id: row.get(2),
                identity_provider_alias: row.get(3),
                federated_user_id: row.get(4),
                federated_username: row.get(5),
                token: row.get(6),
                token_expires_at: row.get(7),
                refresh_token: row.get(8),
                federated_attributes: row.get(9),
                linked_at: row.get(10),
                last_authenticated_at: row.get(11),
                authentication_count: row.get(12),
            })),
            None => Ok(None),
        }
    }

    /// Create identity link
    #[allow(clippy::too_many_arguments)]
    async fn create_identity_link(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
        provider_alias: &str,
        federated_user_id: &str,
        federated_username: Option<&str>,
        token: Option<&str>,
        token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
        refresh_token: Option<&str>,
        federated_attributes: Option<Option<serde_json::Value>>,
    ) -> Result<FederatedIdentityLink> {
        let client = self.db.get_connection().await?;
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let federated_attrs = federated_attributes.flatten();

        client
            .execute(
                "INSERT INTO federated_identity_links
                 (id, user_id, realm_id, identity_provider_alias, federated_user_id,
                  federated_username, token, token_expires_at, refresh_token,
                  federated_attributes, linked_at, last_authenticated_at, authentication_count)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
                &[
                    &id,
                    &user_id,
                    &realm_id,
                    &provider_alias,
                    &federated_user_id,
                    &federated_username,
                    &token,
                    &token_expires_at,
                    &refresh_token,
                    &federated_attrs,
                    &now,
                    &Some(now),
                    &1i32,
                ],
            )
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to create identity link: {}", e))
            })?;

        Ok(FederatedIdentityLink {
            id,
            user_id,
            realm_id,
            identity_provider_alias: provider_alias.to_string(),
            federated_user_id: federated_user_id.to_string(),
            federated_username: federated_username.map(String::from),
            token: token.map(String::from),
            token_expires_at,
            refresh_token: refresh_token.map(String::from),
            federated_attributes: federated_attrs,
            linked_at: now,
            last_authenticated_at: Some(now),
            authentication_count: 1,
        })
    }

    /// Update authentication statistics for identity link
    async fn update_authentication_stats(&self, link_id: &Uuid) -> Result<()> {
        let client = self.db.get_connection().await?;

        client
            .execute(
                "UPDATE federated_identity_links
                 SET last_authenticated_at = NOW(),
                     authentication_count = authentication_count + 1,
                     updated_at = NOW()
                 WHERE id = $1",
                &[link_id],
            )
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to update authentication stats: {}", e))
            })?;

        Ok(())
    }

    /// Update identity link token
    async fn update_identity_link_token(
        &self,
        link_id: &Uuid,
        token: &str,
        token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
        refresh_token: Option<&str>,
    ) -> Result<()> {
        let client = self.db.get_connection().await?;

        client
            .execute(
                "UPDATE federated_identity_links
                 SET token = $2,
                     token_expires_at = $3,
                     refresh_token = $4,
                     updated_at = NOW()
                 WHERE id = $1",
                &[&link_id, &token, &token_expires_at, &refresh_token],
            )
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to update identity link token: {}", e))
            })?;

        Ok(())
    }

    /// Provision user from LDAP data
    async fn provision_user_from_ldap(&self, ldap_user: &User, realm_id: Uuid) -> Result<User> {
        let client = self.db.get_connection().await?;

        // Create new user
        let mut new_user = User::new(
            ldap_user.username.clone(),
            ldap_user.email.clone(),
            ldap_user.satker_code.clone(),
            None, // No password for federated users
            Some(realm_id),
        );

        new_user.first_name = ldap_user.first_name.clone();
        new_user.last_name = ldap_user.last_name.clone();
        new_user.email_verified = true; // Trust LDAP emails
        new_user.federated = true;
        new_user.attributes = ldap_user.attributes.clone();

        // Insert into database
        client
            .execute(
                "INSERT INTO users (id, username, email, satker_code, first_name, last_name,
                                   email_verified, federated, attributes, realm_id, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())",
                &[
                    &new_user.id,
                    &new_user.username,
                    &new_user.email,
                    &new_user.satker_code,
                    &new_user.first_name,
                    &new_user.last_name,
                    &new_user.email_verified,
                    &new_user.federated,
                    &new_user.attributes,
                    &realm_id,
                ],
            )
            .await
            .map_err(|e| AuthencError::database(format!("Failed to provision user: {}", e)))?;

        Ok(new_user)
    }

    /// Provision user from social login profile
    async fn provision_user_from_social(
        &self,
        profile: &serde_json::Value,
        realm_id: Uuid,
        config: &IdentityProviderConfig,
    ) -> Result<User> {
        let client = self.db.get_connection().await?;

        // Extract profile fields
        let email = profile["email"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}@social.federated", Uuid::new_v4()));
        let username = profile["username"]
            .as_str()
            .or_else(|| profile["display_name"].as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| email.split('@').next().unwrap_or("user").to_string());
        let first_name = profile["first_name"].as_str().map(|s| s.to_string());
        let last_name = profile["last_name"].as_str().map(|s| s.to_string());
        let picture_url = profile["picture_url"].as_str().map(|s| s.to_string());

        // Create new user
        let mut new_user = User::new(
            username,
            email,
            String::new(), // No satker_code for social users
            None,           // No password for federated users
            Some(realm_id),
        );

        new_user.first_name = first_name;
        new_user.last_name = last_name;
        new_user.email_verified = config.trust_email;
        new_user.federated = true;

        // Store profile picture in attributes if present
        if let Some(pic) = picture_url {
            let mut attrs = new_user.attributes.clone().unwrap_or_default();
            attrs["picture_url"] = serde_json::json!(pic);
            new_user.attributes = Some(attrs);
        }

        // Insert into database
        client
            .execute(
                "INSERT INTO users (id, username, email, satker_code, first_name, last_name,
                                   email_verified, federated, attributes, realm_id, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())",
                &[
                    &new_user.id,
                    &new_user.username,
                    &new_user.email,
                    &new_user.satker_code,
                    &new_user.first_name,
                    &new_user.last_name,
                    &new_user.email_verified,
                    &new_user.federated,
                    &new_user.attributes,
                    &realm_id,
                ],
            )
            .await
            .map_err(|e| AuthencError::database(format!("Failed to provision social user: {}", e)))?;

        Ok(new_user)
    }

    /// Load user by ID
    async fn load_user_by_id(&self, _user_id: Uuid) -> Result<User> {
        // TODO: Implement proper User loading from row mapping
        Err(AuthencError::not_implemented(
            "User loading from database rows needs proper row-to-User mapping",
        ))
    }

    /// Get provider configuration by alias
    async fn get_provider_config_by_alias(
        &self,
        alias: &str,
        realm_id: Uuid,
    ) -> Result<IdentityProviderConfig> {
        let client = self.db.get_connection().await?;

        let row = client
            .query_one(
                "SELECT id, realm_id, alias, display_name, enabled, provider_type,
                        trust_email, store_token, link_only, config
                 FROM identity_broker_configs
                 WHERE alias = $1 AND realm_id = $2",
                &[&alias, &realm_id],
            )
            .await
            .map_err(|e| AuthencError::database(format!("Failed to get provider config: {}", e)))?;

        let provider_type_str: String = row.get(5);
        let provider_type = match provider_type_str.as_str() {
            "ldap" => FederationProviderType::Ldap,
            "active_directory" => FederationProviderType::ActiveDirectory,
            "social" | "oidc" | "oauth2" => FederationProviderType::Social,
            _ => FederationProviderType::Social,
        };

        Ok(IdentityProviderConfig {
            id: row.get(0),
            realm_id: row.get(1),
            alias: row.get(2),
            display_name: row.get(3),
            enabled: row.get(4),
            provider_type,
            trust_email: row.get(6),
            store_token: row.get(7),
            link_only: row.get(8),
            config: row.get(9),
        })
    }

    /// List all configured identity providers for a realm
    pub async fn list_identity_providers(
        &self,
        realm_id: Uuid,
    ) -> Result<Vec<IdentityProviderConfig>> {
        let client = self.db.get_connection().await?;

        let rows = client
            .query(
                "SELECT id, realm_id, alias, display_name, enabled, provider_type,
                        trust_email, store_token, link_only, config
                 FROM identity_broker_configs
                 WHERE realm_id = $1
                 ORDER BY alias",
                &[&realm_id],
            )
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to list identity providers: {}", e))
            })?;

        let mut configs = Vec::new();

        for row in rows {
            let provider_type_str: String = row.get(5);
            let provider_type = match provider_type_str.as_str() {
                "ldap" => FederationProviderType::Ldap,
                "active_directory" => FederationProviderType::ActiveDirectory,
                "social" | "oidc" | "oauth2" => FederationProviderType::Social,
                _ => continue,
            };

            configs.push(IdentityProviderConfig {
                id: row.get(0),
                realm_id: row.get(1),
                alias: row.get(2),
                display_name: row.get(3),
                enabled: row.get(4),
                provider_type,
                trust_email: row.get(6),
                store_token: row.get(7),
                link_only: row.get(8),
                config: row.get(9),
            });
        }

        Ok(configs)
    }

    // TODO: SPI-dependent methods commented out - needs implementation
    /*
    /// Get social provider by alias
    pub async fn get_social_provider(&self, alias: &str) -> Option<Arc<dyn SocialProvider>> {
        let providers = self.social_providers.read().await;
        providers.get(alias).cloned()
    }

    /// Get LDAP provider by alias
    pub async fn get_ldap_provider(&self, alias: &str) -> Option<Arc<dyn LdapFederationProvider>> {
        let providers = self.ldap_providers.read().await;
        providers.get(alias).cloned()
    }
    */

    /// Get federated identity links for a user
    pub async fn get_user_identity_links(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<FederatedIdentityLink>> {
        let client = self.db.get_connection().await?;

        let rows = client
            .query(
                "SELECT id, user_id, realm_id, identity_provider_alias, federated_user_id,
                        federated_username, token, token_expires_at, refresh_token,
                        federated_attributes, linked_at, last_authenticated_at, authentication_count
                 FROM federated_identity_links
                 WHERE user_id = $1
                 ORDER BY linked_at DESC",
                &[&user_id],
            )
            .await
            .map_err(|e| {
                AuthencError::database(format!("Failed to get user identity links: {}", e))
            })?;

        let mut links = Vec::new();

        for row in rows {
            links.push(FederatedIdentityLink {
                id: row.get(0),
                user_id: row.get(1),
                realm_id: row.get(2),
                identity_provider_alias: row.get(3),
                federated_user_id: row.get(4),
                federated_username: row.get(5),
                token: row.get(6),
                token_expires_at: row.get(7),
                refresh_token: row.get(8),
                federated_attributes: row.get(9),
                linked_at: row.get(10),
                last_authenticated_at: row.get(11),
                authentication_count: row.get(12),
            });
        }

        Ok(links)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_federation_provider_type() {
        let ldap_type = FederationProviderType::Ldap;
        let ad_type = FederationProviderType::ActiveDirectory;
        let social_type = FederationProviderType::Social;

        assert_eq!(ldap_type, FederationProviderType::Ldap);
        assert_eq!(ad_type, FederationProviderType::ActiveDirectory);
        assert_eq!(social_type, FederationProviderType::Social);
    }
}
