//! Cryptographic Services API handlers
//!
//! REST API endpoints for cryptographic operations including HMAC, random number generation,
//! and re-encryption services.

use axum::{
    Router,
    extract::{Path, State},
    response::Json,
    routing::post,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{ApiError, ApiResponse, ApiResult};
use secreton_crypto::transit::algorithms::{generate_random, HashAlgorithm};

use super::AppState;

/// Create Crypto routes
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/hmac", post(compute_hmac))
        .route("/hmac/batch", post(compute_hmac_batch))
        .route("/random", post(generate_random_bytes))
        .route("/reencrypt", post(reencrypt_data))
}

/// HMAC algorithm options
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HmacAlgorithm {
    Sha256,
    Sha384,
    Sha512,
    Sha3_256,
}

impl std::fmt::Display for HmacAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HmacAlgorithm::Sha256 => write!(f, "sha256"),
            HmacAlgorithm::Sha384 => write!(f, "sha384"),
            HmacAlgorithm::Sha512 => write!(f, "sha512"),
            HmacAlgorithm::Sha3_256 => write!(f, "sha3-256"),
        }
    }
}

/// Output format options
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Hex,
    Base64,
    Raw,
}

/// HMAC request
#[derive(Debug, Deserialize, Serialize)]
pub struct HmacRequest {
    /// Name of the key to use for HMAC
    pub key_name: String,
    /// HMAC algorithm to use
    pub algorithm: HmacAlgorithm,
    /// Input data (base64 encoded)
    pub input: String,
    /// Output format
    #[serde(default = "default_output_format")]
    pub output_format: OutputFormat,
}

fn default_output_format() -> OutputFormat {
    OutputFormat::Hex
}

/// HMAC response
#[derive(Debug, Serialize, Deserialize)]
pub struct HmacResponse {
    /// HMAC output in requested format
    pub hmac: String,
    /// Algorithm used
    pub algorithm: HmacAlgorithm,
    /// Output format
    pub format: OutputFormat,
}

/// Batch HMAC request
#[derive(Debug, Deserialize, Serialize)]
pub struct BatchHmacRequest {
    /// Name of the key to use for HMAC
    pub key_name: String,
    /// HMAC algorithm to use
    pub algorithm: HmacAlgorithm,
    /// Multiple inputs (base64 encoded)
    pub inputs: Vec<String>,
    /// Output format
    #[serde(default = "default_output_format")]
    pub output_format: OutputFormat,
}

/// Batch HMAC response
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchHmacResponse {
    /// HMAC outputs in requested format
    pub hmacs: Vec<String>,
    /// Algorithm used
    pub algorithm: HmacAlgorithm,
    /// Output format
    pub format: OutputFormat,
}

/// Random bytes request
#[derive(Debug, Deserialize, Serialize)]
pub struct RandomRequest {
    /// Number of bytes to generate (1-65536)
    pub bytes: usize,
    /// Output format
    #[serde(default = "default_output_format")]
    pub format: OutputFormat,
}

/// Random bytes response
#[derive(Debug, Serialize, Deserialize)]
pub struct RandomResponse {
    /// Random data in requested format
    pub random_bytes: String,
    /// Number of bytes generated
    pub length: usize,
    /// Output format
    pub format: OutputFormat,
}

/// Re-encryption request
#[derive(Debug, Deserialize, Serialize)]
pub struct ReencryptRequest {
    /// Source key name
    pub source_key: String,
    /// Destination key name
    pub destination_key: String,
    /// Ciphertext to re-encrypt
    pub ciphertext: String,
}

/// Re-encryption response
#[derive(Debug, Serialize, Deserialize)]
pub struct ReencryptResponse {
    /// Re-encrypted ciphertext
    pub ciphertext: String,
}

/// Compute HMAC
///
/// # Endpoint
/// `POST /v1/crypto/hmac`
///
/// # Requirements
/// Validates: Requirements 3.1
#[tracing::instrument(skip(state, request), fields(key_name = %request.key_name, algorithm = ?request.algorithm))]
async fn compute_hmac(
    State(state): State<AppState>,
    Json(request): Json<HmacRequest>,
) -> ApiResult<Json<ApiResponse<HmacResponse>>> {
    info!(
        "Computing HMAC with key: {}, algorithm: {}",
        request.key_name, request.algorithm
    );

    // Decode base64 input
    let input_bytes = BASE64
        .decode(&request.input)
        .map_err(|e| ApiError::bad_request(format!("Invalid base64 input: {}", e)))?;

    // Get key from vault storage (stored as a secret)
    let key_path = format!("crypto/hmac-keys/{}", request.key_name);
    let secret = state
        .vault
        .get_secret(&key_path, "system")
        .await
        .map_err(|e| {
            error!("Failed to retrieve HMAC key: {:?}", e);
            ApiError::not_found(format!("HMAC key '{}' not found", request.key_name))
        })?;

    // Extract key data from secret
    let key_data = secret
        .data
        .get("key")
        .ok_or_else(|| ApiError::internal("Invalid key format in secret"))?;

    // Decode key from base64
    let key_bytes = BASE64
        .decode(key_data)
        .map_err(|e| ApiError::internal(format!("Failed to decode key: {}", e)))?;

    // Compute HMAC using the appropriate algorithm
    let hmac_bytes = compute_hmac_with_algorithm(&key_bytes, &input_bytes, request.algorithm)?;

    // Format output
    let hmac = format_output(&hmac_bytes, request.output_format);

    info!("HMAC computed successfully");
    Ok(Json(ApiResponse::success(HmacResponse {
        hmac,
        algorithm: request.algorithm,
        format: request.output_format,
    })))
}

