//! PKI (Public Key Infrastructure) API handlers
//!
//! Provides REST endpoints for certificate lifecycle management including:
//! - Certificate issuance
//! - Certificate revocation
//! - OCSP (Online Certificate Status Protocol) responder
//! - CRL (Certificate Revocation List) generation
//! - Certificate templates
//! - Intermediate CA generation

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

use crate::{ApiResponse, ApiResult, handlers::AppState};
use secreton_core::services::secrets::pki::{
    CertificateTemplate, OcspResponse, OcspStatus, RenewalConfig,
};

/// Create PKI routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/ocsp/:serial", get(get_ocsp_status))
        .route("/renewal/config", get(get_renewal_config))
        .route("/renewal/config", post(set_renewal_config))
        .route("/renewal/check", get(check_renewal))
        .route("/templates", post(create_template))
        .route("/templates", get(list_templates))
        .route("/ca/intermediate", post(generate_intermediate_ca))
}

/// Get OCSP status for a certificate
///
/// # Endpoint
/// `GET /v1/pki/ocsp/{serial}`
///
/// # Parameters
/// - `serial`: Certificate serial number (hex format)
///
/// # Returns
/// OCSP response with certificate status (good, revoked, unknown)
///
/// # Example
/// ```bash
/// curl -X GET http://localhost:8200/v1/pki/ocsp/0000000000000001
/// ```
async fn get_ocsp_status(
    Path(serial): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<OcspResponse>>> {
    let pki_engine = state.pki_engine();

    let response = pki_engine
        .get_ocsp_status(&serial)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get OCSP status: {}", e);
            crate::error::ApiError::Internal {
                message: e.to_string(),
            }
        })?;

    Ok(Json(ApiResponse::success(response)))
}

