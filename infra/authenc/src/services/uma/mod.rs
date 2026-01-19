//! UMA 2.0 (User-Managed Access) Implementation
//!
//! Complete implementation of UMA 2.0 specification for fine-grained authorization.
//!
//! # Components
//! - Resource registration and management
//! - Permission tickets
//! - Requesting Party Token (RPT) issuance
//! - Policy evaluation engine (ABAC)
//! - Claims gathering
//! - Resource owner authorization

pub mod claims_gathering;
/// Modul `init`.
pub mod init;
/// Modul `policy_engine`.
pub mod permission_endpoint;
/// Modul `rpt`.
pub mod policy_engine;
/// Modul `resource_owner_auth`.
pub mod resource_owner_auth;
/// Modul `rpt`.
pub mod rpt;

// Re-exports for convenience
pub use claims_gathering::{
    ClaimsGatheringFlow, ClaimsGatheringRequest, ClaimsGatheringResponse, ClaimsGatheringService,
    ClaimsSubmissionResult, SubmittedClaims,
};
pub use permission_endpoint::{AuthorizationContextBuilder, PermissionEndpoint};
pub use policy_engine::{
    EnvironmentContext, PolicyDecision, PolicyEngine, PolicyEvaluationContext,
    PolicyEvaluationResult, PolicyType, ResourceContext, SubjectContext, UmaPolicy,
};
pub use resource_owner_auth::{
    AuthorizationDecision, DelegationPolicy, ResourceOwnerAuthService,
    ResourceOwnerAuthorizationRequest, ResourceOwnerAuthorizationResponse,
};
pub use rpt::{Permission as RptPermission, Rpt, RptClaims, RptService};

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// UMA 2.0 Authorization request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UmaAuthorizationRequest {
    /// Grant type - must be "urn:ietf:params:oauth:grant-type:uma-ticket"
    pub grant_type: String,
    /// Permission ticket from resource server
    pub ticket: String,
    /// Optional claim token for claims pushing
    pub claim_token: Option<String>,
    /// Format of claim token (e.g., "urn:ietf:params:oauth:token-type:jwt")
    pub claim_token_format: Option<String>,
    /// PCT (Persisted Claims Token) for claims gathering continuation
    pub pct: Option<String>,
    /// RPT for upgrade (optional)
    pub rpt: Option<String>,
}

/// UMA 2.0 Authorization response (success)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UmaAuthorizationResponse {
    /// Access token (RPT - Requesting Party Token)
    pub access_token: String,
    /// Token type - always "Bearer"
    pub token_type: String,
    /// Token expiration in seconds
    pub expires_in: i64,
    /// Refresh token (optional)
    pub refresh_token: Option<String>,
    /// Upgraded RPT flag
    pub upgraded: Option<bool>,
    /// PCT (Persisted Claims Token) for claims gathering
    pub pct: Option<String>,
}

/// UMA 2.0 Permission request (from resource server to AS)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UmaPermissionRequest {
    /// Resource ID
    pub resource_id: String,
    /// Resource scopes requested
    pub resource_scopes: Vec<String>,
}

/// UMA 2.0 Permission response (ticket)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UmaPermissionResponse {
    /// Permission ticket
    pub ticket: String,
}

/// UMA 2.0 Error responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UmaError {
    /// Error code
    pub error: String,
    /// Error description
    pub error_description: Option<String>,
    /// HTTP status code for UMA errors (for example, 403 for need_info)
    pub status: Option<u16>,
    /// Required claims for claims gathering
    pub required_claims: Option<Vec<ClaimRequirement>>,
    /// Redirect URI for claims gathering
    pub redirect_uri: Option<String>,
    /// Ticket for claims gathering continuation
    pub ticket: Option<String>,
}

impl std::fmt::Display for UmaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error_description {
            Some(desc) => write!(f, "{}: {}", self.error, desc),
            None => write!(f, "{}", self.error),
        }
    }
}

/// Claim requirement for claims gathering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimRequirement {
    /// Claim name
    pub claim_name: String,
    /// Claim friendly name
    pub friendly_name: Option<String>,
    /// Claim issuer
    pub issuer: Option<Vec<String>>,
    /// Possible values
    pub possible_values: Option<Vec<String>>,
}

/// UMA service trait for coordinating all UMA operations
#[async_trait]
pub trait UmaService: Send + Sync {
    async fn request_permission_ticket(
        &self,
        resource_server_id: &str,
        permissions: Vec<UmaPermissionRequest>,
    ) -> Result<UmaPermissionResponse>;

    /// Authorize access and issue RPT
    async fn authorize_access(
        &self,
        request: UmaAuthorizationRequest,
        client_id: &str,
        realm_id: Uuid,
    ) -> Result<UmaAuthorizationResponse>;

    /// Validate RPT token
    async fn validate_rpt(&self, rpt: &str, realm_id: Uuid) -> Result<RptClaims>;

    /// Introspect RPT token
    async fn introspect_rpt(
        &self,
        rpt: &str,
        resource_server_id: &str,
    ) -> Result<RptIntrospectionResponse>;
}

/// RPT introspection response
#[derive(Debug, Clone, Serialize, Deserialize)]
    /// Request permission ticket from resource server
pub struct RptIntrospectionResponse {
    /// Whether token is active
    pub active: bool,
    /// Token expiration timestamp
    pub exp: Option<i64>,
    /// Token issued at timestamp
    pub iat: Option<i64>,
    /// Permissions contained in RPT
    pub permissions: Option<Vec<rpt::Permission>>,
}