/// Compute batch HMAC
///
/// # Endpoint
/// `POST /v1/crypto/hmac/batch`
///
/// # Requirements
/// Validates: Requirements 3.4
#[tracing::instrument(skip(state, request), fields(key_name = %request.key_name, algorithm = ?request.algorithm, count = request.inputs.len()))]
async fn compute_hmac_batch(
    State(state): State<AppState>,
    Json(request): Json<BatchHmacRequest>,
) -> ApiResult<Json<ApiResponse<BatchHmacResponse>>> {
    info!(
        "Computing batch HMAC with key: {}, algorithm: {}, count: {}",
        request.key_name,
        request.algorithm,
        request.inputs.len()
    );

    // Get key from vault storage (stored as a secret)
    let key_path = format!("crypto/hmac-keys/{}", request.key_name);
    let secret = state
        .vault
        .get_secret(&key_path, "system")
        .await
        .map_err(|e| {
            error!("Failed to retrieve HMAC key: {:?}", e);
            ApiError::not_found(format!("HMAC key '{}' not found", request.key_name))
        })?;

    // Extract key data from secret
    let key_data = secret
        .data
        .get("key")
        .ok_or_else(|| ApiError::internal("Invalid key format in secret"))?;

    // Decode key from base64
    let key_bytes = BASE64
        .decode(key_data)
        .map_err(|e| ApiError::internal(format!("Failed to decode key: {}", e)))?;

    let mut hmacs = Vec::with_capacity(request.inputs.len());

    // Process each input
    for input in &request.inputs {
        // Decode base64 input
        let input_bytes = BASE64
            .decode(input)
            .map_err(|e| ApiError::bad_request(format!("Invalid base64 input: {}", e)))?;

        // Compute HMAC
        let hmac_bytes = compute_hmac_with_algorithm(&key_bytes, &input_bytes, request.algorithm)?;

        // Format output
        let hmac = format_output(&hmac_bytes, request.output_format);
        hmacs.push(hmac);
    }

    info!("Batch HMAC computed successfully");
    Ok(Json(ApiResponse::success(BatchHmacResponse {
        hmacs,
        algorithm: request.algorithm,
        format: request.output_format,
    })))
}

/// Generate random bytes
///
/// # Endpoint
/// `POST /v1/crypto/random`
///
/// # Requirements
/// Validates: Requirements 3.2, 3.5
#[tracing::instrument(skip(_state, request), fields(bytes = request.bytes))]
async fn generate_random_bytes(
    State(_state): State<AppState>,
    Json(request): Json<RandomRequest>,
) -> ApiResult<Json<ApiResponse<RandomResponse>>> {
    info!("Generating {} random bytes", request.bytes);

    // Validate byte count
    if request.bytes == 0 || request.bytes > 65536 {
        return Err(ApiError::validation(
            "Byte count must be between 1 and 65536",
        ));
    }

    // Generate random bytes
    let random_bytes = generate_random(request.bytes)
        .map_err(|e| ApiError::internal(format!("Failed to generate random bytes: {}", e)))?;

    // Format output
    let random_str = format_output(&random_bytes, request.format);

    info!("Random bytes generated successfully");
    Ok(Json(ApiResponse::success(RandomResponse {
        random_bytes: random_str,
        length: request.bytes,
        format: request.format,
    })))
}

/// Re-encrypt data
///
/// # Endpoint
/// `POST /v1/crypto/reencrypt`
///
/// # Requirements
/// Validates: Requirements 3.3
#[tracing::instrument(skip(state, request), fields(source_key = %request.source_key, destination_key = %request.destination_key))]
async fn reencrypt_data(
    State(state): State<AppState>,
    Json(request): Json<ReencryptRequest>,
) -> ApiResult<Json<ApiResponse<ReencryptResponse>>> {
    info!(
        "Re-encrypting data from key: {} to key: {}",
        request.source_key, request.destination_key
    );

    // Decrypt with source key
    let plaintext = state
        .transit_engine
        .decrypt(&request.source_key, &request.ciphertext, None)
        .await
        .map_err(|e| {
            error!("Failed to decrypt with source key: {:?}", e);
            ApiError::bad_request(format!("Decryption failed: {}", e))
        })?;

    // Encrypt with destination key
    let ciphertext = state
        .transit_engine
        .encrypt(&request.destination_key, &plaintext, None, None)
        .await
        .map_err(|e| {
            error!("Failed to encrypt with destination key: {:?}", e);
            ApiError::internal(format!("Encryption failed: {}", e))
        })?;

    info!("Data re-encrypted successfully");
    Ok(Json(ApiResponse::success(ReencryptResponse { ciphertext })))
}

