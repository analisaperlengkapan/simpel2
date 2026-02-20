//! IAM API state with admin service dependencies

use authenc_core::services::{
    UserManagementServiceImpl, RealmManagementServiceImpl, OAuth2ServiceImpl,
};
use authenc_crypto::jwt::JwtService;
use std::sync::Arc;

/// IAM API state containing all admin service dependencies
#[derive(Clone)]
pub struct IamApiState {
    /// User management service for CRUD operations
    pub user_service: Arc<UserManagementServiceImpl>,

    /// Realm management service
    pub realm_service: Arc<RealmManagementServiceImpl>,

    /// OAuth2 client management service
    pub client_service: Arc<OAuth2ServiceImpl>,

    /// JWT service for token validation
    pub jwt_service: Arc<JwtService>,

    // TODO: Add role service when implemented
    // pub role_service: Arc<RoleManagementService>,

    // TODO: Add federation service when implemented
    // pub federation_service: Arc<FederationService>,
}

impl IamApiState {
    /// Create a new IAM API state with all dependencies
    pub fn new(
        user_service: Arc<UserManagementServiceImpl>,
        realm_service: Arc<RealmManagementServiceImpl>,
        client_service: Arc<OAuth2ServiceImpl>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            user_service,
            realm_service,
            client_service,
            jwt_service,
        }
    }
}
