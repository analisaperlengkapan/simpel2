use crate::database::Database;
use crate::database::operations::{
    client_scope_assignments, client_scopes, scope_validation, user_consent_scopes,
};
use crate::error::{AuthencError, Result};
use crate::models::client_scope::*;
use std::sync::Arc;
use uuid::Uuid;

/// Client Scope Service
/// Provides business logic for managing OAuth2/OIDC client scopes,
/// including scope definitions, client assignments, and user consent.
/// # Features
/// - Reusable scope definitions (like Keycloak's Client Scopes)
/// - Default vs optional scope assignment
/// - User consent management
/// - Scope validation for OAuth2 flows
/// - Standard OIDC scopes pre-configured
/// # Standards Compliance
/// - OAuth 2.0 RFC 6749 (scope parameter)
/// - OpenID Connect Core 1.0 (standard scopes)
/// - OAuth 2.0 Incremental Authorization (consent)
pub struct ClientScopeService {
    db: Arc<Database>,
}

impl ClientScopeService {
    /// Create a new client scope service
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    // =========================================================================
    // SCOPE CRUD OPERATIONS
    // =========================================================================

    /// Create a new client scope
    pub async fn create_scope(
        &self,
        realm_id: Uuid,
        request: CreateClientScopeRequest,
    ) -> Result<ClientScope> {
        // Validate scope name
        if request.name.trim().is_empty() {
            return Err(AuthencError::validation("Scope name cannot be empty"));
        }

        // Check if scope with same name already exists
        if let Some(_existing) =
            client_scopes::get_scope_by_name(&self.db, realm_id, &request.name).await?
        {
            return Err(AuthencError::validation(format!(
                "Scope with name '{}' already exists in this realm",
                request.name
            )));
        }

        client_scopes::create_scope(&self.db, realm_id, request).await
    }

    /// Get a scope by ID
    pub async fn get_scope(&self, scope_id: Uuid) -> Result<Option<ClientScope>> {
        client_scopes::get_scope_by_id(&self.db, scope_id).await
    }

    /// Get a scope by name in a realm
    pub async fn get_scope_by_name(
        &self,
        realm_id: Uuid,
        name: &str,
    ) -> Result<Option<ClientScope>> {
        client_scopes::get_scope_by_name(&self.db, realm_id, name).await
    }

    /// List all scopes in a realm
    pub async fn list_scopes(
        &self,
        realm_id: Uuid,
        enabled_only: bool,
    ) -> Result<Vec<ClientScope>> {
        client_scopes::list_scopes(&self.db, realm_id, enabled_only).await
    }

    /// Update a scope
    pub async fn update_scope(
        &self,
        scope_id: Uuid,
        request: UpdateClientScopeRequest,
    ) -> Result<ClientScope> {
        client_scopes::update_scope(&self.db, scope_id, request).await
    }

    /// Delete a scope
    pub async fn delete_scope(&self, scope_id: Uuid) -> Result<bool> {
        client_scopes::delete_scope(&self.db, scope_id).await
    }

    // =========================================================================
    // CLIENT SCOPE ASSIGNMENT OPERATIONS
    // =========================================================================

    /// Assign scopes to a client
    ///
    /// This replaces all existing scope assignments for the client.
    /// Default scopes are automatically granted without consent.
    /// Optional scopes require user consent (if consent_required = true).
    pub async fn assign_client_scopes(
        &self,
        client_id: Uuid,
        request: AssignClientScopesRequest,
    ) -> Result<()> {
        // Assign default scopes
        client_scope_assignments::assign_default_scopes(
            &self.db,
            client_id,
            &request.default_scope_ids,
        )
        .await?;

        // Assign optional scopes
        client_scope_assignments::assign_optional_scopes(
            &self.db,
            client_id,
            &request.optional_scope_ids,
        )
        .await?;

        Ok(())
    }

    /// Get default scopes for a client
    pub async fn get_default_scopes(&self, client_id: Uuid) -> Result<Vec<ClientScope>> {
        client_scope_assignments::get_default_scopes(&self.db, client_id).await
    }

    /// Get optional scopes for a client
    pub async fn get_optional_scopes(&self, client_id: Uuid) -> Result<Vec<ClientScope>> {
        client_scope_assignments::get_optional_scopes(&self.db, client_id).await
    }