// Helper functions

/// Compute HMAC using the specified algorithm
pub fn compute_hmac_with_algorithm(
    key: &[u8],
    data: &[u8],
    algorithm: HmacAlgorithm,
) -> ApiResult<Vec<u8>> {
    use hmac::{Hmac, Mac};
    use sha2::{Sha256, Sha384, Sha512};
    use sha3::Sha3_256;

    match algorithm {
        HmacAlgorithm::Sha256 => {
            let mut mac = Hmac::<Sha256>::new_from_slice(key)
                .map_err(|e| ApiError::internal(format!("HMAC key initialization failed: {}", e)))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        HmacAlgorithm::Sha384 => {
            let mut mac = Hmac::<Sha384>::new_from_slice(key)
                .map_err(|e| ApiError::internal(format!("HMAC key initialization failed: {}", e)))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        HmacAlgorithm::Sha512 => {
            let mut mac = Hmac::<Sha512>::new_from_slice(key)
                .map_err(|e| ApiError::internal(format!("HMAC key initialization failed: {}", e)))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        HmacAlgorithm::Sha3_256 => {
            let mut mac = Hmac::<Sha3_256>::new_from_slice(key)
                .map_err(|e| ApiError::internal(format!("HMAC key initialization failed: {}", e)))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
    }
}

/// Format output bytes according to the requested format
pub fn format_output(bytes: &[u8], format: OutputFormat) -> String {
    match format {
        OutputFormat::Hex => hex::encode(bytes),
        OutputFormat::Base64 => BASE64.encode(bytes),
        OutputFormat::Raw => {
            // For raw format, we still need to return a string, so use base64
            // In a real implementation, this might return binary data
            BASE64.encode(bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_algorithm_display() {
        assert_eq!(HmacAlgorithm::Sha256.to_string(), "sha256");
        assert_eq!(HmacAlgorithm::Sha384.to_string(), "sha384");
        assert_eq!(HmacAlgorithm::Sha512.to_string(), "sha512");
        assert_eq!(HmacAlgorithm::Sha3_256.to_string(), "sha3-256");
    }

    #[test]
    fn test_format_output() {
        let data = b"test data";

        let hex = format_output(data, OutputFormat::Hex);
        assert_eq!(hex, "746573742064617461");

        let base64 = format_output(data, OutputFormat::Base64);
        assert_eq!(base64, "dGVzdCBkYXRh");
    }

    #[test]
    fn test_compute_hmac_sha256() {
        let key = b"secret_key";
        let data = b"message";

        let result = compute_hmac_with_algorithm(key, data, HmacAlgorithm::Sha256).unwrap();
        assert!(!result.is_empty());
        assert_eq!(result.len(), 32); // SHA-256 produces 32 bytes
    }

    #[test]
    fn test_compute_hmac_sha384() {
        let key = b"secret_key";
        let data = b"message";

        let result = compute_hmac_with_algorithm(key, data, HmacAlgorithm::Sha384).unwrap();
        assert!(!result.is_empty());
        assert_eq!(result.len(), 48); // SHA-384 produces 48 bytes
    }

    #[test]
    fn test_compute_hmac_sha512() {
        let key = b"secret_key";
        let data = b"message";

        let result = compute_hmac_with_algorithm(key, data, HmacAlgorithm::Sha512).unwrap();
        assert!(!result.is_empty());
        assert_eq!(result.len(), 64); // SHA-512 produces 64 bytes
    }

    #[test]
    fn test_compute_hmac_sha3_256() {
        let key = b"secret_key";
        let data = b"message";

        let result = compute_hmac_with_algorithm(key, data, HmacAlgorithm::Sha3_256).unwrap();
        assert!(!result.is_empty());
        assert_eq!(result.len(), 32); // SHA3-256 produces 32 bytes
    }

    #[test]
    fn test_hmac_request_serialization() {
        let request = HmacRequest {
            key_name: "test-key".to_string(),
            algorithm: HmacAlgorithm::Sha256,
            input: "dGVzdA==".to_string(),
            output_format: OutputFormat::Hex,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: HmacRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.key_name, request.key_name);
        assert_eq!(deserialized.algorithm, request.algorithm);
        assert_eq!(deserialized.input, request.input);
        assert_eq!(deserialized.output_format, request.output_format);
    }

    #[test]
    fn test_random_request_validation() {
        let valid_request = RandomRequest {
            bytes: 32,
            format: OutputFormat::Hex,
        };
        assert!(valid_request.bytes > 0 && valid_request.bytes <= 65536);

        let invalid_request = RandomRequest {
            bytes: 0,
            format: OutputFormat::Hex,
        };
        assert!(!(invalid_request.bytes > 0 && invalid_request.bytes <= 65536));

        let invalid_request2 = RandomRequest {
            bytes: 70000,
            format: OutputFormat::Hex,
        };
        assert!(!(invalid_request2.bytes > 0 && invalid_request2.bytes <= 65536));
    }
}
