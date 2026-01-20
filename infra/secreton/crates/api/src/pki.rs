//! PKI Secrets Engine API endpoints
//!
//! Provides REST API for Public Key Infrastructure operations including:
//! - Root CA generation
//! - Role-based certificate issuance
//! - Certificate revocation
//! - CRL generation

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

use secreton_core::services::secrets::pki::{
    IssueCertificateRequest as CoreIssueCertificateRequest, PkiEngine, PkiError, PkiRole,
};

/// PKI API State wrapper
#[derive(Clone)]
pub struct PkiApiState {
    pub engine: Arc<PkiEngine>,
}

impl Default for PkiApiState {
    fn default() -> Self {
        Self {
            engine: Arc::new(PkiEngine::new()),
        }
    }
}

impl PkiApiState {
    /// Create new PKI API state with custom engine
    pub fn new(engine: Arc<PkiEngine>) -> Self {
        Self { engine }
    }

    /// Create new PKI API state with default engine
    pub fn with_default_engine() -> Self {
        Self::default()
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request to generate root CA
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateRootCARequest {
    /// Common Name for the root CA
    pub common_name: String,
    /// TTL in days (default: 3650 = 10 years)
    #[serde(default = "default_root_ca_ttl")]
    pub ttl_days: i64,
}

fn default_root_ca_ttl() -> i64 {
    3650 // 10 years
}

/// Response for root CA generation
#[derive(Debug, Serialize)]
pub struct GenerateRootCAResponse {
    pub success: bool,
    pub message: String,
    pub ca_name: String,
    pub certificate_pem: String,
    pub expires_at: String,
}

/// Request to create PKI role
#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    /// Role TTL in days (default: 90)
    #[serde(default = "default_role_ttl")]
    pub ttl_days: i64,
    /// Maximum TTL in days (default: 365)
    #[serde(default = "default_max_ttl")]
    pub max_ttl_days: i64,
    /// Allow any domain name
    #[serde(default)]
    pub allow_any_name: bool,
    /// Allowed domains (whitelist)
    #[serde(default)]
    pub allowed_domains: Vec<String>,
}

fn default_role_ttl() -> i64 {
    90
}

fn default_max_ttl() -> i64 {
    365
}

/// Response for role creation
#[derive(Debug, Serialize)]
pub struct CreateRoleResponse {
    pub success: bool,
    pub message: String,
    pub role_name: String,
}

/// Response for listing roles
#[derive(Debug, Serialize)]
pub struct ListRolesResponse {
    pub roles: Vec<String>,
}

/// Request to issue certificate
#[derive(Debug, Serialize, Deserialize)]
pub struct IssueCertificateRequest {
    /// Common Name (e.g., simpel.kejaksaan.go.id)
    pub common_name: String,
    /// Subject Alternative Names
    #[serde(default)]
    pub alt_names: Vec<String>,
    /// TTL in days (overrides role default)
    pub ttl_days: Option<i64>,
}

/// Response for certificate issuance
#[derive(Debug, Serialize)]
pub struct IssueCertificateResponse {
    pub success: bool,
    pub message: String,
    pub serial_number: String,
    pub certificate_pem: String,
    pub private_key_pem: String,
    pub ca_chain: Vec<String>,
    pub expires_at: String,
}

/// Request to revoke certificate
#[derive(Debug, Deserialize)]
pub struct RevokeCertificateRequest {
    /// Serial number of certificate to revoke
    pub serial_number: String,
}

/// Response for certificate revocation
#[derive(Debug, Serialize)]
pub struct RevokeCertificateResponse {
    pub success: bool,
    pub message: String,
    pub serial_number: String,
    pub revoked_at: String,
}

/// Response for certificate lookup
#[derive(Debug, Serialize)]
pub struct GetCertificateResponse {
    pub success: bool,
    pub serial_number: String,
    pub certificate_pem: String,
    pub private_key_pem: String,
    pub ca_chain: Vec<String>,
    pub expires_at: String,
}

/// Response for CRL generation
#[derive(Debug, Serialize)]
pub struct GetCRLResponse {
    pub success: bool,
    pub revoked_certificates: Vec<RevokedCertInfo>,
    pub generated_at: String,
}

#[derive(Debug, Serialize)]
pub struct RevokedCertInfo {
    pub serial_number: String,
    pub revoked_at: String,
}

/// Response for listing CAs
#[derive(Debug, Serialize)]
pub struct ListCAsResponse {
    pub cas: Vec<String>,
}

// ============================================================================
// Router Creation
// ============================================================================

/// Create PKI router with all endpoints
pub fn create_pki_router(state: PkiApiState) -> Router {
    Router::new()
        // CA operations
        .route("/ca/root", post(generate_root_ca))
        .route("/ca/list", get(list_cas))
        // Role operations
        .route("/roles", get(list_roles))
        .route("/roles/:role_name", post(create_role))
        // Certificate operations
        .route("/issue/:role_name", post(issue_certificate))
        .route("/revoke", post(revoke_certificate))
        .route("/cert/:serial_number", get(get_certificate))
        // CRL operations
        .route("/crl", get(get_crl))
        .with_state(state)
}

// ============================================================================
// Handler Functions
// ============================================================================

/// Generate root CA
/// POST /pki/ca/root
#[axum::debug_handler]
pub async fn generate_root_ca(
    State(state): State<PkiApiState>,
    Json(request): Json<GenerateRootCARequest>,
) -> Result<Json<GenerateRootCAResponse>, StatusCode> {
    info!(
        "Generating root CA with CN: {}, TTL: {} days",
        request.common_name, request.ttl_days
    );

    match state
        .engine
        .generate_root_ca(request.common_name.clone(), request.ttl_days)
        .await
    {
        Ok(ca) => {
            info!("Root CA generated successfully: {}", ca.name);
            Ok(Json(GenerateRootCAResponse {
                success: true,
                message: format!("Root CA '{}' generated successfully", ca.name),
                ca_name: ca.name,
                certificate_pem: ca.certificate_pem,
                expires_at: ca.expires_at.to_rfc3339(),
            }))
        }
        Err(e) => {
            error!("Failed to generate root CA: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List all CAs
/// GET /pki/ca/list
pub async fn list_cas(State(state): State<PkiApiState>) -> Json<ListCAsResponse> {
    let cas = state.engine.list_cas().await;
    Json(ListCAsResponse { cas })
}

/// Create PKI role
/// POST /pki/roles/{role_name}
#[axum::debug_handler]
pub async fn create_role(
    State(state): State<PkiApiState>,
    Path(role_name): Path<String>,
    Json(request): Json<CreateRoleRequest>,
) -> Result<Json<CreateRoleResponse>, StatusCode> {
    info!("Creating PKI role: {}", role_name);

    let role = PkiRole {
        name: role_name.clone(),
        ttl: chrono::Duration::days(request.ttl_days),
        max_ttl: chrono::Duration::days(request.max_ttl_days),
        allow_any_name: request.allow_any_name,
        allowed_domains: request.allowed_domains,
        ..Default::default()
    };

    match state.engine.create_role(role).await {
        Ok(_) => {
            info!("PKI role '{}' created successfully", role_name);
            Ok(Json(CreateRoleResponse {
                success: true,
                message: format!("Role '{}' created successfully", role_name),
                role_name,
            }))
        }
        Err(e) => {
            error!("Failed to create role '{}': {}", role_name, e);
            match e {
                PkiError::InvalidConfig(_) => Err(StatusCode::BAD_REQUEST),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

/// List all roles
/// GET /pki/roles
pub async fn list_roles(State(state): State<PkiApiState>) -> Json<ListRolesResponse> {
    let roles = state.engine.list_roles().await;
    Json(ListRolesResponse { roles })
}

/// Issue certificate from role
/// POST /pki/issue/{role_name}
#[axum::debug_handler]
pub async fn issue_certificate(
    State(state): State<PkiApiState>,
    Path(role_name): Path<String>,
    Json(request): Json<IssueCertificateRequest>,
) -> Result<Json<IssueCertificateResponse>, StatusCode> {
    info!(
        "Issuing certificate for CN: {} using role: {}",
        request.common_name, role_name
    );

    // Convert TTL from days to chrono::Duration
    let ttl = request.ttl_days.map(chrono::Duration::days);

    let core_request = CoreIssueCertificateRequest {
        common_name: request.common_name.clone(),
        alt_names: request.alt_names,
        ttl,
    };

    match state
        .engine
        .issue_certificate(&role_name, core_request)
        .await
    {
        Ok(cert) => {
            info!(
                "Certificate issued successfully with serial: {}",
                cert.serial_number
            );
            Ok(Json(IssueCertificateResponse {
                success: true,
                message: format!("Certificate issued with serial: {}", cert.serial_number),
                serial_number: cert.serial_number,
                certificate_pem: cert.certificate_pem,
                private_key_pem: cert.private_key_pem,
                ca_chain: cert.ca_chain,
                expires_at: cert.expires_at.to_rfc3339(),
            }))
        }
        Err(e) => {
            error!("Failed to issue certificate: {}", e);
            match e {
                PkiError::RoleNotFound(_) => Err(StatusCode::NOT_FOUND),
                PkiError::InvalidConfig(_) => Err(StatusCode::BAD_REQUEST),
                PkiError::CaNotFound(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

/// Revoke certificate
/// POST /pki/revoke
#[axum::debug_handler]
pub async fn revoke_certificate(
    State(state): State<PkiApiState>,
    Json(request): Json<RevokeCertificateRequest>,
) -> Result<Json<RevokeCertificateResponse>, StatusCode> {
    info!("Revoking certificate: {}", request.serial_number);

    match state
        .engine
        .revoke_certificate(&request.serial_number)
        .await
    {
        Ok(_) => {
            info!(
                "Certificate revoked successfully: {}",
                request.serial_number
            );
            Ok(Json(RevokeCertificateResponse {
                success: true,
                message: format!("Certificate {} revoked successfully", request.serial_number),
                serial_number: request.serial_number,
                revoked_at: chrono::Utc::now().to_rfc3339(),
            }))
        }
        Err(e) => {
            error!("Failed to revoke certificate: {}", e);
            match e {
                PkiError::CertificateNotFound(_) => Err(StatusCode::NOT_FOUND),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

/// Get certificate by serial number
/// GET /pki/cert/{serial_number}
#[axum::debug_handler]
pub async fn get_certificate(
    State(state): State<PkiApiState>,
    Path(serial_number): Path<String>,
) -> Result<Json<GetCertificateResponse>, StatusCode> {
    info!("Retrieving certificate: {}", serial_number);

    match state.engine.get_certificate(&serial_number).await {
        Ok(cert) => {
            info!("Certificate retrieved: {}", serial_number);
            Ok(Json(GetCertificateResponse {
                success: true,
                serial_number: cert.serial_number,
                certificate_pem: cert.certificate_pem,
                private_key_pem: cert.private_key_pem,
                ca_chain: cert.ca_chain,
                expires_at: cert.expires_at.to_rfc3339(),
            }))
        }
        Err(e) => {
            warn!("Certificate not found: {}", serial_number);
            match e {
                PkiError::CertificateNotFound(_) => Err(StatusCode::NOT_FOUND),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

/// Get Certificate Revocation List (CRL)
/// GET /pki/crl
pub async fn get_crl(State(state): State<PkiApiState>) -> Result<Json<GetCRLResponse>, StatusCode> {
    info!("Generating CRL");

    match state.engine.generate_crl().await {
        Ok(revoked_certs) => {
            info!("CRL generated with {} entries", revoked_certs.len());
            let certs_info = revoked_certs
                .into_iter()
                .map(|cert| RevokedCertInfo {
                    serial_number: cert.serial_number,
                    revoked_at: cert.revoked_at.to_rfc3339(),
                })
                .collect();

            Ok(Json(GetCRLResponse {
                success: true,
                revoked_certificates: certs_info,
                generated_at: chrono::Utc::now().to_rfc3339(),
            }))
        }
        Err(e) => {
            error!("Failed to generate CRL: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pki_api_state_creation() {
        let state = PkiApiState::default();
        assert!(Arc::strong_count(&state.engine) == 1);
    }

    #[test]
    fn test_request_deserialization() {
        let json = r#"{"common_name": "test.example.com", "ttl_days": 3650}"#;
        let request: GenerateRootCARequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.common_name, "test.example.com");
        assert_eq!(request.ttl_days, 3650);
    }

    #[test]
    fn test_role_request_defaults() {
        let json = r#"{"allowed_domains": ["example.com"]}"#;
        let request: CreateRoleRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.ttl_days, 90);
        assert_eq!(request.max_ttl_days, 365);
        assert!(!request.allow_any_name);
    }

    #[test]
    fn test_issue_cert_request() {
        let json = r#"{"common_name": "app.example.com", "alt_names": ["www.app.example.com"]}"#;
        let request: IssueCertificateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.common_name, "app.example.com");
        assert_eq!(request.alt_names.len(), 1);
    }
}