    /// Get all scopes for a client with assignment type
    pub async fn get_all_client_scopes(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<ClientScopeAssignment>> {
        client_scope_assignments::get_all_client_scopes(&self.db, client_id).await
    }

    // =========================================================================
    // SCOPE VALIDATION FOR OAUTH2 FLOWS
    // =========================================================================

    /// Validate requested scopes for OAuth2 authorization
    ///
    /// Checks if the requested scopes are:
    /// 1. Allowed for the client (default or optional)
    /// 2. Enabled in the system
    pub async fn validate_requested_scopes(
        &self,
        client_id: Uuid,
        realm_id: Uuid,
        requested_scopes: &str,
    ) -> Result<ScopeValidationResult> {
        let scope_names: Vec<String> = requested_scopes
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if scope_names.is_empty() {
            return Ok(ScopeValidationResult {
                valid: true,
                invalid_scopes: Vec::new(),
                valid_scopes: Vec::new(),
            });
        }

        scope_validation::validate_scopes(&self.db, client_id, realm_id, &scope_names).await
    }

    /// Get effective scopes for a token
    ///
    /// Returns the scopes that should be included in the access token:
    /// - Default scopes (if no specific scopes requested)
    /// - Requested scopes (if valid and consented)
    pub async fn get_effective_scopes(
        &self,
        client_id: Uuid,
        realm_id: Uuid,
        requested_scopes: Option<&str>,
        user_id: Option<Uuid>,
    ) -> Result<Vec<ClientScope>> {
        let requested = match requested_scopes {
            Some(scopes) if !scopes.trim().is_empty() => {
                // Validate requested scopes
                let validation = self
                    .validate_requested_scopes(client_id, realm_id, scopes)
                    .await?;

                if !validation.valid {
                    return Err(AuthencError::validation(format!(
                        "Invalid scopes requested: {}",
                        validation.invalid_scopes.join(", ")
                    )));
                }

                // Check consent if user is present
                if let Some(user_id) = user_id {
                    let scope_names: Vec<String> =
                        scopes.split_whitespace().map(|s| s.to_string()).collect();

                    let consent_check = user_consent_scopes::check_consent(
                        &self.db,
                        user_id,
                        client_id,
                        &scope_names,
                    )
                    .await?;

                    if consent_check.consent_needed {
                        return Err(AuthencError::validation(format!(
                            "User consent required for scopes: {}",
                            consent_check.missing_consent_scopes.join(", ")
                        )));
                    }
                }

                validation.valid_scopes
            }
            _ => {
                // No scopes requested, return default scopes
                self.get_default_scopes(client_id).await?
            }
        };

        // Filter to only include scopes with include_in_token_scope = true
        let token_scopes: Vec<ClientScope> = requested
            .into_iter()
            .filter(|s| s.include_in_token_scope)
            .collect();

        Ok(token_scopes)
    }

    // =========================================================================
    // USER CONSENT OPERATIONS
    // =========================================================================

    /// Check if user consent is required for requested scopes
    pub async fn check_consent_required(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        requested_scopes: &str,
    ) -> Result<ConsentCheckResult> {
        let scope_names: Vec<String> = requested_scopes
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if scope_names.is_empty() {
            return Ok(ConsentCheckResult {
                consent_needed: false,
                scopes_requiring_consent: Vec::new(),
                consented_scopes: Vec::new(),
                missing_consent_scopes: Vec::new(),
            });
        }

        user_consent_scopes::check_consent(&self.db, user_id, client_id, &scope_names).await
    }

    /// Grant user consent for scopes
    pub async fn grant_consent(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        realm_id: Uuid,
        scope_names: &[String],
        expires_in: Option<i64>,
    ) -> Result<Vec<UserConsentScope>> {
        // Get scope IDs from names
        let scopes = client_scopes::get_scopes_by_names(&self.db, realm_id, scope_names).await?;
        let scope_ids: Vec<Uuid> = scopes.iter().map(|s| s.id).collect();

        if scope_ids.is_empty() {
            return Err(AuthencError::validation("No valid scopes found"));
        }

        let request = GrantScopeConsentRequest {
            scope_ids,
            expires_in,
            consent_source: Some("explicit".to_string()),
        };

        user_consent_scopes::grant_consent(&self.db, user_id, client_id, request).await
    }

    /// Revoke user consent for specific scopes or all scopes for a client
    pub async fn revoke_consent(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        scope_names: Option<&[String]>,
    ) -> Result<u64> {
        let scope_ids = if let Some(names) = scope_names {
            // Get scope IDs for the specified names
            let scopes = client_scopes::get_scopes_by_names(&self.db, Uuid::nil(), names).await?;
            Some(scopes.iter().map(|s| s.id).collect::<Vec<Uuid>>())
        } else {
            None
        };

        user_consent_scopes::revoke_consent(&self.db, user_id, client_id, scope_ids.as_deref())
            .await
    }

    /// Get all consented scopes for a user and client
    pub async fn get_consented_scopes(
        &self,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<ClientScope>> {
        user_consent_scopes::get_consented_scopes(&self.db, user_id, client_id).await
    }

    // =========================================================================
    // HELPER METHODS
    // =========================================================================

    /// Get standard OIDC scopes (openid, profile, email, etc.)
    pub async fn get_standard_oidc_scopes(&self, realm_id: Uuid) -> Result<Vec<ClientScope>> {
        let scope_names: Vec<String> = standard_scopes::all()
            .iter()
            .map(|s| s.to_string())
            .collect();

        client_scopes::get_scopes_by_names(&self.db, realm_id, &scope_names).await
    }

    /// Initialize standard scopes for a new realm
    ///
    /// Creates the standard OIDC scopes (openid, profile, email, etc.)
    /// and SIMPelv2-specific scopes for asset management.
    pub async fn initialize_standard_scopes(&self, realm_id: Uuid) -> Result<()> {
        // Standard OIDC scopes
        let standard = vec![
            (
                "openid",
                Some("OpenID Connect"),
                Some("Access to OpenID Connect authentication"),
                false,
                false,
            ),
            (
                "profile",
                Some("User Profile"),
                Some("Access to user profile information (name, username, etc.)"),
                true,
                true,
            ),
            (
                "email",
                Some("Email Address"),
                Some("Access to user email address"),
                true,
                true,
            ),
            (
                "address",
                Some("Physical Address"),
                Some("Access to user physical address"),
                true,
                true,
            ),
            (
                "phone",
                Some("Phone Number"),
                Some("Access to user phone number"),
                true,
                true,
            ),
            (
                "offline_access",
                Some("Offline Access"),
                Some("Access to refresh tokens for offline access"),
                true,
                true,
            ),
            (
                "roles",
                Some("User Roles"),
                Some("Access to user role information"),
                false,
                false,
            ),
            (
                "groups",
                Some("User Groups"),
                Some("Access to user group membership"),
                false,
                false,
            ),
        ];

        for (name, display_name, description, consent_required, display_on_consent) in standard {
            let request = CreateClientScopeRequest {
                name: name.to_string(),
                display_name: display_name.map(|s| s.to_string()),
                description: description.map(|s| s.to_string()),
                protocol: Some("openid-connect".to_string()),
                consent_required: Some(consent_required),
                display_on_consent_screen: Some(display_on_consent),
                consent_screen_text: None,
                include_in_token_scope: Some(true),
                gui_order: None,
                icon_uri: None,
                attributes: None,
            };

            // Ignore if already exists
            if client_scopes::get_scope_by_name(&self.db, realm_id, name)
                .await?
                .is_none()
            {
                let _ = client_scopes::create_scope(&self.db, realm_id, request).await;
            }
        }

        // SIMPelv2-specific scopes
        let simpel = vec![
            (
                "read:aset",
                Some("Read Assets"),
                Some("View asset information"),
            ),
            (
                "write:aset",
                Some("Manage Assets"),
                Some("Create, update, and delete assets"),
            ),
            (
                "read:laporan",
                Some("Read Reports"),
                Some("View reports and analytics"),
            ),
            (
                "write:laporan",
                Some("Manage Reports"),
                Some("Create and manage reports"),
            ),
            (
                "admin:satker",
                Some("Satker Administration"),
                Some("Administrative access to satker (work unit)"),
            ),
            (
                "admin:wilayah",
                Some("Regional Administration"),
                Some("Administrative access to regional level"),
            ),
            (
                "admin:pusat",
                Some("Central Administration"),
                Some("Administrative access to central level"),
            ),
        ];

        for (name, display_name, description) in simpel {
            let request = CreateClientScopeRequest {
                name: name.to_string(),
                display_name: display_name.map(|s| s.to_string()),
                description: description.map(|s| s.to_string()),
                protocol: Some("openid-connect".to_string()),
                consent_required: Some(true),
                display_on_consent_screen: Some(true),
                consent_screen_text: None,
                include_in_token_scope: Some(true),
                gui_order: None,
                icon_uri: None,
                attributes: None,
            };

            // Ignore if already exists
            if client_scopes::get_scope_by_name(&self.db, realm_id, name)
                .await?
                .is_none()
            {
                let _ = client_scopes::create_scope(&self.db, realm_id, request).await;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    // Test helpers would go here
    // Note: Actual tests require database setup and are typically in integration tests
}
