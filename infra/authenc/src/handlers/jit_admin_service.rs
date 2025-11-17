//! Shared JIT (Just-In-Time) Admin Service Implementation
//!
//! This module provides a shared implementation of AdminService for JIT user provisioning
//! used by SAML and federated authentication handlers.

use crate::database::Database;
use crate::services::admin::AdminService;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

/// JIT Admin Service for federated authentication and SAML
///
/// This service provides a minimal implementation of AdminService focused on
/// user creation for Just-In-Time provisioning scenarios. It delegates to
/// database operations for actual user management.
pub struct JitAdminService {
    db: Arc<Database>,
}

impl JitAdminService {
    /// Create a new JIT Admin Service
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AdminService for JitAdminService {
    async fn get_system_stats(&self) -> Result<crate::services::admin::SystemStats, String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn get_users(
        &self,
        _realm_id: &Uuid,
        _page: u32,
        _limit: u32,
    ) -> Result<crate::services::admin::UserListResponse, String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn create_user(
        &self,
        request: crate::services::admin::CreateUserRequest,
    ) -> Result<crate::services::admin::UserResponse, String> {
        use crate::database::operations::users;
        use crate::models::user::CreateUserRequest as DbCreateUserRequest;

        // Extract satker_code from request or attributes
        let satker_code = request
            .satker_code
            .clone()
            .or_else(|| {
                request.attributes.as_ref().and_then(|attrs| {
                    attrs
                        .get("satker_code")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
            })
            .unwrap_or_else(|| "default".to_string());

        // Extract nip from request or attributes
        let nip = request.nip.clone().or_else(|| {
            request.attributes.as_ref().and_then(|attrs| {
                attrs
                    .get("nip")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
        });

        // Extract nama from request or attributes
        let nama = request.nama.clone().or_else(|| {
            request.attributes.as_ref().and_then(|attrs| {
                attrs
                    .get("nama")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
        });

        // Extract jabatan from request or attributes
        let jabatan = request.jabatan.clone().or_else(|| {
            request.attributes.as_ref().and_then(|attrs| {
                attrs
                    .get("jabatan")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
        });

        let db_request = DbCreateUserRequest {
            username: request.username,
            email: request.email,
            satker_code,
            password: request.password,
            first_name: request.first_name,
            last_name: request.last_name,
            nip,
            nama,
            jabatan,
            phone_number: request.phone_number,
            attributes: request.attributes,
            realm_id: Some(request.realm_id),
            organization_id: None,
            roles: Some(
                request
                    .roles
                    .iter()
                    .filter_map(|r| Uuid::parse_str(r).ok())
                    .collect(),
            ),
            secreton_access_policy: None,
        };

        match users::create_user(&self.db, &db_request).await {
            Ok(user) => {
                // Fetch roles from database if user was created
                let roles = if !request.roles.is_empty() {
                    request.roles
                } else {
                    // Query roles from database
                    self.fetch_user_roles(&user.id).await.unwrap_or_default()
                };

                // Fetch groups from database
                let groups = self.fetch_user_groups(&user.id).await.unwrap_or_default();

                Ok(crate::services::admin::UserResponse {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    email_verified: user.email_verified,
                    first_name: user.first_name,
                    last_name: user.last_name,
                    enabled: user.enabled,
                    realm_id: user.realm_id.unwrap_or_default(),
                    roles,
                    groups,
                    created_at: user.created_at,
                    last_login: user.last_login_at,
                    login_attempts: user.failed_login_attempts as u32,
                    locked_until: user.account_locked_until,
                })
            }
            Err(e) => Err(format!("Failed to create user: {}", e)),
        }
    }

    async fn update_user(
        &self,
        _user_id: &Uuid,
        _request: crate::services::admin::UpdateUserRequest,
    ) -> Result<crate::services::admin::UserResponse, String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn delete_user(&self, _user_id: &Uuid) -> Result<(), String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn get_roles(
        &self,
        _realm_id: &Uuid,
    ) -> Result<Vec<crate::services::admin::RoleResponse>, String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn create_role(
        &self,
        _request: crate::services::admin::CreateRoleRequest,
    ) -> Result<crate::services::admin::RoleResponse, String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn get_sessions(
        &self,
        _user_id: Option<Uuid>,
        _page: u32,
        _limit: u32,
    ) -> Result<crate::services::admin::SessionListResponse, String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn terminate_session(&self, _session_id: &str) -> Result<(), String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn get_audit_logs(
        &self,
        _filter: crate::services::admin::AuditLogFilter,
    ) -> Result<crate::services::admin::AuditLogResponse, String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn get_policies(
        &self,
        _realm_id: &Uuid,
    ) -> Result<Vec<crate::services::admin::PolicyResponse>, String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn create_policy(
        &self,
        _request: crate::services::admin::CreatePolicyRequest,
    ) -> Result<crate::services::admin::PolicyResponse, String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn get_zero_trust_dashboard(
        &self,
        _realm_id: &Uuid,
    ) -> Result<crate::services::admin::ZeroTrustDashboard, String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn get_identity_providers(
        &self,
        _realm_id: &Uuid,
    ) -> Result<Vec<crate::services::admin::IdentityProviderResponse>, String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn create_identity_provider(
        &self,
        _request: crate::services::admin::CreateIdentityProviderRequest,
    ) -> Result<crate::services::admin::IdentityProviderResponse, String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn update_identity_provider(
        &self,
        _provider_id: &Uuid,
        _request: crate::services::admin::UpdateIdentityProviderRequest,
    ) -> Result<crate::services::admin::IdentityProviderResponse, String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn delete_identity_provider(&self, _provider_id: &Uuid) -> Result<(), String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn get_identity_provider(
        &self,
        _provider_id: &Uuid,
    ) -> Result<crate::services::admin::IdentityProviderResponse, String> {
        Err("Not implemented for JIT provisioning".to_string())
    }

    async fn test_identity_provider(
        &self,
        _provider_id: &Uuid,
    ) -> Result<crate::services::admin::TestIdentityProviderResponse, String> {
        Err("Not implemented for JIT provisioning".to_string())
    }
}

impl JitAdminService {
    /// Fetch user roles from database
    async fn fetch_user_roles(&self, user_id: &Uuid) -> Result<Vec<String>, String> {
        let query = "
            SELECT r.name
            FROM roles r
            INNER JOIN user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = $1 AND r.deleted_at IS NULL
        ";

        match self.db.query_raw(query, &[user_id]).await {
            Ok(rows) => {
                let roles: Vec<String> = rows
                    .iter()
                    .filter_map(|row| row.try_get::<_, String>(0).ok())
                    .collect();
                Ok(roles)
            }
            Err(e) => {
                tracing::warn!("Failed to fetch user roles: {}", e);
                Ok(Vec::new())
            }
        }
    }

    /// Fetch user groups from database
    async fn fetch_user_groups(&self, user_id: &Uuid) -> Result<Vec<String>, String> {
        let query = "
            SELECT g.name
            FROM groups g
            INNER JOIN user_groups ug ON g.id = ug.group_id
            WHERE ug.user_id = $1 AND g.deleted_at IS NULL
        ";

        match self.db.query_raw(query, &[user_id]).await {
            Ok(rows) => {
                let groups: Vec<String> = rows
                    .iter()
                    .filter_map(|row| row.try_get::<_, String>(0).ok())
                    .collect();
                Ok(groups)
            }
            Err(e) => {
                tracing::warn!("Failed to fetch user groups: {}", e);
                Ok(Vec::new())
            }
        }
    }
}
