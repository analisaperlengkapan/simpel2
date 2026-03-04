//! IAM API state with admin service dependencies

use authenc_core::services::group_store::GroupStore;
use authenc_core::services::{
    AuditService, OAuth2ServiceImpl, RealmManagementServiceImpl, RoleManagementServiceImpl,
    UserManagementServiceImpl,
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

    /// Group management service
    pub group_service: Option<Arc<GroupStore>>,

    /// Role management service
    pub role_service: Option<Arc<RoleManagementServiceImpl>>,

    /// Audit service
    pub audit_service: Option<Arc<AuditService>>,

    /// MFA service facade for generating TOTP secrets and QR codes
    pub mfa_service: Option<Arc<dyn authenc_grpc::service::MfaServiceFacade>>,
    // TODO: Add missing services for full IAM API functionality
    // These services are required by the migrated admin handlers:

    // TODO: Add role service when implemented
    // pub role_service: Arc<RoleManagementService>,
    // Required by: roles.rs

    // TODO: Add group service when implemented
    // pub group_service: Arc<GroupManagementService>,
    // Required by: groups.rs

    // TODO: Add organization service when implemented
    // pub organization_service: Arc<OrganizationService>,
    // Required by: organizations.rs

    // TODO: Add satker service when implemented
    // pub satker_service: Arc<SatkerManagementService>,
    // pub satker_auth_service: Arc<SatkerAuthorizationService>,
    // Required by: satker.rs

    // TODO: Add JIT provisioning service when implemented
    // pub jit_service: Arc<JitProvisioningService>,
    // Required by: jit_admin.rs

    // TODO: Add client registration service when implemented
    // pub client_registration_service: Arc<ClientRegistrationService>,
    // Required by: client_registration.rs, dcr_admin.rs

    // TODO: Add client policy service when implemented
    // pub client_policy_service: Arc<ClientPolicyService>,
    // Required by: client_policy.rs

    // TODO: Add federation service when implemented
    // pub federation_service: Arc<FederationService>,
    // Required by: federation.rs, federation_admin.rs

    // TODO: Add SPI service when implemented
    // pub spi_service: Arc<SpiManagementService>,
    // Required by: spi_management.rs, spi_federation.rs

    // TODO: Add UMA service when implemented
    // pub uma_service: Arc<UmaService>,
    // Required by: uma.rs

    // TODO: Add zero trust service when implemented
    // pub zero_trust_service: Arc<ZeroTrustService>,
    // Required by: zero_trust.rs

    // TODO: Add OID4VC service when implemented
    // pub oid4vc_service: Arc<Oid4VcService>,
    // Required by: oid4vc.rs

    // TODO: Add audit service when implemented
    // pub audit_service: Arc<AuditService>,
    // Required by: audit.rs (for querying and exporting audit logs)
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
            group_service: None,
            role_service: None,
            audit_service: None,
            mfa_service: None,
        }
    }

    /// Set group service (optional, for group management endpoints)
    pub fn with_group_service(mut self, group_service: Arc<GroupStore>) -> Self {
        self.group_service = Some(group_service);
        self
    }

    /// Set role service
    pub fn with_role_service(mut self, role_service: Arc<RoleManagementServiceImpl>) -> Self {
        self.role_service = Some(role_service);
        self
    }

    /// Set audit service
    pub fn with_audit_service(mut self, audit_service: Arc<AuditService>) -> Self {
        self.audit_service = Some(audit_service);
        self
    }

    /// Set MFA service (optional)
    pub fn with_mfa_service(
        mut self,
        mfa_service: Arc<dyn authenc_grpc::service::MfaServiceFacade>,
    ) -> Self {
        self.mfa_service = Some(mfa_service);
        self
    }
}