/// Get renewal configuration
///
/// # Endpoint
/// `GET /v1/pki/renewal/config`
///
/// # Returns
/// Current renewal configuration
async fn get_renewal_config(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<RenewalConfig>>> {
    let pki_engine = state.pki_engine();
    let config = pki_engine.get_renewal_config().await;
    Ok(Json(ApiResponse::success(config)))
}

/// Set renewal configuration
///
/// # Endpoint
/// `POST /v1/pki/renewal/config`
///
/// # Body
/// ```json
/// {
///   "enabled": true,
///   "threshold_days": 30,
///   "check_interval_seconds": 3600
/// }
/// ```
async fn set_renewal_config(
    State(state): State<AppState>,
    Json(config): Json<RenewalConfig>,
) -> ApiResult<Json<ApiResponse<()>>> {
    let pki_engine = state.pki_engine();
    pki_engine.set_renewal_config(config).await;
    Ok(Json(ApiResponse::success(())))
}

/// Check certificates for renewal
///
/// # Endpoint
/// `GET /v1/pki/renewal/check`
///
/// # Returns
/// List of certificate serial numbers that need renewal
async fn check_renewal(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    let pki_engine = state.pki_engine();

    let serials = pki_engine
        .check_certificates_for_renewal()
        .await
        .map_err(|e| {
            tracing::error!("Failed to check certificates for renewal: {}", e);
            crate::error::ApiError::Internal {
                message: e.to_string(),
            }
        })?;

    Ok(Json(ApiResponse::success(serials)))
}

/// Create certificate template
///
/// # Endpoint
/// `POST /v1/pki/templates`
///
/// # Body
/// Certificate template configuration
async fn create_template(
    State(state): State<AppState>,
    Json(template): Json<CertificateTemplate>,
) -> ApiResult<Json<ApiResponse<()>>> {
    let pki_engine = state.pki_engine();

    pki_engine.create_role(template).await.map_err(|e| {
        tracing::error!("Failed to create template: {}", e);
        crate::error::ApiError::Internal {
            message: e.to_string(),
        }
    })?;

    Ok(Json(ApiResponse::success(())))
}

/// List certificate templates
///
/// # Endpoint
/// `GET /v1/pki/templates`
///
/// # Returns
/// List of template names
async fn list_templates(
    State(state): State<AppState>,
) -> ApiResult<Json<ApiResponse<Vec<String>>>> {
    let pki_engine = state.pki_engine();
    let templates = pki_engine.list_roles().await;
    Ok(Json(ApiResponse::success(templates)))
}

/// Generate intermediate CA request
#[derive(Debug, Deserialize, Serialize)]
struct GenerateIntermediateCaRequest {
    common_name: String,
    ttl_days: i64,
    parent_ca_name: String,
}

/// Generate intermediate CA
///
/// # Endpoint
/// `POST /v1/pki/ca/intermediate`
///
/// # Body
/// ```json
/// {
///   "common_name": "Intermediate CA",
///   "ttl_days": 1825,
///   "parent_ca_name": "root"
/// }
/// ```
async fn generate_intermediate_ca(
    State(state): State<AppState>,
    Json(request): Json<GenerateIntermediateCaRequest>,
) -> ApiResult<Json<ApiResponse<String>>> {
    let pki_engine = state.pki_engine();

    let ca = pki_engine
        .generate_intermediate_ca(
            request.common_name,
            request.ttl_days,
            &request.parent_ca_name,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to generate intermediate CA: {}", e);
            crate::error::ApiError::Internal {
                message: e.to_string(),
            }
        })?;

    Ok(Json(ApiResponse::success(ca.name)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use secreton_core::services::secrets::pki::{IssueCertificateRequest, PkiEngine, PkiRole};
    use std::sync::Arc;

    async fn setup_test_pki() -> Arc<PkiEngine> {
        let engine = Arc::new(PkiEngine::new());

        // Generate root CA
        engine
            .generate_root_ca("Test CA".to_string(), 365)
            .await
            .unwrap();

        // Create role
        let role = PkiRole {
            name: "test-role".to_string(),
            allow_any_name: true,
            ..Default::default()
        };
        engine.create_role(role).await.unwrap();

        engine
    }

    #[tokio::test]
    async fn test_ocsp_good_status() {
        let engine = setup_test_pki().await;

        // Issue a certificate
        let request = IssueCertificateRequest {
            common_name: "test.example.com".to_string(),
            alt_names: vec![],
            ttl: None,
        };

        let cert = engine
            .issue_certificate("test-role", request)
            .await
            .unwrap();

        // Check OCSP status
        let response = engine.get_ocsp_status(&cert.serial_number).await.unwrap();

        assert_eq!(response.status, OcspStatus::Good);
        assert_eq!(response.serial_number, cert.serial_number);
        assert!(response.revocation_time.is_none());
    }

    #[tokio::test]
    async fn test_ocsp_revoked_status() {
        let engine = setup_test_pki().await;

        // Issue and revoke a certificate
        let request = IssueCertificateRequest {
            common_name: "test.example.com".to_string(),
            alt_names: vec![],
            ttl: None,
        };

        let cert = engine
            .issue_certificate("test-role", request)
            .await
            .unwrap();

        engine.revoke_certificate(&cert.serial_number).await.unwrap();

        // Check OCSP status
        let response = engine.get_ocsp_status(&cert.serial_number).await.unwrap();

        assert_eq!(response.status, OcspStatus::Revoked);
        assert_eq!(response.serial_number, cert.serial_number);
        assert!(response.revocation_time.is_some());
    }

    #[tokio::test]
    async fn test_ocsp_unknown_status() {
        let engine = setup_test_pki().await;

        // Check status for non-existent certificate
        let response = engine
            .get_ocsp_status("nonexistent")
            .await
            .unwrap();

        assert_eq!(response.status, OcspStatus::Unknown);
        assert_eq!(response.serial_number, "nonexistent");
        assert!(response.revocation_time.is_none());
    }

    #[tokio::test]
    async fn test_renewal_config() {
        let engine = setup_test_pki().await;

        // Get default config
        let config = engine.get_renewal_config().await;
        assert!(config.enabled);
        assert_eq!(config.threshold_days, 30);

        // Update config
        let new_config = RenewalConfig {
            enabled: false,
            threshold_days: 60,
            check_interval_seconds: 7200,
        };
        engine.set_renewal_config(new_config.clone()).await;

        // Verify update
        let updated_config = engine.get_renewal_config().await;
        assert!(!updated_config.enabled);
        assert_eq!(updated_config.threshold_days, 60);
        assert_eq!(updated_config.check_interval_seconds, 7200);
    }

    #[tokio::test]
    async fn test_check_certificates_for_renewal() {
        let engine = setup_test_pki().await;

        // Issue a certificate with short TTL
        let request = IssueCertificateRequest {
            common_name: "short-lived.example.com".to_string(),
            alt_names: vec![],
            ttl: Some(chrono::Duration::days(15)), // Less than default threshold of 30 days
        };

        let cert = engine
            .issue_certificate("test-role", request)
            .await
            .unwrap();

        // Check for certificates needing renewal
        let serials = engine.check_certificates_for_renewal().await.unwrap();

        // Should include our short-lived certificate
        assert!(serials.contains(&cert.serial_number));
    }

    #[tokio::test]
    async fn test_template_enforcement() {
        let engine = setup_test_pki().await;

        // Create a restrictive template
        let template = CertificateTemplate {
            name: "restrictive".to_string(),
            ttl: chrono::Duration::days(7),  // Default TTL within max_ttl
            allow_any_name: false,
            allowed_domains: vec!["example.com".to_string()],
            require_cn: true,
            allow_localhost: false,
            max_ttl: chrono::Duration::days(30),
            ..Default::default()
        };
        engine.create_role(template).await.unwrap();

        // Test 1: Valid domain should succeed
        let request = IssueCertificateRequest {
            common_name: "test.example.com".to_string(),
            alt_names: vec![],
            ttl: None,
        };
        let result = engine.issue_certificate("restrictive", request).await;
        if let Err(ref e) = result {
            eprintln!("Test 1 failed with error: {:?}", e);
        }
        assert!(result.is_ok());

        // Test 2: Invalid domain should fail
        let request = IssueCertificateRequest {
            common_name: "test.invalid.com".to_string(),
            alt_names: vec![],
            ttl: None,
        };
        assert!(engine
            .issue_certificate("restrictive", request)
            .await
            .is_err());

        // Test 3: Localhost should fail
        let request = IssueCertificateRequest {
            common_name: "localhost".to_string(),
            alt_names: vec![],
            ttl: None,
        };
        assert!(engine
            .issue_certificate("restrictive", request)
            .await
            .is_err());

        // Test 4: TTL exceeding max_ttl should fail
        let request = IssueCertificateRequest {
            common_name: "test.example.com".to_string(),
            alt_names: vec![],
            ttl: Some(chrono::Duration::days(60)),
        };
        assert!(engine
            .issue_certificate("restrictive", request)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_intermediate_ca_generation() {
        let engine = setup_test_pki().await;

        // Generate intermediate CA
        let intermediate = engine
            .generate_intermediate_ca("Intermediate CA".to_string(), 365, "root")
            .await
            .unwrap();

        assert_eq!(intermediate.ca_type, secreton_core::services::secrets::pki::CaType::Intermediate);
        assert_eq!(intermediate.parent_ca, Some("root".to_string()));
        assert!(!intermediate.certificate_pem.is_empty());
        assert!(!intermediate.private_key_pem.is_empty());

        // Verify intermediate CA is in the list
        let cas = engine.list_cas().await;
        assert!(cas.contains(&intermediate.name));
    }
}
